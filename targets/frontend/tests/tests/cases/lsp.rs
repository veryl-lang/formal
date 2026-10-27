//! Finite UTF-16 prefix checks and actual production JSON-RPC definition, diagnostic,
//! and semantic-token paths over Unicode LF/CRLF documents. Wait for matching
//! document-version diagnostics before requests. LSP 3.17 position semantics apply:
//! https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#position
//! No blanket guarantee for other LSP features, encoding negotiation, Unicode line
//! separators, or arbitrary diagnostic source implementations.

use ropey::Rope;
use veryl_ls::coordinates::{byte_position, parser_column, utf16_column};
#[test]
fn utf16_positions_match_independent_encoded_prefixes() {
    for column in [0, 1, u32::MAX] {
        let token = veryl_parser::veryl_token::Token::new(
            "",
            0,
            column,
            0,
            0,
            veryl_parser::veryl_token::TokenSource::Builtin,
        );
        assert_eq!(
            token.end_column(),
            column.saturating_sub(1),
            "LSP coordinate output mismatch: empty metadata"
        );
    }
    let mut count = 0;
    for newline in ["\n", "\r\n", "\r"] {
        for word in ["ascii", "é", "漢字", "😀", "e\u{301}", "a𐐀b", "\t日本😀"] {
            let text = format!("{word}{newline}/*{word}*/ value{newline}{word}");
            let rope = Rope::from_str(&text);
            let mut start = 0;
            for (line, part) in text.split(newline).enumerate() {
                for (scalar, (byte, _)) in part
                    .char_indices()
                    .chain(std::iter::once((part.len(), ' ')))
                    .enumerate()
                {
                    let expected = part[..byte].encode_utf16().count();
                    assert_eq!(
                        byte_position(&rope, start + byte),
                        (line as u32, expected as u32),
                        "LSP coordinate output mismatch: byte position"
                    );
                    assert_eq!(
                        parser_column(&rope, line, expected),
                        scalar + 1,
                        "LSP coordinate output mismatch: parser column"
                    );
                    assert_eq!(
                        utf16_column(part, scalar + 1),
                        expected as u32,
                        "LSP coordinate output mismatch: outgoing column"
                    );
                    count += 1;
                }
                assert_eq!(
                    parser_column(&rope, line, usize::MAX),
                    part.chars().count() + 1,
                    "LSP coordinate output mismatch: clamp"
                );
                start += part.len() + newline.len();
            }
        }
    }
    eprintln!("LSP encoded prefix complete positions={count}");
}

use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};
use tower_lsp_server::jsonrpc::Request;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};
struct TestServer {
    req_stream: DuplexStream,
    res_stream: DuplexStream,
    recv_buf: Vec<u8>,
}

impl TestServer {
    fn new<F, S>(init: F) -> Self
    where
        F: FnOnce(Client) -> S,
        S: LanguageServer,
    {
        let (req_client, req_server) = tokio::io::duplex(1024);
        let (res_server, res_client) = tokio::io::duplex(1024);

        let (service, socket) = LspService::new(init);

        tokio::spawn(Server::new(req_server, res_server, socket).serve(service));

        Self {
            req_stream: req_client,
            res_stream: res_client,
            recv_buf: Vec::new(),
        }
    }

    fn encode(payload: &str) -> String {
        format!("Content-Length: {}\r\n\r\n{}", payload.len(), payload)
    }

    fn try_decode_one(buf: &mut Vec<u8>) -> Option<String> {
        let header_end = buf.windows(4).position(|w| w == b"\r\n\r\n")?;
        let header = std::str::from_utf8(&buf[..header_end]).unwrap();
        let len: usize = header
            .strip_prefix("Content-Length: ")
            .unwrap()
            .parse()
            .unwrap();
        let body_start = header_end + 4;
        let body_end = body_start + len;
        if buf.len() < body_end {
            return None;
        }
        let body = String::from_utf8(buf[body_start..body_end].to_vec()).unwrap();
        buf.drain(..body_end);
        Some(body)
    }

    async fn send_request(&mut self, req: Request) {
        let req = serde_json::to_string(&req).unwrap();
        let req = Self::encode(&req);
        self.req_stream.write_all(req.as_bytes()).await.unwrap();
    }

    async fn recv_message(&mut self) -> String {
        loop {
            if let Some(msg) = Self::try_decode_one(&mut self.recv_buf) {
                return msg;
            }
            let mut tmp = [0u8; 4096];
            let n = self.res_stream.read(&mut tmp).await.unwrap();
            assert!(n > 0, "LSP stream closed unexpectedly");
            self.recv_buf.extend_from_slice(&tmp[..n]);
        }
    }

