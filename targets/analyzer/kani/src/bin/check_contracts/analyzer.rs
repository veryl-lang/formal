//! Analyzer-specific contracts and production mutations.
use std::fs;
use std::path::Path;
use veryl_formal_runner::{Contract, Mutation};
const CORE: &str = "veryl/crates/analyzer/src/value.rs";

const PUBLIC_RESIZE_COVERS: [&str; 6] = [
    "public resize request 1 is reachable",
    "public resize request 2 is reachable",
    "public resize request 3 is reachable",
    "public resize request 4 is reachable",
    "public resize request 5 is reachable",
    "public resize request 6 is reachable",
];

const PUBLIC_SMALL_CONDITIONAL_COVERS: [&str; 14] = [
    "public condition zero is reachable",
    "public condition one is reachable",
    "public condition X is reachable",
    "public condition Z is reachable",
    "public common Z becomes X",
    "small conditional one-bit condition is reachable",
    "small conditional two-bit condition is reachable",
    "small conditional zero context is reachable",
    "small conditional unit context is reachable",
    "small conditional wide context is reachable",
    "small conditional known one dominates unknown",
    "small conditional signed extension is reachable",
    "small conditional sign suppression is reachable",
    "small conditional u128 padding is reachable",
];

const PUBLIC_ASSIGNMENT_COVERS: [&str; 22] = [
    "public assignment source width 1 is reachable",
    "public assignment source width 4 is reachable",
    "public assignment source width 63 is reachable",
    "public assignment source width 64 is reachable",
    "public assignment ignored write is reachable",
    "public assignment clipped write is reachable",
    "public assignment reversed endpoints are reachable",
    "public assignment maximum high bound is reachable",
    "public assignment maximum low bound is reachable",
    "public assignment full destination is reachable",
    "public assignment X result is reachable",
    "public assignment Z result is reachable",
    "public assignment different signs are reachable",
    "public assignment u128 padding is reachable",
    "public assignment destination width 1 is reachable",
    "public assignment destination width 4 is reachable",
    "public assignment destination width 14 is reachable",
    "public assignment destination width 32 is reachable",
    "public assignment destination width 63 is reachable",
    "public assignment destination width 64 is reachable",
    "public assignment source width 14 is reachable",
    "public assignment source width 32 is reachable",
];

const PUBLIC_ARITHMETIC_COVERS: [&str; 14] = [
    "public arithmetic operation is reachable",
    "public arithmetic source width 1 is reachable",
    "public arithmetic source width 2 is reachable",
    "public arithmetic source width 3 is reachable",
    "public arithmetic result width 1 is reachable",
    "public arithmetic result width 5 is reachable",
    "public arithmetic signed context is reachable",
    "public arithmetic unsigned context is reachable",
    "public arithmetic growth is reachable",
    "public arithmetic natural width is reachable",
    "public arithmetic source X is reachable",
    "public arithmetic source Z is reachable",
    "public arithmetic signed extension is reachable",
    "public arithmetic sign suppression is reachable",
];

