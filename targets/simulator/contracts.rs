//! Host catalogs and production mutations for binary kernel contracts.
//!
//! Kani and native adapters share definitions but establish different guarantees.
//! Mutation recipes must reach the selected semantic assertion; build, safety,
//! route, and timeout failures alone are not evidence that the contract detected
//! an incorrect result. Snapshots change production copies, never reference models.

use std::fs;
use std::path::Path;
use veryl_formal_runner::{Contract, Mutation};

const ARITHMETIC_COVERS: &[&str] = &[
    "arithmetic first-word carry or borrow is reachable",
    "arithmetic second-word carry or borrow is reachable",
    "arithmetic carry or borrow chain is reachable",
];
const fn arithmetic_contract(name: &'static str, harness: &'static str) -> Contract {
    Contract {
        name,
        group: "arithmetic",
        harness,
        message: "simulator arithmetic semantics mismatch",
        covers: ARITHMETIC_COVERS,
    }
}

const SHIFT_COVERS: &[&str] = &[
    "shift identity is reachable",
    "shift one bit is reachable",
    "shift below word boundary is reachable",
    "shift word boundary is reachable",
    "shift above word boundary is reachable",
    "shift below two words is reachable",
    "shift two words is reachable",
    "shift last storage bit is reachable",
    "shift full capacity is reachable",
    "shift above 32-bit amount is reachable",
    "shift high amount bit is reachable",
    "shift maximum amount is reachable",
    "shift nonzero across word boundary is reachable",
    "shift prefix guard is reachable",
    "shift suffix guard is reachable",
];
const fn shift_contract(name: &'static str, harness: &'static str, proof: bool) -> Contract {
    Contract {
        name,
        group: "shift",
        harness,
        message: "simulator logical shift semantics mismatch",
        covers: if proof { SHIFT_COVERS } else { &[] },
    }
}

const SIGNED_SMALL_COVERS: &[&str] = &[
    "signed shift identity is reachable",
    "signed shift negative one-bit fill is reachable",
    "signed shift negative byte fill is reachable",
    "signed shift positive byte fill is reachable",
    "signed shift prefix guard is reachable",
    "signed shift suffix guard is reachable",
];
const SIGNED_NONNEGATIVE_COVERS: &[&str] = &[
    "signed shift identity is reachable",
    "signed shift word boundary is reachable",
    "signed shift above word boundary is reachable",
    "signed shift last bit is reachable",
    "signed shift oversized positive fill is reachable",
    "signed shift prefix guard is reachable",
    "signed shift suffix guard is reachable",
];
const fn signed_shift_contract(
    name: &'static str,
    harness: &'static str,
    covers: &'static [&'static str],
) -> Contract {
    Contract {
        name,
        group: "signed-shift",
        harness,
        message: "simulator arithmetic shift semantics mismatch",
        covers,
    }
}

const BINARY_BITWISE_COVERS: &[&str] = &[
    "bitwise empty storage is reachable",
    "bitwise one word is reachable",
    "bitwise four disjoint words are reachable",
    "bitwise left alias is reachable",
    "bitwise unaligned left source is reachable",
    "bitwise word boundary is reachable",
    "bitwise last bit is reachable",
    "bitwise prefix guard is reachable",
    "bitwise suffix guard is reachable",
    "bitwise right alias is reachable",
    "bitwise double alias is reachable",
    "bitwise unaligned right source is reachable",
    "bitwise inputs zero zero are reachable",
    "bitwise inputs zero one are reachable",
    "bitwise inputs one zero are reachable",
    "bitwise inputs one one are reachable",
];
const UNARY_BITWISE_COVERS: &[&str] = &[
    "bitwise empty storage is reachable",
    "bitwise one word is reachable",
    "bitwise four disjoint words are reachable",
    "bitwise left alias is reachable",
    "bitwise unaligned left source is reachable",
    "bitwise word boundary is reachable",
    "bitwise last bit is reachable",
    "bitwise prefix guard is reachable",
    "bitwise suffix guard is reachable",
    "bitwise unary zero is reachable",
    "bitwise unary one is reachable",
];
const fn bitwise_contract(
    name: &'static str,
    harness: &'static str,
    covers: &'static [&'static str],
) -> Contract {
    Contract {
        name,
        group: "bitwise",
        harness,
        message: "simulator bitwise semantics mismatch",
        covers,
    }
}

const WINDOW_CROSS_COVERS: &[&str] = &[
    "window first written bit is reachable",
    "window last written bit is reachable",
    "window preceding bit is reachable",
    "window prefix guard is reachable",
    "window suffix guard is reachable",
    "window source word boundary is reachable",
];
const WINDOW_ALIAS_COVERS: &[&str] = &[
    "window first written bit is reachable",
    "window last written bit is reachable",
    "window preceding bit is reachable",
    "window prefix guard is reachable",
    "window suffix guard is reachable",
    "window source word boundary is reachable",
    "window alias snapshot is reachable",
];
const WINDOW_ALIGNED_COVERS: &[&str] = &[
    "window first written bit is reachable",
    "window last written bit is reachable",
    "window preceding bit is reachable",
    "window prefix guard is reachable",
    "window suffix guard is reachable",
    "window following bit is reachable",
];
const WINDOW_CLIPPED_COVERS: &[&str] = &[
    "window first written bit is reachable",
    "window last written bit is reachable",
    "window preceding bit is reachable",
    "window prefix guard is reachable",
    "window suffix guard is reachable",
    "window dirty padding is preserved",
];
const fn window_contract(
    name: &'static str,
    harness: &'static str,
    covers: &'static [&'static str],
) -> Contract {
    Contract {
        name,
        group: "window",
        harness,
        message: "simulator window store semantics mismatch",
        covers,
    }
}

