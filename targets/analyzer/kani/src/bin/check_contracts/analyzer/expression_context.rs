//! Semantic mutations of resolved literal metadata and raw values.
use std::{fs, path::Path};
use veryl_formal_runner::{Contract, Mutation};
pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let Some(&contract) = selected.iter().find(|c| c.name == "literal-leaf-context") else {
        return Vec::new();
    };
    let path = "veryl/crates/analyzer/src/ir/expression.rs";
    let source = fs::read_to_string(repo.join(path)).unwrap();
    let base = source.find("impl Factor {").unwrap();
    [
        ("literal-gather-wrong-width","gather_context",
         "Factor::Value(x) => ExpressionContext {\n                width: x.r#type.total_width().unwrap_or(0),",
         "Factor::Value(x) => ExpressionContext {\n                width: x.r#type.total_width().unwrap_or(0) + 1,"),
        ("literal-apply-inverts-sign","apply_context",
         "Factor::Value(x) => {\n                x.expr_context = expr_context;",
         "Factor::Value(x) => {\n                x.expr_context = ExpressionContext { signed: !expr_context.signed, ..expr_context };"),
        ("literal-apply-truncates-value","apply_context",
         "Factor::Value(x) => {\n                x.expr_context = expr_context;",
         "Factor::Value(x) => {\n                if let ValueVariant::Numeric(v) = &mut x.value { v.trunc(1); }\n                x.expr_context = expr_context;"),
        ("literal-eval-drops-sign","eval_value",
         "Factor::Value(x) => x.get_value().ok().cloned(),",
         "Factor::Value(x) => x.get_value().ok().cloned().map(|mut v| { v.set_signed(false); v }),"),
        ("literal-eval-drops-xz","eval_value",
         "Factor::Value(x) => x.get_value().ok().cloned(),",
         "Factor::Value(x) => x.get_value().ok().cloned().map(|mut v| { if let Value::U64(x) = &mut v { x.mask_xz = 0; } v }),"),
    ].into_iter().map(|(name,method,old,new)| {
        let begin=base+source[base..].find(&format!("    pub fn {method}(")).unwrap();
        let end=source[begin+1..].find("\n    pub fn ").map_or(source.len(),|n|begin+1+n);
        assert_eq!(source[begin..end].matches(old).count(),1,"literal mutation site changed: {name}");
        let mut changed=source.clone();
        changed.replace_range(begin..end,&source[begin..end].replacen(old,new,1));
        Mutation{name,contract,path,source:changed}
    }).collect()
}
