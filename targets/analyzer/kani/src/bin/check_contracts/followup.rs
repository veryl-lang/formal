//! Mutations for original-operand preparation and prepared equality/arithmetic.
use super::replace_once;
use std::{fs, path::Path};
use veryl_formal_runner::{Contract, Mutation};
const OP: &str = "veryl/crates/analyzer/src/ir/op.rs";
fn arm(source: &str, unary: bool, op: &str) -> std::ops::Range<usize> {
    let method = if unary {
        "    pub fn eval_value_unary("
    } else {
        "    pub fn eval_value_binary("
    };
    let base = source.find(method).unwrap();
    let start = base
        + source[base..]
            .find(&format!("            Op::{op} => {{"))
            .unwrap();
    let end = start
        + source[start + 1..]
            .find("\n            Op::")
            .map(|n| n + 1)
            .unwrap_or(source.len() - start);
    start..end
}
fn scoped(source: &str, unary: bool, op: &str, old: &str, new: &str) -> String {
    let range = arm(source, unary, op);
    let replaced = replace_once(&source[range.clone()], old, new);
    let mut result = source.to_owned();
    result.replace_range(range, &replaced);
    result
}
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let source = fs::read_to_string(repo.join(OP)).unwrap();
    let mut out = Vec::new();
    for &contract in selected {
        let mut add = |name, source| {
            out.push(Mutation {
                name,
                contract,
                path: OP,
                source,
            })
        };
        if contract.name == "signedq256-div-positive" {
            add(
                "signedq-div-positive-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let payload = x / y;",
                    "let payload = x % y;",
                ),
            );
            add(
                "signedq-div-positive-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "{let mut result=ValueBigUint::new_bigint(payload,width,signed);*result.payload ^= b1();result}",
                ),
            );
            add(
                "signedq-div-positive-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "signedq-div-positive-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "signedq256-div-negative" {
            add(
                "signedq-div-negative-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let payload = x / y;",
                    "let payload = x % y;",
                ),
            );
            add(
                "signedq-div-negative-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "{let mut result=ValueBigUint::new_bigint(payload,width,signed);*result.payload ^= b1();result}",
                ),
            );
            add(
                "signedq-div-negative-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "signedq-div-negative-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "signedq256-rem-positive" {
            add(
                "signedq-rem-positive-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let payload = x % y;",
                    "let payload = x / y;",
                ),
            );
            add(
                "signedq-rem-positive-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "{let mut result=ValueBigUint::new_bigint(payload,width,signed);*result.payload ^= b1();result}",
                ),
            );
            add(
                "signedq-rem-positive-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "signedq-rem-positive-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "signedq256-rem-negative" {
            add(
                "signedq-rem-negative-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let payload = x % y;",
                    "let payload = x / y;",
                ),
            );
            add(
                "signedq-rem-negative-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "{let mut result=ValueBigUint::new_bigint(payload,width,signed);*result.payload ^= b1();result}",
                ),
            );
            add(
                "signedq-rem-negative-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "signedq-rem-negative-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "signedq256-div-positive-positive-context" {
            add(
                "signedq-div-positive-positive-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
        }
        if contract.name == "signedq256-div-positive-negative-context" {
            add(
                "signedq-div-positive-negative-context-wrong-right-mode",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "signedq256-div-negative-positive-context" {
            add(
                "signedq-div-negative-positive-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
        }
        if contract.name == "signedq256-div-negative-negative-context" {
            add(
                "signedq-div-negative-negative-context-wrong-right-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width - 1, signed);",
                ),
            );
        }
        if contract.name == "signedq256-rem-positive-positive-context" {
            add(
                "signedq-rem-positive-positive-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
        }
        if contract.name == "signedq256-rem-positive-negative-context" {
            add(
                "signedq-rem-positive-negative-context-wrong-right-mode",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "signedq256-rem-negative-positive-context" {
            add(
                "signedq-rem-negative-positive-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
        }
        if contract.name == "signedq256-rem-negative-negative-context" {
            add(
                "signedq-rem-negative-negative-context-wrong-right-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width - 1, signed);",
                ),
            );
        }
        if contract.name == "powerq256-div" {
            add(
                "powerq-div-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Div",
                    "x.payload() / y.payload()",
                    "x.payload() % y.payload()",
                ),
            );
            add(
                "powerq-div-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload ^ b1(), width, signed)",
                ),
            );
            add(
                "powerq-div-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width, !signed)",
                ),
            );
            add(
                "powerq-div-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "powerq256-div-context" {
            add(
                "powerq-div-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "powerq-div-context-wrong-right-mode",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
            add(
                "powerq-div-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "powerq-div-context-wrong-right-source",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = x.as_ref().expand(width, signed);",
                ),
            );
        }
        if contract.name == "powerq256-rem" {
            add(
                "powerq-rem-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "x.payload() % y.payload()",
                    "x.payload() / y.payload()",
                ),
            );
            add(
                "powerq-rem-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload ^ b1(), width, signed)",
                ),
            );
            add(
                "powerq-rem-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width, !signed)",
                ),
            );
            add(
                "powerq-rem-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "powerq256-rem-context" {
            add(
                "powerq-rem-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "powerq-rem-context-wrong-right-mode",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
            add(
                "powerq-rem-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "powerq-rem-context-wrong-right-source",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = x.as_ref().expand(width, signed);",
                ),
            );
        }
        if contract.name == "unitq256-div" {
            add(
                "unitq-div-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Div",
                    "x.payload() / y.payload()",
                    "x.payload() % y.payload()",
                ),
            );
            add(
                "unitq-div-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload ^ b1(), width, signed)",
                ),
            );
            add(
                "unitq-div-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width, !signed)",
                ),
            );
            add(
                "unitq-div-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "unitq256-div-context" {
            add(
                "unitq-div-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "unitq-div-context-wrong-right-mode",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
            add(
                "unitq-div-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "unitq-div-context-wrong-right-source",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = x.as_ref().expand(width, signed);",
                ),
            );
        }
        if contract.name == "unitq256-rem" {
            add(
                "unitq-rem-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "x.payload() % y.payload()",
                    "x.payload() / y.payload()",
                ),
            );
            add(
                "unitq-rem-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload ^ b1(), width, signed)",
                ),
            );
            add(
                "unitq-rem-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width, !signed)",
                ),
            );
            add(
                "unitq-rem-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "unitq256-rem-context" {
            add(
                "unitq-rem-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "unitq-rem-context-wrong-right-mode",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
            add(
                "unitq-rem-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "unitq-rem-context-wrong-right-source",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = x.as_ref().expand(width, signed);",
                ),
            );
        }
        if contract.name == "powerhot256-prepared-signed" {
            add(
                "powerhot-wrong-negative-gate",
                replace_once(
                    &source,
                    "if negative && exp.bit(0) && !ret.is_zero() {",
                    "if !negative && exp.bit(0) && !ret.is_zero() {",
                ),
            );
            add(
                "powerhot-wrong-modulus",
                replace_once(
                    &source,
                    "let modulus = ValueBigUint::gen_mask(width) + BigUint::from(1u32);",
                    "let modulus = ValueBigUint::gen_mask(width - 1) + BigUint::from(1u32);",
                ),
            );
            add(
                "powerhot-wrong-signed-result-flag",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(payload, width, true)",
                    "ValueBigUint::new_biguint(payload, width, false)",
                ),
            );
            let changed = scoped(
                &source,
                false,
                "Pow",
                "ValueBigUint::new_biguint(payload, width, true)",
                "ValueBigUint::new_biguint(payload, width - 1, true)",
            );
            add(
                "powerhot-wrong-result-width",
                scoped(
                    &changed,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(payload, width, false)",
                    "ValueBigUint::new_biguint(payload, width - 1, false)",
                ),
            );
        }
        if contract.name == "powerhot256-prepared-unsigned" {
            add(
                "powerhot-wrong-unsigned-result-flag",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(payload, width, false)",
                    "ValueBigUint::new_biguint(payload, width, true)",
                ),
            );
        }
        if contract.name == "powerhot256-context-signed" {
            add(
                "powerhot-wrong-base-width",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "powerhot-wrong-native-exponent",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "Value::U64(y) => BigUint::from(y.payload),",
                    "Value::U64(y) => BigUint::from(y.payload ^ 1),",
                ),
            );
        }
        if contract.name == "powerhot256-context-unsigned" {
            add(
                "powerhot-wrong-base-mode",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "powerhot-wrong-wide-exponent",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "Value::BigUint(y) => y.payload.as_ref().clone(),",
                    "Value::BigUint(y) => y.payload.as_ref() ^ &b1(),",
                ),
            );
        }
        if contract.name == "powerones256-prepared-witness" {
            add(
                "powerones-wrong-negative-gate",
                replace_once(
                    &source,
                    "if negative && exp.bit(0) && !ret.is_zero() {",
                    "if !negative && exp.bit(0) && !ret.is_zero() {",
                ),
            );
            add(
                "powerones-wrong-exponent-parity",
                replace_once(
                    &source,
                    "if negative && exp.bit(0) && !ret.is_zero() {",
                    "if negative && !exp.bit(0) && !ret.is_zero() {",
                ),
            );
            add(
                "powerones-wrong-signed-result-flag",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(payload, width, true)",
                    "ValueBigUint::new_biguint(payload, width, false)",
                ),
            );
            add(
                "powerones-wrong-unsigned-result-flag",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(payload, width, false)",
                    "ValueBigUint::new_biguint(payload, width, true)",
                ),
            );
            let changed = scoped(
                &source,
                false,
                "Pow",
                "ValueBigUint::new_biguint(payload, width, true)",
                "ValueBigUint::new_biguint(payload, width - 1, true)",
            );
            add(
                "powerones-wrong-result-width",
                scoped(
                    &changed,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(payload, width, false)",
                    "ValueBigUint::new_biguint(payload, width - 1, false)",
                ),
            );
        }
        if contract.name == "powerones256-context-witness" {
            add(
                "powerones-wrong-base-width",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "powerones-wrong-base-mode",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "powerones-wrong-native-exponent",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "Value::U64(y) => BigUint::from(y.payload),",
                    "Value::U64(y) => BigUint::from(y.payload ^ 1),",
                ),
            );
            add(
                "powerones-wrong-wide-exponent",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "Value::BigUint(y) => y.payload.as_ref().clone(),",
                    "Value::BigUint(y) => y.payload.as_ref() ^ &b1(),",
                ),
            );
        }
        if contract.name == "ones256-right" {
            add(
                "ones-right-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "x.payload() * y.payload()",
                    "x.payload() + y.payload()",
                ),
            );
            add(
                "ones-right-missing-truncation",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "payload &= mask_cache.get(width);",
                    "",
                ),
            );
            add(
                "ones-right-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width, !signed)",
                ),
            );
            add(
                "ones-right-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "ones256-right-context" {
            add(
                "ones-right-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "ones-right-context-wrong-right-width",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width - 1, signed);",
                ),
            );
            add(
                "ones-right-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "ones-right-context-wrong-right-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "ones256-left" {
            add(
                "ones-left-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "x.payload() * y.payload()",
                    "x.payload() + y.payload()",
                ),
            );
            add(
                "ones-left-missing-truncation",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "payload &= mask_cache.get(width);",
                    "",
                ),
            );
            add(
                "ones-left-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width, !signed)",
                ),
            );
            add(
                "ones-left-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "ones256-left-context" {
            add(
                "ones-left-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "ones-left-context-wrong-right-width",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width - 1, signed);",
                ),
            );
            add(
                "ones-left-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "ones-left-context-wrong-right-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "signedpow256-div" {
            add(
                "signedpow-div-wrong-signed-operation",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let payload = x / y;",
                    "let payload = x % y;",
                ),
            );
            add(
                "signedpow-div-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "{let mut result=ValueBigUint::new_bigint(payload,width,signed);*result.payload ^= b1();result}",
                ),
            );
            add(
                "signedpow-div-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "signedpow-div-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "signedpow256-div-nonnegative-context" {
            add(
                "signedpow-div-nonnegative-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "signedpow-div-nonnegative-context-wrong-right-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width - 1, signed);",
                ),
            );
            add(
                "signedpow-div-nonnegative-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "signedpow-div-nonnegative-context-wrong-right-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "signedpow256-div-negative-context" {
            add(
                "signedpow-div-negative-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "signedpow-div-negative-context-wrong-right-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width - 1, signed);",
                ),
            );
            add(
                "signedpow-div-negative-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "signedpow-div-negative-context-wrong-right-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "signedpow256-rem" {
            add(
                "signedpow-rem-wrong-signed-operation",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let payload = x % y;",
                    "let payload = x / y;",
                ),
            );
            add(
                "signedpow-rem-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "{let mut result=ValueBigUint::new_bigint(payload,width,signed);*result.payload ^= b1();result}",
                ),
            );
            add(
                "signedpow-rem-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "signedpow-rem-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "signedpow256-rem-nonnegative-context" {
            add(
                "signedpow-rem-nonnegative-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "signedpow-rem-nonnegative-context-wrong-right-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width - 1, signed);",
                ),
            );
            add(
                "signedpow-rem-nonnegative-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "signedpow-rem-nonnegative-context-wrong-right-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "signedpow256-rem-negative-context" {
            add(
                "signedpow-rem-negative-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "signedpow-rem-negative-context-wrong-right-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width - 1, signed);",
                ),
            );
            add(
                "signedpow-rem-negative-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "signedpow-rem-negative-context-wrong-right-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "signedsmall256-div" {
            add(
                "signedsmall-div-wrong-signed-operation",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let payload = x / y;",
                    "let payload = x % y;",
                ),
            );
            add(
                "signedsmall-div-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "{ let mut result=ValueBigUint::new_bigint(payload,width,signed); *result.payload ^= b1(); result }",
                ),
            );
            add(
                "signedsmall-div-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "signedsmall-div-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "signedsmall256-div-nonnegative-context" {
            add(
                "signedsmall-div-nonnegative-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "signedsmall-div-nonnegative-context-wrong-right-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width - 1, signed);",
                ),
            );
            add(
                "signedsmall-div-nonnegative-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "signedsmall-div-nonnegative-context-wrong-right-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "signedsmall256-div-negative-context" {
            add(
                "signedsmall-div-negative-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "signedsmall-div-negative-context-wrong-right-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width - 1, signed);",
                ),
            );
            add(
                "signedsmall-div-negative-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "signedsmall-div-negative-context-wrong-right-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "signedsmall256-rem" {
            add(
                "signedsmall-rem-wrong-signed-operation",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let payload = x % y;",
                    "let payload = x / y;",
                ),
            );
            add(
                "signedsmall-rem-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "{ let mut result=ValueBigUint::new_bigint(payload,width,signed); *result.payload ^= b1(); result }",
                ),
            );
            add(
                "signedsmall-rem-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "signedsmall-rem-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "signedsmall256-rem-nonnegative-context" {
            add(
                "signedsmall-rem-nonnegative-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "signedsmall-rem-nonnegative-context-wrong-right-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width - 1, signed);",
                ),
            );
            add(
                "signedsmall-rem-nonnegative-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "signedsmall-rem-nonnegative-context-wrong-right-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "signedsmall256-rem-negative-context" {
            add(
                "signedsmall-rem-negative-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "signedsmall-rem-negative-context-wrong-right-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width - 1, signed);",
                ),
            );
            add(
                "signedsmall-rem-negative-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "signedsmall-rem-negative-context-wrong-right-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "opposite256-div" {
            add(
                "opposite-div-wrong-signed-operation",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let payload = x / y;",
                    "let payload = x % y;",
                ),
            );
            add(
                "opposite-div-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "{ let mut result = ValueBigUint::new_bigint(payload, width, signed); *result.payload ^= b1(); result }",
                ),
            );
            add(
                "opposite-div-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "opposite-div-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "opposite256-div-positive-context" {
            add(
                "opposite-div-positive-context-wrong-left-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "opposite-div-positive-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "opposite-div-positive-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "opposite-div-positive-context-wrong-right-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "opposite256-div-negative-context" {
            add(
                "opposite-div-negative-context-wrong-left-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "opposite-div-negative-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "opposite-div-negative-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "opposite-div-negative-context-wrong-right-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "opposite256-rem" {
            add(
                "opposite-rem-wrong-signed-operation",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let payload = x % y;",
                    "let payload = x / y;",
                ),
            );
            add(
                "opposite-rem-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "{ let mut result = ValueBigUint::new_bigint(payload, width, signed); *result.payload |= b1(); result }",
                ),
            );
            add(
                "opposite-rem-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "opposite-rem-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "opposite256-rem-positive-context" {
            add(
                "opposite-rem-positive-context-wrong-left-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "opposite-rem-positive-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "opposite-rem-positive-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "opposite-rem-positive-context-wrong-right-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "opposite256-rem-negative-context" {
            add(
                "opposite-rem-negative-context-wrong-left-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "opposite-rem-negative-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "opposite-rem-negative-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "opposite-rem-negative-context-wrong-right-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "multiply256-right" {
            add(
                "multiply-right-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "x.payload() * y.payload()",
                    "x.payload() + y.payload()",
                ),
            );
            add(
                "multiply-right-missing-truncation",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "payload &= mask_cache.get(width);",
                    "",
                ),
            );
            add(
                "multiply-right-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width, !signed)",
                ),
            );
            add(
                "multiply-right-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "multiply256-right-context" {
            add(
                "multiply-right-context-wrong-left-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "multiply-right-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "multiply-right-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "multiply-right-context-wrong-right-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "multiply256-left" {
            add(
                "multiply-left-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "x.payload() * y.payload()",
                    "x.payload() + y.payload()",
                ),
            );
            add(
                "multiply-left-missing-truncation",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "payload &= mask_cache.get(width);",
                    "",
                ),
            );
            add(
                "multiply-left-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width, !signed)",
                ),
            );
            add(
                "multiply-left-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "multiply256-left-context" {
            add(
                "multiply-left-context-wrong-left-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "multiply-left-context-wrong-left-width",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "multiply-left-context-wrong-left-source",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "multiply-left-context-wrong-right-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "division256-self-div-unsigned" {
            add(
                "division-self-div-unsigned-wrong-unsigned-operation",
                scoped(
                    &source,
                    false,
                    "Div",
                    "x.payload() / y.payload()",
                    "x.payload() % y.payload()",
                ),
            );
            add(
                "division-self-div-unsigned-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload ^ b1(), width, signed)",
                ),
            );
            add(
                "division-self-div-unsigned-wrong-result-sign",
                scoped(
                    &scoped(
                        &source,
                        false,
                        "Div",
                        "ValueBigUint::new_biguint(payload, width, signed)",
                        "ValueBigUint::new_biguint(payload, width, !signed)",
                    ),
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "division-self-div-unsigned-wrong-result-width",
                scoped(
                    &scoped(
                        &source,
                        false,
                        "Div",
                        "ValueBigUint::new_biguint(payload, width, signed)",
                        "ValueBigUint::new_biguint(payload, width - 1, signed)",
                    ),
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "division256-self-div-unsigned-context" {
            add(
                "division-self-div-unsigned-context-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "division-self-div-unsigned-context-wrong-lhs-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "division-self-div-unsigned-context-wrong-lhs-source",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "division-self-div-unsigned-context-wrong-rhs-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "division256-self-div-signed" {
            add(
                "division-self-div-signed-wrong-signed-operation",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let payload = x / y;",
                    "let payload = x % y;",
                ),
            );
            add(
                "division-self-div-signed-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "{ let mut ret = ValueBigUint::new_bigint(payload, width, signed); *ret.payload = ret.payload() ^ b1(); ret }",
                ),
            );
            add(
                "division-self-div-signed-wrong-result-sign",
                scoped(
                    &scoped(
                        &source,
                        false,
                        "Div",
                        "ValueBigUint::new_biguint(payload, width, signed)",
                        "ValueBigUint::new_biguint(payload, width, !signed)",
                    ),
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "division-self-div-signed-wrong-result-width",
                scoped(
                    &scoped(
                        &source,
                        false,
                        "Div",
                        "ValueBigUint::new_biguint(payload, width, signed)",
                        "ValueBigUint::new_biguint(payload, width - 1, signed)",
                    ),
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "division256-self-div-signed-context" {
            add(
                "division-self-div-signed-context-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "division-self-div-signed-context-wrong-lhs-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "division-self-div-signed-context-wrong-lhs-source",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "division-self-div-signed-context-wrong-rhs-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "division256-self-rem-unsigned" {
            add(
                "division-self-rem-unsigned-wrong-unsigned-operation",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "x.payload() % y.payload()",
                    "x.payload() / y.payload()",
                ),
            );
            add(
                "division-self-rem-unsigned-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload ^ b1(), width, signed)",
                ),
            );
            add(
                "division-self-rem-unsigned-wrong-result-sign",
                scoped(
                    &scoped(
                        &source,
                        false,
                        "Rem",
                        "ValueBigUint::new_biguint(payload, width, signed)",
                        "ValueBigUint::new_biguint(payload, width, !signed)",
                    ),
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "division-self-rem-unsigned-wrong-result-width",
                scoped(
                    &scoped(
                        &source,
                        false,
                        "Rem",
                        "ValueBigUint::new_biguint(payload, width, signed)",
                        "ValueBigUint::new_biguint(payload, width - 1, signed)",
                    ),
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "division256-self-rem-unsigned-context" {
            add(
                "division-self-rem-unsigned-context-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "division-self-rem-unsigned-context-wrong-lhs-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "division-self-rem-unsigned-context-wrong-lhs-source",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "division-self-rem-unsigned-context-wrong-rhs-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "division256-self-rem-signed" {
            add(
                "division-self-rem-signed-wrong-signed-operation",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let payload = x % y;",
                    "let payload = x / y;",
                ),
            );
            add(
                "division-self-rem-signed-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "{ let mut ret = ValueBigUint::new_bigint(payload, width, signed); *ret.payload = ret.payload() ^ b1(); ret }",
                ),
            );
            add(
                "division-self-rem-signed-wrong-result-sign",
                scoped(
                    &scoped(
                        &source,
                        false,
                        "Rem",
                        "ValueBigUint::new_biguint(payload, width, signed)",
                        "ValueBigUint::new_biguint(payload, width, !signed)",
                    ),
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "division-self-rem-signed-wrong-result-width",
                scoped(
                    &scoped(
                        &source,
                        false,
                        "Rem",
                        "ValueBigUint::new_biguint(payload, width, signed)",
                        "ValueBigUint::new_biguint(payload, width - 1, signed)",
                    ),
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "division256-self-rem-signed-context" {
            add(
                "division-self-rem-signed-context-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "division-self-rem-signed-context-wrong-lhs-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "division-self-rem-signed-context-wrong-lhs-source",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "division-self-rem-signed-context-wrong-rhs-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "division256-smaller-div" {
            add(
                "division-smaller-div-wrong-unsigned-operation",
                scoped(
                    &source,
                    false,
                    "Div",
                    "x.payload() / y.payload()",
                    "x.payload() % y.payload()",
                ),
            );
            add(
                "division-smaller-div-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload ^ b1(), width, signed)",
                ),
            );
            add(
                "division-smaller-div-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width, !signed)",
                ),
            );
            add(
                "division-smaller-div-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "division256-smaller-div-context" {
            add(
                "division-smaller-div-context-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "division-smaller-div-context-wrong-lhs-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "division-smaller-div-context-wrong-lhs-source",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "division-smaller-div-context-wrong-rhs-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "division256-smaller-rem" {
            add(
                "division-smaller-rem-wrong-unsigned-operation",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "x.payload() % y.payload()",
                    "x.payload() / y.payload()",
                ),
            );
            add(
                "division-smaller-rem-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload ^ b1(), width, signed)",
                ),
            );
            add(
                "division-smaller-rem-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width, !signed)",
                ),
            );
            add(
                "division-smaller-rem-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "division256-smaller-rem-context" {
            add(
                "division-smaller-rem-context-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "division-smaller-rem-context-wrong-lhs-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "division-smaller-rem-context-wrong-lhs-source",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "division-smaller-rem-context-wrong-rhs-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "division256-pow2-div" {
            add(
                "division-pow2-div-wrong-unsigned-operation",
                scoped(
                    &source,
                    false,
                    "Div",
                    "x.payload() / y.payload()",
                    "x.payload() % y.payload()",
                ),
            );
            add(
                "division-pow2-div-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload ^ b1(), width, signed)",
                ),
            );
            add(
                "division-pow2-div-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width, !signed)",
                ),
            );
            add(
                "division-pow2-div-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "division256-pow2-div-context" {
            add(
                "division-pow2-div-context-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "division-pow2-div-context-wrong-lhs-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "division-pow2-div-context-wrong-lhs-source",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "division-pow2-div-context-wrong-rhs-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "division256-pow2-rem" {
            add(
                "division-pow2-rem-wrong-unsigned-operation",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "x.payload() % y.payload()",
                    "x.payload() / y.payload()",
                ),
            );
            add(
                "division-pow2-rem-wrong-result-bit",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload ^ b1(), width, signed)",
                ),
            );
            add(
                "division-pow2-rem-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width, !signed)",
                ),
            );
            add(
                "division-pow2-rem-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "division256-pow2-rem-context" {
            add(
                "division-pow2-rem-context-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "division-pow2-rem-context-wrong-lhs-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "division-pow2-rem-context-wrong-lhs-source",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "division-pow2-rem-context-wrong-rhs-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "special256-signed-div-unit" {
            add(
                "special-signed-div-unit-increments-result",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "{ let mut ret = ValueBigUint::new_bigint(payload, width, signed); *ret.payload = ret.payload() + b1(); ret }",
                ),
            );
            add(
                "special-signed-div-unit-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "special-signed-div-unit-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
            add(
                "special-signed-div-unit-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let payload = x / y;",
                    "let payload = x % y;",
                ),
            );
        }
        if contract.name == "special256-signed-div-unit-context" {
            add(
                "special-signed-div-unit-context-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "special-signed-div-unit-context-wrong-lhs-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "special-signed-div-unit-context-wrong-lhs-source",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "special-signed-div-unit-context-wrong-rhs-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "special256-signed-rem-unit" {
            add(
                "special-signed-rem-unit-increments-result",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "{ let mut ret = ValueBigUint::new_bigint(payload, width, signed); *ret.payload = ret.payload() + b1(); ret }",
                ),
            );
            add(
                "special-signed-rem-unit-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "special-signed-rem-unit-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
            add(
                "special-signed-rem-unit-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let payload = x % y;",
                    "let payload = x / y;",
                ),
            );
        }
        if contract.name == "special256-signed-rem-unit-context" {
            add(
                "special-signed-rem-unit-context-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "special-signed-rem-unit-context-wrong-lhs-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "special-signed-rem-unit-context-wrong-lhs-source",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "special-signed-rem-unit-context-wrong-rhs-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "special256-zero-div" {
            add(
                "special-zero-div-increments-result",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "{ let mut ret = ValueBigUint::new_bigint(payload, width, signed); *ret.payload = ret.payload() + b1(); ret }",
                ),
            );
            add(
                "special-zero-div-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "special-zero-div-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
            add(
                "special-zero-div-unsigned-increments-result",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload + b1(), width, signed)",
                ),
            );
        }
        if contract.name == "special256-zero-div-context" {
            add(
                "special-zero-div-context-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "special-zero-div-context-wrong-lhs-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "special-zero-div-context-wrong-lhs-source",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "special-zero-div-context-wrong-rhs-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "special256-zero-rem" {
            add(
                "special-zero-rem-increments-result",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "{ let mut ret = ValueBigUint::new_bigint(payload, width, signed); *ret.payload = ret.payload() + b1(); ret }",
                ),
            );
            add(
                "special-zero-rem-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width, !signed)",
                ),
            );
            add(
                "special-zero-rem-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_bigint(payload, width, signed)",
                    "ValueBigUint::new_bigint(payload, width - 1, signed)",
                ),
            );
            add(
                "special-zero-rem-unsigned-increments-result",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload + b1(), width, signed)",
                ),
            );
        }
        if contract.name == "special256-zero-rem-context" {
            add(
                "special-zero-rem-context-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "special-zero-rem-context-wrong-lhs-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "special-zero-rem-context-wrong-lhs-source",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "special-zero-rem-context-wrong-rhs-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
        }
        if contract.name == "special256-power-one" {
            add(
                "special-power-one-increments-result",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(payload, width, true)",
                    "ValueBigUint::new_biguint(payload + b1(), width, true)",
                ),
            );
            add(
                "special-power-one-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(payload, width, true)",
                    "ValueBigUint::new_biguint(payload, width, false)",
                ),
            );
            add(
                "special-power-one-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(payload, width, true)",
                    "ValueBigUint::new_biguint(payload, width - 1, true)",
                ),
            );
            add(
                "special-power-one-drops-negative-base",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "base.sign() == Sign::Minus,",
                    "false,",
                ),
            );
        }
        if contract.name == "special256-power-one-context" {
            add(
                "special-power-one-context-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "special-power-one-context-wrong-lhs-width",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "special-power-one-context-wrong-lhs-source",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "special-power-one-context-expands-exponent",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, signed);\n                let y = &y.expand(width.max(y.width()), signed).into_owned();",
                ),
            );
        }
        if contract.name == "special256-power-unit-base" {
            add(
                "special-power-unit-base-increments-result",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(payload, width, true)",
                    "ValueBigUint::new_biguint(payload + b1(), width, true)",
                ),
            );
            add(
                "special-power-unit-base-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(payload, width, true)",
                    "ValueBigUint::new_biguint(payload, width, false)",
                ),
            );
            add(
                "special-power-unit-base-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(payload, width, true)",
                    "ValueBigUint::new_biguint(payload, width - 1, true)",
                ),
            );
            add(
                "special-power-unit-base-unsigned-increments-result",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(payload, width, false)",
                    "ValueBigUint::new_biguint(payload + b1(), width, false)",
                ),
            );
        }
        if contract.name == "special256-power-unit-base-context" {
            add(
                "special-power-unit-base-context-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "special-power-unit-base-context-wrong-lhs-width",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width - 1, signed);",
                ),
            );
            add(
                "special-power-unit-base-context-wrong-lhs-source",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "special-power-unit-base-context-expands-exponent",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, signed);\n                let expanded_exponent = y.expand(width.max(y.width()), signed);\n                let y = expanded_exponent.as_ref();",
                ),
            );
        }
        if contract.name == "identity256-mul-right" {
            add(
                "identity-mul-right-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "x.payload() * y.payload()",
                    "x.payload() + y.payload()",
                ),
            );
            add(
                "identity-mul-right-increments-result",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let mut payload = x.payload() * y.payload();",
                    "let mut payload = (x.payload() * y.payload()) + b1();",
                ),
            );
            add(
                "identity-mul-right-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width, !signed)",
                ),
            );
            add(
                "identity-mul-right-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "identity256-mul-right-context" {
            add(
                "identity-mul-right-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "identity-mul-right-wrong-rhs-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
            add(
                "identity-mul-right-wrong-prepare-width",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(x.width(), signed);",
                ),
            );
            add(
                "identity-mul-right-wrong-prepare-rhs",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let y = y.expand(width, signed);",
                    "let y = x.expand(width, signed);",
                ),
            );
        }
        if contract.name == "identity256-mul-left" {
            add(
                "identity-mul-left-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "x.payload() * y.payload()",
                    "x.payload() + y.payload()",
                ),
            );
            add(
                "identity-mul-left-increments-result",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let mut payload = x.payload() * y.payload();",
                    "let mut payload = (x.payload() * y.payload()) + b1();",
                ),
            );
            add(
                "identity-mul-left-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width, !signed)",
                ),
            );
            add(
                "identity-mul-left-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "identity256-mul-left-context" {
            add(
                "identity-mul-left-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "identity-mul-left-wrong-rhs-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
            add(
                "identity-mul-left-wrong-prepare-width",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(x.width(), signed);",
                ),
            );
            add(
                "identity-mul-left-wrong-prepare-rhs",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let y = y.expand(width, signed);",
                    "let y = x.expand(width, signed);",
                ),
            );
        }
        if contract.name == "identity256-div-one" {
            add(
                "identity-div-one-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Div",
                    "x.payload() / y.payload()",
                    "x.payload() + y.payload()",
                ),
            );
            add(
                "identity-div-one-increments-result",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let mut payload = x.payload() / y.payload();",
                    "let mut payload = (x.payload() / y.payload()) + b1();",
                ),
            );
            add(
                "identity-div-one-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width, !signed)",
                ),
            );
            add(
                "identity-div-one-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "identity256-div-one-context" {
            add(
                "identity-div-one-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "identity-div-one-wrong-rhs-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
            add(
                "identity-div-one-wrong-prepare-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(x.width(), signed);",
                ),
            );
            add(
                "identity-div-one-wrong-prepare-rhs",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = x.expand(width, signed);",
                ),
            );
        }
        if contract.name == "identity256-rem-one" {
            add(
                "identity-rem-one-wrong-operation",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "x.payload() % y.payload()",
                    "x.payload() + y.payload()",
                ),
            );
            add(
                "identity-rem-one-increments-result",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let mut payload = x.payload() % y.payload();",
                    "let mut payload = (x.payload() % y.payload()) + b1();",
                ),
            );
            add(
                "identity-rem-one-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width, !signed)",
                ),
            );
            add(
                "identity-rem-one-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "ValueBigUint::new_biguint(payload, width, signed)",
                    "ValueBigUint::new_biguint(payload, width - 1, signed)",
                ),
            );
        }
        if contract.name == "identity256-rem-one-context" {
            add(
                "identity-rem-one-wrong-lhs-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "identity-rem-one-wrong-rhs-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = y.expand(width, !signed);",
                ),
            );
            add(
                "identity-rem-one-wrong-prepare-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(x.width(), signed);",
                ),
            );
            add(
                "identity-rem-one-wrong-prepare-rhs",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = x.expand(width, signed);",
                ),
            );
        }
        if contract.name == "power256-unknown" {
            add(
                "power-unknown-X-becomes-Z",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "return Value::new_x(width, false);",
                    "return Value::new_z(width, false);",
                ),
            );
            add(
                "power-unknown-wrong-width",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "return Value::new_x(width, false);",
                    "return Value::new_x(width - 1, false);",
                ),
            );
            add(
                "power-unknown-wrong-sign",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "return Value::new_x(width, false);",
                    "return Value::new_x(width, true);",
                ),
            );
        }
        if contract.name == "power256-unknown-context" {
            add(
                "power-unknown-wrong-base-sign",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "power-unknown-wrong-base-width",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(x.width(), signed);",
                ),
            );
            add(
                "power-unknown-wrong-base-source",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "power-unknown-expands-exponent",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let prepared_exponent = y.expand(width, signed); let y = prepared_exponent.as_ref(); let x = x.expand(width, signed);",
                ),
            );
        }
        if contract.name == "power256-negative" {
            add(
                "power-negative-zero-base-known",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_x(width, v.signed)",
                    "ValueBigUint::new_biguint(b0(), width, v.signed)",
                ),
            );
            add(
                "power-negative-one-base-zero",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(b1(), width, v.signed)",
                    "ValueBigUint::new_biguint(b0(), width, v.signed)",
                ),
            );
            add(
                "power-negative-wrong-parity",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let p = if exp_odd { mask } else { b1() };",
                    "let p = if !exp_odd { mask } else { b1() };",
                ),
            );
            add(
                "power-negative-unsigned-minus-one",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "else if v.signed && p == mask {",
                    "else if p == mask {",
                ),
            );
            add(
                "power-negative-other-base-one",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(b0(), width, v.signed)",
                    "ValueBigUint::new_biguint(b1(), width, v.signed)",
                ),
            );
            add(
                "power-negative-minus-one-loses-sign",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(p, width, true)",
                    "ValueBigUint::new_biguint(p, width, false)",
                ),
            );
        }
        if contract.name == "power256-negative-context" {
            add(
                "power-negative-wrong-base-sign",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "power-negative-wrong-base-width",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(x.width(), signed);",
                ),
            );
            add(
                "power-negative-wrong-base-source",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "power-negative-expands-exponent",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let prepared_exponent = y.expand(width, signed); let y = prepared_exponent.as_ref(); let x = x.expand(width, signed);",
                ),
            );
        }
        if contract.name == "power256-zero" {
            add(
                "power-zero-signed-result-zero",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(payload, width, true)",
                    "ValueBigUint::new_biguint(b0(), width, true)",
                ),
            );
            add(
                "power-zero-unsigned-result-zero",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(payload, width, false)",
                    "ValueBigUint::new_biguint(b0(), width, false)",
                ),
            );
            add(
                "power-zero-unknown-base-one",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_x(width, x.signed)",
                    "ValueBigUint::new_biguint(b1(), width, x.signed)",
                ),
            );
            add(
                "power-zero-signed-result-loses-sign",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "ValueBigUint::new_biguint(payload, width, true)",
                    "ValueBigUint::new_biguint(payload, width, false)",
                ),
            );
        }
        if contract.name == "power256-zero-context" {
            add(
                "power-zero-wrong-base-sign",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "power-zero-wrong-base-width",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(x.width(), signed);",
                ),
            );
            add(
                "power-zero-wrong-base-source",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let x = y.expand(width, signed);",
                ),
            );
            add(
                "power-zero-expands-exponent",
                scoped(
                    &source,
                    false,
                    "Pow",
                    "let x = x.expand(width, signed);",
                    "let prepared_exponent = y.expand(width, signed); let y = prepared_exponent.as_ref(); let x = x.expand(width, signed);",
                ),
            );
        }
        if contract.name == "exception256-mul" {
            add(
                "exception-mul-wrong-X-as-Z",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "Value::BigUint(ValueBigUint::new_x(width, signed))",
                    "Value::new_z(width, signed)",
                ),
            );
            add(
                "exception-mul-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "Value::BigUint(ValueBigUint::new_x(width, signed))",
                    "Value::BigUint(ValueBigUint::new_x(width - 1, signed))",
                ),
            );
            add(
                "exception-mul-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "Value::BigUint(ValueBigUint::new_x(width, signed))",
                    "Value::BigUint(ValueBigUint::new_x(width, !signed))",
                ),
            );
            add(
                "exception-mul-loses-lhs-unknown-result",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "Value::BigUint(ValueBigUint::new_x(width, signed))",
                    "if x.is_xz() && !y.is_xz() { Value::from_u128(0, 0, width, signed) } else { Value::BigUint(ValueBigUint::new_x(width, signed)) }",
                ),
            );
            add(
                "exception-mul-loses-rhs-unknown-result",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "Value::BigUint(ValueBigUint::new_x(width, signed))",
                    "if !x.is_xz() && y.is_xz() { Value::from_u128(0, 0, width, signed) } else { Value::BigUint(ValueBigUint::new_x(width, signed)) }",
                ),
            );
        }
        if contract.name == "exception256-mul-context" {
            add(
                "exception-mul-wrong-prepare-sign",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "exception-mul-wrong-prepare-width",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(x.width(), signed);",
                ),
            );
            add(
                "exception-mul-wrong-prepare-rhs",
                scoped(
                    &source,
                    false,
                    "Mul",
                    "let y = y.expand(width, signed);",
                    "let y = x.expand(width, signed);",
                ),
            );
        }
        if contract.name == "exception256-div" {
            add(
                "exception-div-wrong-X-as-Z",
                scoped(
                    &source,
                    false,
                    "Div",
                    "Value::BigUint(ValueBigUint::new_x(width, signed))",
                    "Value::new_z(width, signed)",
                ),
            );
            add(
                "exception-div-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "Value::BigUint(ValueBigUint::new_x(width, signed))",
                    "Value::BigUint(ValueBigUint::new_x(width - 1, signed))",
                ),
            );
            add(
                "exception-div-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "Value::BigUint(ValueBigUint::new_x(width, signed))",
                    "Value::BigUint(ValueBigUint::new_x(width, !signed))",
                ),
            );
            add(
                "exception-div-loses-lhs-unknown-result",
                scoped(
                    &source,
                    false,
                    "Div",
                    "Value::BigUint(ValueBigUint::new_x(width, signed))",
                    "if x.is_xz() && !y.is_xz() { Value::from_u128(0, 0, width, signed) } else { Value::BigUint(ValueBigUint::new_x(width, signed)) }",
                ),
            );
            add(
                "exception-div-loses-rhs-unknown-result",
                scoped(
                    &source,
                    false,
                    "Div",
                    "Value::BigUint(ValueBigUint::new_x(width, signed))",
                    "if !x.is_xz() && y.is_xz() { Value::from_u128(0, 0, width, signed) } else { Value::BigUint(ValueBigUint::new_x(width, signed)) }",
                ),
            );
            add(
                "exception-div-loses-known-zero-divisor-result",
                scoped(
                    &source,
                    false,
                    "Div",
                    "Value::BigUint(ValueBigUint::new_x(width, signed))",
                    "if !x.is_xz() && !y.is_xz() && y.payload() == &b0() { Value::from_u128(0, 0, width, signed) } else { Value::BigUint(ValueBigUint::new_x(width, signed)) }",
                ),
            );
        }
        if contract.name == "exception256-div-context" {
            add(
                "exception-div-wrong-prepare-sign",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "exception-div-wrong-prepare-width",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(x.width(), signed);",
                ),
            );
            add(
                "exception-div-wrong-prepare-rhs",
                scoped(
                    &source,
                    false,
                    "Div",
                    "let y = y.expand(width, signed);",
                    "let y = x.expand(width, signed);",
                ),
            );
        }
        if contract.name == "exception256-rem" {
            add(
                "exception-rem-wrong-X-as-Z",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "Value::BigUint(ValueBigUint::new_x(width, signed))",
                    "Value::new_z(width, signed)",
                ),
            );
            add(
                "exception-rem-wrong-result-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "Value::BigUint(ValueBigUint::new_x(width, signed))",
                    "Value::BigUint(ValueBigUint::new_x(width - 1, signed))",
                ),
            );
            add(
                "exception-rem-wrong-result-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "Value::BigUint(ValueBigUint::new_x(width, signed))",
                    "Value::BigUint(ValueBigUint::new_x(width, !signed))",
                ),
            );
            add(
                "exception-rem-loses-lhs-unknown-result",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "Value::BigUint(ValueBigUint::new_x(width, signed))",
                    "if x.is_xz() && !y.is_xz() { Value::from_u128(0, 0, width, signed) } else { Value::BigUint(ValueBigUint::new_x(width, signed)) }",
                ),
            );
            add(
                "exception-rem-loses-rhs-unknown-result",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "Value::BigUint(ValueBigUint::new_x(width, signed))",
                    "if !x.is_xz() && y.is_xz() { Value::from_u128(0, 0, width, signed) } else { Value::BigUint(ValueBigUint::new_x(width, signed)) }",
                ),
            );
            add(
                "exception-rem-loses-known-zero-divisor-result",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "Value::BigUint(ValueBigUint::new_x(width, signed))",
                    "if !x.is_xz() && !y.is_xz() && y.payload() == &b0() { Value::from_u128(0, 0, width, signed) } else { Value::BigUint(ValueBigUint::new_x(width, signed)) }",
                ),
            );
        }
        if contract.name == "exception256-rem-context" {
            add(
                "exception-rem-wrong-prepare-sign",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                ),
            );
            add(
                "exception-rem-wrong-prepare-width",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(x.width(), signed);",
                ),
            );
            add(
                "exception-rem-wrong-prepare-rhs",
                scoped(
                    &source,
                    false,
                    "Rem",
                    "let y = y.expand(width, signed);",
                    "let y = x.expand(width, signed);",
                ),
            );
        }
        if contract.name == "arithmetic256-plus" {
            // Identity interpretation, including partially unknown values.
            for (name, replacement) in [
                (
                    "plus-poisons-unknown",
                    "if x.is_xz() { Value::new_x(width, x.signed()) } else { x.into_owned() }",
                ),
                (
                    "plus-loses-unknown",
                    "Value::from_u128(x.payload_u128(), 0, width, x.signed())",
                ),
                (
                    "plus-Z-becomes-X",
                    "Value::from_u128(x.payload_u128() & !x.mask_xz_u128(), x.mask_xz_u128(), width, x.signed())",
                ),
                (
                    "plus-changes-low-bit",
                    "Value::from_u128(x.payload_u128() ^ 1, x.mask_xz_u128(), width, x.signed())",
                ),
                (
                    "plus-loses-result-sign",
                    "{ let mut ret = x.into_owned(); ret.set_signed(false); ret }",
                ),
            ] {
                add(
                    name,
                    scoped(&source, true, "Add", "x.into_owned()", replacement),
                );
            }
        }
        if contract.name == "wiring256-plus" {
            for (name, replacement) in [
                (
                    "plus-wrong-expand-sign",
                    "let x = x.expand(width, !signed);",
                ),
                (
                    "plus-wrong-expand-width",
                    "let x = x.expand(x.width(), signed);",
                ),
                (
                    "plus-wrong-expand-source",
                    "let other = Value::from_u128(0, 0, 1, false); let x = other.expand(width, signed);",
                ),
            ] {
                add(
                    name,
                    scoped(
                        &source,
                        true,
                        "Add",
                        "let x = x.expand(width, signed);",
                        replacement,
                    ),
                );
            }
        }
        for (name, op) in [("and", "LogicAnd"), ("or", "LogicOr")] {
            if contract.name == format!("logical256-{name}-big-big") {
                let labels = if name == "and" {
                    [
                        "logical-and-loses-zero-dominance",
                        "logical-and-Z-becomes-one",
                        "logical-and-loses-unknown",
                        "logical-and-wrong-expand-sign",
                        "logical-and-wrong-result-width",
                        "logical-and-wrong-rhs",
                    ]
                } else {
                    [
                        "logical-or-loses-one-dominance",
                        "logical-or-Z-becomes-one",
                        "logical-or-loses-unknown",
                        "logical-or-wrong-expand-sign",
                        "logical-or-wrong-result-width",
                        "logical-or-wrong-rhs",
                    ]
                };
                let (old, new) = if name == "and" {
                    (
                        "(x.payload() == &b0() && x.mask_xz() == &b0())\n                            || (y.payload() == &b0() && y.mask_xz() == &b0())",
                        "(x.payload() == &b0() && x.mask_xz() == &b0())\n                            && (y.payload() == &b0() && y.mask_xz() == &b0())",
                    )
                } else {
                    (
                        "(x.payload() & (x.mask_xz() ^ x_mask) != b0())\n                            || (y.payload() & (y.mask_xz() ^ y_mask) != b0())",
                        "(x.payload() & (x.mask_xz() ^ x_mask) != b0())\n                            && (y.payload() & (y.mask_xz() ^ y_mask) != b0())",
                    )
                };
                add(labels[0], scoped(&source, false, op, old, new));
                for (old, new, label) in [
                    (
                        "(x.payload() & (x.mask_xz() ^ x_mask) != b0())",
                        "(x.payload() != &b0())",
                        labels[1],
                    ),
                    ("let is_x = ", "let is_x = false && ", labels[2]),
                    (
                        "let x = x.expand(xy_width, false);",
                        "let x = x.expand(xy_width, true);",
                        labels[3],
                    ),
                    (
                        "ret.expand(width, false).into_owned()",
                        "ret.expand(xy_width, false).into_owned()",
                        labels[4],
                    ),
                    (
                        "let y = y.expand(xy_width, false);",
                        "let y = x.expand(xy_width, false);",
                        labels[5],
                    ),
                ] {
                    add(label, scoped(&source, false, op, old, new));
                }
            }
        }
        let specs: [(&str, &str, bool, &str); 16] = [
            ("wiring256-and", "BitAnd", false, "wiring-and-wrong-sign"),
            ("wiring256-or", "BitOr", false, "wiring-or-wrong-sign"),
            ("wiring256-xor", "BitXor", false, "wiring-xor-wrong-sign"),
            ("wiring256-xnor", "BitXnor", false, "wiring-xnor-wrong-sign"),
            ("wiring256-not", "BitNot", true, "wiring-not-wrong-sign"),
            (
                "wiring256-left",
                "LogicShiftL",
                false,
                "wiring-left-wrong-sign",
            ),
            (
                "wiring256-right",
                "LogicShiftR",
                false,
                "wiring-right-wrong-sign",
            ),
            (
                "wiring256-arith-left",
                "ArithShiftL",
                false,
                "wiring-arith-left-wrong-sign",
            ),
            (
                "wiring256-arith-right",
                "ArithShiftR",
                false,
                "wiring-arith-right-wrong-sign",
            ),
            ("wiring256-eq", "Eq", false, "wiring-eq-wrong-result-width"),
            ("wiring256-ne", "Ne", false, "wiring-ne-wrong-result-width"),
            (
                "wiring256-wild-eq",
                "EqWildcard",
                false,
                "wiring-wild-eq-wrong-result-width",
            ),
            (
                "wiring256-wild-ne",
                "NeWildcard",
                false,
                "wiring-wild-ne-wrong-result-width",
            ),
            ("wiring256-add", "Add", false, "wiring-add-wrong-sign"),
            ("wiring256-sub", "Sub", false, "wiring-sub-wrong-sign"),
            ("wiring256-neg", "Sub", true, "wiring-neg-wrong-sign"),
        ];
        for (name, op, unary, label) in specs {
            if name != contract.name {
                continue;
            }
            let (old, new) = if op == "BitAnd" {
                (
                    "let x = resize(x, width, signed);",
                    "let x = resize(x, width, !signed);",
                )
            } else if name.contains("eq") || name == "wiring256-ne" || name == "wiring256-wild-ne" {
                (
                    "Value::U64(ret).expand(width, false).into_owned()",
                    "Value::U64(ret).expand(xy_width, false).into_owned()",
                )
            } else {
                (
                    "let x = x.expand(width, signed);",
                    "let x = x.expand(width, !signed);",
                )
            };
            add(label, scoped(&source, unary, op, old, new));
        }
        if contract.name == "wiring256-xor" {
            add(
                "wiring-xor-wrong-rhs-identity",
                scoped(
                    &source,
                    false,
                    "BitXor",
                    "let y = y.expand(width, signed);",
                    "let y = x.expand(width, signed);",
                ),
            );
        }
        for (name, op, relation, inclusive) in [
            ("less", "Less", "<", "<="),
            ("less-eq", "LessEq", "<=", "<"),
            ("greater", "Greater", ">", ">="),
            ("greater-eq", "GreaterEq", ">=", ">"),
        ] {
            let labels = match name {
                "less" => [
                    "ordering-less-unsigned-boundary",
                    "ordering-less-signed-direction",
                    "ordering-less-loses-unknown",
                    "ordering-less-ignores-signedness",
                ],
                "less-eq" => [
                    "ordering-less-eq-unsigned-boundary",
                    "ordering-less-eq-signed-direction",
                    "ordering-less-eq-loses-unknown",
                    "ordering-less-eq-ignores-signedness",
                ],
                "greater" => [
                    "ordering-greater-unsigned-boundary",
                    "ordering-greater-signed-direction",
                    "ordering-greater-loses-unknown",
                    "ordering-greater-ignores-signedness",
                ],
                _ => [
                    "ordering-greater-eq-unsigned-boundary",
                    "ordering-greater-eq-signed-direction",
                    "ordering-greater-eq-loses-unknown",
                    "ordering-greater-eq-ignores-signedness",
                ],
            };
            if contract.name == format!("ordering256-{name}-unsigned") {
                add(
                    labels[0],
                    scoped(
                        &source,
                        false,
                        op,
                        &format!("x.payload() {relation} y.payload()"),
                        &format!("x.payload() {inclusive} y.payload()"),
                    ),
                );
            }
            if contract.name == format!("ordering256-{name}-signed") {
                add(
                    labels[1],
                    scoped(
                        &source,
                        false,
                        op,
                        &format!("x.to_bigint() {relation} y.to_bigint()"),
                        &format!("y.to_bigint() {relation} x.to_bigint()"),
                    ),
                );
                add(
                    labels[3],
                    scoped(&source, false, op, "if signed {", "if false && signed {"),
                );
            }
            if contract.name == format!("ordering256-{name}-unknown") {
                add(
                    labels[2],
                    scoped(
                        &source,
                        false,
                        op,
                        "x.mask_xz() != &b0() || y.mask_xz() != &b0()",
                        "x.mask_xz() != &b0() && y.mask_xz() != &b0()",
                    ),
                );
            }
            if contract.name == format!("wiring256-{name}") {
                for (old, new, label) in [
                    (
                        "let x = x.expand(xy_width, signed);",
                        "let x = x.expand(xy_width, !signed);",
                        match name {
                            "less" => "wiring-less-wrong-sign",
                            "less-eq" => "wiring-less-eq-wrong-sign",
                            "greater" => "wiring-greater-wrong-sign",
                            _ => "wiring-greater-eq-wrong-sign",
                        },
                    ),
                    (
                        "Value::U64(ret).expand(width, false).into_owned()",
                        "Value::U64(ret).expand(xy_width, false).into_owned()",
                        match name {
                            "less" => "wiring-less-wrong-result-width",
                            "less-eq" => "wiring-less-eq-wrong-result-width",
                            "greater" => "wiring-greater-wrong-result-width",
                            _ => "wiring-greater-eq-wrong-result-width",
                        },
                    ),
                    (
                        "let y = y.expand(xy_width, signed);",
                        "let y = x.expand(xy_width, signed);",
                        match name {
                            "less" => "wiring-less-wrong-rhs",
                            "less-eq" => "wiring-less-eq-wrong-rhs",
                            "greater" => "wiring-greater-wrong-rhs",
                            _ => "wiring-greater-eq-wrong-rhs",
                        },
                    ),
                ] {
                    add(label, scoped(&source, false, op, old, new));
                }
            }
        }
        let equality = match contract.name {
            "equality256-eq" => Some((
                "Eq",
                "ValueU64::new_bit_0x(mismatch, unknown)",
                "ValueU64::new_bit_0x(mismatch && !unknown, unknown)",
                "equality-known-mismatch-loses-priority",
            )),
            "equality256-ne" => Some((
                "Ne",
                "ValueU64::new_bit_1x(mismatch, unknown)",
                "ValueU64::new_bit_1x(mismatch && !unknown, unknown)",
                "inequality-known-mismatch-loses-priority",
            )),
            "equality256-wild-eq" => Some((
                "EqWildcard",
                "let compare_mask = y.mask_xz() ^ full_mask;",
                "let compare_mask = x.mask_xz() ^ full_mask;",
                "wild-equality-masks-wrong-side",
            )),
            "equality256-wild-ne" => Some((
                "NeWildcard",
                "let compare_mask = y.mask_xz() ^ full_mask;",
                "let compare_mask = x.mask_xz() ^ full_mask;",
                "wild-inequality-masks-wrong-side",
            )),
            _ => None,
        };
        if let Some((op, old, new, name)) = equality {
            add(name, scoped(&source, false, op, old, new));
        }
        let range = if contract.name == "arithmetic256-add" {
            Some((false, "Add"))
        } else if contract.name == "arithmetic256-sub" {
            Some((false, "Sub"))
        } else if contract.name == "arithmetic256-neg" {
            Some((true, "Sub"))
        } else {
            None
        };
        if let Some((unary, op)) = range {
            let region = arm(&source, unary, op);
            let big = region.start + source[region.clone()].find("Value::BigUint(x)").unwrap();
            let replace = |old, new| {
                let mut s = source.clone();
                s.replace_range(
                    big..region.end,
                    &replace_once(&source[big..region.end], old, new),
                );
                s
            };
            match contract.name {
                "arithmetic256-add" => {
                    add(
                        "wide-add-loses-xz-poison",
                        replace("if x.is_xz() | y.is_xz() {", "if x.is_xz() && y.is_xz() {"),
                    );
                    add(
                        "wide-add-leaks-bit128",
                        replace(
                            "payload &= mask_cache.get(width);",
                            "if !(width == 128 && payload.bits() > 128) { payload &= mask_cache.get(width); }",
                        ),
                    );
                }
                "arithmetic256-sub" => {
                    add(
                        "wide-sub-loses-xz-poison",
                        replace("if x.is_xz() | y.is_xz() {", "if x.is_xz() && y.is_xz() {"),
                    );
                    add(
                        "wide-sub-skips-plus-one",
                        replace(
                            "((y.payload() ^ mask) + b1()) & mask",
                            "(y.payload() ^ mask) & mask",
                        ),
                    );
                }
                _ => {
                    add(
                        "wide-neg-loses-xz-poison",
                        replace("if ret.is_xz() {", "if false && ret.is_xz() {"),
                    );
                    add(
                        "wide-neg-leaks-bit128",
                        replace(
                            "*ret.payload &= mask;",
                            "if !(width == 128 && ret.payload.bits() > 128) { *ret.payload &= mask; }",
                        ),
                    );
                    add(
                        "wide-neg-skips-plus-one",
                        replace("*ret.payload += b1();", "// omitted increment"),
                    );
                }
            }
        }
        if contract.name == "conversion256-bigint" {
            let path = "veryl/crates/analyzer/src/value.rs";
            let original = fs::read_to_string(repo.join(path)).unwrap();
            out.push(Mutation {
                name: "conversion-bigint-loses-negative-sign",
                contract,
                path,
                source: replace_once(
                    &original,
                    "let sign = if msb { Sign::Minus } else { Sign::Plus };",
                    "let sign = if msb { Sign::Plus } else { Sign::Plus };",
                ),
            });
        }
    }
    out
}