    /// Receives until a response (a message carrying `result`) arrives,
    /// skipping interleaved notifications.
    async fn recv_response_skipping(&mut self) -> Value {
        loop {
            let msg = self.recv_message().await;
            let value: Value = serde_json::from_str(&msg).unwrap();
            if value.get("result").is_some() {
                return value;
            }
        }
    }
}

fn position(text: &str, byte: usize) -> Value {
    let prefix = &text[..byte];
    json!({"line":prefix.bytes().filter(|&b|b==b'\n').count(),"character":prefix.rsplit('\n').next().unwrap().encode_utf16().count()})
}
#[tokio::test]
async fn lsp_definition_diagnostics_and_semantic_tokens_use_utf16() {
    tokio::time::timeout(std::time::Duration::from_secs(60),async {
        let mut server=TestServer::new(veryl_ls::Backend::new);
        server.send_request(Request::build("initialize").params(json!({"capabilities":{}})).id(1).finish()).await;
        server.recv_response_skipping().await;
        server.send_request(Request::build("initialized").params(json!({})).finish()).await;
        let root=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/fixtures").join(format!("lsp-{}",std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("Veryl.toml"),"[project]\nname=\"lsp_contract\"\nversion=\"0.1.0\"\n[build]\nexclude_std=true\n").unwrap();
        let uri=format!("file://{}",root.join("test.veryl").display());
        let mut request_id=2;
        for newline in ["\n","\r\n"] {for word in ["é","漢","😀e\u{301}"] {
            let text=format!("module Top(/*{word}*/ source:input logic, o:output logic){{{newline}/*{word}*/ assign o=source;{newline}}}");
            server.send_request(Request::build("textDocument/didOpen").params(json!({"textDocument":{"uri":uri,"languageId":"veryl","version":request_id,"text":text}})).finish()).await;
            loop {let notification:Value=serde_json::from_str(&server.recv_message().await).unwrap();
                if notification["method"]=="textDocument/publishDiagnostics" && notification["params"]["version"]==request_id {assert_eq!(notification["params"]["diagnostics"],json!([]),"valid LSP fixture: {notification}");break;}
            }
            let use_byte=text.rfind("source").unwrap();let decl_byte=text.find("source").unwrap();
            request_id+=1;
            server.send_request(Request::build("textDocument/definition").params(json!({"textDocument":{"uri":uri},"position":position(&text,use_byte+5)})).id(request_id).finish()).await;
            let definition=server.recv_response_skipping().await;
            assert_eq!(definition["result"]["range"],json!({"start":position(&text,decl_byte),"end":position(&text,decl_byte+6)}),"LSP wire output mismatch: definition {word:?} {definition}");
            request_id+=1;
            server.send_request(Request::build("textDocument/semanticTokens/full").params(json!({"textDocument":{"uri":uri}})).id(request_id).finish()).await;
            let tokens=server.recv_response_skipping().await;let data=tokens["result"]["data"].as_array().expect("semantic token data");
            let mut line=0;let mut column=0;let mut found=false;
            for token in data.as_chunks::<5>().0{let delta=token[0].as_u64().unwrap();line+=delta;column=if delta==0 {column+token[1].as_u64().unwrap()}else{token[1].as_u64().unwrap()};if json!({"line":line,"character":column})==position(&text,use_byte){assert_eq!(token[2],6,"LSP wire output mismatch: token length");found=true;}}
            assert!(found,"LSP wire output mismatch: semantic position {word:?} {tokens}");
            let bad=text.replace("assign o=source","assign o=absent");let offset=bad.find("absent").unwrap();request_id+=1;
            server.send_request(Request::build("textDocument/didChange").params(json!({"textDocument":{"uri":uri,"version":request_id},"contentChanges":[{"text":bad}]})).finish()).await;
            loop {let notification:Value=serde_json::from_str(&server.recv_message().await).unwrap();if notification["method"]=="textDocument/publishDiagnostics" && notification["params"]["diagnostics"].as_array().is_some_and(|d|!d.is_empty()){
                let diagnostics=notification["params"]["diagnostics"].as_array().unwrap();let diagnostic=diagnostics.iter().find(|d|d["message"].as_str().is_some_and(|m|m.contains("absent"))).expect("undefined identifier diagnostic");
                assert_eq!(diagnostic["range"],json!({"start":position(&bad,offset),"end":position(&bad,offset+6)}),"LSP wire output mismatch: diagnostic {word:?}");break;
            }}
        }}
    }).await.expect("LSP fixture timed out");
}
