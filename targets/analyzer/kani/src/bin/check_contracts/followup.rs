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
    }
    out
}
