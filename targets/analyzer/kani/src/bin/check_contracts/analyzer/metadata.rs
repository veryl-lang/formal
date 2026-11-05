//! Mutations of metadata copying, cache invalidation and width representation.
use std::{fs, path::Path};
use veryl_formal_runner::{Contract, Mutation};

const PATH: &str = "veryl/crates/analyzer/src/ir/comptime.rs";

fn replace(source: &str, old: &str, new: &str) -> String {
    assert_eq!(
        source.matches(old).count(),
        1,
        "metadata mutation site moved"
    );
    source.replacen(old, new, 1)
}

fn method(source: &str, name: &str, old: &str, new: &str) -> String {
    let anchor = format!("    pub fn {name}(");
    let begin = source.find(&anchor).expect("metadata method moved");
    let end = begin + source[begin..].find("\n    }").unwrap() + 6;
    let mut result = source.to_owned();
    result.replace_range(begin..end, &replace(&source[begin..end], old, new));
    result
}

fn type_clone(source: &str, old: &str, new: &str) -> String {
    let source = replace(
        source,
        "#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]\npub struct Type {",
        "#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]\npub struct Type {",
    );
    let body = replace(
        "impl Clone for Type {\n    fn clone(&self) -> Self {\n        Self {\n            kind: self.kind.clone(),\n            signed: self.signed,\n            is_positive: self.is_positive,\n            array: self.array.clone(),\n            width: self.width.clone(),\n            width_expr: self.width_expr.clone(),\n            array_expr: self.array_expr.clone(),\n        }\n    }\n}\n\n",
        old,
        new,
    );
    replace(
        &source,
        "impl Default for Type {",
        &(body + "impl Default for Type {"),
    )
}

pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let source = fs::read_to_string(repo.join(PATH)).unwrap();
    let mut result = Vec::new();
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "resolved-type-copy-update")
    {
        for (name, old, new) in [
            (
                "type-copy-drops-sign",
                "signed: self.signed,",
                "signed: false,",
            ),
            (
                "type-copy-drops-deferred-width",
                "width_expr: self.width_expr.clone(),",
                "width_expr: empty_width_expr(),",
            ),
        ] {
            result.push(Mutation {
                name,
                contract,
                path: PATH,
                source: type_clone(&source, old, new),
            });
        }
        for (name, function, old, new) in [
            (
                "type-set-width-uses-one",
                "set_concrete_width",
                "self.width = shape;",
                "self.width = Shape::new(vec![Some(1)]);",
            ),
            (
                "type-set-width-loses-deferred-width",
                "set_concrete_width",
                "self.width_expr = crate::ir::WidthExpr::from_shape(&shape).into();",
                "self.width_expr = empty_width_expr();",
            ),
            (
                "type-clear-width-keeps-old-shape",
                "clear_width",
                "self.width = Shape::default();",
                "",
            ),
            (
                "type-clear-width-keeps-deferred-width",
                "clear_width",
                "self.width_expr = empty_width_expr();",
                "",
            ),
        ] {
            result.push(Mutation {
                name,
                contract,
                path: PATH,
                source: method(&source, function, old, new),
            });
        }
    }
    if let Some(&contract) = selected.iter().find(|c| c.name == "literal-metadata-copy") {
        let begin = source.find("impl Clone for Comptime {").unwrap();
        let end = begin + source[begin..].find("\nimpl Comptime {").unwrap();
        for (name, old, new) in [
            (
                "metadata-copy-retains-classification",
                "elaboration_const: None,",
                "elaboration_const: self.elaboration_const,",
            ),
            (
                "metadata-copy-drops-runtime-dependency",
                "runtime_dependent: self.runtime_dependent,",
                "runtime_dependent: false,",
            ),
            (
                "metadata-copy-retypes-value-sign",
                "value: self.value.clone(),",
                "value: { let mut value = self.value.clone(); if let ValueVariant::Numeric(v) = &mut value { v.set_signed(self.r#type.signed); } value },",
            ),
            (
                "metadata-copy-drops-context-sign",
                "expr_context: self.expr_context,",
                "expr_context: ExpressionContext { signed: false, ..self.expr_context },",
            ),
        ] {
            let mut changed = source.clone();
            changed.replace_range(begin..end, &replace(&source[begin..end], old, new));
            result.push(Mutation {
                name,
                contract,
                path: PATH,
                source: changed,
            });
        }
    }
    result
}