pub static CONTRACTS: [Contract; 62] = [
    Contract {
        group: "full-nonlinear",
        name: "full-multiply",
        harness: "native_full_multiply_widened",
        message: "full native nonlinear output mismatch",
        covers: &[
            "both full native operands use high bits",
            "full native multiply truncates overflow",
        ],
    },
    Contract {
        group: "values",
        name: "selection",
        harness: "native_selection_matches_reference_semantics",
        message: "selection semantics mismatch",
        covers: &[
            "reversed selection is reachable",
            "single-bit selection is reachable",
            "full-width selection is reachable",
            "invalid selection below bit 64 is reachable",
            "invalid selection above bit 63 is reachable",
            "overlapping selection is reachable",
            "maximum selection bounds are reachable",
            "selected source X is reachable",
            "selected source Z is reachable",
            "signed input selection is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-selection-small",
        harness: "public_selection_small_matches_reference_semantics",
        message: "public selection semantics mismatch",
        covers: &[
            "public selection width 1 is reachable",
            "public selection width 2 is reachable",
            "public selection width 3 is reachable",
            "public selection width 4 is reachable",
            "public selection reversed endpoints are reachable",
            "public selection one bit is reachable",
            "public selection full source is reachable",
            "public selection missing source is reachable",
            "public selection overlap is reachable",
            "public selection highest endpoint is reachable",
            "public selection maximum end is reachable",
            "public selection source X is reachable",
            "public selection source Z is reachable",
            "public selection signed source is reachable",
            "public selection u128 padding is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "partial-assignment",
        harness: "native_assignment_matches_reference_semantics",
        message: "partial-assignment semantics mismatch",
        covers: &[
            "full native width is reachable",
            "ignored writes are reachable",
            "clipped writes are reachable",
            "maximum high bound is reachable",
            "X result is reachable",
            "Z result is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-assignment-native",
        harness: "public_assignment_native_matches_reference_semantics",
        message: "public assignment semantics mismatch",
        covers: &PUBLIC_ASSIGNMENT_COVERS,
    },
    Contract {
        group: "values",
        name: "truncation",
        harness: "native_truncation_matches_reference_semantics",
        message: "truncation semantics mismatch",
        covers: &[
            "truncation from width 64 is reachable",
            "truncation to one bit is reachable",
            "strict truncation is reachable",
            "unchanged truncation is reachable",
            "truncated X is reachable",
            "truncated Z is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-resize-one",
        harness: "public_resize_one_matches_reference_semantics",
        message: "public resize semantics mismatch",
        covers: &PUBLIC_RESIZE_COVERS,
    },
    Contract {
        group: "values",
        name: "public-resize-two",
        harness: "public_resize_two_matches_reference_semantics",
        message: "public resize semantics mismatch",
        covers: &PUBLIC_RESIZE_COVERS,
    },
    Contract {
        group: "values",
        name: "public-resize-three",
        harness: "public_resize_three_matches_reference_semantics",
        message: "public resize semantics mismatch",
        covers: &PUBLIC_RESIZE_COVERS,
    },
    Contract {
        group: "values",
        name: "public-resize-four",
        harness: "public_resize_four_matches_reference_semantics",
        message: "public resize semantics mismatch",
        covers: &PUBLIC_RESIZE_COVERS,
    },
    Contract {
        group: "values",
        name: "extension",
        harness: "native_extension_matches_reference_semantics",
        message: "extension semantics mismatch",
        covers: &[
            "extension to width 64 is reachable",
            "equal-width extension is reachable",
            "smaller extension request is reachable",
            "unsigned zero extension is reachable",
            "disabled sign extension is reachable",
            "extended sign zero is reachable",
            "extended sign one is reachable",
            "extended sign X is reachable",
            "extended sign Z is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "logical-left-shift",
        harness: "native_logical_left_shift_matches_reference_semantics",
        message: "shift semantics mismatch",
        covers: &[
            "one-bit shift is reachable",
            "width-64 shift is reachable",
            "zero shift is reachable",
            "last-bit shift is reachable",
            "width-sized shift is reachable",
            "shift by 64 is reachable",
            "maximum shift is reachable",
            "unknown shift amount is reachable",
            "known unsigned shift is reachable",
            "known signed shift is reachable",
            "shifted source X is reachable",
            "shifted source Z is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "logical-right-shift",
        harness: "native_logical_right_shift_matches_reference_semantics",
        message: "shift semantics mismatch",
        covers: &[
            "one-bit shift is reachable",
            "width-64 shift is reachable",
            "zero shift is reachable",
            "last-bit shift is reachable",
            "width-sized shift is reachable",
            "shift by 64 is reachable",
            "maximum shift is reachable",
            "unknown shift amount is reachable",
            "known unsigned shift is reachable",
            "known signed shift is reachable",
            "shifted source X is reachable",
            "shifted source Z is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "arithmetic-left-shift",
        harness: "native_arithmetic_left_shift_matches_reference_semantics",
        message: "shift semantics mismatch",
        covers: &[
            "one-bit shift is reachable",
            "width-64 shift is reachable",
            "zero shift is reachable",
            "last-bit shift is reachable",
            "width-sized shift is reachable",
            "shift by 64 is reachable",
            "maximum shift is reachable",
            "unknown shift amount is reachable",
            "known unsigned shift is reachable",
            "known signed shift is reachable",
            "shifted source X is reachable",
            "shifted source Z is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "arithmetic-right-shift",
        harness: "native_arithmetic_right_shift_matches_reference_semantics",
        message: "shift semantics mismatch",
        covers: &[
            "one-bit shift is reachable",
            "width-64 shift is reachable",
            "zero shift is reachable",
            "last-bit shift is reachable",
            "width-sized shift is reachable",
            "shift by 64 is reachable",
            "maximum shift is reachable",
            "unknown shift amount is reachable",
            "known unsigned shift is reachable",
            "known signed shift is reachable",
            "shifted source X is reachable",
            "shifted source Z is reachable",
            "shift sign zero fill is reachable",
            "shift sign one fill is reachable",
            "shift sign X fill is reachable",
            "shift sign Z fill is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "equality",
        harness: "native_equality_matches_reference_semantics",
        message: "comparison semantics mismatch",
        covers: &[
            "width-64 comparison is reachable",
            "unequal-width comparison is reachable",
            "unknown sign extension comparison is reachable",
            "one-bit comparison is reachable",
            "mixed-sign comparison is reachable",
            "signed comparison is reachable",
            "unsigned comparison is reachable",
            "comparison zero is reachable",
            "comparison one is reachable",
            "comparison X is reachable",
            "known mismatch despite unknown bits is reachable",
            "one versus unknown equality is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "inequality",
        harness: "native_inequality_matches_reference_semantics",
        message: "comparison semantics mismatch",
        covers: &[
            "width-64 comparison is reachable",
            "unequal-width comparison is reachable",
            "unknown sign extension comparison is reachable",
            "one-bit comparison is reachable",
            "mixed-sign comparison is reachable",
            "signed comparison is reachable",
            "unsigned comparison is reachable",
            "comparison zero is reachable",
            "comparison one is reachable",
            "comparison X is reachable",
            "known mismatch despite unknown bits is reachable",
            "one versus unknown equality is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "wildcard-equality",
        harness: "native_wildcard_equality_matches_reference_semantics",
        message: "comparison semantics mismatch",
        covers: &[
            "width-64 comparison is reachable",
            "unequal-width comparison is reachable",
            "unknown sign extension comparison is reachable",
            "one-bit comparison is reachable",
            "mixed-sign comparison is reachable",
            "signed comparison is reachable",
            "unsigned comparison is reachable",
            "comparison zero is reachable",
            "comparison one is reachable",
            "comparison X is reachable",
            "RHS wildcard match is reachable",
            "unmatched LHS unknown is reachable",
            "wildcard known mismatch despite unknown bits is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "wildcard-inequality",
        harness: "native_wildcard_inequality_matches_reference_semantics",
        message: "comparison semantics mismatch",
        covers: &[
            "width-64 comparison is reachable",
            "unequal-width comparison is reachable",
            "unknown sign extension comparison is reachable",
            "one-bit comparison is reachable",
            "mixed-sign comparison is reachable",
            "signed comparison is reachable",
            "unsigned comparison is reachable",
            "comparison zero is reachable",
            "comparison one is reachable",
            "comparison X is reachable",
            "RHS wildcard match is reachable",
            "unmatched LHS unknown is reachable",
            "wildcard known mismatch despite unknown bits is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "less-than",
        harness: "native_less_than_matches_reference_semantics",
        message: "comparison semantics mismatch",
        covers: &[
            "width-64 comparison is reachable",
            "unequal-width comparison is reachable",
            "unknown sign extension comparison is reachable",
            "one-bit comparison is reachable",
            "mixed-sign comparison is reachable",
            "signed comparison is reachable",
            "unsigned comparison is reachable",
            "comparison zero is reachable",
            "comparison one is reachable",
            "comparison X is reachable",
            "negative versus positive comparison is reachable",
            "positive versus negative comparison is reachable",
            "equal known comparison is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "less-equal",
        harness: "native_less_equal_matches_reference_semantics",
        message: "comparison semantics mismatch",
        covers: &[
            "width-64 comparison is reachable",
            "unequal-width comparison is reachable",
            "unknown sign extension comparison is reachable",
            "one-bit comparison is reachable",
            "mixed-sign comparison is reachable",
            "signed comparison is reachable",
            "unsigned comparison is reachable",
            "comparison zero is reachable",
            "comparison one is reachable",
            "comparison X is reachable",
            "negative versus positive comparison is reachable",
            "positive versus negative comparison is reachable",
            "equal known comparison is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "greater-than",
        harness: "native_greater_than_matches_reference_semantics",
        message: "comparison semantics mismatch",
        covers: &[
            "width-64 comparison is reachable",
            "unequal-width comparison is reachable",
            "unknown sign extension comparison is reachable",
            "one-bit comparison is reachable",
            "mixed-sign comparison is reachable",
            "signed comparison is reachable",
            "unsigned comparison is reachable",
            "comparison zero is reachable",
            "comparison one is reachable",
            "comparison X is reachable",
            "negative versus positive comparison is reachable",
            "positive versus negative comparison is reachable",
            "equal known comparison is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "greater-equal",
        harness: "native_greater_equal_matches_reference_semantics",
        message: "comparison semantics mismatch",
        covers: &[
            "width-64 comparison is reachable",
            "unequal-width comparison is reachable",
            "unknown sign extension comparison is reachable",
            "one-bit comparison is reachable",
            "mixed-sign comparison is reachable",
            "signed comparison is reachable",
            "unsigned comparison is reachable",
            "comparison zero is reachable",
            "comparison one is reachable",
            "comparison X is reachable",
            "negative versus positive comparison is reachable",
            "positive versus negative comparison is reachable",
            "equal known comparison is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "bitwise",
        harness: "native_bitwise_matches_reference_semantics",
        message: "bitwise semantics mismatch",
        covers: &[
            "bitwise AND is reachable",
            "bitwise OR is reachable",
            "bitwise XOR is reachable",
            "bitwise XNOR is reachable",
            "one-bit bitwise operation is reachable",
            "width-64 bitwise operation is reachable",
            "upper stored bit is reachable",
            "AND zero dominates RHS unknown",
            "AND zero dominates LHS unknown",
            "OR one dominates RHS unknown",
            "OR one dominates LHS unknown",
            "bitwise zero is reachable",
            "bitwise one is reachable",
            "bitwise X is reachable",
            "bitwise source Z is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "bit-not",
        harness: "native_bit_not_matches_reference_semantics",
        message: "bit-not semantics mismatch",
        covers: &[
            "one-bit complement is reachable",
            "width-64 complement is reachable",
            "signed complement is reachable",
            "complement upper stored bit is reachable",
            "complement source zero is reachable",
            "complement source one is reachable",
            "complement source X is reachable",
            "complement source Z is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "logical",
        harness: "native_logical_matches_reference_semantics",
        message: "logical semantics mismatch",
        covers: &[
            "logical AND is reachable",
            "logical OR is reachable",
            "width-64 logical operation is reachable",
            "unequal-width logical operation is reachable",
            "logical AND zero dominates RHS unknown",
            "logical AND zero dominates LHS unknown",
            "logical OR one dominates RHS unknown",
            "logical OR one dominates LHS unknown",
            "known truth despite XZ is reachable",
            "logical zero is reachable",
            "logical one is reachable",
            "logical X is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "reductions",
        harness: "native_reductions_matches_reference_semantics",
        message: "reduction semantics mismatch",
        covers: &[
            "reduction AND is reachable",
            "reduction NAND is reachable",
            "reduction OR is reachable",
            "reduction NOR is reachable",
            "reduction XOR is reachable",
            "reduction XNOR is reachable",
            "logical NOT is reachable",
            "one-bit reduction is reachable",
            "width-64 reduction is reachable",
            "reduction zero dominates unknown",
            "reduction one dominates unknown",
            "even parity is reachable",
            "odd parity is reachable",
            "reduction X is reachable",
            "reduction source Z is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "condition-truth",
        harness: "native_condition_truth_matches_reference_semantics",
        message: "condition truth semantics mismatch",
        covers: &[
            "one-bit condition is reachable",
            "width-64 condition is reachable",
            "true condition is reachable",
            "false condition is reachable",
            "ambiguous X condition is reachable",
            "ambiguous Z condition is reachable",
            "condition known one dominates XZ",
        ],
    },
    Contract {
        group: "values",
        name: "conditional",
        harness: "native_conditional_matches_reference_semantics",
        message: "conditional semantics mismatch",
        covers: &[
            "one-bit conditional is reachable",
            "width-64 conditional is reachable",
            "conditional upper stored bit is reachable",
            "true arm selection is reachable",
            "false arm selection is reachable",
            "ambiguous arm merge is reachable",
            "selected true Z is reachable",
            "selected false Z is reachable",
            "merged known zero is reachable",
            "merged known one is reachable",
            "merged common Z becomes X",
            "merged X versus Z is reachable",
            "merged disagreement X is reachable",
            "conditional sign extension is reachable",
            "conditional disabled sign extension is reachable",
            "unselected arm determines width",
        ],
    },
    Contract {
        group: "values",
        name: "public-conditional",
        harness: "public_conditional_small_11_matches_reference_semantics",
        message: "public small conditional semantics mismatch",
        covers: &PUBLIC_SMALL_CONDITIONAL_COVERS,
    },
    Contract {
        group: "values",
        name: "public-conditional-small-12",
        harness: "public_conditional_small_12_matches_reference_semantics",
        message: "public small conditional semantics mismatch",
        covers: &PUBLIC_SMALL_CONDITIONAL_COVERS,
    },
    Contract {
        group: "values",
        name: "public-conditional-small-21",
        harness: "public_conditional_small_21_matches_reference_semantics",
        message: "public small conditional semantics mismatch",
        covers: &PUBLIC_SMALL_CONDITIONAL_COVERS,
    },
    Contract {
        group: "values",
        name: "public-conditional-small-22",
        harness: "public_conditional_small_22_matches_reference_semantics",
        message: "public small conditional semantics mismatch",
        covers: &PUBLIC_SMALL_CONDITIONAL_COVERS,
    },
    Contract {
        group: "values",
        name: "public-conditional-widened",
        harness: "public_conditional_widened_matches_reference_semantics",
        message: "public conditional layout semantics mismatch",
        covers: &[
            "public layout condition X is reachable",
            "public layout condition Z is reachable",
            "public layout signed extension is reachable",
            "public layout unsigned extension is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-conditional-word",
        harness: "public_conditional_word_matches_reference_semantics",
        message: "public conditional layout semantics mismatch",
        covers: &[
            "public layout condition X is reachable",
            "public layout condition Z is reachable",
            "public layout signed extension is reachable",
            "public layout unsigned extension is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "add-sub",
        harness: "native_add_sub_matches_reference_semantics",
        message: "add-sub semantics mismatch",
        covers: &[
            "addition is reachable",
            "subtraction is reachable",
            "one-bit arithmetic is reachable",
            "width-64 arithmetic is reachable",
            "signed arithmetic result is reachable",
            "unsigned arithmetic result is reachable",
            "addition carry wrap is reachable",
            "native addition overflow is reachable",
            "subtraction borrow wrap is reachable",
            "native subtraction underflow is reachable",
            "zero arithmetic is reachable",
            "arithmetic unknown LHS is reachable",
            "arithmetic unknown RHS is reachable",
            "arithmetic source Z is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "negation",
        harness: "native_negation_matches_reference_semantics",
        message: "negation semantics mismatch",
        covers: &[
            "one-bit negation is reachable",
            "width-64 negation is reachable",
            "negation of zero is reachable",
            "native zero negation carry is reachable",
            "minimum signed negation is reachable",
            "native minimum signed negation is reachable",
            "negation of one is reachable",
            "negation source X is reachable",
            "negation source Z is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "unary-plus",
        harness: "native_unary_plus_matches_reference_semantics",
        message: "unary-plus semantics mismatch",
        covers: &[
            "one-bit unary plus is reachable",
            "width-64 unary plus is reachable",
            "unary plus growth is reachable",
            "unary plus equal width is reachable",
            "unary plus smaller request is reachable",
            "unary plus preserves partial X",
            "unary plus preserves Z",
            "unary plus disabled sign is reachable",
            "unary plus extended Z is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-arithmetic-add",
        harness: "public_arithmetic_small_add_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &PUBLIC_ARITHMETIC_COVERS,
    },
    Contract {
        group: "values",
        name: "public-arithmetic-subtract",
        harness: "public_arithmetic_small_subtract_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &PUBLIC_ARITHMETIC_COVERS,
    },
    Contract {
        group: "values",
        name: "public-arithmetic",
        harness: "public_arithmetic_one_bit_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "public addition is reachable",
            "public subtraction is reachable",
            "public negation is reachable",
            "public unary plus is reachable",
            "public plus preserves Z",
        ],
    },
    Contract {
        group: "values",
        name: "public-arithmetic-plus",
        harness: "public_arithmetic_small_plus_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "public plus source width 1 is reachable",
            "public plus source width 2 is reachable",
            "public plus source width 3 is reachable",
            "public plus growth is reachable",
            "public plus natural width is reachable",
            "public plus sign extension is reachable",
            "public plus sign suppression is reachable",
            "public plus preserves X",
            "public plus preserves Z",
            "public plus u128 padding is reachable",
            "public plus extended X is reachable",
            "public plus extended Z is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-arithmetic-negate",
        harness: "public_arithmetic_small_negate_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &PUBLIC_ARITHMETIC_COVERS,
    },
    Contract {
        group: "values",
        name: "context-merge",
        harness: "context_merge_matches_reference_semantics",
        message: "context merge semantics mismatch",
        covers: &[
            "zero context width is reachable",
            "maximum context width is reachable",
            "wider right context is reachable",
            "wider left context is reachable",
            "unsigned sibling is reachable",
            "signed context merge is reachable",
            "nonconstant left context is reachable",
            "nonglobal right context is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "context-unary",
        harness: "unary_context_matches_reference_semantics",
        message: "unary context semantics mismatch",
        covers: &[
            "unary plus context is reachable",
            "unary negation context is reachable",
            "unary complement context is reachable",
            "unary and context is reachable",
            "unary nand context is reachable",
            "unary or context is reachable",
            "unary nor context is reachable",
            "unary xor context is reachable",
            "unary xnor context is reachable",
            "unary not context is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "context-binary",
        harness: "binary_context_matches_reference_semantics",
        message: "binary context semantics mismatch",
        covers: &[
            "binary Add context is reachable",
            "binary Sub context is reachable",
            "binary Mul context is reachable",
            "binary Div context is reachable",
            "binary Rem context is reachable",
            "binary BitAnd context is reachable",
            "binary BitOr context is reachable",
            "binary BitXor context is reachable",
            "binary BitXnor context is reachable",
            "binary Eq context is reachable",
            "binary Ne context is reachable",
            "binary EqWildcard context is reachable",
            "binary NeWildcard context is reachable",
            "binary Less context is reachable",
            "binary LessEq context is reachable",
            "binary Greater context is reachable",
            "binary GreaterEq context is reachable",
            "binary LogicAnd context is reachable",
            "binary LogicOr context is reachable",
            "binary LogicShiftL context is reachable",
            "binary LogicShiftR context is reachable",
            "binary ArithShiftL context is reachable",
            "binary ArithShiftR context is reachable",
            "binary Pow context is reachable",
            "binary As context is reachable",
            "unsized cast target fallback is reachable",
            "cast context narrowing is reachable",
            "cast context growth is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "context-cast",
        harness: "cast_context_matches_reference_semantics",
        message: "cast context semantics mismatch",
        covers: &[
            "type cast context is reachable",
            "numeric cast context is reachable",
            "numeric cast preserves signed source",
            "numeric cast preserves unsigned source",
            "type cast changes signedness",
            "cast operand context growth is reachable",
            "cast operand context retains wider source",
            "cast operand context equal width is reachable",
            "unresolved cast width fallback is reachable",
            "maximum cast context width is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-nonlinear",
        harness: "public_nonlinear_one_bit_matches_reference_semantics",
        message: "public nonlinear semantics mismatch",
        covers: &[
            "public multiply is reachable",
            "public divide is reachable",
            "public remainder is reachable",
            "public power is reachable",
            "public unknown negative exponent is reachable",
            "public zero to zero is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "multiplication",
        harness: "native_multiplication_matches_reference_semantics",
        message: "multiplication semantics mismatch",
        covers: &[
            "one-bit multiplication is reachable",
            "eight-bit multiplication is reachable",
            "signed multiplication is reachable",
            "unsigned multiplication is reachable",
            "zero multiplication is reachable",
            "identity multiplication is reachable",
            "multiplication carry truncation is reachable",
            "multiplication unknown LHS is reachable",
            "multiplication unknown RHS is reachable",
            "multiplication source Z is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "division-remainder",
        harness: "native_division_remainder_matches_reference_semantics",
        message: "division remainder semantics mismatch",
        covers: &[
            "division is reachable",
            "remainder is reachable",
            "one-bit division is reachable",
            "eight-bit division is reachable",
            "signed division is reachable",
            "unsigned division is reachable",
            "zero divisor is reachable",
            "division unknown LHS is reachable",
            "division unknown RHS is reachable",
            "negative quotient is reachable",
            "negative remainder is reachable",
            "small minimum divided by minus one is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "power",
        harness: "native_power_matches_reference_semantics",
        message: "power semantics mismatch",
        covers: &[
            "one-bit power is reachable",
            "eight-bit power is reachable",
            "one-bit exponent is reachable",
            "three-bit exponent is reachable",
            "zero exponent is reachable",
            "positive exponent seven is reachable",
            "negative odd exponent is reachable",
            "negative even exponent is reachable",
            "zero to negative power is reachable",
            "minus one negative odd power is reachable",
            "minus one negative even power is reachable",
            "power source X is reachable",
            "power source Z is reachable",
            "unknown exponent is reachable",
            "unknown negative exponent is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "multiplication-identities",
        harness: "native_multiplication_identities_full_width",
        message: "multiplication identity semantics mismatch",
        covers: &[
            "native identity width 64 is reachable",
            "native multiplication by zero is reachable",
            "native multiplication by one is reachable",
            "unknown times zero is reachable",
            "native multiplication RHS unknown is reachable",
            "native multiplication Z is reachable",
            "native signed multiplication identity is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "division-exceptions",
        harness: "native_division_exceptional_full_width",
        message: "division exception semantics mismatch",
        covers: &[
            "native division width 64 is reachable",
            "native zero divisor is reachable",
            "native divisor one is reachable",
            "native divisor minus one is reachable",
            "native division overflow is reachable",
            "native remainder overflow is reachable",
            "native division LHS unknown is reachable",
            "native division RHS unknown is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "power-exceptions",
        harness: "native_power_exceptional_full_width",
        message: "power exception semantics mismatch",
        covers: &[
            "native power width 64 is reachable",
            "native exponent width 64 is reachable",
            "native zero to zero is reachable",
            "native zero negative power is reachable",
            "native one negative power is reachable",
            "native minus one odd power is reachable",
            "native minus one even power is reachable",
            "native other negative power is reachable",
            "native unknown base is reachable",
            "native unknown exponent is reachable",
            "native minimum exponent is reachable",
            "native unknown negative exponent is reachable",
        ],
    },
    Contract {
        group: "layout",
        name: "shape-total",
        harness: "shape_total_matches_reference_semantics",
        message: "shape total semantics mismatch",
        covers: &[
            "zero first extent is reachable",
            "zero final extent is reachable",
            "unit shape total is reachable",
            "maximum first extent is reachable",
            "maximum final extent is reachable",
            "shape product overflow is reachable",
            "largest doubled fitting shape is reachable",
            "first doubled overflowing shape is reachable",
            "nontrivial shape product is reachable",
        ],
    },
    Contract {
        group: "layout",
        name: "shape-index",
        harness: "shape_index_matches_reference_semantics",
        message: "shape index semantics mismatch",
        covers: &[
            "scalar index shape is reachable",
            "extent-four final index is reachable",
            "unit scalar coercion is reachable",
            "three-dimensional last element is reachable",
            "missing coordinate is reachable",
            "excess coordinates are reachable",
            "unknown index extent is reachable",
            "zero index extent is reachable",
            "inner coordinate spill is reachable",
            "outer coordinate spill is reachable",
            "row-major first element is reachable",
            "row-major second row is reachable",
            "unknown alongside zero extent is reachable",
        ],
    },
    Contract {
        group: "layout",
        name: "shape-range",
        harness: "shape_range_matches_reference_semantics",
        message: "shape range semantics mismatch",
        covers: &[
            "scalar range is reachable",
            "extent-four prefix block is reachable",
            "unit whole range is reachable",
            "three-dimensional whole range is reachable",
            "outer prefix block is reachable",
            "inner prefix block is reachable",
            "full coordinate singleton is reachable",
            "unknown range extent is reachable",
            "empty range is reachable",
            "invalid prefix is reachable",
            "excess prefix is reachable",
        ],
    },
    Contract {
        group: "layout",
        name: "shape-boundary",
        harness: "shape_native_boundary_matches_reference_semantics",
        message: "shape native boundary semantics mismatch",
        covers: &[
            "native empty extent is reachable",
            "native unit coordinate is reachable",
            "maximum valid native coordinate is reachable",
            "maximum invalid native coordinate is reachable",
            "maximum native whole range is reachable",
            "native excess prefix is reachable",
            "native empty whole range is reachable",
            "native unresolved extent is reachable",
        ],
    },
    Contract {
        group: "layout",
        name: "shape-overflow",
        harness: "shape_overflow_rejected",
        message: "shape overflow semantics mismatch",
        covers: &[
            "maximum overflow extent is reachable",
            "first native overflow extent is reachable",
            "overflow whole selection is reachable",
            "overflow partial selection is reachable",
            "overflow full selection is reachable",
        ],
    },
    Contract {
        group: "coordinates",
        name: "selection-endpoints",
        // Kani filters by substring; qualify this name to exclude the checked harness.
        harness: "coordinates::selection_endpoints_match_reference_semantics",
        message: "selection endpoint semantics mismatch",
        covers: &[
            "colon endpoint is reachable",
            "plus endpoint is reachable",
            "minus endpoint is reachable",
            "step endpoint is reachable",
            "maximum plus unit endpoint is reachable",
            "maximum minus unit endpoint is reachable",
            "maximum minus width is reachable",
            "minus endpoint at zero is reachable",
            "minus endpoint underflow is reachable",
            "step endpoint overflow is reachable",
            "zero endpoint width is reachable",
            "array ascending endpoint is reachable",
            "packed descending endpoint is reachable",
        ],
    },
    Contract {
        group: "coordinates",
        name: "packed-unpacked-bits",
        harness: "packed_unpacked_bits_match_reference_semantics",
        message: "packed unpacked size semantics mismatch",
        covers: &[
            "zero packed size is reachable",
            "zero unpacked size is reachable",
            "maximum packed size is reachable",
            "maximum unpacked size is reachable",
            "combined size overflow is reachable",
            "nontrivial combined size is reachable",
        ],
    },
    Contract {
        group: "coordinates",
        name: "checked-selection-endpoints",
        harness: "coordinates::checked_selection_endpoints_match_reference_semantics",
        message: "checked selection endpoint semantics mismatch",
        covers: &[
            "checked colon is reachable",
            "checked plus is reachable",
            "checked minus is reachable",
            "checked step is reachable",
            "checked zero width is reachable",
            "checked maximum plus unit is reachable",
            "checked maximum minus unit is reachable",
            "checked maximum step unit is reachable",
            "checked plus overflow is reachable",
            "checked minus underflow is reachable",
            "checked step overflow is reachable",
            "checked fitting step boundary is reachable",
            "checked full colon endpoints are reachable",
            "checked array order is reachable",
            "checked packed order is reachable",
        ],
    },
];

fn contract(name: &str) -> &'static Contract {
    CONTRACTS
        .iter()
        .find(|contract| contract.name == name)
        .expect("unknown contract")
}

fn replace_once(source: &str, old: &str, new: &str) -> String {
    assert_eq!(
        source.matches(old).count(),
        1,
        "mutation site changed: {old}"
    );
    source.replacen(old, new, 1)
}

fn method_range(source: &str, name: &str) -> std::ops::Range<usize> {
    let start = source
        .find(&format!("    pub fn {name}("))
        .unwrap_or_else(|| panic!("missing mutation method: {name}"));
    let rest = &source[start..];
    let next_method = rest[1..].find("\n    pub fn ").map(|offset| offset + 1);
    let impl_end = rest.find("\n}").expect("missing impl end");
    start..start + next_method.unwrap_or(impl_end).min(impl_end)
}

fn mutate_method(source: &str, name: &str, old: &str, new: &str) -> String {
    let range = method_range(source, name);
    let replacement = replace_once(&source[range.clone()], old, new);
    let mut mutated = source.to_owned();
    mutated.replace_range(range, &replacement);
    mutated
}

pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let mut mutations = Vec::new();
    if selected.iter().any(|c| c.group == "values") {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        let legacy = mutate_method(
            &source,
            "assign",
            "beg < end || end >= self.width as usize || end >= 64",
            "beg < end || end >= 64",
        );
        // Retain a native-width clamp so arbitrary endpoints expose the width
        // violation without also overflowing gen_mask_range's beg + 1.
        let legacy = mutate_method(
            &legacy,
            "assign",
            "        let beg = beg.min(self.width as usize - 1);\n",
            "        let beg = beg.min(63);\n",
        );
        let leaking = mutate_method(
            &source,
            "assign",
            "(value.payload & mask_range)",
            "(value.payload & mask)",
        );
        let leaking = mutate_method(
            &leaking,
            "assign",
            "(value.mask_xz & mask_range)",
            "(value.mask_xz & mask)",
        );
        let payload_truncation =
            mutate_method(&source, "trunc", "        self.payload &= mask;\n", "");
        let xz_truncation = mutate_method(&source, "trunc", "        self.mask_xz &= mask;\n", "");
        let zero_sign = mutate_method(
            &source,
            "expand",
            "if self.signed && use_sign {",
            "if false {",
        );
        let ignored_use_sign = mutate_method(
            &source,
            "expand",
            "if self.signed && use_sign {",
            "if self.signed {",
        );
        let lost_xz = mutate_method(&source, "expand", "mask_xz |= mask;", "");
        let changed_noop = mutate_method(
            &source,
            "expand",
            "return self.clone();",
            "return Self { signed: false, ..self.clone() };",
        );
        let missing_unknown = mutate_method(&source, "select", "mask_xz: mask,", "mask_xz: 0,");
        let missing_partial_unknown = mutate_method(&source, "select", "(mask ^ valid_mask)", "0");
        let wrong_offset = mutate_method(
            &source,
            "select",
            "ret.payload >>= end;",
            "ret.payload >>= 0;",
        );
        let signed_selection = mutate_method(
            &source,
            "select",
            "ret.signed = false;",
            "ret.signed = self.signed;",
        );
        let reversed_left_shift = mutate_method(
            &source,
            "shift_left",
            "ret.payload.unbounded_shl(s)",
            "ret.payload.unbounded_shr(s)",
        );
        let unmasked_left_payload =
            mutate_method(&source, "shift_left", "        ret.payload &= mask;\n", "");
        let unmasked_left_xz =
            mutate_method(&source, "shift_left", "        ret.mask_xz &= mask;\n", "");
        let zero_for_unknown_left_shift = mutate_method(
            &source,
            "shift_left",
            "return Self::new_x(width, false);",
            "return Self::new(0, width, false);",
        );
        let reversed_right_shift = mutate_method(
            &source,
            "shift_right",
            "ret.payload.unbounded_shr(s)",
            "ret.payload.unbounded_shl(s)",
        );
        let clamped_right_shift_63 =
            mutate_method(&source, "shift_right", "amount.min(64)", "amount.min(63)");
        let missing_right_sign_fill = mutate_method(
            &source,
            "shift_right",
            "if arithmetic && signed {",
            "if false {",
        );
        let missing_right_sign_xz = mutate_method(
            &source,
            "shift_right",
            "        ret.mask_xz |= ext_mask_xz;\n",
            "",
        );
        let unsigned_arithmetic_left =
            mutate_method(&source, "shift_left", "if !arithmetic {", "if true {");
        let signed_fill_on_unsigned_right = mutate_method(
            &source,
            "shift_right",
            "if arithmetic && signed {",
            "if arithmetic {",
        );
        let legacy_unknown_equality = mutate_method(
            &source,
            "equal",
            "let mismatch = (self.payload ^ rhs.payload) & known_mask != 0;",
            "let mismatch = (self.payload & !self.mask_xz) != (rhs.payload & !rhs.mask_xz);",
        );
        let legacy_unknown_inequality = mutate_method(
            &source,
            "equal",
            "let mismatch = (self.payload ^ rhs.payload) & known_mask != 0;",
            "let mismatch = (self.payload & !self.mask_xz) != (rhs.payload & !rhs.mask_xz);",
        );
        let unknown_before_known_mismatch = mutate_method(
            &source,
            "equal",
            "Self::new_bit_0x(mismatch, unknown)",
            "Self::new_bit_x0(unknown, mismatch)",
        );
        let lost_inequality_negation =
            mutate_method(&source, "equal", "if negated {", "if false {");
        let ignored_rhs_wildcard = mutate_method(&source, "equal", "if wildcard {", "if false {");
        let ignored_lhs_unknown = mutate_method(
            &source,
            "equal",
            "let unknown = (self.mask_xz | rhs.mask_xz) & compare_mask != 0;",
            "let unknown = false;",
        );
        let unknown_wildcard_mismatch = mutate_method(
            &source,
            "equal",
            "let known_mask = compare_mask & !self.mask_xz & !rhs.mask_xz;",
            "let known_mask = compare_mask;",
        );
        let unsigned_signed_comparison = mutate_method(
            &source,
            "compare",
            "let order = if signed {",
            "let order = if false {",
        );
        let reversed_unsigned_comparison = mutate_method(
            &source,
            "compare",
            "self.payload.cmp(&rhs.payload)",
            "rhs.payload.cmp(&self.payload)",
        );
        let strict_less_equal =
            mutate_method(&source, "compare", "(inclusive && order.is_eq())", "false");
        let strict_greater_equal =
            mutate_method(&source, "compare", "(inclusive && order.is_eq())", "false");
        let ignored_relation_unknown =
            mutate_method(&source, "compare", "self.is_xz() || rhs.is_xz()", "false");
        let and_as_or = mutate_method(
            &source,
            "bit_and",
            "self.payload & rhs.payload",
            "self.payload | rhs.payload",
        );
        let and_loses_zero_dominance = mutate_method(
            &source,
            "bit_and",
            "self.mask_xz & !rhs.mask_xz & rhs.payload",
            "self.mask_xz & !rhs.mask_xz",
        );
        let or_loses_one_dominance = mutate_method(
            &source,
            "bit_or",
            "self.mask_xz & !rhs.mask_xz & !rhs.payload",
            "self.mask_xz & !rhs.mask_xz",
        );
        let xor_loses_unknown = mutate_method(
            &source,
            "bit_xor",
            "self.mask_xz | rhs.mask_xz",
            "self.mask_xz & rhs.mask_xz",
        );
        let xnor_as_xor = mutate_method(&source, "bit_xnor", " ^ Self::gen_mask(width)", "");
        let bitwise_leaks_z = mutate_method(
            &source,
            "bit_xor",
            "        ret.payload &= !ret.mask_xz;\n",
            "",
        );
        let signed_bitwise_result = mutate_method(&source, "bit_or", "width, false", "width, true");
        let complement_leaks_z = mutate_method(
            &source,
            "bit_not",
            "        ret.payload &= ret.mask_xz ^ mask;\n",
            "",
        );
        let complement_loses_sign = mutate_method(
            &source,
            "bit_not",
            "let mut ret = self.clone();",
            "let mut ret = Self { signed: false, ..self.clone() };",
        );
        let reduction_and_unknown_first = mutate_method(
            &source,
            "reduce_and",
            "Self::new_bit_0x(known_zero, self.is_xz())",
            "Self::new_bit_x0(self.is_xz(), known_zero)",
        );
        let reduction_nand_as_and =
            mutate_method(&source, "reduce_and", "if negated {", "if false {");
        let reduction_or_unknown_first = mutate_method(
            &source,
            "reduce_or",
            "Self::new_bit_1x(known_one, self.is_xz())",
            "Self::new_bit_x1(self.is_xz(), known_one)",
        );
        let reduction_nor_as_or = mutate_method(
            &source,
            "reduce_or",
            "Self::new_bit_0x(known_one, self.is_xz())",
            "Self::new_bit_1x(known_one, self.is_xz())",
        );
        let reduction_xor_ignores_unknown =
            mutate_method(&source, "reduce_xor", "if self.is_xz() {", "if false {");
        let reduction_inverts_parity =
            mutate_method(&source, "reduce_xor", "odd != negated", "odd == negated");
        let logical_and_ignores_zero = mutate_method(
            &source,
            "logical_binary",
            "!known_zero && unknown",
            "unknown",
        );
        let logical_or_as_and = mutate_method(
            &source,
            "logical_binary",
            "lhs_one || rhs_one",
            "lhs_one && rhs_one",
        );
        let logical_z_as_one = mutate_method(
            &source,
            "logical_binary",
            "let lhs_one = self.payload & !self.mask_xz != 0;",
            "let lhs_one = self.payload != 0;",
        );
        let logical_ignores_unknown = mutate_method(
            &source,
            "logical_binary",
            "let unknown = self.is_xz() || rhs.is_xz();",
            "let unknown = false;",
        );
        let condition_loses_known_one = mutate_method(
            &source,
            "truth",
            "if self.payload & !self.mask_xz != 0 {",
            "if self.mask_xz == 0 && self.payload & !self.mask_xz != 0 {",
        );
        let condition_z_as_true = mutate_method(
            &source,
            "truth",
            "self.payload & !self.mask_xz",
            "self.payload",
        );
        let condition_unknown_as_false = mutate_method(
            &source,
            "truth",
            "            None\n",
            "            Some(false)\n",
        );
        let reversed_conditional_true_arm = mutate_method(
            &source,
            "conditional",
            "Some(true) => on_true,",
            "Some(true) => on_false,",
        );
        let conditional_ignores_xz_disagreement = mutate_method(
            &source,
            "conditional",
            "(on_true.mask_xz | on_false.mask_xz)",
            "0",
        );
        let conditional_ignores_payload_disagreement = mutate_method(
            &source,
            "conditional",
            "(on_true.payload ^ on_false.payload)",
            "0",
        );
        let conditional_payload_leaks = mutate_method(
            &source,
            "conditional",
            "payload: on_true.payload & same,",
            "payload: on_true.payload,",
        );
        let conditional_loses_disagreement_x =
            mutate_method(&source, "conditional", "(mask ^ same)", "0");
        let conditional_preserves_common_z = mutate_method(
            &source,
            "conditional",
            "(on_true.mask_xz | on_false.mask_xz)",
            "(on_true.mask_xz ^ on_false.mask_xz)",
        );
        let conditional_loses_signedness =
            mutate_method(&source, "conditional", "        ret.signed = signed;\n", "");
        let conditional_ignores_context_width = mutate_method(
            &source,
            "conditional",
            "let width = context_width",
            "let width = 0usize",
        );
        let conditional_ignores_unselected_width =
            mutate_method(&source, "conditional", ".max(otherwise.width as usize)", "");
        let conditional_disabled_sign_fill = mutate_method(
            &source,
            "conditional",
            "self.expand(width, signed)",
            "self.expand(width, false)",
        );
        let conditional_forced_sign_fill = mutate_method(
            &source,
            "conditional",
            "self.expand(width, signed)",
            "self.expand(width, true)",
        );
        let legacy_conditional_false = mutate_method(
            &source,
            "conditional",
            "payload: on_true.payload & same,",
            "payload: on_false.payload,",
        );
        let legacy_conditional_false = mutate_method(
            &legacy_conditional_false,
            "conditional",
            "mask_xz: (on_true.mask_xz & same) | (mask ^ same),",
            "mask_xz: on_false.mask_xz,",
        );
        let public_conditional_ignores_context_width = replace_once(
            &source,
            "return Self::U64(a.conditional(b, condition, context_width, signed));",
            "return Self::U64(a.conditional(b, condition, 1, signed));",
        );
        let public_conditional_truncates_word = replace_once(
            &source,
            "return Self::U64(a.conditional(b, condition, context_width, signed));",
            "let mut value = Self::U64(a.conditional(b, condition, context_width, signed)); value.trunc(32); return value;",
        );
        let addition_as_subtraction = mutate_method(
            &source,
            "add_sub",
            "self.payload.wrapping_add(rhs.payload)",
            "self.payload.wrapping_sub(rhs.payload)",
        );
        let subtraction_as_addition = mutate_method(
            &source,
            "add_sub",
            "self.payload.wrapping_sub(rhs.payload)",
            "self.payload.wrapping_add(rhs.payload)",
        );
        let unmasked_arithmetic_result = mutate_method(
            &source,
            "add_sub",
            "payload & Self::gen_mask(width)",
            "payload",
        );
        let arithmetic_ignores_lhs_unknown = mutate_method(
            &source,
            "add_sub",
            "self.is_xz() | rhs.is_xz()",
            "rhs.is_xz()",
        );
        let arithmetic_ignores_rhs_unknown = mutate_method(
            &source,
            "add_sub",
            "self.is_xz() | rhs.is_xz()",
            "self.is_xz()",
        );
        let arithmetic_loses_signedness = mutate_method(
            &source,
            "add_sub",
            "Self::new(payload & Self::gen_mask(width), width, signed)",
            "Self::new(payload & Self::gen_mask(width), width, false)",
        );
        let arithmetic_overflow_to_zero = mutate_method(
            &source,
            "add_sub",
            "self.payload.wrapping_add(rhs.payload)",
            "self.payload.checked_add(rhs.payload).unwrap_or(0)",
        );
        let arithmetic_underflow_to_zero = mutate_method(
            &source,
            "add_sub",
            "self.payload.wrapping_sub(rhs.payload)",
            "self.payload.checked_sub(rhs.payload).unwrap_or(0)",
        );
        let negation_missing_carry =
            mutate_method(&source, "negate", "wrapping_add(1)", "wrapping_add(0)");
        let unmasked_negation =
            mutate_method(&source, "negate", "        ret.payload &= mask;\n", "");
        let negation_ignores_unknown =
            mutate_method(&source, "negate", "if self.is_xz() {", "if false {");
        let negation_loses_signedness = mutate_method(
            &source,
            "negate",
            "let mut ret = self.clone();",
            "let mut ret = Self { signed: false, ..self.clone() };",
        );
        let unary_plus_loses_unknown = mutate_method(
            &source,
            "expand",
            "return self.clone();",
            "return Self { mask_xz: 0, ..self.clone() };",
        );
        let unary_plus_loses_extended_z = mutate_method(&source, "expand", "mask_xz |= mask;", "");
        for (name, contract, source) in [
            (
                "missing-unknown-selection",
                contract("selection"),
                missing_unknown,
            ),
            (
                "missing-partial-unknown-selection",
                contract("selection"),
                missing_partial_unknown,
            ),
            (
                "wrong-selection-offset",
                contract("selection"),
                wrong_offset,
            ),
            ("signed-selection", contract("selection"), signed_selection),
            (
                "legacy-out-of-bounds",
                contract("partial-assignment"),
                legacy,
            ),
            (
                "neighbour-bit-leak",
                contract("partial-assignment"),
                leaking,
            ),
            (
                "untruncated-payload",
                contract("truncation"),
                payload_truncation,
            ),
            ("untruncated-xz", contract("truncation"), xz_truncation),
            ("zero-instead-of-sign", contract("extension"), zero_sign),
            ("ignored-use-sign", contract("extension"), ignored_use_sign),
            ("lost-sign-xz", contract("extension"), lost_xz),
            (
                "changed-noop-signedness",
                contract("extension"),
                changed_noop,
            ),
            (
                "reversed-left-shift",
                contract("logical-left-shift"),
                reversed_left_shift,
            ),
            (
                "unmasked-left-payload",
                contract("logical-left-shift"),
                unmasked_left_payload,
            ),
            (
                "unmasked-left-xz",
                contract("logical-left-shift"),
                unmasked_left_xz,
            ),
            (
                "zero-for-unknown-left-shift",
                contract("logical-left-shift"),
                zero_for_unknown_left_shift,
            ),
            (
                "reversed-right-shift",
                contract("logical-right-shift"),
                reversed_right_shift,
            ),
            (
                "wrapped-right-shift-64",
                contract("logical-right-shift"),
                clamped_right_shift_63,
            ),
            (
                "missing-right-sign-fill",
                contract("arithmetic-right-shift"),
                missing_right_sign_fill,
            ),
            (
                "missing-right-sign-xz",
                contract("arithmetic-right-shift"),
                missing_right_sign_xz,
            ),
            (
                "unsigned-arithmetic-left",
                contract("arithmetic-left-shift"),
                unsigned_arithmetic_left,
            ),
            (
                "signed-fill-on-unsigned-right",
                contract("arithmetic-right-shift"),
                signed_fill_on_unsigned_right,
            ),
            (
                "legacy-unknown-equality",
                contract("equality"),
                legacy_unknown_equality,
            ),
            (
                "legacy-unknown-inequality",
                contract("inequality"),
                legacy_unknown_inequality,
            ),
            (
                "unknown-before-known-mismatch",
                contract("equality"),
                unknown_before_known_mismatch,
            ),
            (
                "lost-inequality-negation",
                contract("inequality"),
                lost_inequality_negation,
            ),
            (
                "ignored-RHS-wildcard",
                contract("wildcard-equality"),
                ignored_rhs_wildcard,
            ),
            (
                "ignored-LHS-unknown",
                contract("wildcard-inequality"),
                ignored_lhs_unknown,
            ),
            (
                "unknown-wildcard-mismatch",
                contract("wildcard-inequality"),
                unknown_wildcard_mismatch,
            ),
            (
                "unsigned-signed-comparison",
                contract("less-than"),
                unsigned_signed_comparison,
            ),
            (
                "reversed-unsigned-comparison",
                contract("greater-than"),
                reversed_unsigned_comparison,
            ),
            (
                "strict-less-equal",
                contract("less-equal"),
                strict_less_equal,
            ),
            (
                "strict-greater-equal",
                contract("greater-equal"),
                strict_greater_equal,
            ),
            (
                "ignored-relation-unknown",
                contract("greater-equal"),
                ignored_relation_unknown,
            ),
            ("and-as-or", contract("bitwise"), and_as_or),
            (
                "and-loses-zero-dominance",
                contract("bitwise"),
                and_loses_zero_dominance,
            ),
            (
                "or-loses-one-dominance",
                contract("bitwise"),
                or_loses_one_dominance,
            ),
            ("xor-loses-unknown", contract("bitwise"), xor_loses_unknown),
            ("xnor-as-xor", contract("bitwise"), xnor_as_xor),
            ("bitwise-leaks-Z", contract("bitwise"), bitwise_leaks_z),
            (
                "signed-bitwise-result",
                contract("bitwise"),
                signed_bitwise_result,
            ),
            (
                "complement-leaks-Z",
                contract("bit-not"),
                complement_leaks_z,
            ),
            (
                "complement-loses-sign",
                contract("bit-not"),
                complement_loses_sign,
            ),
            (
                "reduction-and-unknown-first",
                contract("reductions"),
                reduction_and_unknown_first,
            ),
            (
                "reduction-nand-as-and",
                contract("reductions"),
                reduction_nand_as_and,
            ),
            (
                "reduction-or-unknown-first",
                contract("reductions"),
                reduction_or_unknown_first,
            ),
            (
                "reduction-nor-as-or",
                contract("reductions"),
                reduction_nor_as_or,
            ),
            (
                "reduction-xor-ignores-unknown",
                contract("reductions"),
                reduction_xor_ignores_unknown,
            ),
            (
                "reduction-inverts-parity",
                contract("reductions"),
                reduction_inverts_parity,
            ),
            (
                "logical-and-ignores-zero",
                contract("logical"),
                logical_and_ignores_zero,
            ),
            ("logical-or-as-and", contract("logical"), logical_or_as_and),
            ("logical-Z-as-one", contract("logical"), logical_z_as_one),
            (
                "logical-ignores-unknown",
                contract("logical"),
                logical_ignores_unknown,
            ),
            (
                "condition-loses-known-one",
                contract("condition-truth"),
                condition_loses_known_one,
            ),
            (
                "condition-Z-as-true",
                contract("condition-truth"),
                condition_z_as_true,
            ),
            (
                "condition-unknown-as-false",
                contract("condition-truth"),
                condition_unknown_as_false,
            ),
            (
                "reversed-conditional-true-arm",
                contract("conditional"),
                reversed_conditional_true_arm,
            ),
            (
                "conditional-ignores-XZ-disagreement",
                contract("conditional"),
                conditional_ignores_xz_disagreement,
            ),
            (
                "conditional-ignores-payload-disagreement",
                contract("conditional"),
                conditional_ignores_payload_disagreement,
            ),
            (
                "conditional-payload-leaks",
                contract("conditional"),
                conditional_payload_leaks,
            ),
            (
                "conditional-loses-disagreement-X",
                contract("conditional"),
                conditional_loses_disagreement_x,
            ),
            (
                "conditional-preserves-common-Z",
                contract("conditional"),
                conditional_preserves_common_z,
            ),
            (
                "conditional-loses-signedness",
                contract("conditional"),
                conditional_loses_signedness,
            ),
            (
                "conditional-ignores-context-width",
                contract("conditional"),
                conditional_ignores_context_width,
            ),
            (
                "conditional-ignores-unselected-width",
                contract("conditional"),
                conditional_ignores_unselected_width,
            ),
            (
                "conditional-disabled-sign-fill",
                contract("conditional"),
                conditional_disabled_sign_fill,
            ),
            (
                "conditional-forced-sign-fill",
                contract("conditional"),
                conditional_forced_sign_fill,
            ),
            (
                "legacy-conditional-false",
                contract("public-conditional"),
                legacy_conditional_false,
            ),
            (
                "public-conditional-ignores-context-width",
                contract("public-conditional-widened"),
                public_conditional_ignores_context_width,
            ),
            (
                "public-conditional-truncates-word",
                contract("public-conditional-word"),
                public_conditional_truncates_word,
            ),
            (
                "addition-as-subtraction",
                contract("add-sub"),
                addition_as_subtraction,
            ),
            (
                "subtraction-as-addition",
                contract("add-sub"),
                subtraction_as_addition,
            ),
            (
                "unmasked-arithmetic-result",
                contract("add-sub"),
                unmasked_arithmetic_result,
            ),
            (
                "arithmetic-ignores-LHS-unknown",
                contract("add-sub"),
                arithmetic_ignores_lhs_unknown,
            ),
            (
                "arithmetic-ignores-RHS-unknown",
                contract("add-sub"),
                arithmetic_ignores_rhs_unknown,
            ),
            (
                "arithmetic-loses-signedness",
                contract("add-sub"),
                arithmetic_loses_signedness,
            ),
            (
                "arithmetic-overflow-to-zero",
                contract("add-sub"),
                arithmetic_overflow_to_zero,
            ),
            (
                "arithmetic-underflow-to-zero",
                contract("add-sub"),
                arithmetic_underflow_to_zero,
            ),
            (
                "negation-missing-carry",
                contract("negation"),
                negation_missing_carry,
            ),
            ("unmasked-negation", contract("negation"), unmasked_negation),
            (
                "negation-ignores-unknown",
                contract("public-arithmetic-negate"),
                negation_ignores_unknown,
            ),
            (
                "negation-loses-signedness",
                contract("negation"),
                negation_loses_signedness,
            ),
            (
                "unary-plus-loses-unknown",
                contract("unary-plus"),
                unary_plus_loses_unknown,
            ),
            (
                "unary-plus-loses-extended-Z",
                contract("unary-plus"),
                unary_plus_loses_extended_z,
            ),
        ] {
            if selected.iter().any(|c| c.name == contract.name) {
                mutations.push(Mutation {
                    name,
                    contract,
                    path: CORE,
                    source,
                });
            }
        }
        for (name, contract_name, path, method, old, new) in [
            (
                "unary-plus-poisons-unknown",
                "public-arithmetic-plus",
                "veryl/crates/analyzer/src/ir/op.rs",
                "eval_value_unary",
                "Op::Add => {\n                let x = x.expand(width, signed);\n                x.into_owned()\n            }",
                "Op::Add => {\n                let x = x.expand(width, signed);\n                if x.is_xz() { Value::new_x(x.width(), x.signed()) } else { x.into_owned() }\n            }",
            ),
            (
                "merge-loses-wider-width",
                "context-merge",
                "veryl/crates/analyzer/src/ir/comptime.rs",
                "merge",
                "self.width.max(other.width)",
                "self.width.min(other.width)",
            ),
            (
                "merge-keeps-mixed-sign",
                "context-merge",
                "veryl/crates/analyzer/src/ir/comptime.rs",
                "merge",
                "self.signed & other.signed",
                "self.signed | other.signed",
            ),
            (
                "merge-keeps-runtime-const",
                "context-merge",
                "veryl/crates/analyzer/src/ir/comptime.rs",
                "merge",
                "self.is_const & other.is_const",
                "self.is_const | other.is_const",
            ),
            (
                "merge-keeps-local-global",
                "context-merge",
                "veryl/crates/analyzer/src/ir/comptime.rs",
                "merge",
                "self.is_global & other.is_global",
                "self.is_global | other.is_global",
            ),
            (
                "unary-loses-operand-width",
                "context-unary",
                "veryl/crates/analyzer/src/ir/op.rs",
                "eval_context_unary",
                "Op::Add | Op::Sub | Op::BitNot => x,",
                "Op::Add | Op::Sub | Op::BitNot => ExpressionContext { width: 1, ..x },",
            ),
            (
                "unary-predicate-wrong-width",
                "context-unary",
                "veryl/crates/analyzer/src/ir/op.rs",
                "eval_context_unary",
                "width: 1,",
                "width: 2,",
            ),
            (
                "unary-propagates-into-reduction",
                "context-unary",
                "veryl/crates/analyzer/src/ir/op.rs",
                "unary_x_self_determined",
                "matches!(",
                "false && matches!(",
            ),
            (
                "binary-loses-right-width",
                "context-binary",
                "veryl/crates/analyzer/src/ir/op.rs",
                "eval_context_binary",
                "width: x.width.max(y.width),",
                "width: x.width,",
            ),
            (
                "binary-keeps-mixed-sign",
                "context-binary",
                "veryl/crates/analyzer/src/ir/op.rs",
                "eval_context_binary",
                "signed: x.signed & y.signed,",
                "signed: x.signed | y.signed,",
            ),
            (
                "binary-predicate-wrong-width",
                "context-binary",
                "veryl/crates/analyzer/src/ir/op.rs",
                "eval_context_binary",
                "width: 1,",
                "width: 2,",
            ),
            (
                "shift-uses-amount-width",
                "context-binary",
                "veryl/crates/analyzer/src/ir/op.rs",
                "eval_context_binary",
                "width: x.width,",
                "width: y.width,",
            ),
            (
                "cast-loses-width-fallback",
                "context-binary",
                "veryl/crates/analyzer/src/ir/op.rs",
                "eval_context_binary",
                "if y.width > 0 { y.width } else { x.width }",
                "if y.width > 0 { y.width } else { 0 }",
            ),
            (
                "logical-propagates-left",
                "context-binary",
                "veryl/crates/analyzer/src/ir/op.rs",
                "binary_x_self_determined",
                "matches!(",
                "false && matches!(",
            ),
            (
                "shift-propagates-amount",
                "context-binary",
                "veryl/crates/analyzer/src/ir/op.rs",
                "binary_y_self_determined",
                "matches!(",
                "false && matches!(",
            ),
            (
                "comparison-inherits-outer-context",
                "context-binary",
                "veryl/crates/analyzer/src/ir/op.rs",
                "binary_op_self_determined",
                "matches!(",
                "false && matches!(",
            ),
            (
                "cast-narrows-inner-context",
                "context-cast",
                "veryl/crates/analyzer/src/ir/comptime.rs",
                "cast_operand",
                "self.width.max(width)",
                "width",
            ),
            (
                "numeric-cast-uses-target-sign",
                "context-cast",
                "veryl/crates/analyzer/src/ir/op.rs",
                "eval_context_cast",
                "if !target_is_type {",
                "if false {",
            ),
            (
                "type-cast-keeps-source-sign",
                "context-cast",
                "veryl/crates/analyzer/src/ir/op.rs",
                "eval_context_cast",
                "if !target_is_type {",
                "if true {",
            ),
        ] {
            let contract = contract(contract_name);
            if !selected.iter().any(|c| c.name == contract.name) {
                continue;
            }
            let original = fs::read_to_string(repo.join(path)).unwrap();
            let source = mutate_method(&original, method, old, new);
            mutations.push(Mutation {
                name,
                contract,
                path,
                source,
            });
        }
        for (name, contract_name, method, old, new) in [
            (
                "multiply-as-addition",
                "multiplication",
                "multiply",
                "self.payload.wrapping_mul(rhs.payload)",
                "self.payload.wrapping_add(rhs.payload)",
            ),
            (
                "unmasked-multiplication",
                "multiplication",
                "multiply",
                "self.payload.wrapping_mul(rhs.payload) & Self::gen_mask(width)",
                "self.payload.wrapping_mul(rhs.payload)",
            ),
            (
                "multiply-ignores-LHS-unknown",
                "multiplication-identities",
                "multiply",
                "self.is_xz() || rhs.is_xz()",
                "rhs.is_xz()",
            ),
            (
                "multiply-ignores-RHS-unknown",
                "multiplication-identities",
                "multiply",
                "self.is_xz() || rhs.is_xz()",
                "self.is_xz()",
            ),
            (
                "multiply-loses-signedness",
                "multiplication",
                "multiply",
                "                signed,",
                "                false,",
            ),
            (
                "unsigned-division-as-remainder",
                "division-remainder",
                "divide_remainder",
                "self.payload / rhs.payload",
                "self.payload % rhs.payload",
            ),
            (
                "unsigned-remainder-as-division",
                "division-remainder",
                "divide_remainder",
                "self.payload % rhs.payload",
                "self.payload / rhs.payload",
            ),
            (
                "division-ignores-signed-mode",
                "division-remainder",
                "divide_remainder",
                "if signed {",
                "if false {",
            ),
            (
                "division-zero-to-zero",
                "division-exceptions",
                "divide_remainder",
                "Self::new_x(width, signed)",
                "if !self.is_xz() && !rhs.is_xz() && rhs.payload & mask == 0 { Self::new(0, width, signed) } else { Self::new_x(width, signed) }",
            ),
            (
                "division-ignores-LHS-unknown",
                "division-exceptions",
                "divide_remainder",
                "self.is_xz() || rhs.is_xz()",
                "rhs.is_xz()",
            ),
            (
                "division-ignores-RHS-unknown",
                "division-exceptions",
                "divide_remainder",
                "self.is_xz() || rhs.is_xz()",
                "self.is_xz()",
            ),
            (
                "division-overflow-to-zero",
                "division-exceptions",
                "divide_remainder",
                "checked_div(rhs).unwrap_or(lhs)",
                "checked_div(rhs).unwrap_or(0)",
            ),
            (
                "remainder-overflow-to-dividend",
                "division-exceptions",
                "divide_remainder",
                "checked_rem(rhs).unwrap_or(0)",
                "checked_rem(rhs).unwrap_or(lhs)",
            ),
            (
                "unmasked-division-result",
                "division-remainder",
                "divide_remainder",
                "Self::new(payload & mask, width, signed)",
                "Self::new(payload, width, signed)",
            ),
            (
                // Keep the exceptional path: removing its guard also enters
                // the general power loop outside this harness's unwind bound.
                "power-unknown-exponent-to-zero",
                "power-exceptions",
                "power",
                "return Self::new_x(width, false);",
                "return Self::new(0, width, false);",
            ),
            (
                "power-ignores-base-unknown",
                "power-exceptions",
                "power",
                "if self.is_xz() {",
                "if false {",
            ),
            (
                "power-ignores-negative-exponent",
                "power",
                "power",
                "let payload = if negative {",
                "let payload = if false {",
            ),
            (
                "power-inverts-negative-parity",
                "power-exceptions",
                "power",
                "if exponent.payload & 1 != 0 {",
                "if exponent.payload & 1 == 0 {",
            ),
            (
                "power-zero-exponent-to-zero",
                "power",
                "power",
                "let mut result = 1u64;",
                "let mut result = 0u64;",
            ),
            (
                "power-multiplies-by-addition",
                "power",
                "power",
                "result.wrapping_mul(base)",
                "result.wrapping_add(base)",
            ),
            (
                "unmasked-power-result",
                "power",
                "power",
                "result & mask",
                "result",
            ),
            (
                "power-loses-signedness",
                "power-exceptions",
                "power",
                "Self::new(payload, width, self.signed)",
                "Self::new(payload, width, false)",
            ),
        ] {
            let contract = contract(contract_name);
            if !selected.iter().any(|c| c.name == contract.name) {
                continue;
            }
            let source = mutate_method(&source, method, old, new);
            mutations.push(Mutation {
                name,
                contract,
                path: CORE,
                source,
            });
        }
    }
    if selected.iter().any(|c| c.group == "layout") {
        let path = "veryl/crates/analyzer/src/ir/shape.rs";
        let original = fs::read_to_string(repo.join(path)).unwrap();
        for (name, contract_name, method, edits) in [
            (
                "shape-total-wraps-overflow",
                "shape-total",
                "total",
                &[(
                    "total.checked_mul((*extent)?)",
                    "Some(total.wrapping_mul((*extent)?))",
                )][..],
            ),
            (
                "shape-unknown-total-known",
                "shape-boundary",
                "total",
                &[("return None;", "return Some(1);")][..],
            ),
            (
                "shape-zero-total-one",
                "shape-total",
                "total",
                &[("return Some(0);", "return Some(1);")][..],
            ),
            (
                "shape-scalar-total-zero",
                "shape-boundary",
                "total",
                &[("try_fold(1usize", "try_fold(0usize")][..],
            ),
            (
                "shape-index-ignores-bounds",
                "shape-index",
                "calc_index",
                &[("if index[i] >= extent {", "if false {")][..],
            ),
            (
                "shape-index-reverses-storage",
                "shape-index",
                "calc_index",
                &[("self.iter().enumerate().rev()", "self.iter().enumerate()")][..],
            ),
            (
                "shape-index-ignores-stride",
                "shape-index",
                "calc_index",
                &[("base.checked_mul(extent)?", "base")][..],
            ),
            (
                "shape-index-loses-offset",
                "shape-index",
                "calc_index",
                &[("ret.checked_add(index[i].checked_mul(base)?)?", "ret")][..],
            ),
            (
                "shape-range-off-by-one",
                "shape-range",
                "calc_range",
                &[("length.checked_sub(1)?", "length")][..],
            ),
            (
                "shape-range-unscaled-prefix",
                "shape-range",
                "calc_range",
                &[("index[dimension].checked_mul(stride)?", "index[dimension]")][..],
            ),
            (
                "shape-range-ignores-prefix-bounds",
                "shape-range",
                "calc_range",
                &[("if index[dimension] >= extent {", "if false {")][..],
            ),
            (
                "shape-range-loses-start",
                "shape-range",
                "calc_range",
                &[("Some((start,", "Some((0,")][..],
            ),
            (
                "shape-index-overflow-alias",
                "shape-overflow",
                "calc_index",
                &[
                    ("            self.total()?;\n", ""),
                    ("base.checked_mul(extent)?", "base.wrapping_mul(extent)"),
                ][..],
            ),
            (
                "shape-range-overflow-alias",
                "shape-overflow",
                "calc_range",
                &[
                    ("if self.total()? == 0 {", "if false {"),
                    ("stride.checked_mul(extent)?", "stride.wrapping_mul(extent)"),
                ][..],
            ),
        ] {
            let contract = contract(contract_name);
            if !selected.iter().any(|c| c.name == contract.name) {
                continue;
            }
            let source = edits.iter().fold(original.clone(), |source, (old, new)| {
                mutate_method(&source, method, old, new)
            });
            mutations.push(Mutation {
                name,
                contract,
                path,
                source,
            });
        }
    }
    if selected.iter().any(|c| c.group == "coordinates") {
        let variable = "veryl/crates/analyzer/src/ir/variable.rs";
        let comptime = "veryl/crates/analyzer/src/ir/comptime.rs";
        for (name, contract_name, path, old, new) in [
            (
                "endpoint-plus-premature-saturation",
                "selection-endpoints",
                variable,
                "beg.saturating_add(end - 1)",
                "beg.saturating_add(end).saturating_sub(1)",
            ),
            (
                "endpoint-minus-premature-saturation",
                "selection-endpoints",
                variable,
                "beg.saturating_sub(end - 1)",
                "beg.saturating_add(1).saturating_sub(end)",
            ),
            (
                "endpoint-step-premature-saturation",
                "selection-endpoints",
                variable,
                "low.saturating_add(end - 1)",
                "low.saturating_add(end).saturating_sub(1)",
            ),
            (
                "endpoint-colon-swaps-array",
                "selection-endpoints",
                variable,
                "VarSelectOp::Colon => (beg, end, false)",
                "VarSelectOp::Colon => (beg, end, is_array)",
            ),
            (
                "endpoint-ignores-array-order",
                "selection-endpoints",
                variable,
                "if swap { (end, beg) } else { (beg, end) }",
                "if false { (end, beg) } else { (beg, end) }",
            ),
            (
                "endpoint-plus-off-by-one",
                "selection-endpoints",
                variable,
                "beg.saturating_add(end - 1)",
                "beg.saturating_add(end)",
            ),
            (
                "endpoint-minus-off-by-one",
                "selection-endpoints",
                variable,
                "beg.saturating_sub(end - 1)",
                "beg.saturating_sub(end)",
            ),
            (
                "endpoint-step-adds-instead",
                "selection-endpoints",
                variable,
                "let low = beg.saturating_mul(end);",
                "let low = beg.saturating_add(end);",
            ),
            (
                "size-total-bits-wraps",
                "packed-unpacked-bits",
                comptime,
                "self.total_width()?.checked_mul(self.array.total()?)",
                "Some(self.total_width()?.wrapping_mul(self.array.total()?))",
            ),
            (
                "size-total-bits-adds",
                "packed-unpacked-bits",
                comptime,
                "self.total_width()?.checked_mul(self.array.total()?)",
                "self.total_width()?.checked_add(self.array.total()?)",
            ),
            (
                "size-total-bits-ignores-array",
                "packed-unpacked-bits",
                comptime,
                "self.total_width()?.checked_mul(self.array.total()?)",
                "self.total_width()",
            ),
        ] {
            let contract = contract(contract_name);
            if selected.iter().any(|c| c.name == contract.name) {
                let original = fs::read_to_string(repo.join(path)).unwrap();
                mutations.push(Mutation {
                    name,
                    contract,
                    path,
                    source: replace_once(&original, old, new),
                });
            }
        }
    }
    if selected
        .iter()
        .any(|c| c.name == "checked-selection-endpoints")
    {
        let path = "veryl/crates/analyzer/src/ir/variable.rs";
        let original = fs::read_to_string(repo.join(path)).unwrap();
        for (name, edits) in [
            (
                "checked-plus-clamps-overflow",
                &[(
                    "beg.checked_add(end.checked_sub(1)?)?",
                    "beg.saturating_add(end.saturating_sub(1))",
                )][..],
            ),
            (
                "checked-minus-clamps-underflow",
                &[(
                    "beg.checked_sub(end.checked_sub(1)?)?",
                    "beg.saturating_sub(end.checked_sub(1)?)",
                )][..],
            ),
            (
                "checked-step-clamps-overflow",
                &[
                    ("beg.checked_mul(end)?", "beg.saturating_mul(end)"),
                    (
                        "low.checked_add(end.checked_sub(1)?)?",
                        "low.saturating_add(end.checked_sub(1)?)",
                    ),
                ][..],
            ),
        ] {
            let source = edits.iter().fold(original.clone(), |source, (old, new)| {
                mutate_method(&source, "checked_eval_value", old, new)
            });
            mutations.push(Mutation {
                name,
                contract: contract("checked-selection-endpoints"),
                path,
                source,
            });
        }
    }
    for (contract_name, name, method, old, new) in [
        (
            "shape-index",
            "shape-index-rejects-four-extent",
            "calc_index",
            " -> Option<usize> {\n",
            " -> Option<usize> {\n        if self.0.contains(&Some(4)) { return None; }\n",
        ),
        (
            "shape-index",
            "shape-total-zero-before-unresolved",
            "total",
            "if self.0.iter().any(Option::is_none) {",
            "if !self.0.contains(&Some(0)) && self.0.iter().any(Option::is_none) {",
        ),
        (
            "shape-range",
            "shape-range-rejects-four-extent",
            "calc_range",
            " -> Option<(usize, usize)> {\n",
            " -> Option<(usize, usize)> {\n        if self.0.contains(&Some(4)) { return None; }\n",
        ),
    ] {
        if let Some(&contract) = selected.iter().find(|c| c.name == contract_name) {
            let path = "veryl/crates/analyzer/src/ir/shape.rs";
            let source = fs::read_to_string(repo.join(path)).unwrap();
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, method, old, new),
            });
        }
    }
    for (contract_name, name, old, new) in [
        (
            "public-conditional-small-22",
            "public-conditional-ignores-condition-high-bit",
            "let condition = self.truth();",
            "let mut condition_value = self.clone(); condition_value.trunc(1); let condition = condition_value.truth();",
        ),
        (
            "public-conditional",
            "public-conditional-zero-context-widens",
            "let context_width = width;",
            "let context_width = if width == 0 { 4 } else { width };",
        ),
        (
            "public-conditional-small-12",
            "public-conditional-zero-context-selects-width",
            "return Self::U64(a.conditional(b, condition, context_width, signed));",
            "let mut value = Self::U64(a.conditional(b, condition, context_width, signed)); if context_width == 0 && condition == Some(true) { value.trunc(a.width as usize); } return value;",
        ),
    ] {
        if let Some(&contract) = selected.iter().find(|c| c.name == contract_name) {
            let source = fs::read_to_string(repo.join(CORE)).unwrap();
            mutations.push(Mutation {
                name,
                contract,
                path: CORE,
                source: replace_once(&source, old, new),
            });
        }
    }
    for (contract_name, name, method, old, new) in [
        (
            "public-arithmetic-add",
            "public-add-disables-left-sign",
            "eval_value_binary",
            "Op::Add => {\n                let x = x.expand(width, signed);",
            "Op::Add => {\n                let x = x.expand(width, false);",
        ),
        (
            "public-arithmetic-add",
            "public-add-forces-biguint-context",
            "eval_value_binary",
            "Op::Add => {\n                let x = x.expand(width, signed);",
            "Op::Add => {\n                let width = width.max(65); let x = x.expand(width, signed);",
        ),
        (
            "public-arithmetic-subtract",
            "public-subtract-skips-right-extension",
            "eval_value_binary",
            "Op::Sub => {\n                let x = x.expand(width, signed);\n                let y = y.expand(width, signed);",
            "Op::Sub => {\n                let x = x.expand(width, signed);\n                let y = std::borrow::Cow::Borrowed(y);",
        ),
        (
            "public-arithmetic-negate",
            "public-negate-adopts-caller-sign",
            "eval_value_unary",
            "Value::U64(x) => Value::U64(x.negate(width)),",
            "Value::U64(x) => { let mut ret = x.negate(width); ret.signed = signed; Value::U64(ret) },",
        ),
    ] {
        if let Some(&contract) = selected.iter().find(|c| c.name == contract_name) {
            let path = "veryl/crates/analyzer/src/ir/op.rs";
            let source = fs::read_to_string(repo.join(path)).unwrap();
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, method, old, new),
            });
        }
    }
    if let Some(&contract) = selected.iter().find(|c| c.name == "public-arithmetic-plus") {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        for (name, new) in [
            ("public-plus-skips-growth", "Op::Add => { x.clone() }"),
            (
                "public-plus-adopts-caller-sign",
                "Op::Add => { let x = x.expand(width, signed); let mut ret = x.into_owned(); if let Value::U64(v) = &mut ret { v.signed = signed; } ret }",
            ),
        ] {
            mutations.push(Mutation {
                name, contract, path,
                source: mutate_method(&source, "eval_value_unary",
                    "Op::Add => {\n                let x = x.expand(width, signed);\n                x.into_owned()\n            }", new),
            });
        }
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "public-plus-loses-extended-Z",
            contract,
            path: CORE,
            source: mutate_method(&source, "expand", "mask_xz |= mask;", ""),
        });
        for (name, old, new) in [
            (
                "public-plus-upper-payload-leak",
                "Self::U64(x) => x.payload as u128,",
                "Self::U64(x) => (x.payload as u128) | if x.width == 5 { 1u128 << 100 } else { 0 },",
            ),
            (
                "public-plus-upper-xz-leak",
                "Self::U64(x) => x.mask_xz as u128,",
                "Self::U64(x) => (x.mask_xz as u128) | if x.width == 5 { 1u128 << 64 } else { 0 },",
            ),
        ] {
            mutations.push(Mutation {
                name,
                contract,
                path: CORE,
                source: replace_once(&source, old, new),
            });
        }
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-assignment-native")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "public-assignment-reaches-biguint-constructor",
            contract,
            path: CORE,
            source: mutate_method(&source, "from_u128", "if width <= 64 {", "if width < 64 {"),
        });
        for (name, new) in [
            (
                "public-assignment-swaps-endpoints",
                "x.assign(value, end, beg)",
            ),
            (
                "public-assignment-clears-source-xz",
                "x.assign(ValueU64 { mask_xz: 0, ..value }, beg, end)",
            ),
            (
                "public-assignment-adopts-source-sign",
                "{ x.signed = value.signed; x.assign(value, beg, end) }",
            ),
            (
                "public-assignment-adopts-source-width",
                "{ x.width = value.width; x.assign(value, beg, end) }",
            ),
        ] {
            mutations.push(Mutation {
                name,
                contract,
                path: CORE,
                source: replace_once(
                    &source,
                    "x.assign(value, beg, end)\n            }\n            Self::BigUint(x) => {",
                    &format!("{new}\n            }}\n            Self::BigUint(x) => {{"),
                ),
            });
        }
    }
    if let Some(&contract) = selected.iter().find(|c| c.name == "public-selection-small") {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        let old = "Self::U64(x) => Self::U64(x.select(beg, end)),";
        for (name, new) in [
            (
                "public-selection-sorts-ascending",
                "Self::U64(x) => Self::U64(x.select(beg.min(end), beg.max(end))),",
            ),
            (
                "public-selection-preserves-source-sign",
                "Self::U64(x) => { let mut ret = x.select(beg, end); ret.signed = x.signed; Self::U64(ret) },",
            ),
            (
                "public-selection-clips-missing-width",
                "Self::U64(x) => { let mut ret = x.select(beg, end); if ret.width > x.width { ret.trunc(x.width as usize); } Self::U64(ret) },",
            ),
        ] {
            mutations.push(Mutation {
                name,
                contract,
                path: CORE,
                source: replace_once(&source, old, new),
            });
        }
    }
    if let Some(&contract) = selected.iter().find(|c| c.name == "public-resize-four") {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        for (name, old, new) in [
            (
                "public-truncation-skips-native",
                "Self::U64(x) => x.trunc(width),",
                "Self::U64(_) => {},",
            ),
            (
                "public-truncation-grows-on-noop",
                "if self_width <= width {",
                "if self_width == width {",
            ),
            (
                "public-extension-disables-sign",
                "Cow::Owned(Value::U64(x.expand(width, use_sign)))",
                "Cow::Owned(Value::U64(x.expand(width, false)))",
            ),
        ] {
            mutations.push(Mutation {
                name,
                contract,
                path: CORE,
                source: replace_once(&source, old, new),
            });
        }
        mutations.push(Mutation {
            name: "public-resize-constructor-clears-xz",
            contract,
            path: CORE,
            source: mutate_method(
                &source,
                "from_u128",
                "mask_xz: mask_xz as u64,",
                "mask_xz: 0,",
            ),
        });
    }
    if let Some(&contract) = selected.iter().find(|c| c.name == "full-multiply") {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        for (name, old, new) in [
            (
                "full-multiply-add",
                "self.payload.wrapping_mul(rhs.payload)",
                "self.payload.wrapping_add(rhs.payload)",
            ),
            (
                "full-multiply-low32",
                "self.payload.wrapping_mul(rhs.payload)",
                "(self.payload as u32 as u64).wrapping_mul(rhs.payload)",
            ),
        ] {
            mutations.push(Mutation {
                name,
                contract,
                path: CORE,
                source: mutate_method(&source, "multiply", old, new),
            });
        }
    }
    mutations
}