const WINDOW_LOG_COVERS: &[&str] = &[
    "window log zero offset is reachable",
    "window log highest safe offset is reachable",
    "window log unaligned offset is reachable",
    "window log first payload byte is reachable",
    "window log last payload byte is reachable",
    "window log nonidentity merge is reachable",
    "window log prefix guard is reachable",
    "window log suffix guard is reachable",
];
const fn window_log_contract(
    name: &'static str,
    harness: &'static str,
    covers: &'static [&'static str],
) -> Contract {
    Contract {
        name,
        group: "window-log",
        harness,
        message: "simulator window log semantics mismatch",
        covers,
    }
}

const COMMIT_WIDE_COVERS: &[&str] = &[
    "wide commit exact end is reachable",
    "wide commit unaligned is reachable",
    "wide commit outside range is reachable",
    "wide commit unchanged is reachable",
    "wide commit changed is reachable",
];
const COMMIT_ORDER_COVERS: &[&str] = &[
    "commit repeated byte differs is reachable",
    "commit restored value still marks is reachable",
    "commit all writes unchanged is reachable",
];
const fn commit_contract(
    name: &'static str,
    harness: &'static str,
    covers: &'static [&'static str],
    order: bool,
) -> Contract {
    Contract {
        name,
        group: "commit",
        harness,
        message: if order {
            "simulator commit order mismatch"
        } else {
            "simulator commit semantics mismatch"
        },
        covers,
    }
}

