//! Seeded finite programs/options, not a grammar or option cross-product. Reparse
//! and compare syntax after removing location/identity and optional-comma metadata;
//! retain token text/order and decoded comment order. A second format must be
//! identical and a rendering witness must demonstrate changed input.

use serde_json::Value;
use veryl_formatter::Formatter;
use veryl_metadata::{Metadata, NewlineStyle};
use veryl_parser::{
    Parser,
    resource_table::{self, StrId},
    veryl_grammar_trait::Veryl,
};

fn text(token: &Value) -> String {
    resource_table::get_str_value(StrId(token["text"].as_u64().unwrap() as usize)).unwrap()
}
fn normalized(ast: &Veryl) -> (Value, Vec<String>) {
    fn visit(value: &mut Value, comments: &mut Vec<(u64, String)>) {
        match value {
            Value::Object(map) => {
                if let Some(tokens) = map.remove("comments") {
                    for token in tokens.as_array().unwrap() {
                        comments.push((
                            token["pos"].as_u64().unwrap(),
                            text(token).trim_end().to_string(),
                        ));
                    }
                }
                if map.contains_key("pos") && map.contains_key("source") && map.contains_key("text")
                {
                    let token = text(&Value::Object(map.clone()));
                    *value = serde_json::json!({"text":token});
                } else {
                    let trailing_comma = map.len() == 1 && map.contains_key("comma");
                    for child in map.values_mut() {
                        visit(child, comments);
                    }
                    if trailing_comma {
                        *value = Value::Null;
                    }
                }
            }
            Value::Array(values) => {
                for value in values {
                    visit(value, comments);
                }
            }
            _ => {}
        }
    }
    let mut value = serde_json::to_value(ast).unwrap();
    let mut comments = Vec::new();
    visit(&mut value, &mut comments);
    comments.sort_by_key(|(position, _)| *position);
    (value, comments.into_iter().map(|(_, text)| text).collect())
}
fn formatted(source: &str, metadata: &Metadata) -> String {
    let parser = Parser::parse(source, &"").unwrap();
    let mut formatter = Formatter::new(metadata);
    formatter.format(&parser.veryl, source);
    formatter.as_str().to_string()
}
#[test]
fn formatting_preserves_normalized_ast_comments_and_idempotence() {
    let mut changed = 0;
    for seed in 0..128 {
        for windows in [false, true] {
            let width = 1 + seed % 8;
            let expression = match seed % 4 {
                0 => "(a+b)*(a^b)",
                1 => "if a[0] ? (a+b) : (a-b)",
                2 => "(a<<1)^(b>>1)",
                _ => "~((a&b)|(a^b))",
            };
            let mut source = format!(
                "// 日本😀 seed {seed}\n/* inline α日本😀 */ #[allow(unused_variable)] module Sample{seed}(a:input logic<{width}>,b:input logic<{width}>,result:output logic<{width}>){{// lead α\nvar value:logic<{width}>;always_comb{{value={expression}; // step 中文\nif a[0]{{value=value^b;}}else{{value=b;}}}}assign result=value; // tail β\n}}// end 😀\n"
            );
            if windows {
                source = source.replace('\n', "\r\n");
            }
            let before = Parser::parse(&source, &"").unwrap();
            let expected = normalized(&before.veryl);
            let metadata = Metadata::create_default("prj").unwrap();
            let mut metadata = metadata;
            metadata.format.indent_width = 2 + seed % 3;
            metadata.format.max_width = 40 + (seed % 5) * 20;
            metadata.format.vertical_align = seed % 2 == 0;
            metadata.format.newline_style = if windows {
                NewlineStyle::Windows
            } else {
                NewlineStyle::Unix
            };
            let first = formatted(&source, &metadata);
            let after = Parser::parse(&first, &"").unwrap();
            let actual = normalized(&after.veryl);
            if expected != actual {
                let path =
                    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/fixtures");
                std::fs::create_dir_all(&path).unwrap();
                std::fs::write(
                    path.join(format!("format-{}-{seed}-before.json", std::process::id())),
                    serde_json::to_vec_pretty(&expected).unwrap(),
                )
                .unwrap();
                std::fs::write(
                    path.join(format!("format-{}-{seed}-after.json", std::process::id())),
                    serde_json::to_vec_pretty(&actual).unwrap(),
                )
                .unwrap();
            }
            assert!(
                expected == actual,
                "formatter semantics mismatch: normalized AST: seed={seed} windows={windows}"
            );
            assert_eq!(
                formatted(&first, &metadata),
                first,
                "formatter semantics mismatch: idempotence: seed={seed} windows={windows}"
            );
            changed += usize::from(first != source);
        }
    }
    assert!(changed > 0, "formatter rendering path absent");
    eprintln!("formatter complete cases=256");
}
