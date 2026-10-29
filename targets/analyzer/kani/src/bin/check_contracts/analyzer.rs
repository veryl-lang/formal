//! Analyzer-specific contracts and production mutations.
use std::fs;
use std::path::Path;
use veryl_formal_runner::{Contract, Mutation};
const CORE: &str = "veryl/crates/analyzer/src/value.rs";

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

const PUBLIC_ARITHMETIC_COVERS: [&str; 15] = [
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
    "public arithmetic u128 padding is reachable",
];

const SHIFT_CONTINUATION_COVERS: &[&str] = &[
    "public shift continuation signed amount is reachable",
    "public shift continuation signed one fill is reachable",
    "public shift continuation signed X fill is reachable",
    "public shift continuation signed Z fill is reachable",
    "public shift continuation unknown amount is reachable",
    "public shift continuation arithmetic left overshift is reachable",
    "public shift continuation upper Z amount is reachable",
];

pub static CONTRACTS: [Contract; 99] = [
    Contract {
        group: "public-nonlinear",
        name: "public-small-remainder-width-1-unsigned",
        harness: "public_small_remainder_width_1_unsigned_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "nonlinear remainder numeric case is reachable",
            "nonlinear fixed width unknown LHS is reachable",
            "nonlinear fixed width unknown RHS is reachable",
            "nonlinear fixed width zero RHS is reachable",
            "nonlinear fixed width mixed signs are reachable",
            "nonlinear fixed width natural call is reachable",
        ],
    },
    Contract {
        group: "public-nonlinear",
        name: "public-small-remainder-width-1-signed",
        harness: "public_small_remainder_width_1_signed_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "nonlinear remainder numeric case is reachable",
            "nonlinear fixed width unknown LHS is reachable",
            "nonlinear fixed width unknown RHS is reachable",
            "nonlinear fixed width zero RHS is reachable",
            "nonlinear fixed width mixed signs are reachable",
            "nonlinear fixed width natural call is reachable",
        ],
    },
    Contract {
        group: "public-nonlinear",
        name: "public-small-remainder-width-2-unsigned",
        harness: "public_small_remainder_width_2_unsigned_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "nonlinear remainder numeric case is reachable",
            "nonlinear fixed width unknown LHS is reachable",
            "nonlinear fixed width unknown RHS is reachable",
            "nonlinear fixed width zero RHS is reachable",
            "nonlinear fixed width mixed signs are reachable",
            "nonlinear fixed width natural call is reachable",
            "nonlinear fixed width widened call is reachable",
        ],
    },
    Contract {
        group: "public-nonlinear",
        name: "public-small-remainder-width-2-signed",
        harness: "public_small_remainder_width_2_signed_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "nonlinear remainder numeric case is reachable",
            "nonlinear fixed width unknown LHS is reachable",
            "nonlinear fixed width unknown RHS is reachable",
            "nonlinear fixed width zero RHS is reachable",
            "nonlinear fixed width mixed signs are reachable",
            "nonlinear fixed width natural call is reachable",
            "nonlinear fixed width widened call is reachable",
        ],
    },
    Contract {
        group: "public-nonlinear",
        name: "public-small-remainder-width-3-unsigned",
        harness: "public_small_remainder_width_3_unsigned_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "nonlinear remainder numeric case is reachable",
            "nonlinear fixed width unknown LHS is reachable",
            "nonlinear fixed width unknown RHS is reachable",
            "nonlinear fixed width zero RHS is reachable",
            "nonlinear fixed width mixed signs are reachable",
            "nonlinear fixed width natural call is reachable",
            "nonlinear fixed width widened call is reachable",
        ],
    },
    Contract {
        group: "public-nonlinear",
        name: "public-small-remainder-width-3-signed",
        harness: "public_small_remainder_width_3_signed_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "nonlinear remainder numeric case is reachable",
            "nonlinear fixed width unknown LHS is reachable",
            "nonlinear fixed width unknown RHS is reachable",
            "nonlinear fixed width zero RHS is reachable",
            "nonlinear fixed width mixed signs are reachable",
            "nonlinear fixed width natural call is reachable",
            "nonlinear fixed width widened call is reachable",
        ],
    },
    Contract {
        group: "public-nonlinear",
        name: "public-small-remainder-width-4-unsigned",
        harness: "public_small_remainder_width_4_unsigned_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "nonlinear remainder numeric case is reachable",
            "nonlinear fixed width unknown LHS is reachable",
            "nonlinear fixed width unknown RHS is reachable",
            "nonlinear fixed width zero RHS is reachable",
            "nonlinear fixed width mixed signs are reachable",
            "nonlinear fixed width widened call is reachable",
        ],
    },
    Contract {
        group: "public-nonlinear",
        name: "public-small-remainder-width-4-signed",
        harness: "public_small_remainder_width_4_signed_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "nonlinear remainder numeric case is reachable",
            "nonlinear fixed width unknown LHS is reachable",
            "nonlinear fixed width unknown RHS is reachable",
            "nonlinear fixed width zero RHS is reachable",
            "nonlinear fixed width mixed signs are reachable",
            "nonlinear fixed width widened call is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-small-divide-width-1-unsigned",
        harness: "public_small_divide_width_1_unsigned_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "nonlinear fixed width unknown LHS is reachable",
            "nonlinear fixed width unknown RHS is reachable",
            "nonlinear fixed width zero RHS is reachable",
            "nonlinear fixed width mixed signs are reachable",
            "nonlinear fixed width natural call is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-small-divide-width-1-signed",
        harness: "public_small_divide_width_1_signed_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "nonlinear fixed width unknown LHS is reachable",
            "nonlinear fixed width unknown RHS is reachable",
            "nonlinear fixed width zero RHS is reachable",
            "nonlinear fixed width mixed signs are reachable",
            "nonlinear fixed width natural call is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-small-divide-width-2-unsigned",
        harness: "public_small_divide_width_2_unsigned_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "nonlinear fixed width unknown LHS is reachable",
            "nonlinear fixed width unknown RHS is reachable",
            "nonlinear fixed width zero RHS is reachable",
            "nonlinear fixed width mixed signs are reachable",
            "nonlinear fixed width natural call is reachable",
            "nonlinear fixed width widened call is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-small-divide-width-2-signed",
        harness: "public_small_divide_width_2_signed_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "nonlinear fixed width unknown LHS is reachable",
            "nonlinear fixed width unknown RHS is reachable",
            "nonlinear fixed width zero RHS is reachable",
            "nonlinear fixed width mixed signs are reachable",
            "nonlinear fixed width natural call is reachable",
            "nonlinear fixed width widened call is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-small-divide-width-3-unsigned",
        harness: "public_small_divide_width_3_unsigned_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "nonlinear fixed width unknown LHS is reachable",
            "nonlinear fixed width unknown RHS is reachable",
            "nonlinear fixed width zero RHS is reachable",
            "nonlinear fixed width mixed signs are reachable",
            "nonlinear fixed width natural call is reachable",
            "nonlinear fixed width widened call is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-small-divide-width-3-signed",
        harness: "public_small_divide_width_3_signed_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "nonlinear fixed width unknown LHS is reachable",
            "nonlinear fixed width unknown RHS is reachable",
            "nonlinear fixed width zero RHS is reachable",
            "nonlinear fixed width mixed signs are reachable",
            "nonlinear fixed width natural call is reachable",
            "nonlinear fixed width widened call is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-small-divide-width-4-unsigned",
        harness: "public_small_divide_width_4_unsigned_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "nonlinear fixed width unknown LHS is reachable",
            "nonlinear fixed width unknown RHS is reachable",
            "nonlinear fixed width zero RHS is reachable",
            "nonlinear fixed width mixed signs are reachable",
            "nonlinear fixed width widened call is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-small-divide-width-4-signed",
        harness: "public_small_divide_width_4_signed_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "nonlinear fixed width unknown LHS is reachable",
            "nonlinear fixed width unknown RHS is reachable",
            "nonlinear fixed width zero RHS is reachable",
            "nonlinear fixed width mixed signs are reachable",
            "nonlinear fixed width widened call is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-small-multiply",
        harness: "public_small_multiply_initial_calls_match_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "public small product LHS width 1 is reachable",
            "public small product LHS width 2 is reachable",
            "public small product LHS width 3 is reachable",
            "public small product RHS width 1 is reachable",
            "public small product RHS width 2 is reachable",
            "public small product RHS width 3 is reachable",
            "public small product wider RHS is reachable",
            "public small product wider LHS is reachable",
            "public small product mixed signs are reachable",
            "public small product unknown input is reachable",
            "public small product zero input is reachable",
            "public small product sign extension is reachable",
        ],
    },
    Contract {
        group: "public-wide-resize",
        name: "public-native-zero-extension-wide",
        harness: "public_native_zero_extension_wide_matches_reference_semantics",
        message: "public wide resize semantics mismatch",
        covers: &[
            "public native wide zero extension width 1 zero is reachable",
            "public native wide zero extension width 1 one is reachable",
            "public native wide zero extension width 1 X is reachable",
            "public native wide zero extension width 1 Z is reachable",
            "public native wide zero extension width 64 zero is reachable",
            "public native wide zero extension width 64 one is reachable",
            "public native wide zero extension width 64 X is reachable",
            "public native wide zero extension width 64 Z is reachable",
            "public native wide zero extension interior widths are reachable",
        ],
    },
    Contract {
        group: "public-wide-resize",
        name: "public-native-zero-fill-extension-wide",
        harness: "public_native_zero_fill_extension_wide_matches_reference_semantics",
        message: "public wide resize semantics mismatch",
        covers: &[
            "public native zero fill extension signed endpoint is reachable",
            "public native zero fill extension unsigned high Z is reachable",
            "public native zero fill extension signed lower X is reachable",
            "public native zero fill extension signed interior Z is reachable",
            "public native zero fill extension unsigned one is reachable",
        ],
    },
    Contract {
        group: "public-getters",
        name: "public-big-getters-128",
        harness: "public_big_getters_match_reference_bits",
        message: "public BigUint getter semantics mismatch",
        covers: &[
            "public BigUint getters empty digits are reachable",
            "public BigUint getters high one is reachable",
            "public BigUint getters high X is reachable",
            "public BigUint getters high Z and padding are reachable",
        ],
    },
    Contract {
        group: "public-native-sequences",
        name: "public-small-shift-continuation-width-1",
        harness: "public_small_shift_continuation_width_1_matches_reference_semantics",
        message: "public shift semantics mismatch",
        covers: SHIFT_CONTINUATION_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-small-shift-continuation-width-2",
        harness: "public_small_shift_continuation_width_2_matches_reference_semantics",
        message: "public shift semantics mismatch",
        covers: SHIFT_CONTINUATION_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-small-shift-continuation-width-3",
        harness: "public_small_shift_continuation_width_3_matches_reference_semantics",
        message: "public shift semantics mismatch",
        covers: SHIFT_CONTINUATION_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-small-shift-continuation-width-4",
        harness: "public_small_shift_continuation_width_4_matches_reference_semantics",
        message: "public shift semantics mismatch",
        covers: SHIFT_CONTINUATION_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-boundary-width-31",
        harness: "public_shift_boundary_width_31_matches_reference_semantics",
        message: "public shift semantics mismatch",
        covers: SHIFT_CONTINUATION_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-boundary-width-32",
        harness: "public_shift_boundary_width_32_matches_reference_semantics",
        message: "public shift semantics mismatch",
        covers: SHIFT_CONTINUATION_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-boundary-width-63",
        harness: "public_shift_boundary_width_63_matches_reference_semantics",
        message: "public shift semantics mismatch",
        covers: SHIFT_CONTINUATION_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-boundary-width-64",
        harness: "public_shift_boundary_width_64_matches_reference_semantics",
        message: "public shift semantics mismatch",
        covers: SHIFT_CONTINUATION_COVERS,
    },
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
        name: "public-selection-native",
        harness: "public_selection_native_matches_reference_semantics",
        message: "public selection semantics mismatch",
        covers: &[
            "public native selection source width 1 is reachable",
            "public native selection source width 64 is reachable",
            "public native selection reversed endpoints are reachable",
            "public native selection single bit is reachable",
            "public native selection 64-bit result is reachable",
            "public native selection full source is reachable",
            "public native selection missing source is reachable",
            "public native selection partial overlap is reachable",
            "public native selection maximum single endpoint is reachable",
            "public native selection maximum range is reachable",
            "public native selection maximum reversed endpoint is reachable",
            "public native selection signed source is reachable",
            "public native selection unsigned source is reachable",
            "public native selection source X is reachable",
            "public native selection source Z is reachable",
            "public native selection u128 padding is reachable",
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
        name: "public-resize-native",
        harness: "public_resize_native_matches_reference_semantics",
        message: "public resize semantics mismatch",
        covers: &[
            "public native resize source width 1 is reachable",
            "public native resize source width 64 is reachable",
            "public native resize request 1 is reachable",
            "public native resize request 64 is reachable",
            "public native resize smaller request is reachable",
            "public native resize equal request is reachable",
            "public native resize larger request is reachable",
            "public native resize signed source is reachable",
            "public native resize unsigned source is reachable",
            "public native resize u128 padding is reachable",
            "public native resize truncated X is reachable",
            "public native resize truncated Z is reachable",
            "public native resize disables sign extension",
            "public native resize zero-extends unsigned source",
            "public native resize extends zero",
            "public native resize extends one",
            "public native resize extends X",
            "public native resize extends Z",
        ],
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
        name: "public-shift-unsigned-native",
        harness: "public_shift_unsigned_initial_calls_match_reference_semantics",
        message: "public shift semantics mismatch",
        covers: &[
            "public shift source width 1 is reachable",
            "public shift source width 64 is reachable",
            "public shift zero amount is reachable",
            "public shift last-bit amount is reachable",
            "public shift source-width amount is reachable",
            "public shift amount 64 is reachable",
            "public shift maximum amount is reachable",
            "public shift unknown amount is reachable",
            "public shift amount contains X",
            "public shift amount contains Z",
            "public shift amount has an upper unknown",
            "public shift source contains X",
            "public shift source contains Z",
            "public shift source high bit is reachable",
            "public shift highest native observation is reachable",
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
        name: "public-equality-native",
        harness: "public_equality_native_pair_matches_reference_semantics",
        message: "public comparison semantics mismatch",
        covers: &[
            "public comparison LHS width 1 is reachable",
            "public comparison LHS width 2 is reachable",
            "public comparison LHS width 3 is reachable",
            "public comparison RHS width 1 is reachable",
            "public comparison RHS width 2 is reachable",
            "public comparison RHS width 3 is reachable",
            "public comparison LHS width 64 is reachable",
            "public comparison RHS width 64 is reachable",
            "public comparison wider LHS is reachable",
            "public comparison wider RHS is reachable",
            "public comparison signed mode is reachable",
            "public comparison mixed signs are reachable",
            "public comparison X result is reachable",
            "public comparison true result is reachable",
            "public comparison false result is reachable",
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
        name: "public-bitwise-native",
        harness: "public_bitwise_native_sequence_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public bitwise LHS width 1 is reachable",
            "public bitwise LHS width 64 is reachable",
            "public bitwise RHS width 1 is reachable",
            "public bitwise RHS width 64 is reachable",
            "public bitwise wider LHS is reachable",
            "public bitwise wider RHS is reachable",
            "public bitwise unchanged context is reachable",
            "public bitwise width 64 is reachable",
            "public bitwise signed context is reachable",
            "public bitwise mixed signs are reachable",
            "public bitwise source X is reachable",
            "public bitwise source Z is reachable",
            "public bitwise u128 padding is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-not-native",
        harness: "public_bit_not_native_sequence_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public NOT source width 1 is reachable",
            "public NOT source width 64 is reachable",
            "public NOT result width 64 is reachable",
            "public NOT widened context is reachable",
            "public NOT unchanged context is reachable",
            "public NOT signed source is reachable",
            "public NOT unsigned source is reachable",
            "public NOT source X is reachable",
            "public NOT source Z is reachable",
            "public NOT u128 padding is reachable",
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
        name: "public-conditional-native",
        harness: "public_conditional_native_matches_reference_semantics",
        message: "public conditional semantics mismatch",
        covers: &[
            "public native conditional condition width 1 is reachable",
            "public native conditional condition width 2 is reachable",
            "public native conditional true width 1 is reachable",
            "public native conditional true width 64 is reachable",
            "public native conditional false width 1 is reachable",
            "public native conditional false width 64 is reachable",
            "public native conditional context 0 is reachable",
            "public native conditional context 1 is reachable",
            "public native conditional context 64 is reachable",
            "public native conditional false condition is reachable",
            "public native conditional true condition is reachable",
            "public native conditional ambiguous condition is reachable",
            "public native conditional known one dominates unknown",
            "public native conditional common Z becomes X",
            "public native conditional context widens branches",
            "public native conditional context preserves branch width",
            "public native conditional sign extension is reachable",
            "public native conditional sign suppression is reachable",
            "public native conditional signed result is reachable",
            "public native conditional unsigned result is reachable",
            "public native conditional u128 padding is reachable",
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
        name: "public-arithmetic-initial-pair",
        harness: "public_arithmetic_initial_pair_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "arithmetic initial pair observes addition",
            "arithmetic initial pair observes subtraction",
            "arithmetic initial pair LHS width 1 is reachable",
            "arithmetic initial pair LHS width 2 is reachable",
            "arithmetic initial pair LHS width 3 is reachable",
            "arithmetic initial pair RHS width 1 is reachable",
            "arithmetic initial pair RHS width 2 is reachable",
            "arithmetic initial pair RHS width 3 is reachable",
            "arithmetic initial pair wider LHS is reachable",
            "arithmetic initial pair wider RHS is reachable",
            "arithmetic initial pair mixed source signs are reachable",
            "arithmetic initial pair X result is reachable",
            "arithmetic initial pair u128 padding is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-arithmetic-unsigned-growth-plus",
        harness: "public_arithmetic_unsigned_growth_plus_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "unary widened sequence source width 1 is reachable",
            "unary widened sequence source width 2 is reachable",
            "unary widened sequence source width 3 is reachable",
            "unary widened sequence result width 3 is reachable",
            "unary widened sequence result width 4 is reachable",
            "unary widened sequence result width 5 is reachable",
            "unary widened sequence signed source is reachable",
            "unary widened sequence unsigned source is reachable",
            "unary widened sequence suppresses sign extension",
            "unary widened sequence X result is reachable",
            "unary widened sequence known one is reachable",
            "unary widened sequence Z result is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-arithmetic-unsigned-growth-negate",
        harness: "public_arithmetic_unsigned_growth_negate_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "unary widened sequence source width 1 is reachable",
            "unary widened sequence source width 2 is reachable",
            "unary widened sequence source width 3 is reachable",
            "unary widened sequence result width 3 is reachable",
            "unary widened sequence result width 4 is reachable",
            "unary widened sequence result width 5 is reachable",
            "unary widened sequence signed source is reachable",
            "unary widened sequence unsigned source is reachable",
            "unary widened sequence suppresses sign extension",
            "unary widened sequence X result is reachable",
            "unary widened sequence known one is reachable",
            "unary widened sequence u128 padding is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-arithmetic-signed-growth-plus",
        harness: "public_arithmetic_signed_growth_plus_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "unary signed growth source width 1 is reachable",
            "unary signed growth source width 2 is reachable",
            "unary signed growth source width 3 is reachable",
            "unary signed growth result width 3 is reachable",
            "unary signed growth result width 4 is reachable",
            "unary signed growth result width 5 is reachable",
            "unary signed growth signed source is reachable",
            "unary signed growth unsigned source is reachable",
            "unary signed growth negative signed source is reachable",
            "unary signed growth X result is reachable",
            "unary signed growth known one is reachable",
            "unary signed growth u128 padding is reachable",
            "unary signed growth extends one",
            "unary signed growth extends X",
            "unary signed growth extends Z",
            "unary signed growth zero-extends unsigned source",
        ],
    },
    Contract {
        group: "values",
        name: "public-arithmetic-signed-growth-negate",
        harness: "public_arithmetic_signed_growth_negate_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "unary signed growth source width 1 is reachable",
            "unary signed growth source width 2 is reachable",
            "unary signed growth source width 3 is reachable",
            "unary signed growth result width 3 is reachable",
            "unary signed growth result width 4 is reachable",
            "unary signed growth result width 5 is reachable",
            "unary signed growth signed source is reachable",
            "unary signed growth unsigned source is reachable",
            "unary signed growth negative signed source is reachable",
            "unary signed growth X result is reachable",
            "unary signed growth known one is reachable",
            "unary signed growth u128 padding is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-binary-natural-signed-add",
        harness: "public_binary_natural_signed_add_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "binary cache sequence LHS width 1 is reachable",
            "binary cache sequence LHS width 2 is reachable",
            "binary cache sequence LHS width 3 is reachable",
            "binary cache sequence RHS width 1 is reachable",
            "binary cache sequence RHS width 2 is reachable",
            "binary cache sequence RHS width 3 is reachable",
            "binary cache sequence wider LHS is reachable",
            "binary cache sequence wider RHS is reachable",
            "binary cache sequence mixed source signs are reachable",
            "binary cache sequence X result is reachable",
            "binary cache sequence known one is reachable",
            "binary cache sequence padding observation is reachable",
            "binary cache sequence left negative extension input is reachable",
            "binary cache sequence right negative extension input is reachable",
            "binary cache sequence minimum result width is reachable",
            "binary cache sequence maximum result width is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-binary-natural-signed-subtract",
        harness: "public_binary_natural_signed_subtract_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "binary cache sequence LHS width 1 is reachable",
            "binary cache sequence LHS width 2 is reachable",
            "binary cache sequence LHS width 3 is reachable",
            "binary cache sequence RHS width 1 is reachable",
            "binary cache sequence RHS width 2 is reachable",
            "binary cache sequence RHS width 3 is reachable",
            "binary cache sequence wider LHS is reachable",
            "binary cache sequence wider RHS is reachable",
            "binary cache sequence mixed source signs are reachable",
            "binary cache sequence X result is reachable",
            "binary cache sequence known one is reachable",
            "binary cache sequence padding observation is reachable",
            "binary cache sequence left negative extension input is reachable",
            "binary cache sequence right negative extension input is reachable",
            "binary cache sequence minimum result width is reachable",
            "binary cache sequence maximum result width is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-binary-widened-unsigned-add",
        harness: "public_binary_widened_unsigned_add_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "binary cache sequence LHS width 1 is reachable",
            "binary cache sequence LHS width 2 is reachable",
            "binary cache sequence LHS width 3 is reachable",
            "binary cache sequence RHS width 1 is reachable",
            "binary cache sequence RHS width 2 is reachable",
            "binary cache sequence RHS width 3 is reachable",
            "binary cache sequence wider LHS is reachable",
            "binary cache sequence wider RHS is reachable",
            "binary cache sequence mixed source signs are reachable",
            "binary cache sequence X result is reachable",
            "binary cache sequence known one is reachable",
            "binary cache sequence padding observation is reachable",
            "binary cache sequence left negative extension input is reachable",
            "binary cache sequence right negative extension input is reachable",
            "binary cache sequence minimum result width is reachable",
            "binary cache sequence maximum result width is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-binary-widened-unsigned-subtract",
        harness: "public_binary_widened_unsigned_subtract_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "binary cache sequence LHS width 1 is reachable",
            "binary cache sequence LHS width 2 is reachable",
            "binary cache sequence LHS width 3 is reachable",
            "binary cache sequence RHS width 1 is reachable",
            "binary cache sequence RHS width 2 is reachable",
            "binary cache sequence RHS width 3 is reachable",
            "binary cache sequence wider LHS is reachable",
            "binary cache sequence wider RHS is reachable",
            "binary cache sequence mixed source signs are reachable",
            "binary cache sequence X result is reachable",
            "binary cache sequence known one is reachable",
            "binary cache sequence padding observation is reachable",
            "binary cache sequence left negative extension input is reachable",
            "binary cache sequence right negative extension input is reachable",
            "binary cache sequence minimum result width is reachable",
            "binary cache sequence maximum result width is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-binary-widened-signed-add",
        harness: "public_binary_widened_signed_add_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "binary cache sequence LHS width 1 is reachable",
            "binary cache sequence LHS width 2 is reachable",
            "binary cache sequence LHS width 3 is reachable",
            "binary cache sequence RHS width 1 is reachable",
            "binary cache sequence RHS width 2 is reachable",
            "binary cache sequence RHS width 3 is reachable",
            "binary cache sequence wider LHS is reachable",
            "binary cache sequence wider RHS is reachable",
            "binary cache sequence mixed source signs are reachable",
            "binary cache sequence X result is reachable",
            "binary cache sequence known one is reachable",
            "binary cache sequence padding observation is reachable",
            "binary cache sequence left negative extension input is reachable",
            "binary cache sequence right negative extension input is reachable",
            "binary cache sequence minimum result width is reachable",
            "binary cache sequence maximum result width is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-binary-widened-signed-subtract",
        harness: "public_binary_widened_signed_subtract_matches_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "binary cache sequence LHS width 1 is reachable",
            "binary cache sequence LHS width 2 is reachable",
            "binary cache sequence LHS width 3 is reachable",
            "binary cache sequence RHS width 1 is reachable",
            "binary cache sequence RHS width 2 is reachable",
            "binary cache sequence RHS width 3 is reachable",
            "binary cache sequence wider LHS is reachable",
            "binary cache sequence wider RHS is reachable",
            "binary cache sequence mixed source signs are reachable",
            "binary cache sequence X result is reachable",
            "binary cache sequence known one is reachable",
            "binary cache sequence padding observation is reachable",
            "binary cache sequence left negative extension input is reachable",
            "binary cache sequence right negative extension input is reachable",
            "binary cache sequence minimum result width is reachable",
            "binary cache sequence maximum result width is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-binary-word64",
        harness: "public_binary_word64_initial_calls_match_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "public word64 addition overflow is reachable",
            "public word64 subtraction borrow is reachable",
            "public word64 highest stored bit is reachable",
            "public word64 source X is reachable",
            "public word64 source Z is reachable",
            "public word64 unknown result is reachable",
            "public word64 mixed source signs are reachable",
            "public word64 signed sources are reachable",
            "public word64 unsigned sources are reachable",
            "public word64 u128 padding is reachable",
        ],
    },
    Contract {
        group: "values",
        name: "public-arithmetic-unary-native",
        harness: "public_unary_native_initial_calls_match_reference_semantics",
        message: "public arithmetic semantics mismatch",
        covers: &[
            "public native unary source width 1 is reachable",
            "public native unary source width 64 is reachable",
            "public native unary signed source is reachable",
            "public native unary unsigned source is reachable",
            "public native unary source X is reachable",
            "public native unary source Z is reachable",
            "public native unary unknown negate is reachable",
            "public native unary known negate is reachable",
            "public native unary minimum signed value is reachable",
            "public native unary u128 padding is reachable",
        ],
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

// Conditional cover statements for other width instantiations remain in
// Kani's output. They are diagnostic only unless listed in Contract::covers.
pub const AUXILIARY_NONLINEAR_COVERS: &[&str] = &[
    "nonlinear fixed width natural call is reachable",
    "nonlinear fixed width widened call is reachable",
];

// Named covers are mandatory for this profile; the shared runner validates them.
// Only these new public proofs omit Kani's automatic per-assertion diagnostics.
pub fn uses_explicit_covers(contract: &Contract) -> bool {
    matches!(
        contract.group,
        "public-nonlinear" | "public-wide-resize" | "public-getters" | "public-native-sequences"
    )
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
                contract("public-conditional-native"),
                legacy_conditional_false,
            ),
            (
                "public-conditional-ignores-context-width",
                contract("public-conditional-native"),
                public_conditional_ignores_context_width,
            ),
            (
                "public-conditional-truncates-word",
                contract("public-conditional-native"),
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
            "public-conditional-native",
            "public-conditional-ignores-condition-high-bit",
            "let condition = self.truth();",
            "let mut condition_value = self.clone(); condition_value.trunc(1); let condition = condition_value.truth();",
        ),
        (
            "public-conditional-native",
            "public-conditional-zero-context-widens",
            "let context_width = width;",
            "let context_width = if width == 0 { 4 } else { width };",
        ),
        (
            "public-conditional-native",
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
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-arithmetic-initial-pair")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "arithmetic-initial-pair-subtracts-as-add",
            contract,
            path,
            source: mutate_method(
                &source,
                "eval_value_binary",
                "Value::U64(x.add_sub(y, width, signed, true))",
                "Value::U64(x.add_sub(y, width, signed, false))",
            ),
        });
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "arithmetic-initial-pair-subtraction-upper-leak",
            contract, path: CORE,
            source: mutate_method(&source, "add_sub",
                "Self::new(payload & Self::gen_mask(width), width, signed)",
                "{ let mut result = Self::new(payload & Self::gen_mask(width), width, signed); if subtract { result.payload |= 1u64 << 63; } result }"),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-arithmetic-unary-native")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "unary-initial-pair-negates-as-plus",
            contract,
            path,
            source: mutate_method(
                &source,
                "eval_value_unary",
                "Value::U64(x) => Value::U64(x.negate(width)),",
                "Value::U64(x) => Value::U64(x.clone()),",
            ),
        });
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "unary-initial-pair-negation-upper-leak",
            contract,
            path: CORE,
            source: mutate_method(
                &source,
                "negate",
                "ret.payload &= mask;",
                "ret.payload = (ret.payload & mask) | if width < 64 { 1u64 << 63 } else { 0 };",
            ),
        });
    }
    // These mutations preserve the unsigned prefix and corrupt only signed calls.
    for (contract_name, name, old, new) in [
        (
            "public-arithmetic-unary-native",
            "unary-signed-plus-grows-natural-width",
            "Op::Add => {\n                let x = x.expand(width, signed);\n                x.into_owned()\n            }",
            "Op::Add => { let x = x.expand(width + usize::from(signed), signed); x.into_owned() }",
        ),
        (
            "public-arithmetic-unary-native",
            "unary-signed-plus-upper-leak",
            "Op::Add => {\n                let x = x.expand(width, signed);\n                x.into_owned()\n            }",
            "Op::Add => { let mut result = x.expand(width, signed).into_owned(); if signed { if let Value::U64(x) = &mut result { if x.width < 64 { x.payload |= 1u64 << 63; } } } result }",
        ),
        (
            "public-arithmetic-unary-native",
            "unary-signed-negate-returns-plus",
            "Value::U64(x) => Value::U64(x.negate(width)),",
            "Value::U64(x) => Value::U64(if signed { x.clone() } else { x.negate(width) }),",
        ),
        (
            "public-arithmetic-unary-native",
            "unary-signed-negate-upper-leak",
            "Value::U64(x) => Value::U64(x.negate(width)),",
            "Value::U64(x) => { let mut result = x.negate(width); if signed && width < 64 { result.payload |= 1u64 << 63; } Value::U64(result) },",
        ),
    ] {
        if let Some(&contract) = selected.iter().find(|c| c.name == contract_name) {
            let path = "veryl/crates/analyzer/src/ir/op.rs";
            let source = fs::read_to_string(repo.join(path)).unwrap();
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_unary", old, new),
            });
        }
    }
    for (contract_name, name, method, old, new) in [
        (
            "public-arithmetic-unsigned-growth-plus",
            "unary-unsigned-growth-plus-skips-growth",
            "eval_value_unary",
            "Op::Add => {\n                let x = x.expand(width, signed);\n                x.into_owned()\n            }",
            "Op::Add => { let x = x.expand(if signed { width } else { x.width() }, signed); x.into_owned() }",
        ),
        (
            "public-arithmetic-unsigned-growth-plus",
            "unary-unsigned-growth-plus-poisons-added-bit",
            "eval_value_unary",
            "Op::Add => {\n                let x = x.expand(width, signed);\n                x.into_owned()\n            }",
            "Op::Add => { let natural = x.width(); let mut result = x.expand(width, signed).into_owned(); if !signed && width > natural { if let Value::U64(x) = &mut result { x.payload |= 1u64 << natural; } } result }",
        ),
        (
            "public-arithmetic-unsigned-growth-negate",
            "unary-unsigned-growth-negate-returns-plus",
            "eval_value_unary",
            "Value::U64(x) => Value::U64(x.negate(width)),",
            "Value::U64(x) => Value::U64(if !signed && width > 3 { x.clone() } else { x.negate(width) }),",
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
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-arithmetic-unsigned-growth-negate")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "unary-unsigned-growth-negate-ignores-unknown",
            contract,
            path: CORE,
            source: mutate_method(
                &source,
                "negate",
                "if self.is_xz() {",
                "if self.is_xz() && width <= 3 {",
            ),
        });
    }
    for (contract_name, name, old, new) in [
        (
            "public-arithmetic-signed-growth-plus",
            "unary-signed-growth-plus-zero-extends",
            "Op::Add => {\n                let x = x.expand(width, signed);\n                x.into_owned()\n            }",
            "Op::Add => { let x = x.expand(width, false); x.into_owned() }",
        ),
        (
            "public-arithmetic-signed-growth-negate",
            "unary-signed-growth-negate-returns-plus",
            "Value::U64(x) => Value::U64(x.negate(width)),",
            "Value::U64(x) => Value::U64(if signed && width > 3 { x.clone() } else { x.negate(width) }),",
        ),
        (
            "public-arithmetic-signed-growth-negate",
            "unary-signed-growth-negate-upper-leak",
            "Value::U64(x) => Value::U64(x.negate(width)),",
            "Value::U64(x) => { let mut result = x.negate(width); if signed && width > 3 { result.payload |= 1u64 << 63; } Value::U64(result) },",
        ),
    ] {
        if let Some(&contract) = selected.iter().find(|c| c.name == contract_name) {
            let path = "veryl/crates/analyzer/src/ir/op.rs";
            let source = fs::read_to_string(repo.join(path)).unwrap();
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_unary", old, new),
            });
        }
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-arithmetic-signed-growth-plus")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "unary-signed-growth-plus-loses-extended-xz",
            contract,
            path: CORE,
            source: mutate_method(&source, "expand", "mask_xz |= mask;", "mask_xz |= 0;"),
        });
    }
    if let Some(&contract) = selected.iter().find(|c| c.name == "public-equality-native") {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        for (name, old, new) in [
            (
                "public-equality-inverted",
                "x.equal(y, false, false)",
                "x.equal(y, false, true)",
            ),
            (
                "public-equality-uses-wildcard",
                "x.equal(y, false, false)",
                "x.equal(y, true, false)",
            ),
            (
                "public-equality-ignores-upper-payload",
                "x.equal(y, false, false)",
                "{ let mut x = x.clone(); let mut y = y.clone(); x.payload &= u32::MAX as u64; y.payload &= u32::MAX as u64; x.equal(&y, false, false) }",
            ),
        ] {
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_binary", old, new),
            });
        }
        let old = "Value::U64(ret).expand(width, false).into_owned()\n            }\n            Op::Ne =>";
        for (name, new) in [
            (
                "public-equality-skips-result-growth",
                "Value::U64(ret)\n            }\n            Op::Ne =>",
            ),
            (
                "public-equality-upper-padding-leak",
                "{ let mut ret = ret; ret.payload |= 1u64 << 63; Value::U64(ret).expand(width, false).into_owned() }\n            }\n            Op::Ne =>",
            ),
        ] {
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: replace_once(&source, old, new),
            });
        }
    }
    for (contract_name, name, method, old, new) in [
        (
            "public-bitwise-native",
            "public-bitwise-and-as-or",
            "eval_value_binary",
            "Value::U64(x.bit_and(y, width))",
            "Value::U64(x.bit_or(y, width))",
        ),
        (
            "public-bitwise-native",
            "public-bitwise-or-as-and",
            "eval_value_binary",
            "Value::U64(x.bit_or(y, width))",
            "Value::U64(x.bit_and(y, width))",
        ),
        (
            "public-bitwise-native",
            "public-bitwise-xor-as-xnor",
            "eval_value_binary",
            "Value::U64(x.bit_xor(y, width))",
            "Value::U64(x.bit_xnor(y, width))",
        ),
        (
            "public-bitwise-native",
            "public-bitwise-xnor-as-xor",
            "eval_value_binary",
            "Value::U64(x.bit_xnor(y, width))",
            "Value::U64(x.bit_xor(y, width))",
        ),
        (
            "public-bitwise-native",
            "public-bitwise-and-disables-left-sign",
            "eval_value_binary",
            "let x = resize(x, width, signed);",
            "let x = resize(x, width, false);",
        ),
        (
            "public-bitwise-native",
            "public-bitwise-and-upper-padding-leak",
            "eval_value_binary",
            "Value::U64(x.bit_and(y, width))",
            "{ let mut result = x.bit_and(y, width); if width < 64 { result.payload |= 1u64 << 63; } Value::U64(result) }",
        ),
        (
            "public-not-native",
            "public-not-returns-input",
            "eval_value_unary",
            "Value::U64(x) => Value::U64(x.bit_not(width)),",
            "Value::U64(x) => Value::U64(x.clone()),",
        ),
        (
            "public-not-native",
            "public-not-disables-sign",
            "eval_value_unary",
            "Op::BitNot => {\n                let x = x.expand(width, signed);",
            "Op::BitNot => {\n                let x = x.expand(width, false);",
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
    if let Some(&contract) = selected.iter().find(|c| c.name == "public-not-native") {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "public-not-unknown-becomes-z",
            contract,
            path: CORE,
            source: mutate_method(
                &source,
                "bit_not",
                "ret.payload &= ret.mask_xz ^ mask;",
                "ret.payload &= mask;",
            ),
        });
    }
    for (contract_name, name, old, new) in [
        (
            "public-binary-natural-signed-add",
            "public-binary-natural-signed-add-disables-left-sign",
            "Op::Add => {\n                let x = x.expand(width, signed);",
            "Op::Add => {\n                let x = x.expand(width, false);",
        ),
        (
            "public-binary-natural-signed-add",
            "public-binary-natural-signed-add-swaps-operation",
            "Value::U64(x.add_sub(y, width, signed, false))",
            "Value::U64(x.add_sub(y, width, signed, true))",
        ),
        (
            "public-binary-natural-signed-subtract",
            "public-binary-natural-signed-subtract-swaps-operation",
            "Value::U64(x.add_sub(y, width, signed, true))",
            "Value::U64(x.add_sub(y, width, signed, false))",
        ),
        (
            "public-binary-natural-signed-subtract",
            "public-binary-natural-signed-subtract-drops-result-sign",
            "Value::U64(x.add_sub(y, width, signed, true))",
            "Value::U64(x.add_sub(y, width, false, true))",
        ),
        (
            "public-binary-widened-unsigned-add",
            "public-binary-widened-unsigned-add-swaps-operation",
            "Value::U64(x.add_sub(y, width, signed, false))",
            "Value::U64(x.add_sub(y, width, signed, true))",
        ),
        (
            "public-binary-widened-unsigned-add",
            "public-binary-widened-unsigned-add-upper-padding-leak",
            "Value::U64(x.add_sub(y, width, signed, false))",
            "{ let mut result = x.add_sub(y, width, signed, false); if !signed && width > 3 { result.payload |= 1u64 << 63; } Value::U64(result) }",
        ),
        (
            "public-binary-widened-unsigned-subtract",
            "public-binary-widened-unsigned-subtract-swaps-operation",
            "Value::U64(x.add_sub(y, width, signed, true))",
            "Value::U64(x.add_sub(y, width, signed, false))",
        ),
        (
            "public-binary-widened-unsigned-subtract",
            "public-binary-widened-unsigned-subtract-upper-padding-leak",
            "Value::U64(x.add_sub(y, width, signed, true))",
            "{ let mut result = x.add_sub(y, width, signed, true); if !signed && width > 3 { result.payload |= 1u64 << 63; } Value::U64(result) }",
        ),
        (
            "public-binary-widened-signed-add",
            "public-binary-widened-signed-add-disables-left-sign",
            "Op::Add => {\n                let x = x.expand(width, signed);",
            "Op::Add => {\n                let x = x.expand(width, false);",
        ),
        (
            "public-binary-widened-signed-add",
            "public-binary-widened-signed-add-upper-padding-leak",
            "Value::U64(x.add_sub(y, width, signed, false))",
            "{ let mut result = x.add_sub(y, width, signed, false); if signed && width > 3 { result.payload |= 1u64 << 63; } Value::U64(result) }",
        ),
        (
            "public-binary-widened-signed-subtract",
            "public-binary-widened-signed-subtract-swaps-operation",
            "Value::U64(x.add_sub(y, width, signed, true))",
            "Value::U64(x.add_sub(y, width, signed, false))",
        ),
        (
            "public-binary-widened-signed-subtract",
            "public-binary-widened-signed-subtract-upper-padding-leak",
            "Value::U64(x.add_sub(y, width, signed, true))",
            "{ let mut result = x.add_sub(y, width, signed, true); if signed && width > 3 { result.payload |= 1u64 << 63; } Value::U64(result) }",
        ),
        (
            "public-binary-word64",
            "public-word64-adds-as-subtract",
            "Value::U64(x.add_sub(y, width, signed, false))",
            "Value::U64(x.add_sub(y, width, signed, true))",
        ),
        (
            "public-binary-word64",
            "public-word64-subtracts-as-add",
            "Value::U64(x.add_sub(y, width, signed, true))",
            "Value::U64(x.add_sub(y, width, signed, false))",
        ),
        (
            "public-binary-word64",
            "public-word64-drops-signed-subtract-metadata",
            "Value::U64(x.add_sub(y, width, signed, true))",
            "Value::U64(x.add_sub(y, width, false, true))",
        ),
    ] {
        if let Some(&contract) = selected.iter().find(|c| c.name == contract_name) {
            let path = "veryl/crates/analyzer/src/ir/op.rs";
            let source = fs::read_to_string(repo.join(path)).unwrap();
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_binary", old, new),
            });
        }
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-arithmetic-unary-native")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        for (name, old, new) in [
            (
                "public-native-unary-negate-drops-upper-payload",
                "Value::U64(x) => Value::U64(x.negate(width)),",
                "Value::U64(x) => { let mut result = x.negate(width); result.payload &= u32::MAX as u64; Value::U64(result) },",
            ),
            (
                "public-native-unary-plus-drops-upper-payload",
                "Op::Add => {\n                let x = x.expand(width, signed);\n                x.into_owned()\n            }",
                "Op::Add => { let mut result = x.expand(width, signed).into_owned(); if let Value::U64(x) = &mut result { x.payload &= u32::MAX as u64; } result }",
            ),
        ] {
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_unary", old, new),
            });
        }
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-shift-unsigned-native")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        for (name, old, new) in [
            (
                "public-shift-logical-left-as-right",
                "x.shift_left(y, width, false)",
                "x.shift_right(y, width, signed, false)",
            ),
            (
                "public-shift-logical-right-as-left",
                "x.shift_right(y, width, signed, false)",
                "x.shift_left(y, width, false)",
            ),
            (
                "public-shift-arithmetic-left-as-right",
                "x.shift_left(y, width, true)",
                "x.shift_right(y, width, signed, true)",
            ),
            (
                "public-shift-arithmetic-right-as-left",
                "x.shift_right(y, width, signed, true)",
                "x.shift_left(y, width, true)",
            ),
            (
                "public-shift-left-upper-padding-leak",
                "let ret = Value::U64(x.shift_left(y, width, false));",
                "let mut result = x.shift_left(y, width, false); if width < 64 { result.payload |= 1u64 << 63; } let ret = Value::U64(result);",
            ),
        ] {
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_binary", old, new),
            });
        }
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        for (name, old, new) in [
            (
                "public-shift-amount-ignores-unknown",
                "Self::U64(x) if x.is_xz() => None,",
                "Self::U64(x) if x.is_xz() && x.payload == 0 => None,",
            ),
            (
                "public-shift-amount-reinterprets-signed",
                "Self::U64(x) => Some(x.payload.to_usize().unwrap_or(usize::MAX)),",
                "Self::U64(x) => Some((x.payload as i64).to_usize().unwrap_or(0)),",
            ),
        ] {
            mutations.push(Mutation {
                name,
                contract,
                path: CORE,
                source: mutate_method(&source, "to_usize_saturating", old, new),
            });
        }
    }
    for (contract_name, name, path, method, old, new) in [
        (
            "public-small-shift-continuation-width-1",
            "public-small-shift-arithmetic-right-as-logical",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.shift_right(y, width, signed, true)",
            "x.shift_right(y, width, signed, false)",
        ),
        (
            "public-small-shift-continuation-width-2",
            "public-small-shift-Z-fill-as-X",
            CORE,
            "shift_right",
            "if payload_msb { ext_mask } else { 0 }",
            "if payload_msb && !mask_xz_msb { ext_mask } else { 0 }",
        ),
        (
            "public-small-shift-continuation-width-3",
            "public-small-shift-drops-XZ-fill",
            CORE,
            "shift_right",
            "ret.mask_xz |= ext_mask_xz;",
            "ret.mask_xz |= 0;",
        ),
        (
            "public-small-shift-continuation-width-4",
            "public-small-shift-arithmetic-left-clears-sign",
            CORE,
            "shift_left",
            "if !arithmetic {\n            ret.signed = false;\n        }",
            "ret.signed = false;",
        ),
        (
            "public-small-shift-continuation-width-4",
            "public-small-shift-ignores-Z-amount",
            CORE,
            "to_usize_saturating",
            "Self::U64(x) if x.is_xz() => None,",
            "Self::U64(x) if x.is_xz() && x.payload == 0 => None,",
        ),
    ] {
        if let Some(&contract) = selected.iter().find(|c| c.name == contract_name) {
            let source = fs::read_to_string(repo.join(path)).unwrap();
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, method, old, new),
            });
        }
    }
    for (contract_name, name, method, old, new) in [
        (
            "public-shift-boundary-width-31",
            "public-word-shift-reads-previous-sign-bit",
            "shift_right",
            "let payload_msb = ((self.payload >> (self.width - 1)) & 1) == 1;",
            "let payload_msb = ((self.payload >> self.width.saturating_sub(2)) & 1) == 1;",
        ),
        (
            "public-shift-boundary-width-32",
            "public-word-shift-XZ-fill-loses-bit-31",
            "shift_right",
            "ret.mask_xz |= ext_mask_xz;",
            "ret.mask_xz |= ext_mask_xz & 0x7fff_ffff;",
        ),
        (
            "public-shift-boundary-width-63",
            "public-word-shift-left-clamps-at-32",
            "shift_left",
            "let s = amount.min(64) as u32;",
            "let s = amount.min(32) as u32;",
        ),
        (
            "public-shift-boundary-width-64",
            "public-word-shift-word64-mask-loses-high-bit",
            "gen_mask",
            "u64::MAX",
            "u64::MAX >> 1",
        ),
        (
            "public-shift-boundary-width-64",
            "public-word-shift-signed-amount-as-negative",
            "to_usize_saturating",
            "Self::U64(x) => Some(x.payload.to_usize().unwrap_or(usize::MAX)),",
            "Self::U64(x) => Some(if x.signed { (x.payload as i64).to_usize().unwrap_or(0) } else { x.payload.to_usize().unwrap_or(usize::MAX) }),",
        ),
    ] {
        if let Some(&contract) = selected.iter().find(|c| c.name == contract_name) {
            let source = fs::read_to_string(repo.join(CORE)).unwrap();
            mutations.push(Mutation {
                name,
                contract,
                path: CORE,
                source: mutate_method(&source, method, old, new),
            });
        }
    }
    for (contract_name, name, old, new) in [
        (
            "public-small-divide-width-1-unsigned",
            "public-small-divide-zero-divisor-as-zero",
            "Value::U64(x.divide_remainder(y, width, signed, false))",
            "Value::U64(if y.payload == 0 && y.mask_xz == 0 { ValueU64::new(0, width, signed) } else { x.divide_remainder(y, width, signed, false) })",
        ),
        (
            "public-small-divide-width-2-signed",
            "public-small-divide-upper-padding-leak",
            "Value::U64(x.divide_remainder(y, width, signed, false))",
            "{ let mut result = x.divide_remainder(y, width, signed, false); result.payload |= 1u64 << 63; Value::U64(result) }",
        ),
        (
            "public-small-divide-width-3-unsigned",
            "public-small-divide-as-remainder",
            "Value::U64(x.divide_remainder(y, width, signed, false))",
            "Value::U64(x.divide_remainder(y, width, signed, true))",
        ),
        (
            "public-small-divide-width-4-signed",
            "public-small-divide-skips-left-sign-extension",
            "Op::Div => {\n                let x = x.expand(width, signed);",
            "Op::Div => {\n                let x = x.expand(width, false);",
        ),
    ] {
        if let Some(&contract) = selected.iter().find(|c| c.name == contract_name) {
            let path = "veryl/crates/analyzer/src/ir/op.rs";
            let source = fs::read_to_string(repo.join(path)).unwrap();
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_binary", old, new),
            });
        }
    }
    for (contract_name, name, old, new) in [
        (
            "public-small-remainder-width-1-unsigned",
            "public-small-remainder-zero-divisor-as-zero",
            "Value::U64(x.divide_remainder(y, width, signed, true))",
            "{ let mut result = x.divide_remainder(y, width, signed, true); if y.payload == 0 && y.mask_xz == 0 { result.mask_xz = 0; } Value::U64(result) }",
        ),
        (
            "public-small-remainder-width-2-signed",
            "public-small-remainder-upper-padding-leak",
            "Value::U64(x.divide_remainder(y, width, signed, true))",
            "{ let mut result = x.divide_remainder(y, width, signed, true); result.payload |= 1u64 << 63; Value::U64(result) }",
        ),
        (
            "public-small-remainder-width-3-unsigned",
            "public-small-remainder-as-quotient",
            "Value::U64(x.divide_remainder(y, width, signed, true))",
            "Value::U64(x.divide_remainder(y, width, signed, false))",
        ),
        (
            "public-small-remainder-width-4-signed",
            "public-small-remainder-skips-left-sign-extension",
            "Op::Rem => {\n                let x = x.expand(width, signed);",
            "Op::Rem => {\n                let x = x.expand(width, false);",
        ),
    ] {
        if let Some(&contract) = selected.iter().find(|c| c.name == contract_name) {
            let path = "veryl/crates/analyzer/src/ir/op.rs";
            let source = fs::read_to_string(repo.join(path)).unwrap();
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_binary", old, new),
            });
        }
    }
    if let Some(&contract) = selected.iter().find(|c| c.name == "public-small-multiply") {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        for (name, old, new) in [
            (
                "public-small-multiply-as-add",
                "Value::U64(x.multiply(y, width, signed))",
                "Value::U64(x.add_sub(y, width, signed, false))",
            ),
            (
                "public-small-multiply-padding-leak",
                "Value::U64(x.multiply(y, width, signed))",
                "{ let mut result = x.multiply(y, width, signed); result.payload |= 1u64 << 63; Value::U64(result) }",
            ),
            (
                "public-small-multiply-skips-left-sign-extension",
                "Op::Mul => {\n                let x = x.expand(width, signed);",
                "Op::Mul => {\n                let x = x.expand(width, false);",
            ),
        ] {
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_binary", old, new),
            });
        }
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-native-zero-extension-wide")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        for (name, old, new) in [
            (
                "public-wide-zero-extension-loses-payload",
                "let mut payload = Box::new(BigUint::from(x.payload));",
                "let mut payload = Box::new(BigUint::from(0u64));",
            ),
            (
                "public-wide-zero-extension-loses-xz",
                "let mut mask_xz = Box::new(BigUint::from(x.mask_xz));",
                "let mut mask_xz = Box::new(BigUint::from(0u64));",
            ),
            (
                "public-wide-zero-extension-retains-sign",
                "let signed = if use_sign { x.signed } else { false };",
                "let signed = x.signed;",
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
        .find(|c| c.name == "public-native-zero-extension-wide")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        let old = "                    ValueBigUint {\n                        payload,\n                        mask_xz,\n                        width: width as u32,\n                        signed,\n                    }\n                }\n                Self::BigUint(x) => {";
        let new = "                    ValueBigUint {\n                        payload,\n                        mask_xz,\n                        width: 64,\n                        signed,\n                    }\n                }\n                Self::BigUint(x) => {";
        mutations.push(Mutation {
            name: "public-wide-zero-extension-reports-native-width",
            contract,
            path: CORE,
            source: replace_once(&source, old, new),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-native-zero-extension-wide")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "public-word64-zero-extension-loses-high-payload",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "let mut payload = Box::new(BigUint::from(x.payload));",
                "let mut payload = Box::new(BigUint::from(x.payload & !(1u64 << 63)));",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-native-zero-extension-wide")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "public-word64-zero-extension-loses-high-mask_xz",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "let mut mask_xz = Box::new(BigUint::from(x.mask_xz));",
                "let mut mask_xz = Box::new(BigUint::from(x.mask_xz & !(1u64 << 63)));",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-native-zero-fill-extension-wide")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "public-zero-fill-extension-clears-sign",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "let signed = if use_sign { x.signed } else { false };",
                "let signed = false;",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-native-zero-fill-extension-wide")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "public-zero-fill-extension-empty-digits-as-one",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "        0 => 0,\n        1 => digits[0] as u128,",
                "        0 => 1,\n        1 => digits[0] as u128,",
            ),
        });
    }
    if let Some(&contract) = selected.iter().find(|c| c.name == "public-big-getters-128") {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "public-big-getters-drops-high-word",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "_ => (digits[0] as u128) | ((digits[1] as u128) << 64),",
                "_ => digits[0] as u128,",
            ),
        });
    }
    if let Some(&contract) = selected.iter().find(|c| c.name == "public-big-getters-128") {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "public-big-getters-xz-reads-payload",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "Self::BigUint(x) => biguint_to_u128(&x.mask_xz),",
                "Self::BigUint(x) => biguint_to_u128(&x.payload),",
            ),
        });
    }
    if let Some(&contract) = selected.iter().find(|c| c.name == "public-arithmetic-add") {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        for (name, old, new) in [
            (
                "public-add-upper-payload-leak",
                "Self::U64(x) => x.payload as u128,",
                "Self::U64(x) => (x.payload as u128) | if x.width == 5 { 1u128 << 100 } else { 0 },",
            ),
            (
                "public-add-upper-xz-leak",
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
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-selection-native")
    {
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
            (
                "public-selection-drops-upper-source",
                "Self::U64(x) => { let mut x = x.clone(); x.payload &= u32::MAX as u64; Self::U64(x.select(beg, end)) },",
            ),
            (
                "public-selection-missing-bits-are-zero",
                "Self::U64(x) => { let mut ret = x.select(beg, end); if end >= x.width as usize { ret.mask_xz = 0; } Self::U64(ret) },",
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
    if let Some(&contract) = selected.iter().find(|c| c.name == "public-resize-native") {
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
        for (name, old, new) in [
            (
                "public-resize-constructor-drops-upper-payload",
                "payload: payload as u64,",
                "payload: payload as u32 as u64,",
            ),
            (
                "public-resize-constructor-drops-upper-xz",
                "mask_xz: mask_xz as u64,",
                "mask_xz: mask_xz as u32 as u64,",
            ),
        ] {
            mutations.push(Mutation {
                name,
                contract,
                path: CORE,
                source: mutate_method(&source, "from_u128", old, new),
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