pub static KANI: [Contract; 45] = [
    commit_contract(
        "commit-narrow",
        "narrow_commit_matches_reference",
        &[
            "commit one byte is reachable",
            "commit two bytes is reachable",
            "commit four bytes is reachable",
            "commit unaligned eight bytes is reachable",
            "commit outside range is reachable",
            "commit unchanged value is reachable",
            "commit changed value is reachable",
        ],
        false,
    ),
    commit_contract(
        "commit-wide3",
        "wide_three_commit_matches_reference",
        COMMIT_WIDE_COVERS,
        false,
    ),
    commit_contract(
        "commit-wide7",
        "wide_seven_commit_matches_reference",
        COMMIT_WIDE_COVERS,
        false,
    ),
    commit_contract(
        "commit-wide15",
        "wide_fifteen_commit_matches_reference",
        COMMIT_WIDE_COVERS,
        false,
    ),
    commit_contract(
        "commit-wide17",
        "large_seventeen_commit_matches_reference",
        COMMIT_WIDE_COVERS,
        false,
    ),
    commit_contract(
        "commit-wide56",
        "large_fifty_six_commit_matches_reference",
        COMMIT_WIDE_COVERS,
        false,
    ),
    commit_contract(
        "commit-narrow-order",
        "narrow_commit_order_matches_reference",
        COMMIT_ORDER_COVERS,
        true,
    ),
    commit_contract(
        "commit-wide-order",
        "wide_commit_order_matches_reference",
        COMMIT_ORDER_COVERS,
        true,
    ),
    commit_contract(
        "commit-mixed-order",
        "mixed_commit_order_matches_reference",
        &[
            "commit repeated byte differs is reachable",
            "commit restored value still marks is reachable",
            "commit all writes unchanged is reachable",
            "commit wide pool wins over later narrow push is reachable",
        ],
        true,
    ),
    window_log_contract(
        "log-only",
        "window_log_only_matches_reference",
        WINDOW_LOG_COVERS,
    ),
    window_log_contract(
        "log-only-alias",
        "window_log_only_alias_matches_reference",
        WINDOW_LOG_COVERS,
    ),
    window_log_contract(
        "log-direct",
        "window_log_direct_matches_reference",
        WINDOW_LOG_COVERS,
    ),
    window_log_contract(
        "log-direct-alias",
        "window_log_direct_alias_matches_reference",
        WINDOW_LOG_COVERS,
    ),
    window_log_contract(
        "log-noop",
        "window_log_empty_noop_matches_reference",
        &[
            "window log zero width is reachable",
            "window log empty destination is reachable",
            "window log outside destination is reachable",
            "window log maximum amount is reachable",
        ],
    ),
    window_contract(
        "window-cross",
        "wide_window_cross_words_matches_reference",
        WINDOW_CROSS_COVERS,
    ),
    window_contract(
        "window-cross-alias",
        "wide_window_cross_words_alias_matches_reference",
        WINDOW_ALIAS_COVERS,
    ),
    window_contract(
        "window-aligned",
        "wide_window_aligned_matches_reference",
        WINDOW_ALIGNED_COVERS,
    ),
    window_contract(
        "window-aligned-alias",
        "wide_window_aligned_alias_matches_reference",
        WINDOW_ALIGNED_COVERS,
    ),
    window_contract(
        "window-clipped",
        "wide_window_clipped_matches_reference",
        WINDOW_CLIPPED_COVERS,
    ),
    window_contract(
        "window-clipped-alias",
        "wide_window_clipped_alias_matches_reference",
        WINDOW_CLIPPED_COVERS,
    ),
    Contract {
        name: "window-noop",
        group: "window",
        harness: "wide_window_noop_preserves_storage",
        message: "simulator window noop semantics mismatch",
        covers: &[
            "window zero width is reachable",
            "window empty destination is reachable",
            "window outside destination is reachable",
            "window maximum amount is reachable",
        ],
    },
    bitwise_contract(
        "wide-band",
        "wide_band_matches_reference",
        BINARY_BITWISE_COVERS,
    ),
    bitwise_contract(
        "wide-bor",
        "wide_bor_matches_reference",
        BINARY_BITWISE_COVERS,
    ),
    bitwise_contract(
        "wide-bxor",
        "wide_bxor_matches_reference",
        BINARY_BITWISE_COVERS,
    ),
    bitwise_contract(
        "wide-bxor-not",
        "wide_bxor_not_matches_reference",
        BINARY_BITWISE_COVERS,
    ),
    bitwise_contract(
        "wide-band-not",
        "wide_band_not_matches_reference",
        BINARY_BITWISE_COVERS,
    ),
    bitwise_contract(
        "wide-bnot",
        "wide_bnot_matches_reference",
        UNARY_BITWISE_COVERS,
    ),
    signed_shift_contract(
        "wide-ashr-small",
        "wide_ashr_small_matches_reference",
        SIGNED_SMALL_COVERS,
    ),
    signed_shift_contract(
        "wide-ashr-small-inplace",
        "wide_ashr_small_inplace_matches_reference",
        SIGNED_SMALL_COVERS,
    ),
    signed_shift_contract(
        "wide-ashr-nonnegative",
        "wide_ashr_nonnegative_matches_reference",
        SIGNED_NONNEGATIVE_COVERS,
    ),
    signed_shift_contract(
        "wide-ashr-nonnegative-inplace",
        "wide_ashr_nonnegative_inplace_matches_reference",
        SIGNED_NONNEGATIVE_COVERS,
    ),
    Contract {
        name: "wide-ashr-empty",
        group: "signed-shift",
        harness: "wide_ashr_empty_metadata_preserves_storage",
        message: "simulator empty shift semantics mismatch",
        covers: &[
            "signed shift zero width is reachable",
            "signed shift zero storage is reachable",
            "signed shift empty metadata is reachable",
        ],
    },
    shift_contract("wide-shl", "wide_shl_matches_reference", true),
    shift_contract("wide-lshr", "wide_lshr_matches_reference", true),
    shift_contract(
        "wide-shl-inplace",
        "wide_shl_inplace_matches_reference",
        true,
    ),
    shift_contract(
        "wide-lshr-inplace",
        "wide_lshr_inplace_matches_reference",
        true,
    ),
    arithmetic_contract("wide-add", "wide_add_three_words_matches_reference"),
    arithmetic_contract("wide-sub", "wide_sub_three_words_matches_reference"),
    arithmetic_contract(
        "wide-add-left-alias",
        "wide_add_left_alias_matches_reference",
    ),
    arithmetic_contract(
        "wide-add-right-alias",
        "wide_add_right_alias_matches_reference",
    ),
    arithmetic_contract(
        "wide-sub-left-alias",
        "wide_sub_left_alias_matches_reference",
    ),
    arithmetic_contract(
        "wide-sub-right-alias",
        "wide_sub_right_alias_matches_reference",
    ),
    Contract {
        name: "wide-mask",
        group: "memory",
        harness: "wide_mask_matches_reference",
        message: "simulator mask semantics mismatch",
        covers: &[
            "mask empty storage is reachable",
            "mask zero width sentinel is reachable",
            "mask width 63 is reachable",
            "mask word boundary is reachable",
            "mask width 65 is reachable",
            "mask full capacity is reachable",
            "mask above capacity is reachable",
            "mask unaligned destination is reachable",
            "mask prefix guard is reachable",
            "mask suffix guard is reachable",
        ],
    },
    Contract {
        name: "wide-copy",
        group: "memory",
        harness: "wide_copy_matches_reference",
        message: "simulator copy semantics mismatch",
        covers: &[
            "copy empty buffer is reachable",
            "copy four words is reachable",
            "copy exact alias is reachable",
            "copy unaligned source is reachable",
            "copy unaligned destination is reachable",
            "copy prefix guard is reachable",
            "copy suffix guard is reachable",
        ],
    },
    Contract {
        name: "wide-resize",
        group: "memory",
        harness: "wide_resize_matches_reference",
        message: "simulator resize semantics mismatch",
        covers: &[
            "resize empty source is reachable",
            "resize empty destination is reachable",
            "resize signed one-bit growth is reachable",
            "resize width 63 is reachable",
            "resize width 64 is reachable",
            "resize width 65 is reachable",
            "resize above native width is reachable",
            "resize shrinking is reachable",
            "resize exact alias is reachable",
            "resize unaligned source is reachable",
            "resize unaligned destination is reachable",
            "resize prefix guard is reachable",
            "resize suffix guard is reachable",
        ],
    },
];
pub static NATIVE: [Contract; 26] = [
    commit_contract(
        "commit-bounds",
        "commit::entry_sizes_and_bounds",
        &[],
        false,
    ),
    commit_contract(
        "commit-mixed",
        "commit::mixed_order_and_marking",
        &[],
        false,
    ),
    commit_contract(
        "commit-history",
        "commit::restoration_and_live_prefix",
        &[],
        false,
    ),
    commit_contract(
        "commit-marking",
        "commit::marking_sees_each_byte",
        &[],
        false,
    ),
    window_log_contract("log-records", "window_log::records_and_modes", &[]),
    window_log_contract(
        "log-history",
        "window_log::growth_and_append_preserve_history",
        &[],
    ),
    window_log_contract(
        "log-noop",
        "window_log::prefilled_noops_preserve_entries",
        &[],
    ),
    window_contract("window-small", "window::small_disjoint_and_alias", &[]),
    window_contract(
        "window-boundaries",
        "window::word_boundaries_clipping_and_heap",
        &[],
    ),
    Contract {
        name: "window-overlap",
        group: "window",
        harness: "window::shared_allocation_overlap_uses_snapshot",
        message: "simulator window overlap semantics mismatch",
        covers: &[],
    },
    bitwise_contract("wide-band", "bitwise_and_buffers_and_alias", &[]),
    bitwise_contract("wide-bor", "bitwise_or_buffers_and_alias", &[]),
    bitwise_contract("wide-bxor", "bitwise_xor_buffers_and_alias", &[]),
    bitwise_contract("wide-bxor-not", "bitwise_xnor_buffers_and_alias", &[]),
    bitwise_contract("wide-band-not", "bitwise_and_not_buffers_and_alias", &[]),
    bitwise_contract("wide-bnot", "bitwise_not_buffers_and_alias", &[]),
    signed_shift_contract("wide-ashr", "arithmetic_right_shift_storage_and_alias", &[]),
    Contract {
        name: "wide-ashr-prepared",
        group: "signed-shift",
        harness: "arithmetic_right_shift_preparation_and_mask",
        message: "simulator prepared shift semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "wide-ashr-empty",
        group: "signed-shift",
        harness: "arithmetic_right_shift_empty_metadata",
        message: "simulator empty shift semantics mismatch",
        covers: &[],
    },
    shift_contract("wide-shl", "logical_left_shift_boundaries_and_alias", false),
    shift_contract(
        "wide-lshr",
        "logical_right_shift_boundaries_and_alias",
        false,
    ),
    Contract {
        name: "wide-mask",
        group: "memory",
        harness: "mask_width_sentinel_padding_and_guards",
        message: "simulator mask semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "wide-copy",
        group: "memory",
        harness: "copy_disjoint_unaligned_and_exact_alias",
        message: "simulator copy semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "wide-resize",
        group: "memory",
        harness: "resize_minimal_source_dirty_padding_and_guards",
        message: "simulator resize semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "wide-resize-alias",
        group: "memory",
        harness: "resize_exact_alias_grows_and_shrinks",
        message: "simulator inplace resize semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "wide-arithmetic",
        group: "arithmetic",
        harness: "modular_add_sub_unaligned_and_alias",
        message: "simulator arithmetic semantics mismatch",
        covers: &[],
    },
];
fn replace(source: &str, function: &str, old: &str, new: &str, count: usize) -> String {
    let declaration = if function == "sext_word" {
        format!("unsafe fn {function}(")
    } else {
        format!("pub unsafe extern \"C\" fn {function}(")
    };
    let start = source.find(&declaration).expect("mutation function moved");
    let end = source[start + declaration.len()..]
        .find("\npub unsafe extern \"C\" fn ")
        .map(|i| start + declaration.len() + i)
        .unwrap_or(source.len());
    assert_eq!(
        source[start..end].matches(old).count(),
        count,
        "mutation site changed: {function}: {old}"
    );
    let replacement = source[start..end].replace(old, new);
    format!("{}{}{}", &source[..start], replacement, &source[end..])
}
pub fn mutations(repo: &Path, selected: &[&'static Contract], kani: bool) -> Vec<Mutation> {
    let path = "veryl/crates/simulator/src/wide_ops.rs";
    let original = fs::read_to_string(repo.join(path)).unwrap();
    let mut result = Vec::new();
    for (name, proof, native, function, old, new, count) in [
        (
            "copy-skips-final-word",
            "wide-copy",
            "wide-copy",
            "wide_copy",
            "for i in 0..nw(nb)",
            "for i in 0..nw(nb).saturating_sub(1)",
            1,
        ),
        (
            "copy-repeats-first-word",
            "wide-copy",
            "wide-copy",
            "wide_copy",
            "rd(src, i)",
            "rd(src, 0)",
            1,
        ),
        (
            "resize-fills-empty-with-ones",
            "wide-resize",
            "wide-resize",
            "wide_resize",
            "wr(dst, i, 0);",
            "wr(dst, i, u64::MAX);",
            1,
        ),
        (
            "resize-disables-sign-extension",
            "wide-resize",
            "wide-resize",
            "wide_resize",
            "let signed = (src_info >> 32) & 1 == 1;",
            "let signed = false;",
            1,
        ),
        (
            "resize-keeps-dirty-padding",
            "wide-resize",
            "wide-resize",
            "sext_word",
            "(raw & mask)",
            "raw",
            1,
        ),
        (
            "resize-skips-final-word",
            "wide-resize",
            "wide-resize",
            "wide_resize",
            "wr(dst, i, sext_word(src, i, src_w, sign));",
            "if i+1<nw(dst_nb) { wr(dst, i, sext_word(src, i, src_w, sign)); }",
            1,
        ),
        (
            "resize-clears-alias-before-read",
            "wide-resize",
            "wide-resize-alias",
            "wide_resize",
            "record(ProfOp::Resize);",
            "record(ProfOp::Resize); unsafe {for i in 0..nw(dst_nb) {wr(dst,i,0);}}",
            1,
        ),
        (
            "addition-loses-word-carry",
            "wide-add",
            "wide-arithmetic",
            "wide_add",
            "carry = (c1 as u64) + (c2 as u64);",
            "carry = 0;",
            1,
        ),
        (
            "subtraction-loses-word-borrow",
            "wide-sub",
            "wide-arithmetic",
            "wide_sub",
            "borrow = (b1 as u64) + (b2 as u64);",
            "borrow = 0;",
            1,
        ),
        (
            "subtraction-adds-instead",
            "wide-sub",
            "wide-arithmetic",
            "wide_sub",
            "overflowing_sub",
            "overflowing_add",
            2,
        ),
        (
            "shl-drops-neighbor",
            "wide-shl",
            "wide-shl",
            "wide_shl",
            "(lo << bit_shift) | (hi >> (64 - bit_shift))",
            "lo << bit_shift",
            1,
        ),
        (
            "lshr-drops-neighbor",
            "wide-lshr",
            "wide-lshr",
            "wide_lshr",
            "(lo >> bit_shift) | (hi << (64 - bit_shift))",
            "lo >> bit_shift",
            1,
        ),
        (
            "shl-wrong-word-offset",
            "wide-shl",
            "wide-shl",
            "wide_shl",
            "amount / 64",
            "amount / 32",
            1,
        ),
        (
            "lshr-wrong-word-offset",
            "wide-lshr",
            "wide-lshr",
            "wide_lshr",
            "amount / 64",
            "amount / 32",
            1,
        ),
        (
            "shl-oversized-fills-ones",
            "wide-shl",
            "wide-shl",
            "wide_shl",
            "wr(dst, i, 0);",
            "wr(dst, i, u64::MAX);",
            1,
        ),
        (
            "lshr-oversized-fills-ones",
            "wide-lshr",
            "wide-lshr",
            "wide_lshr",
            "wr(dst, i, 0);",
            "wr(dst, i, u64::MAX);",
            1,
        ),
        (
            "shl-truncates-amount",
            "wide-shl",
            "wide-shl",
            "wide_shl",
            "(amount / 64) as usize",
            "((amount as u32 as u64) / 64) as usize",
            1,
        ),
        (
            "lshr-truncates-amount",
            "wide-lshr",
            "wide-lshr",
            "wide_lshr",
            "(amount / 64) as usize",
            "((amount as u32 as u64) / 64) as usize",
            1,
        ),
        (
            "shl-corrupts-word-shift",
            "wide-shl",
            "wide-shl",
            "wide_shl",
            "if bit_shift == 0 {\n                    lo",
            "if bit_shift == 0 {\n                    lo ^ 1",
            1,
        ),
        (
            "lshr-corrupts-word-shift",
            "wide-lshr",
            "wide-lshr",
            "wide_lshr",
            "if bit_shift == 0 {\n                    lo",
            "if bit_shift == 0 {\n                    lo ^ 1",
            1,
        ),
        (
            "shl-reverses-alias-order",
            "wide-shl-inplace",
            "wide-shl",
            "wide_shl",
            "for i in (0..n).rev() {",
            "for i in 0..n {",
            1,
        ),
        (
            "lshr-reverses-alias-order",
            "wide-lshr-inplace",
            "wide-lshr",
            "wide_lshr",
            "for i in 0..n {\n            let src_idx",
            "for i in (0..n).rev() {\n            let src_idx",
            1,
        ),
        (
            "ashr-disables-sign-fill",
            "wide-ashr-small",
            "wide-ashr",
            "wide_ashr",
            "if sign == 1 && amount > 0 {",
            "if false {",
            1,
        ),
        (
            "ashr-wrong-sign-bit",
            "wide-ashr-small",
            "wide-ashr",
            "wide_ashr",
            "let sign_bit = (width as usize - 1) % 64;",
            "let sign_bit = ((width as usize - 1) % 64) ^ 1;",
            1,
        ),
        (
            "ashr-misses-first-fill-bit",
            "wide-ashr-small",
            "wide-ashr",
            "wide_ashr",
            "(width as u64 - amount) as usize",
            "(width as u64 - amount) as usize + 1",
            1,
        ),
        (
            "ashr-clears-shifted-bits",
            "wide-ashr-small",
            "wide-ashr",
            "wide_ashr",
            "| (1u64 << bit)",
            "& (1u64 << bit)",
            1,
        ),
        (
            "ashr-reads-sign-after-shift",
            "wide-ashr-small-inplace",
            "wide-ashr",
            "wide_ashr",
            "wide_lshr(dst, a, amount, nb);",
            "wide_lshr(dst, a, amount, nb); let sign = (rd(a, sign_word) >> sign_bit) & 1;",
            1,
        ),
        (
            "ashr-reverses-inner-alias-order",
            "wide-ashr-small-inplace",
            "wide-ashr",
            "wide_lshr",
            "for i in 0..n {\n            let src_idx",
            "for i in (0..n).rev() {\n            let src_idx",
            1,
        ),
        (
            "ashr-misses-lowest-negative-bit",
            "",
            "wide-ashr",
            "wide_ashr",
            "if amount >= width as u64 {\n                0",
            "if amount >= width as u64 {\n                1",
            1,
        ),
        (
            "ashr-shifts-wrong-direction",
            "wide-ashr-nonnegative",
            "wide-ashr",
            "wide_ashr",
            "wide_lshr(dst, a, amount, nb);",
            "wide_shl(dst, a, amount, nb);",
            1,
        ),
        (
            "ashr-truncates-amount",
            "wide-ashr-nonnegative",
            "wide-ashr",
            "wide_ashr",
            "wide_lshr(dst, a, amount, nb);",
            "wide_lshr(dst, a, amount as u32 as u64, nb);",
            1,
        ),
        (
            "ashr-nonnegative-alias-order",
            "wide-ashr-nonnegative-inplace",
            "",
            "wide_lshr",
            "for i in 0..n {\n            let src_idx",
            "for i in (0..n).rev() {\n            let src_idx",
            1,
        ),
        (
            "ashr-clears-empty-width",
            "wide-ashr-empty",
            "wide-ashr-empty",
            "wide_ashr",
            "if nb == 0 || width == 0 {",
            "if width == 0 {unsafe {for i in 0..nw(nb) {wr(dst,i,0);}}} if nb == 0 || width == 0 {",
            1,
        ),
        (
            "ashr-preparation-loses-sign",
            "",
            "wide-ashr-prepared",
            "wide_resize",
            "let signed = (src_info >> 32) & 1 == 1;",
            "let signed = false;",
            1,
        ),
        (
            "ashr-preparation-keeps-dirty-padding",
            "",
            "wide-ashr-prepared",
            "sext_word",
            "(raw & mask)",
            "raw",
            1,
        ),
        (
            "ashr-result-keeps-padding",
            "",
            "wide-ashr-prepared",
            "wide_apply_mask",
            "& mask",
            "| mask",
            1,
        ),
        (
            "window-loses-untouched-bits",
            "window-cross",
            "window-small",
            "wide_window_store",
            "(old & !mask)",
            "0",
            1,
        ),
        (
            "window-drops-source-high-word",
            "window-cross",
            "window-boundaries",
            "wide_window_store",
            "let s_hi = if sw + 1 < src_words {",
            "let s_hi = if false {",
            1,
        ),
        (
            "window-skips-final-word",
            "window-cross",
            "window-small",
            "wide_window_store",
            "wr(dst, w0 + k, v);",
            "if k+1<n {wr(dst, w0+k, v);}",
            1,
        ),
        (
            "window-disables-direct-write",
            "window-cross-alias",
            "window-small",
            "wide_window_store",
            "let in_place = (log_and_flags >> 32) & 1 == 1;",
            "let in_place = false;",
            1,
        ),
        (
            "window-writes-before-source-snapshot",
            "window-cross-alias",
            "window-overlap",
            "wide_window_store",
            "*out = (old & !mask) | ((sv << shift) & mask);",
            "*out = (old & !mask) | ((sv << shift) & mask); if in_place {wr(dst,word,*out);}",
            1,
        ),
        (
            "window-full-word-mask-misses-top-bit",
            "window-aligned",
            "window-boundaries",
            "wide_window_store",
            "if cnt >= 64 {\n                u64::MAX",
            "if cnt >= 64 {\n                u64::MAX >> 1",
            1,
        ),
        (
            "window-ignores-first-word-offset",
            "window-aligned",
            "window-boundaries",
            "wide_window_store",
            "wr(dst, w0 + k, v);",
            "wr(dst,k,v);",
            1,
        ),
        (
            "window-clears-source-alias-first",
            "window-aligned-alias",
            "window-small",
            "wide_window_store",
            "let width = width.min(dst_width - amount);",
            "let width = width.min(dst_width - amount); if in_place {unsafe {wr(dst,0,0);}}",
            1,
        ),
        (
            "window-extends-last-word-mask",
            "window-clipped",
            "window-small",
            "wide_window_store",
            "let hi = ((word + 1) * 64).min(amount + width);",
            "let hi = (word + 1) * 64;",
            1,
        ),
        (
            "window-omits-mask-offset",
            "window-clipped-alias",
            "window-small",
            "wide_window_store",
            "((1u64 << cnt) - 1) << shift",
            "(1u64 << cnt) - 1",
            1,
        ),
        (
            "window-clears-zero-width",
            "window-noop",
            "window-small",
            "wide_window_store",
            "if width == 0 || amount >= dst_width {",
            "if width == 0 && dst_width > 0 {unsafe {wr(dst,0,0);}} if width == 0 || amount >= dst_width {",
            1,
        ),
        (
            "window-log-skips-record",
            "log-only",
            "log-records",
            "wide_window_store",
            "if !log_buf.is_null() {",
            "if false {",
            1,
        ),
        (
            "window-log-wrong-word-offset",
            "log-only",
            "log-records",
            "wide_window_store",
            "log_offset + (w0 * 8) as u32",
            "log_offset",
            1,
        ),
        (
            "window-log-forces-direct-write",
            "log-only-alias",
            "log-records",
            "wide_window_store",
            "let in_place = (log_and_flags >> 32) & 1 == 1;",
            "let in_place = true;",
            1,
        ),
        (
            "window-log-disables-direct-write",
            "log-direct",
            "log-records",
            "wide_window_store",
            "let in_place = (log_and_flags >> 32) & 1 == 1;",
            "let in_place = false;",
            1,
        ),
        (
            "window-log-direct-wrong-word",
            "log-direct-alias",
            "log-records",
            "wide_window_store",
            "wr(dst, w0 + k, v);",
            "wr(dst,k,v);",
            1,
        ),
        (
            "window-log-noop-increments-counter",
            "log-noop",
            "log-noop",
            "wide_window_store",
            "if width == 0 || amount >= dst_width {",
            "if width == 0 && !log_buf.is_null() {unsafe {(*log_buf).wide_count += 1;}} if width == 0 || amount >= dst_width {",
            1,
        ),
        (
            "mask-keeps-padding",
            "wide-mask",
            "wide-mask",
            "wide_apply_mask",
            "& mask",
            "| mask",
            1,
        ),
        (
            "addition-clears-alias-before-read",
            "wide-add-left-alias",
            "wide-arithmetic",
            "wide_add",
            "record(ProfOp::Add);",
            "record(ProfOp::Add); if nb > 0 { unsafe {wr(dst,0,0);} }",
            1,
        ),
        (
            "subtraction-clears-alias-before-read",
            "wide-sub-right-alias",
            "wide-arithmetic",
            "wide_sub",
            "record(ProfOp::Sub);",
            "record(ProfOp::Sub); if nb > 0 { unsafe {wr(dst,0,0);} }",
            1,
        ),
        (
            "mask-keeps-upper-words",
            "wide-mask",
            "wide-mask",
            "wide_apply_mask",
            "wr(dst, i, 0);",
            "wr(dst, i, 1);",
            1,
        ),
        (
            "mask-clears-zero-sentinel",
            "wide-mask",
            "wide-mask",
            "wide_apply_mask",
            "if width == 0 || nb == 0 {",
            "if width == 0 { unsafe {for i in 0..nw(nb) {wr(dst,i,0);}} } if width == 0 || nb == 0 {",
            1,
        ),
    ] {
        let name_of_contract = if kani { proof } else { native };
        let Some(contract) = selected.iter().find(|c| c.name == name_of_contract) else {
            continue;
        };
        result.push(Mutation {
            name,
            contract,
            path,
            source: replace(&original, function, old, new, count),
        });
    }
    let log_path = "veryl/crates/simulator/src/ir/write_log.rs";
    let log_original = fs::read_to_string(repo.join(log_path)).unwrap();
    for (name, proof, native, function, old, new) in [
        (
            "window-log-short-record",
            "log-only",
            "log-records",
            "push_wide",
            "native_bytes: native_bytes as u8,",
            "native_bytes: native_bytes.saturating_sub(1) as u8,",
        ),
        (
            "window-log-repeats-first-byte",
            "log-only-alias",
            "log-records",
            "push_wide",
            "std::ptr::copy_nonoverlapping(payload, p.as_mut_ptr(), native_bytes);",
            "p[..native_bytes].fill(*payload);",
        ),
        (
            "window-log-zero-payload",
            "log-direct",
            "log-records",
            "push_wide",
            "std::ptr::copy_nonoverlapping(payload, p.as_mut_ptr(), native_bytes);",
            "std::ptr::copy_nonoverlapping(payload, p.as_mut_ptr(), 0);",
        ),
        (
            "window-log-loses-append-count",
            "",
            "log-history",
            "push_wide",
            "self.wide_count += 1;",
            "self.wide_count = 1;",
        ),
        (
            "window-log-overwrites-entry-zero",
            "",
            "log-history",
            "push_wide",
            "let idx = self.wide_count as usize;",
            "let idx = 0;",
        ),
        (
            "window-log-growth-drops-history",
            "",
            "log-history",
            "grow_wide_to",
            "wide[..live].copy_from_slice(&self._wide_owner[..live]);",
            "wide[..live].fill(WriteLogWideEntry::default());",
        ),
        (
            "window-log-clears-narrow-count",
            "",
            "log-history",
            "push_wide",
            "self.wide_count += 1;",
            "self.wide_count += 1; self.narrow_count = 0;",
        ),
        (
            "window-log-chunk-skips-byte",
            "",
            "log-records",
            "push_wide_range",
            "done += chunk;",
            "done += chunk + 1;",
        ),
        (
            "window-log-chunk-repeats-prefix",
            "",
            "log-records",
            "push_wide_range",
            "payload.add(done)",
            "payload",
        ),
        (
            "window-log-chunk-reuses-offset",
            "",
            "log-records",
            "push_wide_range",
            "offset + done as u32",
            "offset",
        ),
    ] {
        let contract_name = if kani { proof } else { native };
        if let Some(contract) = selected.iter().find(|c| c.name == contract_name) {
            let declaration = format!("fn {function}(");
            let start = log_original
                .find(&declaration)
                .expect("log mutation method moved");
            let end = start
                + log_original[start..]
                    .find("\n    }")
                    .expect("log method closing brace moved")
                + 6;
            assert_eq!(
                log_original[start..end].matches(old).count(),
                1,
                "log mutation site changed: {function}: {old}"
            );
            let source = format!(
                "{}{}{}",
                &log_original[..start],
                log_original[start..end].replace(old, new),
                &log_original[end..]
            );
            result.push(Mutation {
                name,
                contract,
                path: log_path,
                source,
            });
        }
    }
    let commit_start = log_original
        .find("fn commit_from_log_impl<const MARKING: bool>(")
        .expect("commit declaration moved");
    let commit_end = commit_start
        + log_original[commit_start..]
            .find("\n}")
            .expect("commit closing brace moved")
        + 2;
    let commit_body = &log_original[commit_start..commit_end];
    for (name, proof, native, old, new) in [
        (
            "commit-narrow-data",
            "commit-narrow",
            "commit-bounds",
            "(p as *mut u64).write_unaligned(entry.payload);",
            "(p as *mut u64).write_unaligned(!entry.payload);",
        ),
        (
            "commit-narrow-end",
            "commit-narrow",
            "commit-bounds",
            "if offset + nb > len {",
            "if offset + nb >= len {",
        ),
        (
            "commit-narrow-mark",
            "commit-narrow",
            "commit-bounds",
            "MARKING && *p != entry.payload as u8",
            "MARKING && *p == entry.payload as u8",
        ),
        (
            "commit-narrow-count",
            "commit-narrow",
            "commit-bounds",
            "let narrow_limit = buffer.narrow_count as usize;",
            "let narrow_limit = 0;",
        ),
        (
            "commit-narrow-high-payload",
            "commit-narrow",
            "commit-bounds",
            "MARKING && *p != entry.payload as u8",
            "MARKING && u64::from(*p) != entry.payload",
        ),
        (
            "commit-pair-tail",
            "commit-wide3",
            "commit-bounds",
            "let tail = nb - $w;",
            "let tail = 0;",
        ),
        (
            "commit-pair-data",
            "commit-wide7",
            "commit-bounds",
            "(p.add(tail) as *mut $t).write_unaligned(hi_v);",
            "(p.add(tail) as *mut $t).write_unaligned(lo);",
        ),
        (
            "commit-pair-mark",
            "commit-wide15",
            "commit-marking",
            "|| (p.add(tail) as *const $t).read_unaligned() != hi_v",
            "|| false",
        ),
        (
            "commit-large-copy",
            "commit-wide17",
            "commit-bounds",
            "std::ptr::copy_nonoverlapping(s, p, nb);",
            "std::ptr::copy_nonoverlapping(s, p, nb - 1);",
        ),
        (
            "commit-large-mark",
            "commit-wide56",
            "commit-marking",
            "std::slice::from_raw_parts(p, nb) != std::slice::from_raw_parts(s, nb)",
            "std::slice::from_raw_parts(p, 8) != std::slice::from_raw_parts(s, 8)",
        ),
        (
            "commit-wide-end",
            "commit-wide17",
            "commit-bounds",
            "nb == 0 || nb > WRITE_LOG_WIDE_ENTRY_PAYLOAD_BYTES || offset + nb > len",
            "nb == 0 || nb > WRITE_LOG_WIDE_ENTRY_PAYLOAD_BYTES || offset + nb >= len",
        ),
        (
            "commit-wide-max",
            "commit-wide56",
            "commit-bounds",
            "nb > WRITE_LOG_WIDE_ENTRY_PAYLOAD_BYTES",
            "nb >= WRITE_LOG_WIDE_ENTRY_PAYLOAD_BYTES",
        ),
        (
            "commit-narrow-reverse",
            "commit-narrow-order",
            "commit-history",
            "buffer.narrow_entries_slice().iter().take(narrow_limit)",
            "buffer.narrow_entries_slice().iter().take(narrow_limit).rev()",
        ),
        (
            "commit-wide-reverse",
            "commit-wide-order",
            "commit-history",
            "buffer.wide_entries_slice().iter().take(wide_limit)",
            "buffer.wide_entries_slice().iter().take(wide_limit).rev()",
        ),
        (
            "commit-narrow-inactive",
            "",
            "commit-history",
            "let narrow_limit = buffer.narrow_count as usize;",
            "let narrow_limit = buffer.narrow_capacity as usize;",
        ),
        (
            "commit-wide-inactive",
            "",
            "commit-history",
            "let wide_limit = buffer.wide_count as usize;",
            "let wide_limit = buffer.wide_capacity as usize;",
        ),
        (
            "commit-unknown-class",
            "",
            "commit-bounds",
            "_ => {}",
            "_ => {if nb > 0 {*p = entry.payload as u8;}}",
        ),
    ] {
        let contract_name = if kani { proof } else { native };
        if let Some(contract) = selected.iter().find(|c| c.name == contract_name) {
            assert_eq!(
                commit_body.matches(old).count(),
                1,
                "commit mutation site changed: {old}"
            );
            result.push(Mutation {
                name,
                contract,
                path: log_path,
                source: format!(
                    "{}{}{}",
                    &log_original[..commit_start],
                    commit_body.replace(old, new),
                    &log_original[commit_end..]
                ),
            });
        }
    }
    let contract_name = if kani {
        "commit-mixed-order"
    } else {
        "commit-mixed"
    };
    if let Some(contract) = selected.iter().find(|c| c.name == contract_name) {
        let narrow = commit_body
            .find("    // Narrow path: single word store per width class.")
            .expect("narrow block moved");
        let wide = commit_body
            .find("    // Wide path.")
            .expect("wide block moved");
        let end = commit_body.len() - 2;
        result.push(Mutation {
            name: "commit-pool-order",
            contract,
            path: log_path,
            source: format!(
                "{}{}{}{}{}{}",
                &log_original[..commit_start],
                &commit_body[..narrow],
                &commit_body[wide..end],
                &commit_body[narrow..wide],
                &commit_body[end..],
                &log_original[commit_end..]
            ),
        });
    }
    let contract_name = if kani {
        "commit-narrow"
    } else {
        "commit-marking"
    };
    if let Some(contract) = selected.iter().find(|c| c.name == contract_name) {
        let old = "commit_from_log_impl::<true>(ff_values, buffer, changed)";
        assert_eq!(
            log_original.matches(old).count(),
            1,
            "marking wrapper moved"
        );
        result.push(Mutation {
            name: "commit-marking-disabled",
            contract,
            path: log_path,
            source: log_original.replace(
                old,
                "commit_from_log_impl::<false>(ff_values, buffer, changed)",
            ),
        });
    }
    for (contract_name, function, profile, wrong, skip, repeat, alias, old, new) in [
        (
            "wide-band",
            "wide_band",
            "Band",
            "band-wrong-operation",
            "band-skips-final-word",
            "band-repeats-left-word",
            "band-clears-alias",
            "rd(a, i) & rd(b, i)",
            "rd(a, i) | rd(b, i)",
        ),
        (
            "wide-bor",
            "wide_bor",
            "Bor",
            "bor-wrong-operation",
            "bor-skips-final-word",
            "bor-repeats-left-word",
            "bor-clears-alias",
            "rd(a, i) | rd(b, i)",
            "rd(a, i) & rd(b, i)",
        ),
        (
            "wide-bxor",
            "wide_bxor",
            "Bxor",
            "bxor-wrong-operation",
            "bxor-skips-final-word",
            "bxor-repeats-left-word",
            "bxor-clears-alias",
            "rd(a, i) ^ rd(b, i)",
            "rd(a, i) | rd(b, i)",
        ),
        (
            "wide-bxor-not",
            "wide_bxor_not",
            "BxorNot",
            "bxor-not-wrong-operation",
            "bxor-not-skips-final-word",
            "bxor-not-repeats-left-word",
            "bxor-not-clears-alias",
            "!(rd(a, i) ^ rd(b, i))",
            "rd(a, i) ^ rd(b, i)",
        ),
        (
            "wide-band-not",
            "wide_band_not",
            "BandNot",
            "band-not-wrong-operation",
            "band-not-skips-final-word",
            "band-not-repeats-left-word",
            "band-not-clears-alias",
            "rd(a, i) & !rd(b, i)",
            "rd(a, i) & rd(b, i)",
        ),
        (
            "wide-bnot",
            "wide_bnot",
            "Bnot",
            "bnot-wrong-operation",
            "bnot-skips-final-word",
            "bnot-repeats-left-word",
            "bnot-clears-alias",
            "!rd(a, i)",
            "rd(a, i)",
        ),
    ] {
        let Some(contract) = selected.iter().find(|c| c.name == contract_name) else {
            continue;
        };
        for (name, old, new) in [
            (wrong, old, new),
            (
                skip,
                "for i in 0..nw(nb)",
                "for i in 0..nw(nb).saturating_sub(1)",
            ),
            (repeat, "rd(a, i)", "rd(a, 0)"),
        ] {
            result.push(Mutation {
                name,
                contract,
                path,
                source: replace(&original, function, old, new, 1),
            });
        }
        let record = format!("record(ProfOp::{profile});");
        let clobber = format!("{record} if nb > 0 {{unsafe {{wr(dst,0,0);}}}}");
        result.push(Mutation {
            name: alias,
            contract,
            path,
            source: replace(&original, function, &record, &clobber, 1),
        });
    }
    if kani {
        for (name, proof, function, old, new, count) in [
            (
                "addition-left-alias-loses-carry",
                "wide-add-left-alias",
                "wide_add",
                "carry = (c1 as u64) + (c2 as u64);",
                "carry = 0;",
                1,
            ),
            (
                "addition-right-alias-loses-carry",
                "wide-add-right-alias",
                "wide_add",
                "carry = (c1 as u64) + (c2 as u64);",
                "carry = 0;",
                1,
            ),
            (
                "subtraction-left-alias-loses-borrow",
                "wide-sub-left-alias",
                "wide_sub",
                "borrow = (b1 as u64) + (b2 as u64);",
                "borrow = 0;",
                1,
            ),
            (
                "subtraction-right-alias-loses-borrow",
                "wide-sub-right-alias",
                "wide_sub",
                "borrow = (b1 as u64) + (b2 as u64);",
                "borrow = 0;",
                1,
            ),
            (
                "subtraction-left-alias-adds",
                "wide-sub-left-alias",
                "wide_sub",
                "overflowing_sub",
                "overflowing_add",
                2,
            ),
            (
                "subtraction-right-alias-adds",
                "wide-sub-right-alias",
                "wide_sub",
                "overflowing_sub",
                "overflowing_add",
                2,
            ),
        ] {
            if let Some(contract) = selected.iter().find(|c| c.name == proof) {
                result.push(Mutation {
                    name,
                    contract,
                    path,
                    source: replace(&original, function, old, new, count),
                });
            }
        }
    }
    result
}

pub fn catalog(kani: bool) -> &'static [Contract] {
    if kani { &KANI } else { &NATIVE }
}
