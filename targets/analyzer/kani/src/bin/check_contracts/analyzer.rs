//! Analyzer-specific contracts and production mutations.
use std::fs;
use std::path::Path;
use veryl_formal_runner::{Contract, Mutation};
#[path = "followup.rs"]
mod followup;
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

const SHIFT_WIDENING_COVERS: &[&str] = &[
    "public shift widening signed one before left shift is reachable",
    "public shift widening signed X before logical right is reachable",
    "public shift widening signed Z fill is reachable",
    "public shift widening unsigned arithmetic left is reachable",
    "public shift widening upper Z amount is reachable",
    "public shift widening overshift padding is reachable",
];

const WORD_BIG_AMOUNT_COVERS: &[&str] = &[
    "public wide amount trace signed operand is reachable",
    "public wide amount trace negative source is reachable",
    "public wide amount trace signed X source is reachable",
    "public wide amount trace unsigned Z source is reachable",
    "public wide amount trace arithmetic left is reachable",
];

const PUBLIC_INEQUALITY_NATIVE_UNSIGNED_COVERS: &[&str] = &[
    "public comparison LHS width 1 is reachable",
    "public comparison LHS width 2 is reachable",
    "public comparison LHS width 3 is reachable",
    "public comparison RHS width 1 is reachable",
    "public comparison RHS width 2 is reachable",
    "public comparison RHS width 3 is reachable",
    "public comparison wider LHS is reachable",
    "public comparison wider RHS is reachable",
    "public comparison X result is reachable",
    "public comparison true result is reachable",
    "public comparison false result is reachable",
    "public comparison unsigned sources are reachable",
    "public comparison signed RHS only is reachable",
    "public comparison signed LHS only is reachable",
    "public comparison partition sizing is reachable",
    "public comparison LHS width 64 is reachable",
    "public comparison RHS width 64 is reachable",
];

const PUBLIC_INEQUALITY_NATIVE_SIGNED_COVERS: &[&str] = &[
    "public comparison LHS width 1 is reachable",
    "public comparison LHS width 2 is reachable",
    "public comparison LHS width 3 is reachable",
    "public comparison RHS width 1 is reachable",
    "public comparison RHS width 2 is reachable",
    "public comparison RHS width 3 is reachable",
    "public comparison wider LHS is reachable",
    "public comparison wider RHS is reachable",
    "public comparison X result is reachable",
    "public comparison true result is reachable",
    "public comparison false result is reachable",
    "public comparison signed sources are reachable",
    "public comparison partition sizing is reachable",
    "public comparison LHS width 64 is reachable",
    "public comparison RHS width 64 is reachable",
];

const PUBLIC_SMALL_WILDCARD_INEQUALITY_UNSIGNED_COVERS: &[&str] = &[
    "public comparison LHS width 1 is reachable",
    "public comparison LHS width 2 is reachable",
    "public comparison LHS width 3 is reachable",
    "public comparison RHS width 1 is reachable",
    "public comparison RHS width 2 is reachable",
    "public comparison RHS width 3 is reachable",
    "public comparison wider LHS is reachable",
    "public comparison wider RHS is reachable",
    "public comparison X result is reachable",
    "public comparison true result is reachable",
    "public comparison false result is reachable",
    "public comparison unsigned sources are reachable",
    "public comparison signed RHS only is reachable",
    "public comparison signed LHS only is reachable",
    "public comparison partition sizing is reachable",
    "public comparison RHS wildcard is reachable",
    "public comparison LHS X is significant",
];

const PUBLIC_SMALL_ORDERING_UNSIGNED_COVERS: &[&str] = &[
    "public comparison LHS width 1 is reachable",
    "public comparison LHS width 2 is reachable",
    "public comparison LHS width 3 is reachable",
    "public comparison RHS width 1 is reachable",
    "public comparison RHS width 2 is reachable",
    "public comparison RHS width 3 is reachable",
    "public comparison wider LHS is reachable",
    "public comparison wider RHS is reachable",
    "public comparison X result is reachable",
    "public comparison true result is reachable",
    "public comparison false result is reachable",
    "public comparison unsigned sources are reachable",
    "public comparison signed RHS only is reachable",
    "public comparison signed LHS only is reachable",
    "public comparison partition sizing is reachable",
];

const PUBLIC_SMALL_WILDCARD_INEQUALITY_SIGNED_COVERS: &[&str] = &[
    "public comparison LHS width 1 is reachable",
    "public comparison LHS width 2 is reachable",
    "public comparison LHS width 3 is reachable",
    "public comparison RHS width 1 is reachable",
    "public comparison RHS width 2 is reachable",
    "public comparison RHS width 3 is reachable",
    "public comparison wider LHS is reachable",
    "public comparison wider RHS is reachable",
    "public comparison X result is reachable",
    "public comparison true result is reachable",
    "public comparison false result is reachable",
    "public comparison signed sources are reachable",
    "public comparison partition sizing is reachable",
    "public comparison RHS wildcard is reachable",
    "public comparison LHS X is significant",
];

const PUBLIC_SMALL_ORDERING_SIGNED_COVERS: &[&str] = &[
    "public comparison LHS width 1 is reachable",
    "public comparison LHS width 2 is reachable",
    "public comparison LHS width 3 is reachable",
    "public comparison RHS width 1 is reachable",
    "public comparison RHS width 2 is reachable",
    "public comparison RHS width 3 is reachable",
    "public comparison wider LHS is reachable",
    "public comparison wider RHS is reachable",
    "public comparison X result is reachable",
    "public comparison true result is reachable",
    "public comparison false result is reachable",
    "public comparison signed sources are reachable",
    "public comparison partition sizing is reachable",
];

const PUBLIC_WILDCARD_CONTINUATION_COVERS: &[&str] = &[
    "public comparison LHS width 1 is reachable",
    "public comparison LHS width 2 is reachable",
    "public comparison LHS width 3 is reachable",
    "public comparison RHS width 1 is reachable",
    "public comparison RHS width 2 is reachable",
    "public comparison RHS width 3 is reachable",
    "public comparison wider LHS is reachable",
    "public comparison wider RHS is reachable",
    "public comparison signed mode is reachable",
    "public comparison mixed signs are reachable",
    "public comparison X result is reachable",
    "public comparison true result is reachable",
    "public comparison false result is reachable",
    "public comparison sign extension is reachable",
    "public comparison RHS wildcard is reachable",
    "public comparison LHS X is significant",
];

const PUBLIC_SMALL_FIXED_ORDERING_COVERS: &[&str] = &[
    "public comparison unsigned sources are reachable",
    "public comparison signed RHS only is reachable",
    "public comparison signed LHS only is reachable",
    "public comparison signed sources are reachable",
    "public comparison X result is reachable",
    "public comparison true result is reachable",
    "public comparison false result is reachable",
];

const PUBLIC_WIDTH4_WILDCARD_EQUAL_COVERS: &[&str] = &[
    "public comparison unsigned sources are reachable",
    "public comparison signed RHS only is reachable",
    "public comparison signed LHS only is reachable",
    "public comparison signed sources are reachable",
    "public comparison X result is reachable",
    "public comparison true result is reachable",
    "public comparison false result is reachable",
    "public width4 wildcard equality RHS X matches",
    "public width4 wildcard equality RHS Z matches",
    "public width4 wildcard equality LHS X remains significant",
    "public width4 wildcard equality LHS Z remains significant",
    "public width4 wildcard equality known mismatch dominates LHS unknown",
    "public width4 wildcard equality known mismatch survives RHS wildcard",
];

const PUBLIC_WIDTH4_WILDCARD_NOT_EQUAL_COVERS: &[&str] = &[
    "public comparison unsigned sources are reachable",
    "public comparison signed RHS only is reachable",
    "public comparison signed LHS only is reachable",
    "public comparison signed sources are reachable",
    "public comparison X result is reachable",
    "public comparison true result is reachable",
    "public comparison false result is reachable",
    "public width4 wildcard inequality RHS X matches",
    "public width4 wildcard inequality RHS Z matches",
    "public width4 wildcard inequality LHS X remains significant",
    "public width4 wildcard inequality LHS Z remains significant",
    "public width4 wildcard inequality known mismatch dominates LHS unknown",
    "public width4 wildcard inequality known mismatch survives RHS wildcard",
];

pub static CONTRACTS: [Contract; 473] = [
    Contract {
        group: "public-big-values",
        name: "signedq256-byte-api",
        harness: "public_big_ops::signed_power_quotient::signed_quotient_numeric_api_matches_byte_division",
        message: "signed quotient byte API mismatch",
        covers: &[
            "signed quotient API positive positive is reachable",
            "signed quotient API positive negative is reachable",
            "signed quotient API negative positive is reachable",
            "signed quotient API negative negative is reachable",
            "signed quotient API zero remainder is reachable",
            "signed quotient API minimum over minus one is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedq256-quotient-word-api",
        harness: "public_big_ops::signed_power_quotient::signed_quotient_quotient_word_matches_reference",
        message: "signed quotient word mismatch",
        covers: &[
            "signed quotient word unit width is reachable",
            "signed quotient word nonpower divisor is reachable",
            "signed quotient word minimum overflow is reachable",
            "signed quotient word negative quotient is reachable",
            "signed quotient word word crossing is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedq256-remainder-word-api",
        harness: "public_big_ops::signed_power_quotient::signed_quotient_remainder_word_matches_reference",
        message: "signed quotient word mismatch",
        covers: &[
            "signed quotient word unit width is reachable",
            "signed quotient word nonpower divisor is reachable",
            "signed quotient word negative remainder is reachable",
            "signed quotient word zero remainder is reachable",
            "signed quotient word maximum remainder is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedq256-domain-api",
        harness: "public_big_ops::signed_power_quotient::signed_quotient_original_domain_preserves_prepared_values",
        message: "signed quotient original domain mismatch",
        covers: &[
            "signed quotient domain native native is reachable",
            "signed quotient domain native big is reachable",
            "signed quotient domain big native is reachable",
            "signed quotient domain big big is reachable",
            "signed quotient domain left signed only is reachable",
            "signed quotient domain right signed only is reachable",
            "signed quotient domain minimum dividend is reachable",
            "signed quotient domain largest quotient is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedq256-div-positive",
        harness: "public_big_ops::signed_power_quotient::wide_signed_quotient_div_positive_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "signed quotient class minimum nonpower divisor is reachable",
            "signed quotient class word crossing is reachable",
            "signed quotient class zero remainder is reachable",
            "signed quotient class maximum nonzero remainder is reachable",
            "signed quotient class maximum positive dividend is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedq256-div-negative",
        harness: "public_big_ops::signed_power_quotient::wide_signed_quotient_div_negative_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "signed quotient class minimum nonpower divisor is reachable",
            "signed quotient class word crossing is reachable",
            "signed quotient class zero remainder is reachable",
            "signed quotient class maximum nonzero remainder is reachable",
            "signed quotient class minimum over minus one is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedq256-rem-positive",
        harness: "public_big_ops::signed_power_quotient::wide_signed_quotient_rem_positive_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "signed quotient class minimum nonpower divisor is reachable",
            "signed quotient class word crossing is reachable",
            "signed quotient class zero remainder is reachable",
            "signed quotient class maximum nonzero remainder is reachable",
            "signed quotient class maximum positive dividend is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedq256-rem-negative",
        harness: "public_big_ops::signed_power_quotient::wide_signed_quotient_rem_negative_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "signed quotient class minimum nonpower divisor is reachable",
            "signed quotient class word crossing is reachable",
            "signed quotient class zero remainder is reachable",
            "signed quotient class maximum nonzero remainder is reachable",
            "signed quotient class minimum over minus one is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedq256-div-positive-positive-context",
        harness: "public_big_ops::signed_power_quotient::wide_signed_quotient_div_positive_positive_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["signed quotient class positive positive context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "signedq256-div-positive-negative-context",
        harness: "public_big_ops::signed_power_quotient::wide_signed_quotient_div_positive_negative_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["signed quotient class positive negative context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "signedq256-div-negative-positive-context",
        harness: "public_big_ops::signed_power_quotient::wide_signed_quotient_div_negative_positive_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["signed quotient class negative positive context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "signedq256-div-negative-negative-context",
        harness: "public_big_ops::signed_power_quotient::wide_signed_quotient_div_negative_negative_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["signed quotient class negative negative context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "signedq256-rem-positive-positive-context",
        harness: "public_big_ops::signed_power_quotient::wide_signed_quotient_rem_positive_positive_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["signed quotient class positive positive context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "signedq256-rem-positive-negative-context",
        harness: "public_big_ops::signed_power_quotient::wide_signed_quotient_rem_positive_negative_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["signed quotient class positive negative context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "signedq256-rem-negative-positive-context",
        harness: "public_big_ops::signed_power_quotient::wide_signed_quotient_rem_negative_positive_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["signed quotient class negative positive context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "signedq256-rem-negative-negative-context",
        harness: "public_big_ops::signed_power_quotient::wide_signed_quotient_rem_negative_negative_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["signed quotient class negative negative context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "powerq256-byte-api",
        harness: "public_big_ops::power_quotient::power_quotient_numeric_api_matches_byte_division",
        message: "power quotient byte API mismatch",
        covers: &[
            "power quotient API unit quotient is reachable",
            "power quotient API nonpower divisor is reachable",
            "power quotient API maximum dividend is reachable",
            "power quotient API largest byte quotient is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "powerq256-product-api",
        harness: "public_big_ops::power_quotient::power_quotient_full_product_matches_scalar_coordinates",
        message: "power quotient product mismatch",
        covers: &[
            "power quotient product unit quotient is reachable",
            "power quotient product bit254 is reachable",
            "power quotient product word crossing is reachable",
            "power quotient product high half is reachable",
            "power quotient product zero divisor is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "powerq256-euclidean-api",
        harness: "public_big_ops::power_quotient::power_quotient_numeric_api_satisfies_euclidean_identity",
        message: "power quotient Euclidean mismatch",
        covers: &[
            "power quotient Euclidean unit width is reachable",
            "power quotient Euclidean maximum dividend is reachable",
            "power quotient Euclidean largest quotient is reachable",
            "power quotient Euclidean nonpower divisor is reachable",
            "power quotient Euclidean maximum remainder is reachable",
            "power quotient Euclidean word borrow is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "powerq256-domain-api",
        harness: "public_big_ops::power_quotient::power_quotient_original_domain_preserves_prepared_values",
        message: "power quotient original domain mismatch",
        covers: &[
            "power quotient domain native native is reachable",
            "power quotient domain native big is reachable",
            "power quotient domain big native is reachable",
            "power quotient domain big big is reachable",
            "power quotient domain left signed only is reachable",
            "power quotient domain right signed only is reachable",
            "power quotient domain largest quotient is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "powerq256-div",
        harness: "public_big_ops::power_quotient::wide_power_quotient_div_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "power quotient class minimum nonunit quotient is reachable",
            "power quotient class largest quotient is reachable",
            "power quotient class unit quotient is reachable",
            "power quotient class nonpower divisor is reachable",
            "power quotient class maximum remainder is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "powerq256-div-context",
        harness: "public_big_ops::power_quotient::wide_power_quotient_div_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["power quotient class original context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "powerq256-rem",
        harness: "public_big_ops::power_quotient::wide_power_quotient_rem_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "power quotient class minimum nonunit quotient is reachable",
            "power quotient class largest quotient is reachable",
            "power quotient class unit quotient is reachable",
            "power quotient class nonpower divisor is reachable",
            "power quotient class maximum remainder is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "powerq256-rem-context",
        harness: "public_big_ops::power_quotient::wide_power_quotient_rem_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["power quotient class original context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "unitq256-byte-api",
        harness: "public_big_ops::unit_quotient::unit_quotient_numeric_api_matches_byte_division",
        message: "unit quotient byte API mismatch",
        covers: &[
            "unit quotient API unit divisor is reachable",
            "unit quotient API maximum dividend is reachable",
            "unit quotient API exact division is reachable",
            "unit quotient API upper divisor is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "unitq256-euclidean-api",
        harness: "public_big_ops::unit_quotient::unit_quotient_numeric_api_satisfies_euclidean_identity",
        message: "unit quotient Euclidean mismatch",
        covers: &[
            "unit quotient Euclidean unit width is reachable",
            "unit quotient Euclidean maximum dividend is reachable",
            "unit quotient Euclidean upper divisor is reachable",
            "unit quotient Euclidean maximum remainder is reachable",
            "unit quotient Euclidean exact division is reachable",
            "unit quotient Euclidean word borrow is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "unitq256-domain-api",
        harness: "public_big_ops::unit_quotient::unit_quotient_original_domain_preserves_prepared_values",
        message: "unit quotient original domain mismatch",
        covers: &[
            "unit quotient domain native native is reachable",
            "unit quotient domain native big is reachable",
            "unit quotient domain big native is reachable",
            "unit quotient domain big big is reachable",
            "unit quotient domain left signed only is reachable",
            "unit quotient domain right signed only is reachable",
            "unit quotient domain upper divisor is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "unitq256-div",
        harness: "public_big_ops::unit_quotient::wide_unit_quotient_div_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "unit quotient class minimum width is reachable",
            "unit quotient class maximum upper divisor is reachable",
            "unit quotient class exact division is reachable",
            "unit quotient class adjacent operands is reachable",
            "unit quotient class maximum remainder is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "unitq256-div-context",
        harness: "public_big_ops::unit_quotient::wide_unit_quotient_div_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["unit quotient class original context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "unitq256-rem",
        harness: "public_big_ops::unit_quotient::wide_unit_quotient_rem_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "unit quotient class minimum width is reachable",
            "unit quotient class maximum upper divisor is reachable",
            "unit quotient class exact division is reachable",
            "unit quotient class adjacent operands is reachable",
            "unit quotient class maximum remainder is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "unitq256-rem-context",
        harness: "public_big_ops::unit_quotient::wide_unit_quotient_rem_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["unit quotient class original context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "powerhot256-expansion-api",
        harness: "public_big_ops::hot_power::single_hot_expansion_membership_is_exact",
        message: "single hot expansion membership mismatch",
        covers: &[
            "single hot membership unit width is reachable",
            "single hot membership equal width is reachable",
            "single hot membership sign extension rejects is reachable",
            "single hot membership zero extension accepts is reachable",
            "single hot membership upper bit is reachable",
            "single hot membership missing source bit is reachable",
            "single hot membership multiple bits rejects is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "powerhot256-byte-api",
        harness: "public_big_ops::hot_power::single_hot_power_api_matches_byte_products",
        message: "single hot power byte API mismatch",
        covers: &[
            "single hot power API zero exponent is reachable",
            "single hot power API unit base is reachable",
            "single hot power API clipped product is reachable",
            "single hot power API maximum byte base is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "powerhot256-coordinates-api",
        harness: "public_big_ops::hot_power::single_hot_power_api_matches_scalar_coordinates",
        message: "single hot power coordinate mismatch",
        covers: &[
            "single hot power coordinates unit width is reachable",
            "single hot power coordinates full exponent is reachable",
            "single hot power coordinates high result bit is reachable",
            "single hot power coordinates upper exponent is reachable",
            "single hot power coordinates top base bit is reachable",
            "single hot power coordinates outside width is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "powerhot256-successor-api",
        harness: "public_big_ops::hot_power::single_hot_power_api_satisfies_bounded_successor_law",
        message: "single hot power successor mismatch",
        covers: &[
            "single hot power recurrence unit base is reachable",
            "single hot power recurrence unit width is reachable",
            "single hot power recurrence full exponent is reachable",
            "single hot power recurrence high result bit is reachable",
            "single hot power recurrence first zero result is reachable",
            "single hot power recurrence top base bit is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "powerhot256-domain-api",
        harness: "public_big_ops::hot_power::single_hot_power_original_domain_preserves_prepared_base",
        message: "single hot power original domain mismatch",
        covers: &[
            "single hot power domain native native is reachable",
            "single hot power domain native big is reachable",
            "single hot power domain big native is reachable",
            "single hot power domain big big is reachable",
            "single hot power domain signed minimum is reachable",
            "single hot power domain exponent exceeds result width is reachable",
            "single hot power domain equal width retains sign is reachable",
            "single hot power domain unit base is reachable",
            "single hot power domain wide signed exponent is reachable",
            "single hot power domain source sign bit zero extends is reachable",
            "single hot power domain narrow signed positive is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "powerhot256-prepared-signed",
        harness: "public_big_ops::hot_power::wide_single_hot_power_prepared_signed_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "single hot power class minimum signed width is reachable",
            "single hot power class unit base is reachable",
            "single hot power class signed minimum is reachable",
            "single hot power class wide signed exponent is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "powerhot256-prepared-unsigned",
        harness: "public_big_ops::hot_power::wide_single_hot_power_prepared_unsigned_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "single hot power class maximum unsigned width is reachable",
            "single hot power class full unsigned exponent is reachable",
            "single hot power class high result bit is reachable",
            "single hot power class first zero result is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "powerhot256-context-signed",
        harness: "public_big_ops::hot_power::wide_single_hot_power_context_signed_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["single hot power class signed original context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "powerhot256-context-unsigned",
        harness: "public_big_ops::hot_power::wide_single_hot_power_context_unsigned_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["single hot power class unsigned original context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "powerones256-byte-api",
        harness: "public_big_ops::ones_power::all_ones_power_api_matches_byte_products",
        message: "ones power byte API mismatch",
        covers: &[
            "ones power API zero exponent is reachable",
            "ones power API even exponent is reachable",
            "ones power API odd exponent is reachable",
            "ones power API maximum byte base is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "powerones256-successor-api",
        harness: "public_big_ops::ones_power::all_ones_power_api_satisfies_bounded_successor_law",
        message: "ones power successor mismatch",
        covers: &[
            "ones power recurrence unit width is reachable",
            "ones power recurrence full exponent is reachable",
            "ones power recurrence zero is reachable",
            "ones power recurrence even is reachable",
            "ones power recurrence odd is reachable",
            "ones power recurrence upper exponent is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "powerones256-domain-api",
        harness: "public_big_ops::ones_power::all_ones_power_original_domain_preserves_prepared_base",
        message: "ones power original domain mismatch",
        covers: &[
            "ones power domain native native is reachable",
            "ones power domain native big is reachable",
            "ones power domain big native is reachable",
            "ones power domain big big is reachable",
            "ones power domain signed base is reachable",
            "ones power domain unsigned base is reachable",
            "ones power domain exponent exceeds result width is reachable",
            "ones power domain wide signed exponent is reachable",
            "ones power domain equal width retains sign is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "powerones256-prepared",
        harness: "public_big_ops::ones_power::wide_all_ones_power_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "ones power class minimum signed base is reachable",
            "ones power class maximum unsigned base is reachable",
            "ones power class even exponent is reachable",
            "ones power class odd exponent is reachable",
            "ones power class full unsigned exponent is reachable",
            "ones power class wide signed exponent is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "powerones256-context",
        harness: "public_big_ops::ones_power::wide_all_ones_power_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["ones power class original context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "ones256-byte-api",
        harness: "public_big_ops::ones::all_ones_numeric_api_matches_byte_products",
        message: "ones numeric API mismatch",
        covers: &[
            "ones API unit factor is reachable",
            "ones API maximum byte product is reachable",
            "ones API zero product is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ones256-inverse-api",
        harness: "public_big_ops::ones::all_ones_full_product_satisfies_nonoverflowing_inverse",
        message: "ones full product inverse mismatch",
        covers: &[
            "ones inverse unit width is reachable",
            "ones inverse bit255 is reachable",
            "ones inverse zero is reachable",
            "ones inverse high borrow is reachable",
            "ones inverse cross word is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ones256-word-api",
        harness: "public_big_ops::ones::all_ones_modular_product_matches_scalar_subtraction_carry",
        message: "ones modular product mismatch",
        covers: &[
            "ones word minimum width is reachable",
            "ones word maximum width is reachable",
            "ones word zero is reachable",
            "ones word one is reachable",
            "ones word high truncation is reachable",
            "ones word all ones square is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ones256-right-domain-api",
        harness: "public_big_ops::ones::all_ones_right_original_domain_preserves_prepared_values",
        message: "ones original domain mismatch",
        covers: &[
            "ones domain native native is reachable",
            "ones domain native big is reachable",
            "ones domain big native is reachable",
            "ones domain big big is reachable",
            "ones domain minimum signed width is reachable",
            "ones domain maximum unsigned width is reachable",
            "ones domain mixed source signs is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ones256-left-domain-api",
        harness: "public_big_ops::ones::all_ones_left_original_domain_preserves_prepared_values",
        message: "ones original domain mismatch",
        covers: &[
            "ones domain native native is reachable",
            "ones domain native big is reachable",
            "ones domain big native is reachable",
            "ones domain big big is reachable",
            "ones domain minimum signed width is reachable",
            "ones domain maximum unsigned width is reachable",
            "ones domain mixed source signs is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ones256-right",
        harness: "public_big_ops::ones::wide_all_ones_right_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "ones class minimum signed width is reachable",
            "ones class maximum unsigned width is reachable",
            "ones class zero number is reachable",
            "ones class truncated high product is reachable",
            "ones class signed minimum number is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ones256-right-context",
        harness: "public_big_ops::ones::wide_all_ones_right_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["ones class original context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "ones256-left",
        harness: "public_big_ops::ones::wide_all_ones_left_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "ones class minimum signed width is reachable",
            "ones class maximum unsigned width is reachable",
            "ones class zero number is reachable",
            "ones class truncated high product is reachable",
            "ones class signed minimum number is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ones256-left-context",
        harness: "public_big_ops::ones::wide_all_ones_left_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["ones class original context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "signedpow256-byte-api",
        harness: "public_big_ops::signed_power_two::signed_power_arithmetic_matches_byte_division",
        message: "signed power numeric API mismatch",
        covers: &[
            "power signed API zero dividend is reachable",
            "power signed API negative quotient is reachable",
            "power signed API positive quotient is reachable",
            "power signed API minimum divisor is reachable",
            "power signed API minimum over minus one is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedpow256-coordinates-api",
        harness: "public_big_ops::signed_power_two::signed_power_magnitude_matches_scalar_coordinates",
        message: "signed power coordinate API mismatch",
        covers: &[
            "power signed coordinates unit divisor is reachable",
            "power signed coordinates zero dividend is reachable",
            "power signed coordinates word crossing is reachable",
            "power signed coordinates top divisor is reachable",
            "power signed coordinates full magnitude is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedpow256-word-api",
        harness: "public_big_ops::signed_power_two::signed_power_word_matches_signed_decoding",
        message: "signed power word class mismatch",
        covers: &[
            "power signed word minimum width is reachable",
            "power signed word maximum width is reachable",
            "power signed word zero dividend is reachable",
            "power signed word minimum over minus one is reachable",
            "power signed word minimum divisor is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedpow256-domain-api",
        harness: "public_big_ops::signed_power_two::signed_power_original_domain_preserves_prepared_values",
        message: "signed power context domain mismatch",
        covers: &[
            "power signed domain native native is reachable",
            "power signed domain native big is reachable",
            "power signed domain big native is reachable",
            "power signed domain big big is reachable",
            "power signed domain positive positive is reachable",
            "power signed domain positive negative is reachable",
            "power signed domain negative positive is reachable",
            "power signed domain negative negative is reachable",
            "power signed domain minimum zero is reachable",
            "power signed domain minimum over minus one is reachable",
            "power signed domain minimum divisor is reachable",
            "power signed domain mixed source signs is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedpow256-div",
        harness: "public_big_ops::signed_power_two::wide_signed_power_div_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "power signed class minimum nonunit divisor is reachable",
            "power signed class minimum over minus one is reachable",
            "power signed class zero dividend is reachable",
            "power signed class word crossing is reachable",
            "power signed class minimum divisor is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedpow256-div-nonnegative-context",
        harness: "public_big_ops::signed_power_two::wide_signed_power_div_nonnegative_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["power signed class nonnegative context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "signedpow256-div-negative-context",
        harness: "public_big_ops::signed_power_two::wide_signed_power_div_negative_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["power signed class negative context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "signedpow256-rem",
        harness: "public_big_ops::signed_power_two::wide_signed_power_rem_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "power signed class minimum nonunit divisor is reachable",
            "power signed class minimum over minus one is reachable",
            "power signed class zero dividend is reachable",
            "power signed class word crossing is reachable",
            "power signed class minimum divisor is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedpow256-rem-nonnegative-context",
        harness: "public_big_ops::signed_power_two::wide_signed_power_rem_nonnegative_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["power signed class nonnegative context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "signedpow256-rem-negative-context",
        harness: "public_big_ops::signed_power_two::wide_signed_power_rem_negative_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["power signed class negative context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "signedsmall256-conversion-api",
        harness: "public_big_ops::signed_small::small_signed_conversion_includes_zero_and_minimum",
        message: "small signed conversion witness mismatch",
        covers: &[
            "small signed conversion minimum width is reachable",
            "small signed conversion maximum width is reachable",
            "small signed conversion zero is reachable",
            "small signed conversion minimum integer is reachable",
            "small signed conversion maximum positive is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedsmall256-byte-api",
        harness: "public_big_ops::signed_small::small_signed_arithmetic_matches_byte_division",
        message: "small signed numeric API mismatch",
        covers: &[
            "small signed API zero dividend is reachable",
            "small signed API positive dividend is reachable",
            "small signed API negative dividend is reachable",
            "small signed API minimum divisor is reachable",
            "small signed API opposite signs is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedsmall256-word-api",
        harness: "public_big_ops::signed_small::small_signed_word_class_matches_decoded_magnitudes",
        message: "small signed word class mismatch",
        covers: &[
            "small signed word minimum width is reachable",
            "small signed word maximum width is reachable",
            "small signed word zero dividend is reachable",
            "small signed word minimum divisor is reachable",
            "small signed word both negative is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedsmall256-domain-api",
        harness: "public_big_ops::signed_small::small_signed_original_domain_preserves_prepared_values",
        message: "small signed context domain mismatch",
        covers: &[
            "small signed domain native native is reachable",
            "small signed domain native big is reachable",
            "small signed domain big native is reachable",
            "small signed domain big big is reachable",
            "small signed domain positive positive is reachable",
            "small signed domain positive negative is reachable",
            "small signed domain negative positive is reachable",
            "small signed domain negative negative is reachable",
            "small signed domain minimum zero is reachable",
            "small signed domain maximum minimum divisor is reachable",
            "small signed domain mixed source signs is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedsmall256-div",
        harness: "public_big_ops::signed_small::wide_small_signed_div_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "small signed class minimum positive pair is reachable",
            "small signed class minimum divisor is reachable",
            "small signed class zero dividend is reachable",
            "small signed class negative dividend is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedsmall256-div-nonnegative-context",
        harness: "public_big_ops::signed_small::wide_small_signed_div_nonnegative_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["small signed class nonnegative context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "signedsmall256-div-negative-context",
        harness: "public_big_ops::signed_small::wide_small_signed_div_negative_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["small signed class negative context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "signedsmall256-rem",
        harness: "public_big_ops::signed_small::wide_small_signed_rem_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "small signed class minimum positive pair is reachable",
            "small signed class minimum divisor is reachable",
            "small signed class zero dividend is reachable",
            "small signed class negative dividend is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "signedsmall256-rem-nonnegative-context",
        harness: "public_big_ops::signed_small::wide_small_signed_rem_nonnegative_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["small signed class nonnegative context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "signedsmall256-rem-negative-context",
        harness: "public_big_ops::signed_small::wide_small_signed_rem_negative_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["small signed class negative context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "opposite256-conversion-api",
        harness: "public_big_ops::opposite::opposite_conversion_witness_matches_signed_decoding",
        message: "opposite conversion witness mismatch",
        covers: &[
            "opposite conversion minimum width is reachable",
            "opposite conversion maximum width is reachable",
            "opposite conversion unit magnitude is reachable",
            "opposite conversion maximum magnitude is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "opposite256-byte-api",
        harness: "public_big_ops::opposite::numeric_opposite_division_matches_byte_arithmetic",
        message: "opposite numeric API mismatch",
        covers: &[
            "opposite API positive dividend is reachable",
            "opposite API negative dividend is reachable",
            "opposite API byte minimum magnitude is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "opposite256-context-domain-api",
        harness: "public_big_ops::opposite::opposite_original_domain_preserves_prepared_values",
        message: "opposite context domain mismatch",
        covers: &[
            "opposite class minimum positive dividend is reachable",
            "opposite class maximum negative dividend is reachable",
            "opposite class maximum magnitude is reachable",
            "opposite class mixed source signs is reachable",
            "opposite class native native sources is reachable",
            "opposite class native big sources is reachable",
            "opposite class big native sources is reachable",
            "opposite class big big sources is reachable",
            "opposite class prepared sources is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "opposite256-div",
        harness: "public_big_ops::opposite::wide_opposite_div_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "opposite class minimum positive dividend is reachable",
            "opposite class maximum negative dividend is reachable",
            "opposite class maximum magnitude is reachable",
            "opposite class mixed source signs is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "opposite256-div-positive-context",
        harness: "public_big_ops::opposite::wide_opposite_div_positive_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["opposite class positive context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "opposite256-div-negative-context",
        harness: "public_big_ops::opposite::wide_opposite_div_negative_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["opposite class negative context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "opposite256-rem",
        harness: "public_big_ops::opposite::wide_opposite_rem_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "opposite class minimum positive dividend is reachable",
            "opposite class maximum negative dividend is reachable",
            "opposite class maximum magnitude is reachable",
            "opposite class mixed source signs is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "opposite256-rem-positive-context",
        harness: "public_big_ops::opposite::wide_opposite_rem_positive_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["opposite class positive context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "opposite256-rem-negative-context",
        harness: "public_big_ops::opposite::wide_opposite_rem_negative_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &["opposite class negative context is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "multiply256-byte-api",
        harness: "public_big_ops::multiplication::numeric_one_hot_multiplication_matches_byte_arithmetic",
        message: "multiplication numeric API mismatch",
        covers: &[
            "multiplication API zero number is reachable",
            "multiplication API byte high product is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "multiply256-coordinates-api",
        harness: "public_big_ops::multiplication::numeric_one_hot_multiplication_matches_full_product_coordinates",
        message: "multiplication numeric API mismatch",
        covers: &[
            "multiplication API factor one is reachable",
            "multiplication API word crossing is reachable",
            "multiplication API bit254 is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "multiply256-right",
        harness: "public_big_ops::multiplication::wide_one_hot_multiply_right_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "multiplication class minimum signed width is reachable",
            "multiplication class maximum unsigned width is reachable",
            "multiplication class mixed source signs is reachable",
            "multiplication class factor one is reachable",
            "multiplication class word crossing is reachable",
            "multiplication class truncated high product is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "multiply256-right-context",
        harness: "public_big_ops::multiplication::wide_one_hot_multiply_right_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "multiplication class minimum signed width is reachable",
            "multiplication class maximum unsigned width is reachable",
            "multiplication class mixed source signs is reachable",
            "multiplication class factor one is reachable",
            "multiplication class word crossing is reachable",
            "multiplication class truncated high product is reachable",
            "multiplication class native native sources is reachable",
            "multiplication class native big sources is reachable",
            "multiplication class big native sources is reachable",
            "multiplication class big big sources is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "multiply256-left",
        harness: "public_big_ops::multiplication::wide_one_hot_multiply_left_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "multiplication class minimum signed width is reachable",
            "multiplication class maximum unsigned width is reachable",
            "multiplication class mixed source signs is reachable",
            "multiplication class factor one is reachable",
            "multiplication class word crossing is reachable",
            "multiplication class truncated high product is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "multiply256-left-context",
        harness: "public_big_ops::multiplication::wide_one_hot_multiply_left_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "multiplication class minimum signed width is reachable",
            "multiplication class maximum unsigned width is reachable",
            "multiplication class mixed source signs is reachable",
            "multiplication class factor one is reachable",
            "multiplication class word crossing is reachable",
            "multiplication class truncated high product is reachable",
            "multiplication class native native sources is reachable",
            "multiplication class native big sources is reachable",
            "multiplication class big native sources is reachable",
            "multiplication class big big sources is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-byte-api",
        harness: "numeric_division_classes_match_byte_arithmetic",
        message: "division numeric API mismatch",
        covers: &[
            "division API equal words are reachable",
            "division API smaller words are reachable",
            "division API byte top bit is reachable",
            "division API negative self is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-pow2-api",
        harness: "numeric_power_two_division_matches_full_word_coordinates",
        message: "division numeric API mismatch",
        covers: &[
            "division API zero shift is reachable",
            "division API high word crossing is reachable",
            "division API bit255 is reachable",
            "division API zero number is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-self-signed-domain-api",
        harness: "signed_self_context_domain_preserves_prepared_inputs",
        message: "division context domain mismatch",
        covers: &[
            "division class native native sources is reachable",
            "division class native big sources is reachable",
            "division class big native sources is reachable",
            "division class big big sources is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-self-div-unsigned",
        harness: "wide_division_self_div_unsigned_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "division class minimum width is reachable",
            "division class maximum width is reachable",
            "division class mixed source signs is reachable",
            "division class unsigned nonunit self is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-self-div-unsigned-context",
        harness: "wide_division_self_div_unsigned_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "division class minimum width is reachable",
            "division class maximum width is reachable",
            "division class mixed source signs is reachable",
            "division class unsigned nonunit self is reachable",
            "division class native native sources is reachable",
            "division class native big sources is reachable",
            "division class big native sources is reachable",
            "division class big big sources is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-self-div-signed",
        harness: "wide_division_self_div_signed_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "division class minimum width is reachable",
            "division class maximum width is reachable",
            "division class mixed source signs is reachable",
            "division class minimum signed self is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-self-div-signed-context",
        harness: "wide_division_self_div_signed_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "division class minimum width is reachable",
            "division class maximum width is reachable",
            "division class mixed source signs is reachable",
            "division class minimum signed self is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-self-rem-unsigned",
        harness: "wide_division_self_rem_unsigned_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "division class minimum width is reachable",
            "division class maximum width is reachable",
            "division class mixed source signs is reachable",
            "division class unsigned nonunit self is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-self-rem-unsigned-context",
        harness: "wide_division_self_rem_unsigned_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "division class minimum width is reachable",
            "division class maximum width is reachable",
            "division class mixed source signs is reachable",
            "division class unsigned nonunit self is reachable",
            "division class native native sources is reachable",
            "division class native big sources is reachable",
            "division class big native sources is reachable",
            "division class big big sources is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-self-rem-signed",
        harness: "wide_division_self_rem_signed_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "division class minimum width is reachable",
            "division class maximum width is reachable",
            "division class mixed source signs is reachable",
            "division class minimum signed self is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-self-rem-signed-context",
        harness: "wide_division_self_rem_signed_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "division class minimum width is reachable",
            "division class maximum width is reachable",
            "division class mixed source signs is reachable",
            "division class minimum signed self is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-smaller-div",
        harness: "wide_division_smaller_div_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "division class minimum width is reachable",
            "division class maximum width is reachable",
            "division class mixed source signs is reachable",
            "division class zero smaller numerator is reachable",
            "division class upper smaller divisor is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-smaller-div-context",
        harness: "wide_division_smaller_div_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "division class minimum width is reachable",
            "division class maximum width is reachable",
            "division class mixed source signs is reachable",
            "division class zero smaller numerator is reachable",
            "division class upper smaller divisor is reachable",
            "division class native native sources is reachable",
            "division class native big sources is reachable",
            "division class big native sources is reachable",
            "division class big big sources is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-smaller-rem",
        harness: "wide_division_smaller_rem_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "division class minimum width is reachable",
            "division class maximum width is reachable",
            "division class mixed source signs is reachable",
            "division class zero smaller numerator is reachable",
            "division class upper smaller divisor is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-smaller-rem-context",
        harness: "wide_division_smaller_rem_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "division class minimum width is reachable",
            "division class maximum width is reachable",
            "division class mixed source signs is reachable",
            "division class zero smaller numerator is reachable",
            "division class upper smaller divisor is reachable",
            "division class native native sources is reachable",
            "division class native big sources is reachable",
            "division class big native sources is reachable",
            "division class big big sources is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-pow2-div",
        harness: "wide_division_pow2_div_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "division class minimum width is reachable",
            "division class maximum width is reachable",
            "division class mixed source signs is reachable",
            "division class divisor one is reachable",
            "division class word crossing is reachable",
            "division class top divisor bit is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-pow2-div-context",
        harness: "wide_division_pow2_div_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "division class minimum width is reachable",
            "division class maximum width is reachable",
            "division class mixed source signs is reachable",
            "division class divisor one is reachable",
            "division class word crossing is reachable",
            "division class top divisor bit is reachable",
            "division class native native sources is reachable",
            "division class native big sources is reachable",
            "division class big native sources is reachable",
            "division class big big sources is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-pow2-rem",
        harness: "wide_division_pow2_rem_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "division class minimum width is reachable",
            "division class maximum width is reachable",
            "division class mixed source signs is reachable",
            "division class divisor one is reachable",
            "division class word crossing is reachable",
            "division class top divisor bit is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "division256-pow2-rem-context",
        harness: "wide_division_pow2_rem_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "division class minimum width is reachable",
            "division class maximum width is reachable",
            "division class mixed source signs is reachable",
            "division class divisor one is reachable",
            "division class word crossing is reachable",
            "division class top divisor bit is reachable",
            "division class native native sources is reachable",
            "division class native big sources is reachable",
            "division class big native sources is reachable",
            "division class big big sources is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "special256-signed-api",
        harness: "special_signed_numeric_apis_match_native_arithmetic",
        message: "special numeric API mismatch",
        covers: &[
            "special API zero over upper divisor is reachable",
            "special API minimum over minus one is reachable",
            "special API negative quotient is reachable",
            "special API negative divisor is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "special256-power-api",
        harness: "special_unsigned_and_power_apis_match_scalar_identities",
        message: "special numeric API mismatch",
        covers: &[
            "special API bit128 modulus is reachable",
            "special API zero power upper exponent is reachable",
            "special API one power is reachable",
            "special API upper base is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "special256-signed-div-unit",
        harness: "wide_special_signed_div_unit_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "special minimum width is reachable",
            "special maximum width is reachable",
            "special mixed source signs is reachable",
            "special zero dividend is reachable",
            "special minimum signed dividend is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "special256-signed-div-unit-context",
        harness: "wide_special_signed_div_unit_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "special minimum width is reachable",
            "special maximum width is reachable",
            "special mixed source signs is reachable",
            "special zero dividend is reachable",
            "special minimum signed dividend is reachable",
            "special native native sources is reachable",
            "special native big sources is reachable",
            "special big native sources is reachable",
            "special big big sources is reachable",
            "special signed one bit minus one is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "special256-signed-rem-unit",
        harness: "wide_special_signed_rem_unit_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "special minimum width is reachable",
            "special maximum width is reachable",
            "special mixed source signs is reachable",
            "special zero dividend is reachable",
            "special minimum signed dividend is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "special256-signed-rem-unit-context",
        harness: "wide_special_signed_rem_unit_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "special minimum width is reachable",
            "special maximum width is reachable",
            "special mixed source signs is reachable",
            "special zero dividend is reachable",
            "special minimum signed dividend is reachable",
            "special native native sources is reachable",
            "special native big sources is reachable",
            "special big native sources is reachable",
            "special big big sources is reachable",
            "special signed one bit minus one is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "special256-zero-div",
        harness: "wide_special_zero_div_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "special minimum width is reachable",
            "special maximum width is reachable",
            "special mixed source signs is reachable",
            "special signed zero division is reachable",
            "special unsigned zero division is reachable",
            "special all ones divisor is reachable",
            "special upper divisor bit is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "special256-zero-div-context",
        harness: "wide_special_zero_div_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "special minimum width is reachable",
            "special maximum width is reachable",
            "special mixed source signs is reachable",
            "special signed zero division is reachable",
            "special unsigned zero division is reachable",
            "special all ones divisor is reachable",
            "special upper divisor bit is reachable",
            "special native native sources is reachable",
            "special native big sources is reachable",
            "special big native sources is reachable",
            "special big big sources is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "special256-zero-rem",
        harness: "wide_special_zero_rem_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "special minimum width is reachable",
            "special maximum width is reachable",
            "special mixed source signs is reachable",
            "special signed zero division is reachable",
            "special unsigned zero division is reachable",
            "special all ones divisor is reachable",
            "special upper divisor bit is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "special256-zero-rem-context",
        harness: "wide_special_zero_rem_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "special minimum width is reachable",
            "special maximum width is reachable",
            "special mixed source signs is reachable",
            "special signed zero division is reachable",
            "special unsigned zero division is reachable",
            "special all ones divisor is reachable",
            "special upper divisor bit is reachable",
            "special native native sources is reachable",
            "special native big sources is reachable",
            "special big native sources is reachable",
            "special big big sources is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "special256-power-one",
        harness: "wide_special_power_one_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "special minimum width is reachable",
            "special maximum width is reachable",
            "special mixed source signs is reachable",
            "special exponent wider than result is reachable",
            "special one bit positive exponent is reachable",
            "special wide signed positive exponent is reachable",
            "special power minimum signed base is reachable",
            "special power upper unsigned base is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "special256-power-one-context",
        harness: "wide_special_power_one_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "special minimum width is reachable",
            "special maximum width is reachable",
            "special mixed source signs is reachable",
            "special exponent wider than result is reachable",
            "special one bit positive exponent is reachable",
            "special wide signed positive exponent is reachable",
            "special power minimum signed base is reachable",
            "special power upper unsigned base is reachable",
            "special native native sources is reachable",
            "special native big sources is reachable",
            "special big native sources is reachable",
            "special big big sources is reachable",
            "special signed one bit base is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "special256-power-unit-base",
        harness: "wide_special_power_unit_base_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "special minimum width is reachable",
            "special maximum width is reachable",
            "special mixed source signs is reachable",
            "special exponent wider than result is reachable",
            "special one bit positive exponent is reachable",
            "special wide signed positive exponent is reachable",
            "special power zero base is reachable",
            "special power one base is reachable",
            "special power upper exponent bit is reachable",
            "special power positive even exponent is reachable",
            "special power positive odd exponent is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "special256-power-unit-base-context",
        harness: "wide_special_power_unit_base_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "special minimum width is reachable",
            "special maximum width is reachable",
            "special mixed source signs is reachable",
            "special exponent wider than result is reachable",
            "special one bit positive exponent is reachable",
            "special wide signed positive exponent is reachable",
            "special power zero base is reachable",
            "special power one base is reachable",
            "special power upper exponent bit is reachable",
            "special power positive even exponent is reachable",
            "special power positive odd exponent is reachable",
            "special native native sources is reachable",
            "special native big sources is reachable",
            "special big native sources is reachable",
            "special big big sources is reachable",
            "special unit base zero extension is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "unit256-div-one-reference",
        harness: "restoring_division_by_one_preserves_number",
        message: "unit division reference mismatch",
        covers: &[
            "unit division reference minimum width is reachable",
            "unit division reference maximum width is reachable",
            "unit division reference upper bit is reachable",
            "unit division reference zero is reachable",
            "unit division reference maximum number is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "unit256-api",
        harness: "numeric_unit_apis_preserve_native_half_arithmetic",
        message: "unit numeric API mismatch",
        covers: &[
            "unit API zero factor is reachable",
            "unit API one factor is reachable",
            "unit API bit255 is reachable",
            "unit API zero number is reachable",
            "unit API full magnitude is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "identity256-mul-right",
        harness: "wide_unit_mul_right_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "unit identity minimum width is reachable",
            "unit identity maximum width is reachable",
            "unit identity zero number is reachable",
            "unit identity all ones number is reachable",
            "unit identity upper bit is reachable",
            "unit identity mixed source signs is reachable",
            "unit identity unsigned mode signed sources is reachable",
            "unit identity alternating number is reachable",
            "unit identity zero factor is reachable",
            "unit identity one factor is reachable",
            "unit identity signed multiplication is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "identity256-mul-right-context",
        harness: "wide_unit_mul_right_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "unit identity minimum width is reachable",
            "unit identity maximum width is reachable",
            "unit identity zero number is reachable",
            "unit identity all ones number is reachable",
            "unit identity upper bit is reachable",
            "unit identity mixed source signs is reachable",
            "unit identity unsigned mode signed sources is reachable",
            "unit identity alternating number is reachable",
            "unit identity zero factor is reachable",
            "unit identity one factor is reachable",
            "unit identity signed multiplication is reachable",
            "unit identity native native sources is reachable",
            "unit identity native big sources is reachable",
            "unit identity big native sources is reachable",
            "unit identity big big sources is reachable",
            "unit identity signed one bit one zero extension is reachable",
            "unit identity signed one bit zero sign extension is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "identity256-mul-left",
        harness: "wide_unit_mul_left_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "unit identity minimum width is reachable",
            "unit identity maximum width is reachable",
            "unit identity zero number is reachable",
            "unit identity all ones number is reachable",
            "unit identity upper bit is reachable",
            "unit identity mixed source signs is reachable",
            "unit identity unsigned mode signed sources is reachable",
            "unit identity alternating number is reachable",
            "unit identity zero factor is reachable",
            "unit identity one factor is reachable",
            "unit identity signed multiplication is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "identity256-mul-left-context",
        harness: "wide_unit_mul_left_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "unit identity minimum width is reachable",
            "unit identity maximum width is reachable",
            "unit identity zero number is reachable",
            "unit identity all ones number is reachable",
            "unit identity upper bit is reachable",
            "unit identity mixed source signs is reachable",
            "unit identity unsigned mode signed sources is reachable",
            "unit identity alternating number is reachable",
            "unit identity zero factor is reachable",
            "unit identity one factor is reachable",
            "unit identity signed multiplication is reachable",
            "unit identity native native sources is reachable",
            "unit identity native big sources is reachable",
            "unit identity big native sources is reachable",
            "unit identity big big sources is reachable",
            "unit identity signed one bit one zero extension is reachable",
            "unit identity signed one bit zero sign extension is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "identity256-div-one",
        harness: "wide_unit_div_one_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "unit identity minimum width is reachable",
            "unit identity maximum width is reachable",
            "unit identity zero number is reachable",
            "unit identity all ones number is reachable",
            "unit identity upper bit is reachable",
            "unit identity mixed source signs is reachable",
            "unit identity unsigned mode signed sources is reachable",
            "unit identity alternating number is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "identity256-div-one-context",
        harness: "wide_unit_div_one_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "unit identity minimum width is reachable",
            "unit identity maximum width is reachable",
            "unit identity zero number is reachable",
            "unit identity all ones number is reachable",
            "unit identity upper bit is reachable",
            "unit identity mixed source signs is reachable",
            "unit identity unsigned mode signed sources is reachable",
            "unit identity alternating number is reachable",
            "unit identity native native sources is reachable",
            "unit identity native big sources is reachable",
            "unit identity big native sources is reachable",
            "unit identity big big sources is reachable",
            "unit identity signed one bit one zero extension is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "identity256-rem-one",
        harness: "wide_unit_rem_one_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "unit identity minimum width is reachable",
            "unit identity maximum width is reachable",
            "unit identity zero number is reachable",
            "unit identity all ones number is reachable",
            "unit identity upper bit is reachable",
            "unit identity mixed source signs is reachable",
            "unit identity unsigned mode signed sources is reachable",
            "unit identity alternating number is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "identity256-rem-one-context",
        harness: "wide_unit_rem_one_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "unit identity minimum width is reachable",
            "unit identity maximum width is reachable",
            "unit identity zero number is reachable",
            "unit identity all ones number is reachable",
            "unit identity upper bit is reachable",
            "unit identity mixed source signs is reachable",
            "unit identity unsigned mode signed sources is reachable",
            "unit identity alternating number is reachable",
            "unit identity native native sources is reachable",
            "unit identity native big sources is reachable",
            "unit identity big native sources is reachable",
            "unit identity big big sources is reachable",
            "unit identity signed one bit one zero extension is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "power256-zero-api",
        harness: "power_zero_api_and_signed_bridge_preserve_values",
        message: "power numeric API mismatch",
        covers: &[
            "power bridge negative zero is reachable",
            "power bridge no sign is reachable",
            "power bridge wide magnitude is reachable",
            "power zero API modulus one is reachable",
            "power zero API bit128 modulus is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "power256-unknown",
        harness: "wide_power_unknown_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide power minimum result is reachable",
            "wide power maximum result is reachable",
            "wide power minimum exponent is reachable",
            "wide power maximum exponent is reachable",
            "wide power native exponent is reachable",
            "wide power big exponent is reachable",
            "wide power exponent wider than result is reachable",
            "wide power signed context is reachable",
            "wide power unsigned context is reachable",
            "wide power only exponent unknown is reachable",
            "wide power both operands unknown is reachable",
            "wide power unknown negative exponent is reachable",
            "wide power upper exponent unknown is reachable",
            "wide power unknown exponent clears result sign is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "power256-unknown-context",
        harness: "wide_power_unknown_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide power minimum result is reachable",
            "wide power maximum result is reachable",
            "wide power minimum exponent is reachable",
            "wide power maximum exponent is reachable",
            "wide power native exponent is reachable",
            "wide power big exponent is reachable",
            "wide power exponent wider than result is reachable",
            "wide power signed context is reachable",
            "wide power unsigned context is reachable",
            "wide power only exponent unknown is reachable",
            "wide power both operands unknown is reachable",
            "wide power unknown negative exponent is reachable",
            "wide power upper exponent unknown is reachable",
            "wide power unknown exponent clears result sign is reachable",
            "wide power native native source is reachable",
            "wide power native big source is reachable",
            "wide power big native source is reachable",
            "wide power big big source is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "power256-negative",
        harness: "wide_power_negative_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide power minimum result is reachable",
            "wide power maximum result is reachable",
            "wide power minimum exponent is reachable",
            "wide power maximum exponent is reachable",
            "wide power native exponent is reachable",
            "wide power big exponent is reachable",
            "wide power exponent wider than result is reachable",
            "wide power signed context is reachable",
            "wide power unsigned context is reachable",
            "wide power negative exponent zero base is reachable",
            "wide power negative exponent one base is reachable",
            "wide power negative exponent unknown base is reachable",
            "wide power minus one odd negative is reachable",
            "wide power minus one even negative is reachable",
            "wide power negative exponent other base is reachable",
            "wide power minimum signed exponent is reachable",
            "wide power unsigned all ones base is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "power256-negative-context",
        harness: "wide_power_negative_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide power minimum result is reachable",
            "wide power maximum result is reachable",
            "wide power minimum exponent is reachable",
            "wide power maximum exponent is reachable",
            "wide power native exponent is reachable",
            "wide power big exponent is reachable",
            "wide power exponent wider than result is reachable",
            "wide power signed context is reachable",
            "wide power unsigned context is reachable",
            "wide power negative exponent zero base is reachable",
            "wide power negative exponent one base is reachable",
            "wide power negative exponent unknown base is reachable",
            "wide power minus one odd negative is reachable",
            "wide power minus one even negative is reachable",
            "wide power negative exponent other base is reachable",
            "wide power minimum signed exponent is reachable",
            "wide power unsigned all ones base is reachable",
            "wide power native native source is reachable",
            "wide power native big source is reachable",
            "wide power big native source is reachable",
            "wide power big big source is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "power256-zero",
        harness: "wide_power_zero_prepared_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide power minimum result is reachable",
            "wide power maximum result is reachable",
            "wide power minimum exponent is reachable",
            "wide power maximum exponent is reachable",
            "wide power native exponent is reachable",
            "wide power big exponent is reachable",
            "wide power exponent wider than result is reachable",
            "wide power signed context is reachable",
            "wide power unsigned context is reachable",
            "wide power zero to zero is reachable",
            "wide power other base to zero is reachable",
            "wide power signed minimum base to zero is reachable",
            "wide power unknown base to zero is reachable",
            "wide power zero exponent retains equal width sign is reachable",
            "wide power zero exponent unsigned base is reachable",
            "wide power signed zero exponent is reachable",
            "wide power all ones to zero is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "power256-zero-context",
        harness: "wide_power_zero_context_matches_reference",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide power minimum result is reachable",
            "wide power maximum result is reachable",
            "wide power minimum exponent is reachable",
            "wide power maximum exponent is reachable",
            "wide power native exponent is reachable",
            "wide power big exponent is reachable",
            "wide power exponent wider than result is reachable",
            "wide power signed context is reachable",
            "wide power unsigned context is reachable",
            "wide power zero to zero is reachable",
            "wide power other base to zero is reachable",
            "wide power signed minimum base to zero is reachable",
            "wide power unknown base to zero is reachable",
            "wide power zero exponent retains equal width sign is reachable",
            "wide power zero exponent unsigned base is reachable",
            "wide power signed zero exponent is reachable",
            "wide power all ones to zero is reachable",
            "wide power native native source is reachable",
            "wide power native big source is reachable",
            "wide power big native source is reachable",
            "wide power big big source is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "exception256-mul",
        harness: "wide_mul_exception_prepared_matches_all_x",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide exception minimum width is reachable",
            "wide exception maximum width is reachable",
            "wide exception only lhs unknown is reachable",
            "wide exception only rhs unknown is reachable",
            "wide exception both unknown is reachable",
            "wide exception upper unknown is reachable",
            "wide exception signed mixed sources is reachable",
            "wide exception unsigned mode signed sources is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "exception256-mul-context",
        harness: "wide_mul_exception_preparation_matches_all_x",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide exception minimum width is reachable",
            "wide exception maximum width is reachable",
            "wide exception only lhs unknown is reachable",
            "wide exception only rhs unknown is reachable",
            "wide exception both unknown is reachable",
            "wide exception upper unknown is reachable",
            "wide exception signed mixed sources is reachable",
            "wide exception unsigned mode signed sources is reachable",
            "wide exception native native preparation is reachable",
            "wide exception native big preparation is reachable",
            "wide exception big native preparation is reachable",
            "wide exception big big preparation is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "exception256-div",
        harness: "wide_div_exception_prepared_matches_all_x",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide exception minimum width is reachable",
            "wide exception maximum width is reachable",
            "wide exception only lhs unknown is reachable",
            "wide exception only rhs unknown is reachable",
            "wide exception both unknown is reachable",
            "wide exception upper unknown is reachable",
            "wide exception signed mixed sources is reachable",
            "wide exception unsigned mode signed sources is reachable",
            "wide exception known zero divisor is reachable",
            "wide exception zero over zero is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "exception256-div-context",
        harness: "wide_div_exception_preparation_matches_all_x",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide exception minimum width is reachable",
            "wide exception maximum width is reachable",
            "wide exception only lhs unknown is reachable",
            "wide exception only rhs unknown is reachable",
            "wide exception both unknown is reachable",
            "wide exception upper unknown is reachable",
            "wide exception signed mixed sources is reachable",
            "wide exception unsigned mode signed sources is reachable",
            "wide exception native native preparation is reachable",
            "wide exception native big preparation is reachable",
            "wide exception big native preparation is reachable",
            "wide exception big big preparation is reachable",
            "wide exception known zero divisor is reachable",
            "wide exception zero over zero is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "exception256-rem",
        harness: "wide_rem_exception_prepared_matches_all_x",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide exception minimum width is reachable",
            "wide exception maximum width is reachable",
            "wide exception only lhs unknown is reachable",
            "wide exception only rhs unknown is reachable",
            "wide exception both unknown is reachable",
            "wide exception upper unknown is reachable",
            "wide exception signed mixed sources is reachable",
            "wide exception unsigned mode signed sources is reachable",
            "wide exception known zero divisor is reachable",
            "wide exception zero over zero is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "exception256-rem-context",
        harness: "wide_rem_exception_preparation_matches_all_x",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide exception minimum width is reachable",
            "wide exception maximum width is reachable",
            "wide exception only lhs unknown is reachable",
            "wide exception only rhs unknown is reachable",
            "wide exception both unknown is reachable",
            "wide exception upper unknown is reachable",
            "wide exception signed mixed sources is reachable",
            "wide exception unsigned mode signed sources is reachable",
            "wide exception native native preparation is reachable",
            "wide exception native big preparation is reachable",
            "wide exception big native preparation is reachable",
            "wide exception big big preparation is reachable",
            "wide exception known zero divisor is reachable",
            "wide exception zero over zero is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "arithmetic256-plus",
        harness: "wide_unary_plus_preserves_prepared_planes",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide plus minimum zero is reachable",
            "wide plus upper one is reachable",
            "wide plus upper X is reachable",
            "wide plus upper Z is reachable",
            "wide plus partial unknown is reachable",
            "wide plus equal width retains sign is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-plus",
        harness: "wide_unary_plus_original_preparation_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide plus native sign extension is reachable",
            "wide plus native zero extension is reachable",
            "wide plus X sign extension is reachable",
            "wide plus Z sign extension is reachable",
            "wide plus equal width preserves signed metadata is reachable",
            "wide plus unsigned source stays unsigned is reachable",
            "wide plus disabled sign extension is reachable",
            "wide plus maximum unknown input is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "logical256-truth-reference",
        harness: "logical_truth_and_zero_extension_match_scalar",
        message: "wide logical truth mismatch",
        covers: &[
            "logical truth full zero extension is reachable",
            "logical truth wide zero is reachable",
            "logical truth upper one is reachable",
            "logical truth upper X is reachable",
            "logical truth upper Z is reachable",
            "logical truth one dominates unknown is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "logical256-and-native-big",
        harness: "wide_logical_and_native_big_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide logical false is reachable",
            "wide logical true is reachable",
            "wide logical unknown is reachable",
            "wide logical zero with unknown is reachable",
            "wide logical one with unknown is reachable",
            "wide logical known one with XZ is reachable",
            "wide logical upper source widths narrow result is reachable",
            "wide logical lower source widths wide result is reachable",
            "wide logical independent signs are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "logical256-and-big-native",
        harness: "wide_logical_and_big_native_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide logical false is reachable",
            "wide logical true is reachable",
            "wide logical unknown is reachable",
            "wide logical zero with unknown is reachable",
            "wide logical one with unknown is reachable",
            "wide logical known one with XZ is reachable",
            "wide logical upper source widths narrow result is reachable",
            "wide logical lower source widths wide result is reachable",
            "wide logical independent signs are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "logical256-and-big-big",
        harness: "wide_logical_and_big_big_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide logical false is reachable",
            "wide logical true is reachable",
            "wide logical unknown is reachable",
            "wide logical zero with unknown is reachable",
            "wide logical one with unknown is reachable",
            "wide logical known one with XZ is reachable",
            "wide logical upper source widths narrow result is reachable",
            "wide logical lower source widths wide result is reachable",
            "wide logical independent signs are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "logical256-or-native-big",
        harness: "wide_logical_or_native_big_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide logical false is reachable",
            "wide logical true is reachable",
            "wide logical unknown is reachable",
            "wide logical zero with unknown is reachable",
            "wide logical one with unknown is reachable",
            "wide logical known one with XZ is reachable",
            "wide logical upper source widths narrow result is reachable",
            "wide logical lower source widths wide result is reachable",
            "wide logical independent signs are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "logical256-or-big-native",
        harness: "wide_logical_or_big_native_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide logical false is reachable",
            "wide logical true is reachable",
            "wide logical unknown is reachable",
            "wide logical zero with unknown is reachable",
            "wide logical one with unknown is reachable",
            "wide logical known one with XZ is reachable",
            "wide logical upper source widths narrow result is reachable",
            "wide logical lower source widths wide result is reachable",
            "wide logical independent signs are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "logical256-or-big-big",
        harness: "wide_logical_or_big_big_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide logical false is reachable",
            "wide logical true is reachable",
            "wide logical unknown is reachable",
            "wide logical zero with unknown is reachable",
            "wide logical one with unknown is reachable",
            "wide logical known one with XZ is reachable",
            "wide logical upper source widths narrow result is reachable",
            "wide logical lower source widths wide result is reachable",
            "wide logical independent signs are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "conversion256-bigint",
        harness: "wide_bigint_conversion_matches_signed_decoding",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide signed conversion minimum is reachable",
            "wide signed conversion zero is reachable",
            "wide signed conversion upper unknown is reachable",
            "wide signed conversion positive is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "numeric256-order-api",
        harness: "numeric256_ordering_api_matches_scalar_order",
        message: "wide ordering numeric API mismatch",
        covers: &[
            "ordering API high half difference is reachable",
            "ordering API negative magnitudes are reachable",
            "ordering API negative zero normalization is reachable",
            "ordering API no-sign normalization is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ordering256-less-unsigned",
        harness: "wide_ordering_less_unsigned_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide ordering known true is reachable",
            "wide ordering known false is reachable",
            "wide ordering opposite high bits are reachable",
            "wide ordering equal operands are reachable",
            "wide ordering both high bits set are reachable",
            "wide ordering widens boolean result is reachable",
            "wide ordering narrow result keeps full comparison is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ordering256-less-signed",
        harness: "wide_ordering_less_signed_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide ordering known true is reachable",
            "wide ordering known false is reachable",
            "wide ordering opposite high bits are reachable",
            "wide ordering equal operands are reachable",
            "wide ordering both high bits set are reachable",
            "wide ordering widens boolean result is reachable",
            "wide ordering narrow result keeps full comparison is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ordering256-less-unknown",
        harness: "wide_ordering_less_unknown_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide ordering upper unknown is reachable",
            "wide ordering only lhs unknown is reachable",
            "wide ordering only rhs unknown is reachable",
            "wide ordering both unknown is reachable",
            "wide ordering known mismatch under unknown is reachable",
            "wide ordering widens boolean result is reachable",
            "wide ordering narrow result keeps full comparison is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-less",
        harness: "wide_preparation_less_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring mixed storage and signs are reachable",
            "wiring ordering signed operands are reachable",
            "wiring comparison separates result width",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ordering256-less-eq-unsigned",
        harness: "wide_ordering_less_eq_unsigned_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide ordering known true is reachable",
            "wide ordering known false is reachable",
            "wide ordering opposite high bits are reachable",
            "wide ordering equal operands are reachable",
            "wide ordering both high bits set are reachable",
            "wide ordering widens boolean result is reachable",
            "wide ordering narrow result keeps full comparison is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ordering256-less-eq-signed",
        harness: "wide_ordering_less_eq_signed_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide ordering known true is reachable",
            "wide ordering known false is reachable",
            "wide ordering opposite high bits are reachable",
            "wide ordering equal operands are reachable",
            "wide ordering both high bits set are reachable",
            "wide ordering widens boolean result is reachable",
            "wide ordering narrow result keeps full comparison is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ordering256-less-eq-unknown",
        harness: "wide_ordering_less_eq_unknown_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide ordering upper unknown is reachable",
            "wide ordering only lhs unknown is reachable",
            "wide ordering only rhs unknown is reachable",
            "wide ordering both unknown is reachable",
            "wide ordering known mismatch under unknown is reachable",
            "wide ordering widens boolean result is reachable",
            "wide ordering narrow result keeps full comparison is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-less-eq",
        harness: "wide_preparation_less_eq_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring mixed storage and signs are reachable",
            "wiring ordering signed operands are reachable",
            "wiring comparison separates result width",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ordering256-greater-unsigned",
        harness: "wide_ordering_greater_unsigned_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide ordering known true is reachable",
            "wide ordering known false is reachable",
            "wide ordering opposite high bits are reachable",
            "wide ordering equal operands are reachable",
            "wide ordering both high bits set are reachable",
            "wide ordering widens boolean result is reachable",
            "wide ordering narrow result keeps full comparison is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ordering256-greater-signed",
        harness: "wide_ordering_greater_signed_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide ordering known true is reachable",
            "wide ordering known false is reachable",
            "wide ordering opposite high bits are reachable",
            "wide ordering equal operands are reachable",
            "wide ordering both high bits set are reachable",
            "wide ordering widens boolean result is reachable",
            "wide ordering narrow result keeps full comparison is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ordering256-greater-unknown",
        harness: "wide_ordering_greater_unknown_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide ordering upper unknown is reachable",
            "wide ordering only lhs unknown is reachable",
            "wide ordering only rhs unknown is reachable",
            "wide ordering both unknown is reachable",
            "wide ordering known mismatch under unknown is reachable",
            "wide ordering widens boolean result is reachable",
            "wide ordering narrow result keeps full comparison is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-greater",
        harness: "wide_preparation_greater_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring mixed storage and signs are reachable",
            "wiring ordering signed operands are reachable",
            "wiring comparison separates result width",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ordering256-greater-eq-unsigned",
        harness: "wide_ordering_greater_eq_unsigned_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide ordering known true is reachable",
            "wide ordering known false is reachable",
            "wide ordering opposite high bits are reachable",
            "wide ordering equal operands are reachable",
            "wide ordering both high bits set are reachable",
            "wide ordering widens boolean result is reachable",
            "wide ordering narrow result keeps full comparison is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ordering256-greater-eq-signed",
        harness: "wide_ordering_greater_eq_signed_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide ordering known true is reachable",
            "wide ordering known false is reachable",
            "wide ordering opposite high bits are reachable",
            "wide ordering equal operands are reachable",
            "wide ordering both high bits set are reachable",
            "wide ordering widens boolean result is reachable",
            "wide ordering narrow result keeps full comparison is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "ordering256-greater-eq-unknown",
        harness: "wide_ordering_greater_eq_unknown_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide ordering upper unknown is reachable",
            "wide ordering only lhs unknown is reachable",
            "wide ordering only rhs unknown is reachable",
            "wide ordering both unknown is reachable",
            "wide ordering known mismatch under unknown is reachable",
            "wide ordering widens boolean result is reachable",
            "wide ordering narrow result keeps full comparison is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-greater-eq",
        harness: "wide_preparation_greater_eq_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring mixed storage and signs are reachable",
            "wiring ordering signed operands are reachable",
            "wiring comparison separates result width",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "numeric256-add-api",
        harness: "numeric256_addition_api_preserves_high_half",
        message: "wide numeric addition API mismatch",
        covers: &[
            "addition API carry into high half is reachable",
            "addition API bit255 is reachable",
            "addition API zero sum is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "numeric256-add",
        harness: "numeric256_addition_matches_scalar_carry",
        message: "wide numeric addition mismatch",
        covers: &[
            "numeric addition carry into bit128 is reachable",
            "numeric addition carry beyond bit255 is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "equality256-eq",
        harness: "wide_equality_eq_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide equality upper unknown is reachable",
            "wide equality known equal is reachable",
            "wide equality mixed sign is reachable",
            "wide equality widens boolean result is reachable",
            "wide equality narrow result keeps full comparison is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "equality256-ne",
        harness: "wide_equality_ne_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide equality upper unknown is reachable",
            "wide equality known equal is reachable",
            "wide equality mixed sign is reachable",
            "wide equality widens boolean result is reachable",
            "wide equality narrow result keeps full comparison is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "equality256-wild-eq",
        harness: "wide_equality_wild_eq_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide equality upper unknown is reachable",
            "wide equality known equal is reachable",
            "wide equality mixed sign is reachable",
            "wide equality widens boolean result is reachable",
            "wide equality narrow result keeps full comparison is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "equality256-wild-ne",
        harness: "wide_equality_wild_ne_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide equality upper unknown is reachable",
            "wide equality known equal is reachable",
            "wide equality mixed sign is reachable",
            "wide equality widens boolean result is reachable",
            "wide equality narrow result keeps full comparison is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "arithmetic256-add",
        harness: "wide_arithmetic_add_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide arithmetic carry boundary is reachable",
            "wide arithmetic upper unknown is reachable",
            "wide arithmetic signed lower boundary is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "arithmetic256-sub",
        harness: "wide_arithmetic_sub_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide arithmetic carry boundary is reachable",
            "wide arithmetic upper unknown is reachable",
            "wide arithmetic signed lower boundary is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "arithmetic256-neg",
        harness: "wide_arithmetic_neg_matches_scalar",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide arithmetic carry boundary is reachable",
            "wide arithmetic upper unknown is reachable",
            "wide arithmetic signed lower boundary is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-and",
        harness: "wide_preparation_and_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring mixed storage and signs are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-or",
        harness: "wide_preparation_or_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring mixed storage and signs are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-xor",
        harness: "wide_preparation_xor_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring mixed storage and signs are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-xnor",
        harness: "wide_preparation_xnor_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring mixed storage and signs are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-not",
        harness: "wide_preparation_not_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-left",
        harness: "wide_preparation_left_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring unknown wide amount is reachable",
            "wiring known wide amount is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-right",
        harness: "wide_preparation_right_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring unknown wide amount is reachable",
            "wiring known wide amount is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-arith-left",
        harness: "wide_preparation_arith_left_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring unknown wide amount is reachable",
            "wiring known wide amount is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-arith-right",
        harness: "wide_preparation_arith_right_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring unknown wide amount is reachable",
            "wiring known wide amount is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-eq",
        harness: "wide_preparation_eq_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring mixed storage and signs are reachable",
            "wiring comparison ignores outer sign",
            "wiring comparison separates result width",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-ne",
        harness: "wide_preparation_ne_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring mixed storage and signs are reachable",
            "wiring comparison ignores outer sign",
            "wiring comparison separates result width",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-wild-eq",
        harness: "wide_preparation_wild_eq_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring mixed storage and signs are reachable",
            "wiring comparison ignores outer sign",
            "wiring comparison separates result width",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-wild-ne",
        harness: "wide_preparation_wild_ne_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring mixed storage and signs are reachable",
            "wiring comparison ignores outer sign",
            "wiring comparison separates result width",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-add",
        harness: "wide_preparation_add_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring mixed storage and signs are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-sub",
        harness: "wide_preparation_sub_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
            "wiring mixed storage and signs are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "wiring256-neg",
        harness: "wide_preparation_neg_wiring",
        message: "wide preparation wiring mismatch",
        covers: &[
            "wiring native signed source reaches wide context",
            "wiring BigUint source reaches wide context",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "reduction256-and",
        harness: "wide_reduction_and_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide reduction maximum widths are reachable",
            "wide reduction unknown source is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "reduction256-nand",
        harness: "wide_reduction_nand_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide reduction maximum widths are reachable",
            "wide reduction unknown source is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "reduction256-or",
        harness: "wide_reduction_or_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide reduction maximum widths are reachable",
            "wide reduction unknown source is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "reduction256-nor",
        harness: "wide_reduction_nor_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide reduction maximum widths are reachable",
            "wide reduction unknown source is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "reduction256-xor",
        harness: "wide_reduction_xor_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide reduction maximum widths are reachable",
            "wide reduction unknown source is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "reduction256-xnor",
        harness: "wide_reduction_xnor_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide reduction maximum widths are reachable",
            "wide reduction unknown source is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "reduction256-logic-not",
        harness: "wide_reduction_logic_not_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide reduction maximum widths are reachable",
            "wide reduction unknown source is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "assignment256-high",
        harness: "assignment_high_intermediate_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &["assignment bit254 intermediate is reachable"],
    },
    Contract {
        group: "public-big-values",
        name: "bitwise256-and",
        harness: "wide_bitwise_and_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide bitwise lower boundary is reachable",
            "wide bitwise upper unknown is reachable",
            "wide bitwise signed operands are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "bitwise256-or",
        harness: "wide_bitwise_or_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide bitwise lower boundary is reachable",
            "wide bitwise upper unknown is reachable",
            "wide bitwise signed operands are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "bitwise256-xor",
        harness: "wide_bitwise_xor_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide bitwise lower boundary is reachable",
            "wide bitwise upper unknown is reachable",
            "wide bitwise signed operands are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "bitwise256-xnor",
        harness: "wide_bitwise_xnor_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide bitwise lower boundary is reachable",
            "wide bitwise upper unknown is reachable",
            "wide bitwise signed operands are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "not256",
        harness: "wide_bit_not_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide NOT sign expansion is reachable",
            "wide NOT unknown is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "shift256-left",
        harness: "wide_shift_left_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide shift oversized amount is reachable",
            "wide shift unknown amount is reachable",
            "wide shift source sign partition is reachable",
            "wide shift bit255 intermediate input is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "shift256-right",
        harness: "wide_shift_right_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide shift oversized amount is reachable",
            "wide shift unknown amount is reachable",
            "wide shift source sign partition is reachable",
            "wide shift bit255 intermediate input is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "shift256-arith-left",
        harness: "wide_shift_arith_left_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide shift oversized amount is reachable",
            "wide shift unknown amount is reachable",
            "wide shift source sign partition is reachable",
            "wide shift bit255 intermediate input is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "shift256-arith-right-signed",
        harness: "wide_shift_arith_right_signed_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide shift oversized amount is reachable",
            "wide shift unknown amount is reachable",
            "wide shift source sign partition is reachable",
            "wide shift bit255 intermediate input is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "shift256-arith-right-unsigned",
        harness: "wide_shift_arith_right_unsigned_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide shift oversized amount is reachable",
            "wide shift unknown amount is reachable",
            "wide shift source sign partition is reachable",
            "wide shift bit255 intermediate input is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "numeric256-shifts",
        harness: "numeric256_shifts_follow_scalar_coordinates",
        message: "wide numeric semantics mismatch",
        covers: &[
            "numeric256 left crosses128 is reachable",
            "numeric256 right crosses128 is reachable",
            "numeric256 bit255 is reachable",
            "numeric256 oversized shift is reachable",
            "numeric256 zero mask is reachable",
            "numeric256 full mask is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "assignment256-native-native",
        harness: "assignment_native_native_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide assignment full window is reachable",
            "wide assignment last bit is reachable",
            "wide assignment shifted high RHS is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "assignment256-native-big",
        harness: "assignment_native_big_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide assignment full window is reachable",
            "wide assignment last bit is reachable",
            "wide assignment shifted high RHS is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "assignment256-big-native",
        harness: "assignment_big_native_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide assignment full window is reachable",
            "wide assignment last bit is reachable",
            "wide assignment shifted high RHS is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "assignment256-big-big",
        harness: "assignment_big_big_matches_scalar_bits",
        message: "wide operator semantics mismatch",
        covers: &[
            "wide assignment full window is reachable",
            "wide assignment last bit is reachable",
            "wide assignment shifted high RHS is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "numeric256-api",
        harness: "numeric256_equality_and_high_bits_match_scalar",
        message: "wide numeric semantics mismatch",
        covers: &[
            "numeric256 high difference is reachable",
            "numeric256 high equality is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "conditional-expansion-value-summary",
        harness: "wide_expansion_value_summary_matches_reference_semantics",
        message: "public BigUint semantics mismatch",
        covers: &[
            "wide expansion value summary borrowed sign is reachable",
            "wide expansion value summary native Z is reachable",
            "wide expansion value summary BigUint X is reachable",
            "wide expansion value summary unsigned zero is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "conditional-expansion-summary",
        harness: "expansion_summary_matches_scalar_bits",
        message: "public BigUint semantics mismatch",
        covers: &[
            "expansion summary native Z fill is reachable",
            "expansion summary BigUint X fill is reachable",
            "expansion summary borrowed sign is reachable",
            "expansion summary unsigned zero fill is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "big-numeric-wide-api",
        harness: "bounded_biguint_wide_api_follows_scalar_bits",
        message: "public BigUint semantics mismatch",
        covers: &[
            "wide numeric API zero zero is reachable",
            "wide numeric API zero one is reachable",
            "wide numeric API high one zero is reachable",
            "wide numeric API boundary one one is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "public-big-truncation",
        harness: "public_big_truncation_matches_reference_semantics",
        message: "public BigUint semantics mismatch",
        covers: &[
            "big truncation native low X is reachable",
            "big truncation native boundary Z is reachable",
            "big truncation discarded high Z is reachable",
            "big truncation wide high one is reachable",
            "big truncation no growth is reachable",
            "big truncation maximum identity is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "public-big-expansion-zero",
        harness: "public_big_zero_expansion_matches_reference_semantics",
        message: "public BigUint semantics mismatch",
        covers: &[
            "big expansion borrowed signed source is reachable",
            "big expansion unsigned zero fill is reachable",
            "big expansion signed zero fill is reachable",
            "big expansion signed one fill is reachable",
            "big expansion signed X fill is reachable",
            "big expansion signed Z fill is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "public-big-expansion",
        harness: "public_big_signed_expansion_matches_reference_semantics",
        message: "public BigUint semantics mismatch",
        covers: &[
            "big expansion borrowed signed source is reachable",
            "big expansion unsigned zero fill is reachable",
            "big expansion signed zero fill is reachable",
            "big expansion signed one fill is reachable",
            "big expansion signed X fill is reachable",
            "big expansion signed Z fill is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "public-big-selection-wide",
        harness: "public_big_wide_selection_matches_reference_semantics",
        message: "public BigUint semantics mismatch",
        covers: &[
            "big wide selection full width65 is reachable",
            "big wide selection full width128 is reachable",
            "big wide selection word crossing is reachable",
            "big wide selection partial overlap is reachable",
            "big wide selection maximum endpoints are reachable",
            "big wide selection high padding is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "public-conditional-wide-native-native",
        harness: "public_wide_conditional_native_native_matches_reference_semantics",
        message: "public BigUint semantics mismatch",
        covers: &[
            "wide conditional known zero is reachable",
            "wide conditional high known one is reachable",
            "wide conditional high unknown is reachable",
            "wide conditional common Z becomes X is reachable",
            "wide conditional mixed branch signs are reachable",
            "wide conditional natural context is reachable",
            "wide conditional maximum context is reachable",
            "wide conditional upper one observation is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "public-conditional-wide-native-big",
        harness: "public_wide_conditional_native_big_matches_reference_semantics",
        message: "public BigUint semantics mismatch",
        covers: &[
            "wide conditional known zero is reachable",
            "wide conditional high known one is reachable",
            "wide conditional high unknown is reachable",
            "wide conditional common Z becomes X is reachable",
            "wide conditional mixed branch signs are reachable",
            "wide conditional natural context is reachable",
            "wide conditional maximum context is reachable",
            "wide conditional upper one observation is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "public-conditional-wide-big-native",
        harness: "public_wide_conditional_big_native_matches_reference_semantics",
        message: "public BigUint semantics mismatch",
        covers: &[
            "wide conditional known zero is reachable",
            "wide conditional high known one is reachable",
            "wide conditional high unknown is reachable",
            "wide conditional common Z becomes X is reachable",
            "wide conditional mixed branch signs are reachable",
            "wide conditional natural context is reachable",
            "wide conditional maximum context is reachable",
            "wide conditional upper one observation is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "public-conditional-wide-big-big",
        harness: "public_wide_conditional_big_big_matches_reference_semantics",
        message: "public BigUint semantics mismatch",
        covers: &[
            "wide conditional known zero is reachable",
            "wide conditional high known one is reachable",
            "wide conditional high unknown is reachable",
            "wide conditional common Z becomes X is reachable",
            "wide conditional mixed branch signs are reachable",
            "wide conditional natural context is reachable",
            "wide conditional maximum context is reachable",
            "wide conditional upper one observation is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "big-numeric-api",
        harness: "bounded_biguint_numeric_api_follows_scalar_bits",
        message: "public BigUint semantics mismatch",
        covers: &[
            "big numeric API zero and maximum shift are reachable",
            "big numeric API two full words are reachable",
            "big numeric API word crossing is reachable",
            "big numeric API opposite high bits are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "big-numeric-bridge",
        harness: "bounded_biguint_numeric_bridge_preserves_bits",
        message: "public BigUint semantics mismatch",
        covers: &[
            "big numeric bridge zero is reachable",
            "big numeric bridge two words are reachable",
            "big numeric bridge word boundary is reachable",
            "big numeric bridge inline word is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "big-truth-reference",
        harness: "big_truth_reference_matches_scalar_fold",
        message: "public BigUint semantics mismatch",
        covers: &[
            "big truth reference zero is reachable",
            "big truth reference one is reachable",
            "big truth reference X is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "public-big-truth",
        harness: "public_big_truth_matches_reference_semantics",
        message: "public BigUint semantics mismatch",
        covers: &[
            "big truth zero endpoint is reachable",
            "big truth high one is reachable",
            "big truth high X is reachable",
            "big truth high Z is reachable",
            "big truth known one dominates X is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "public-big-conditional-native",
        harness: "public_big_condition_matches_reference_semantics",
        message: "public BigUint semantics mismatch",
        covers: &[
            "big conditional zero endpoint is reachable",
            "big conditional high one is reachable",
            "big conditional high X is reachable",
            "big conditional high Z is reachable",
            "big conditional mixed signs and known one are reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "public-native-sign-extension-wide",
        harness: "public_native_sign_extension_wide_matches_reference_semantics",
        message: "public BigUint semantics mismatch",
        covers: &[
            "big sign extension one is reachable",
            "big sign extension X is reachable",
            "big sign extension Z is reachable",
            "big sign extension interior is reachable",
        ],
    },
    Contract {
        group: "public-big-values",
        name: "public-big-selection-native",
        harness: "public_big_selection_matches_reference_semantics",
        message: "public BigUint semantics mismatch",
        covers: &[
            "big selection crosses word boundary is reachable",
            "big selection upper word is reachable",
            "big selection partial overlap is reachable",
            "big selection maximum endpoint is reachable",
            "big selection reversed endpoints are reachable",
            "big selection high Z is reachable",
        ],
    },
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
            "public BigUint getters u64 constructor zero is reachable",
            "public BigUint getters u64 constructor one is reachable",
            "public BigUint getters u64 constructor X is reachable",
            "public BigUint getters u64 constructor Z is reachable",
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
        group: "public-native-sequences",
        name: "public-shift-widening-4-to-8",
        harness: "public_shift_widening_4_to_8_matches_reference_semantics",
        message: "public shift semantics mismatch",
        covers: SHIFT_WIDENING_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-widening-4-to-64",
        harness: "public_shift_widening_4_to_64_matches_reference_semantics",
        message: "public shift semantics mismatch",
        covers: SHIFT_WIDENING_COVERS,
    },
    Contract {
        group: "public-getters",
        name: "public-big-amount-decoding-128",
        harness: "public_big_amount_decoding_matches_reference",
        message: "public BigUint amount semantics mismatch",
        covers: &[
            "public BigUint amount zero is reachable",
            "public BigUint amount usize maximum is reachable",
            "public BigUint oversized amount is reachable",
            "public BigUint upper X amount is reachable",
            "public BigUint upper Z amount is reachable",
        ],
    },
    Contract {
        group: "public-native-sequences",
        name: "public-word64-big-amount-small",
        harness: "public_word64_shift_small_amount_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-word64-big-amount-high",
        harness: "public_word64_shift_high_amount_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-word64-big-amount-upper-x",
        harness: "public_word64_shift_upper_x_amount_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-word64-big-amount-upper-z",
        harness: "public_word64_shift_upper_z_amount_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-word32-big-amount-small",
        harness: "public_word32_shift_small_amount_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-word32-big-amount-high",
        harness: "public_word32_shift_high_amount_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-word32-big-amount-upper-x",
        harness: "public_word32_shift_upper_x_amount_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-word32-big-amount-upper-z",
        harness: "public_word32_shift_upper_z_amount_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-big-amount-width-1-small",
        harness: "public_shift_big_amount_width_1_small_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-big-amount-width-1-high",
        harness: "public_shift_big_amount_width_1_high_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-big-amount-width-1-upper-x",
        harness: "public_shift_big_amount_width_1_upper_x_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-big-amount-width-1-upper-z",
        harness: "public_shift_big_amount_width_1_upper_z_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-big-amount-width-2-small",
        harness: "public_shift_big_amount_width_2_small_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-big-amount-width-2-high",
        harness: "public_shift_big_amount_width_2_high_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-big-amount-width-2-upper-x",
        harness: "public_shift_big_amount_width_2_upper_x_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-big-amount-width-2-upper-z",
        harness: "public_shift_big_amount_width_2_upper_z_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-big-amount-width-31-small",
        harness: "public_shift_big_amount_width_31_small_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-big-amount-width-31-high",
        harness: "public_shift_big_amount_width_31_high_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-big-amount-width-31-upper-x",
        harness: "public_shift_big_amount_width_31_upper_x_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-big-amount-width-31-upper-z",
        harness: "public_shift_big_amount_width_31_upper_z_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-big-amount-width-63-small",
        harness: "public_shift_big_amount_width_63_small_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-big-amount-width-63-high",
        harness: "public_shift_big_amount_width_63_high_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-big-amount-width-63-upper-x",
        harness: "public_shift_big_amount_width_63_upper_x_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-native-sequences",
        name: "public-shift-big-amount-width-63-upper-z",
        harness: "public_shift_big_amount_width_63_upper_z_matches_reference",
        message: "public shift semantics mismatch",
        covers: WORD_BIG_AMOUNT_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-less-equal-left-width4",
        harness: "public_less_equal_left_width4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: &[
            "public comparison unsigned sources are reachable",
            "public comparison signed RHS only is reachable",
            "public comparison signed LHS only is reachable",
            "public comparison signed sources are reachable",
            "public comparison X result is reachable",
            "public comparison true result is reachable",
            "public comparison false result is reachable",
            "public less equal width4 narrow RHS is reachable",
            "public less equal width4 equal widths are reachable",
        ],
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-less-equal-right-width4",
        harness: "public_less_equal_right_width4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: &[
            "public comparison unsigned sources are reachable",
            "public comparison signed RHS only is reachable",
            "public comparison signed LHS only is reachable",
            "public comparison signed sources are reachable",
            "public comparison X result is reachable",
            "public comparison true result is reachable",
            "public comparison false result is reachable",
            "public less equal width4 narrow LHS is reachable",
            "public less equal width4 three bit LHS is reachable",
        ],
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-equal-width-1-by-4",
        harness: "public_small_greater_equal_width_1_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-equal-width-2-by-4",
        harness: "public_small_greater_equal_width_2_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-equal-width-3-by-4",
        harness: "public_small_greater_equal_width_3_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-equal-width-4-by-1",
        harness: "public_small_greater_equal_width_4_by_1_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-equal-width-4-by-2",
        harness: "public_small_greater_equal_width_4_by_2_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-equal-width-4-by-3",
        harness: "public_small_greater_equal_width_4_by_3_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-equal-width-4-by-4",
        harness: "public_small_greater_equal_width_4_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-equal-width-1-by-1",
        harness: "public_small_greater_equal_width_1_by_1_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-equal-width-1-by-2",
        harness: "public_small_greater_equal_width_1_by_2_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-equal-width-1-by-3",
        harness: "public_small_greater_equal_width_1_by_3_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-equal-width-2-by-1",
        harness: "public_small_greater_equal_width_2_by_1_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-equal-width-2-by-2",
        harness: "public_small_greater_equal_width_2_by_2_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-equal-width-2-by-3",
        harness: "public_small_greater_equal_width_2_by_3_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-equal-width-3-by-1",
        harness: "public_small_greater_equal_width_3_by_1_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-equal-width-3-by-2",
        harness: "public_small_greater_equal_width_3_by_2_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-equal-width-3-by-3",
        harness: "public_small_greater_equal_width_3_by_3_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-width-1-by-4",
        harness: "public_small_greater_width_1_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-width-2-by-4",
        harness: "public_small_greater_width_2_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-width-3-by-4",
        harness: "public_small_greater_width_3_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-width-4-by-1",
        harness: "public_small_greater_width_4_by_1_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-width-4-by-2",
        harness: "public_small_greater_width_4_by_2_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-width-4-by-3",
        harness: "public_small_greater_width_4_by_3_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-width-4-by-4",
        harness: "public_small_greater_width_4_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-width-1-by-1",
        harness: "public_small_greater_width_1_by_1_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-width-1-by-2",
        harness: "public_small_greater_width_1_by_2_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-width-1-by-3",
        harness: "public_small_greater_width_1_by_3_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-width-2-by-1",
        harness: "public_small_greater_width_2_by_1_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-width-2-by-2",
        harness: "public_small_greater_width_2_by_2_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-width-2-by-3",
        harness: "public_small_greater_width_2_by_3_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-width-3-by-1",
        harness: "public_small_greater_width_3_by_1_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-width-3-by-2",
        harness: "public_small_greater_width_3_by_2_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-greater-width-3-by-3",
        harness: "public_small_greater_width_3_by_3_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-equal-width-1-by-1",
        harness: "public_small_less_equal_width_1_by_1_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-equal-width-1-by-2",
        harness: "public_small_less_equal_width_1_by_2_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-equal-width-1-by-3",
        harness: "public_small_less_equal_width_1_by_3_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-equal-width-2-by-1",
        harness: "public_small_less_equal_width_2_by_1_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-equal-width-2-by-2",
        harness: "public_small_less_equal_width_2_by_2_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-equal-width-2-by-3",
        harness: "public_small_less_equal_width_2_by_3_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-equal-width-3-by-1",
        harness: "public_small_less_equal_width_3_by_1_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-equal-width-3-by-2",
        harness: "public_small_less_equal_width_3_by_2_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-equal-width-3-by-3",
        harness: "public_small_less_equal_width_3_by_3_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-width-1-by-4",
        harness: "public_small_less_width_1_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-width-2-by-4",
        harness: "public_small_less_width_2_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-width-3-by-4",
        harness: "public_small_less_width_3_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-width-4-by-1",
        harness: "public_small_less_width_4_by_1_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-width-4-by-2",
        harness: "public_small_less_width_4_by_2_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-width-4-by-3",
        harness: "public_small_less_width_4_by_3_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-width-4-by-4",
        harness: "public_small_less_width_4_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_FIXED_ORDERING_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-unsigned-continuation",
        harness: "public_small_less_unsigned_continuation_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_ORDERING_UNSIGNED_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-less-signed-continuation",
        harness: "public_small_less_signed_continuation_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_ORDERING_SIGNED_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-inequality-native-unsigned-continuation",
        harness: "public_inequality_native_unsigned_continuation_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_INEQUALITY_NATIVE_UNSIGNED_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-inequality-native-signed-continuation",
        harness: "public_inequality_native_signed_continuation_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_INEQUALITY_NATIVE_SIGNED_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-wildcard-not-equal-width-1-by-4",
        harness: "public_small_wildcard_not_equal_width_1_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_WIDTH4_WILDCARD_NOT_EQUAL_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-wildcard-not-equal-width-2-by-4",
        harness: "public_small_wildcard_not_equal_width_2_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_WIDTH4_WILDCARD_NOT_EQUAL_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-wildcard-not-equal-width-3-by-4",
        harness: "public_small_wildcard_not_equal_width_3_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_WIDTH4_WILDCARD_NOT_EQUAL_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-wildcard-not-equal-width-4-by-1",
        harness: "public_small_wildcard_not_equal_width_4_by_1_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_WIDTH4_WILDCARD_NOT_EQUAL_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-wildcard-not-equal-width-4-by-2",
        harness: "public_small_wildcard_not_equal_width_4_by_2_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_WIDTH4_WILDCARD_NOT_EQUAL_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-wildcard-not-equal-width-4-by-3",
        harness: "public_small_wildcard_not_equal_width_4_by_3_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_WIDTH4_WILDCARD_NOT_EQUAL_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-wildcard-not-equal-width-4-by-4",
        harness: "public_small_wildcard_not_equal_width_4_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_WIDTH4_WILDCARD_NOT_EQUAL_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-wildcard-inequality-unsigned-continuation",
        harness: "public_small_wildcard_inequality_unsigned_continuation_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_WILDCARD_INEQUALITY_UNSIGNED_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-wildcard-inequality-signed-continuation",
        harness: "public_small_wildcard_inequality_signed_continuation_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_SMALL_WILDCARD_INEQUALITY_SIGNED_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-wildcard-equal-width-1-by-4",
        harness: "public_small_wildcard_equal_width_1_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_WIDTH4_WILDCARD_EQUAL_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-wildcard-equal-width-2-by-4",
        harness: "public_small_wildcard_equal_width_2_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_WIDTH4_WILDCARD_EQUAL_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-wildcard-equal-width-3-by-4",
        harness: "public_small_wildcard_equal_width_3_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_WIDTH4_WILDCARD_EQUAL_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-wildcard-equal-width-4-by-1",
        harness: "public_small_wildcard_equal_width_4_by_1_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_WIDTH4_WILDCARD_EQUAL_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-wildcard-equal-width-4-by-2",
        harness: "public_small_wildcard_equal_width_4_by_2_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_WIDTH4_WILDCARD_EQUAL_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-wildcard-equal-width-4-by-3",
        harness: "public_small_wildcard_equal_width_4_by_3_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_WIDTH4_WILDCARD_EQUAL_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-wildcard-equal-width-4-by-4",
        harness: "public_small_wildcard_equal_width_4_by_4_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_WIDTH4_WILDCARD_EQUAL_COVERS,
    },
    Contract {
        group: "public-comparison-sequences",
        name: "public-small-wildcard-equality-continuation",
        harness: "public_small_wildcard_equality_continuation_matches_reference",
        message: "public comparison semantics mismatch",
        covers: PUBLIC_WILDCARD_CONTINUATION_COVERS,
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
        group: "public-gate-sequences",
        name: "public-bitwise-widened-mutation-witness",
        harness: "public_bitwise_widened_mutation_witness_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public bitwise witness unsigned sources is reachable",
            "public bitwise witness signed sources is reachable",
            "public bitwise witness signed LHS only is reachable",
            "public bitwise witness signed RHS only is reachable",
            "public bitwise witness source X is reachable",
            "public bitwise witness source Z is reachable",
            "public bitwise witness u128 padding is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-bitwise-widened-native-trace",
        harness: "public_bitwise_widened_native_trace_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public bitwise trace LHS width 1 is reachable",
            "public bitwise trace LHS width 64 is reachable",
            "public bitwise trace RHS width 1 is reachable",
            "public bitwise trace RHS width 64 is reachable",
            "public bitwise trace wider LHS is reachable",
            "public bitwise trace wider RHS is reachable",
            "public bitwise trace unchanged context is reachable",
            "public bitwise trace widened context is reachable",
            "public bitwise trace context width 64 is reachable",
            "public bitwise trace unsigned sources is reachable",
            "public bitwise trace signed sources is reachable",
            "public bitwise trace signed LHS only is reachable",
            "public bitwise trace signed RHS only is reachable",
            "public bitwise trace source X is reachable",
            "public bitwise trace source Z is reachable",
            "public bitwise trace signed Z extension is reachable",
            "public bitwise trace u128 padding is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-logical-and-mutation-witness",
        harness: "public_logical_and_mutation_witness_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "logical AND witness unsigned sources is reachable",
            "logical AND witness signed sources is reachable",
            "logical AND witness signed LHS only is reachable",
            "logical AND witness signed RHS only is reachable",
            "logical AND witness false result is reachable",
            "logical AND witness true result is reachable",
            "logical AND witness X result is reachable",
            "logical AND witness zero dominates RHS unknown is reachable",
            "logical AND witness zero dominates LHS unknown is reachable",
            "logical AND witness source Z is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "logical-truth-reference",
        harness: "logical_truth_scan_matches_scalar_fold",
        message: "logical truth reference mismatch",
        covers: &[
            "logical truth reference width 1 is reachable",
            "logical truth reference width 64 is reachable",
            "logical truth reference false is reachable",
            "logical truth reference true is reachable",
            "logical truth reference X is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-logical-and-native-continuation",
        harness: "public_logical_and_native_continuation_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public native logical AND LHS width 1 is reachable",
            "public native logical AND LHS width 64 is reachable",
            "public native logical AND RHS width 1 is reachable",
            "public native logical AND RHS width 64 is reachable",
            "public native logical AND unsigned sources is reachable",
            "public native logical AND signed sources is reachable",
            "public native logical AND signed LHS only is reachable",
            "public native logical AND signed RHS only is reachable",
            "public native logical AND false result is reachable",
            "public native logical AND true result is reachable",
            "public native logical AND X result is reachable",
            "public native logical AND zero dominates RHS unknown is reachable",
            "public native logical AND zero dominates LHS unknown is reachable",
            "public native logical AND source Z is reachable",
            "public native logical AND known ones dominate unknown is reachable",
            "public native logical AND unchanged context is reachable",
            "public native logical AND widened context is reachable",
            "public native logical AND context width 64 is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-and-native-continuation",
        harness: "public_reduction_and_native_continuation_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public reduction AND source width 1 is reachable",
            "public reduction AND source width 64 is reachable",
            "public reduction AND unchanged NOT context is reachable",
            "public reduction AND widened NOT context is reachable",
            "public reduction AND NOT context width 64 is reachable",
            "public reduction AND signed source is reachable",
            "public reduction AND unsigned source is reachable",
            "public reduction AND false result is reachable",
            "public reduction AND true result is reachable",
            "public reduction AND X result is reachable",
            "public reduction AND zero dominates unknown is reachable",
            "public reduction AND source Z is reachable",
            "public reduction AND source X is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "reduction-and-reference",
        harness: "native_reduction_and_tree_matches_scalar_fold",
        message: "reduction AND reference mismatch",
        covers: &[
            "reduction AND reference width 1 is reachable",
            "reduction AND reference width 64 is reachable",
            "reduction AND reference zero is reachable",
            "reduction AND reference one is reachable",
            "reduction AND reference X is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-and-width65",
        harness: "public_reduction_and_width65_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public wide reduction AND width 1 zero unchanged is reachable",
            "public wide reduction AND width 64 signed one is reachable",
            "public wide reduction AND widened X and Z condition is reachable",
            "public wide reduction AND widened zero dominates X and Z is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-and-width128",
        harness: "public_reduction_and_width128_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public wide reduction AND width 1 zero unchanged is reachable",
            "public wide reduction AND width 64 signed one is reachable",
            "public wide reduction AND widened X and Z condition is reachable",
            "public wide reduction AND widened zero dominates X and Z is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-nand-native-continuation",
        harness: "public_reduction_nand_native_continuation_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public native reduction NAND width 1 zero unchanged is reachable",
            "public native reduction NAND width 64 signed one is reachable",
            "public native reduction NAND widened X and Z is reachable",
            "public native reduction NAND widened zero dominates X and Z is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-nand-width65",
        harness: "public_reduction_nand_width65_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public wide reduction NAND width 1 zero unchanged is reachable",
            "public wide reduction NAND width 64 signed one is reachable",
            "public wide reduction NAND widened X and Z is reachable",
            "public wide reduction NAND widened zero dominates X and Z is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-nand-width128",
        harness: "public_reduction_nand_width128_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public wide reduction NAND width 1 zero unchanged is reachable",
            "public wide reduction NAND width 64 signed one is reachable",
            "public wide reduction NAND widened X and Z is reachable",
            "public wide reduction NAND widened zero dominates X and Z is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "reduction-or-reference",
        harness: "native_reduction_or_tree_matches_scalar_fold",
        message: "reduction OR reference mismatch",
        covers: &[
            "reduction OR reference width 1 is reachable",
            "reduction OR reference width 64 is reachable",
            "reduction OR reference zero is reachable",
            "reduction OR reference one is reachable",
            "reduction OR reference X is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-or-native-continuation",
        harness: "public_reduction_or_native_continuation_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public native reduction OR width 1 zero unchanged is reachable",
            "public native reduction OR width 64 signed high one is reachable",
            "public native reduction OR widened X and Z is reachable",
            "public native reduction OR widened one dominates X and Z is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-or-width65",
        harness: "public_reduction_or_width65_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public wide reduction OR width 1 zero unchanged is reachable",
            "public wide reduction OR width 64 signed high one is reachable",
            "public wide reduction OR widened X and Z is reachable",
            "public wide reduction OR widened one dominates X and Z is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-or-width128",
        harness: "public_reduction_or_width128_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public wide reduction OR width 1 zero unchanged is reachable",
            "public wide reduction OR width 64 signed high one is reachable",
            "public wide reduction OR widened X and Z is reachable",
            "public wide reduction OR widened one dominates X and Z is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-nor-native-continuation",
        harness: "public_reduction_nor_native_continuation_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public native reduction NOR width 1 zero unchanged is reachable",
            "public native reduction NOR width 64 signed high one is reachable",
            "public native reduction NOR widened X and Z is reachable",
            "public native reduction NOR widened one dominates X and Z is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-nor-width65",
        harness: "public_reduction_nor_width65_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public wide reduction NOR width 1 zero unchanged is reachable",
            "public wide reduction NOR width 64 signed high one is reachable",
            "public wide reduction NOR widened X and Z is reachable",
            "public wide reduction NOR widened one dominates X and Z is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-nor-width128",
        harness: "public_reduction_nor_width128_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public wide reduction NOR width 1 zero unchanged is reachable",
            "public wide reduction NOR width 64 signed high one is reachable",
            "public wide reduction NOR widened X and Z is reachable",
            "public wide reduction NOR widened one dominates X and Z is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "reduction-xor-reference",
        harness: "native_reduction_xor_tree_matches_scalar_fold",
        message: "reduction XOR reference mismatch",
        covers: &[
            "reduction XOR reference width 1 is reachable",
            "reduction XOR reference width 64 is reachable",
            "reduction XOR reference zero is reachable",
            "reduction XOR reference one is reachable",
            "reduction XOR reference X is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-xor-native-continuation",
        harness: "public_reduction_xor_native_continuation_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public native reduction XOR width 1 zero unchanged is reachable",
            "public native reduction XOR width 64 signed odd parity is reachable",
            "public native reduction XOR width 64 even parity is reachable",
            "public native reduction XOR widened X and Z is reachable",
            "public native reduction XOR widened unknown dominates one is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-xor-width65",
        harness: "public_reduction_xor_width65_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public wide reduction XOR width 1 zero unchanged is reachable",
            "public wide reduction XOR width 64 signed odd parity is reachable",
            "public wide reduction XOR width 64 even parity is reachable",
            "public wide reduction XOR widened X and Z is reachable",
            "public wide reduction XOR widened unknown dominates one is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-xor-width128",
        harness: "public_reduction_xor_width128_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public wide reduction XOR width 1 zero unchanged is reachable",
            "public wide reduction XOR width 64 signed odd parity is reachable",
            "public wide reduction XOR width 64 even parity is reachable",
            "public wide reduction XOR widened X and Z is reachable",
            "public wide reduction XOR widened unknown dominates one is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-xnor-native-continuation",
        harness: "public_reduction_xnor_native_continuation_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public native reduction XNOR width 1 zero unchanged is reachable",
            "public native reduction XNOR width 64 signed odd parity is reachable",
            "public native reduction XNOR width 64 even parity is reachable",
            "public native reduction XNOR widened X and Z is reachable",
            "public native reduction XNOR widened unknown dominates one is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-xnor-width65",
        harness: "public_reduction_xnor_width65_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public wide reduction XNOR width 1 zero unchanged is reachable",
            "public wide reduction XNOR width 64 signed odd parity is reachable",
            "public wide reduction XNOR width 64 even parity is reachable",
            "public wide reduction XNOR widened X and Z is reachable",
            "public wide reduction XNOR widened unknown dominates one is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-reduction-xnor-width128",
        harness: "public_reduction_xnor_width128_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public wide reduction XNOR width 1 zero unchanged is reachable",
            "public wide reduction XNOR width 64 signed odd parity is reachable",
            "public wide reduction XNOR width 64 even parity is reachable",
            "public wide reduction XNOR widened X and Z is reachable",
            "public wide reduction XNOR widened unknown dominates one is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-logical-not-native-continuation",
        harness: "public_logical_not_native_continuation_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public logical NOT width 1 zero unchanged is reachable",
            "public logical NOT width 64 signed high one is reachable",
            "public logical NOT widened X and Z is reachable",
            "public logical NOT widened one dominates X and Z is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-logical-not-width65",
        harness: "public_logical_not_width65_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public logical NOT width 1 zero unchanged is reachable",
            "public logical NOT width 64 signed high one is reachable",
            "public logical NOT widened X and Z is reachable",
            "public logical NOT widened one dominates X and Z is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-logical-not-width128",
        harness: "public_logical_not_width128_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public logical NOT width 1 zero unchanged is reachable",
            "public logical NOT width 64 signed high one is reachable",
            "public logical NOT widened X and Z is reachable",
            "public logical NOT widened one dominates X and Z is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "logical-truth-unrolled-reference",
        harness: "unrolled_native_logical_truth_matches_reference_semantics",
        message: "logical unrolled truth reference mismatch",
        covers: &[
            "logical unrolled truth width 1 is reachable",
            "logical unrolled truth width 64 is reachable",
            "logical unrolled truth zero is reachable",
            "logical unrolled truth one is reachable",
            "logical unrolled truth X is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-logical-binary-domain",
        harness: "public_logical_binary_domain_is_reachable",
        message: "public gate semantics mismatch",
        covers: &[
            "public logical binary unsigned narrow LHS and high RHS are reachable",
            "public logical binary signed high LHS and narrow RHS are reachable",
            "public logical binary signed LHS known ones dominate X and Z",
            "public logical binary signed RHS zero and unknown LHS are reachable",
            "public logical binary zero LHS and Z RHS are reachable",
            "public logical binary both unknown widened sources are reachable",
            "public logical binary both zero unchanged sources are reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-logical-and-width65",
        harness: "public_logical_and_width65_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &["public logical binary full continuation is reachable"],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-logical-or-native-continuation",
        harness: "public_logical_or_native_continuation_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &["public logical binary full continuation is reachable"],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-logical-or-width65",
        harness: "public_logical_or_width65_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &["public logical binary full continuation is reachable"],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-logical-and-width65-witness",
        harness: "public_logical_and_width65_witness_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public logical binary witness zero result is reachable",
            "public logical binary witness one result is reachable",
            "public logical binary witness X result is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-logical-or-native-witness",
        harness: "public_logical_or_native_witness_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public logical binary witness zero result is reachable",
            "public logical binary witness one result is reachable",
            "public logical binary witness X result is reachable",
        ],
    },
    Contract {
        group: "public-gate-sequences",
        name: "public-logical-or-width65-witness",
        harness: "public_logical_or_width65_witness_matches_reference_semantics",
        message: "public gate semantics mismatch",
        covers: &[
            "public logical binary witness zero result is reachable",
            "public logical binary witness one result is reachable",
            "public logical binary witness X result is reachable",
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
        group: "public-gate-sequences",
        name: "public-conditional-native",
        harness: "public_conditional_native_matches_reference_semantics",
        message: "public conditional semantics mismatch",
        covers: &[
            "public native conditional condition width 1 is reachable",
            "public native conditional condition width 2 is reachable",
            "public native conditional condition width 64 is reachable",
            "public native conditional high bit determines true",
            "public native conditional high bit determines unknown",
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

// Conditional cover statements for other generic instantiations remain in
// Kani's output. They are diagnostic only unless listed in Contract::covers.
pub const AUXILIARY_CONDITIONAL_COVERS: &[&str] = &[
    "signed quotient class minimum nonpower divisor is reachable",
    "signed quotient class word crossing is reachable",
    "signed quotient class zero remainder is reachable",
    "signed quotient class maximum nonzero remainder is reachable",
    "signed quotient class minimum over minus one is reachable",
    "signed quotient class maximum positive dividend is reachable",
    "signed quotient class positive positive context is reachable",
    "signed quotient class positive negative context is reachable",
    "signed quotient class negative positive context is reachable",
    "signed quotient class negative negative context is reachable",
    "power quotient class minimum nonunit quotient is reachable",
    "power quotient class largest quotient is reachable",
    "power quotient class unit quotient is reachable",
    "power quotient class nonpower divisor is reachable",
    "power quotient class maximum remainder is reachable",
    "power quotient class original context is reachable",
    "unit quotient class minimum width is reachable",
    "unit quotient class maximum upper divisor is reachable",
    "unit quotient class exact division is reachable",
    "unit quotient class adjacent operands is reachable",
    "unit quotient class maximum remainder is reachable",
    "unit quotient class original context is reachable",
    "single hot power class minimum signed width is reachable",
    "single hot power class maximum unsigned width is reachable",
    "single hot power class unit base is reachable",
    "single hot power class signed minimum is reachable",
    "single hot power class full unsigned exponent is reachable",
    "single hot power class high result bit is reachable",
    "single hot power class first zero result is reachable",
    "single hot power class wide signed exponent is reachable",
    "single hot power class signed original context is reachable",
    "single hot power class unsigned original context is reachable",
    "ones power class minimum signed base is reachable",
    "ones power class maximum unsigned base is reachable",
    "ones power class even exponent is reachable",
    "ones power class odd exponent is reachable",
    "ones power class full unsigned exponent is reachable",
    "ones power class wide signed exponent is reachable",
    "ones power class original context is reachable",
    "ones class minimum signed width is reachable",
    "ones class maximum unsigned width is reachable",
    "ones class zero number is reachable",
    "ones class truncated high product is reachable",
    "ones class signed minimum number is reachable",
    "ones class original context is reachable",
    "power signed class minimum nonunit divisor is reachable",
    "power signed class minimum over minus one is reachable",
    "power signed class zero dividend is reachable",
    "power signed class word crossing is reachable",
    "power signed class minimum divisor is reachable",
    "power signed class nonnegative context is reachable",
    "power signed class negative context is reachable",
    "small signed class minimum positive pair is reachable",
    "small signed class minimum divisor is reachable",
    "small signed class zero dividend is reachable",
    "small signed class negative dividend is reachable",
    "small signed class nonnegative context is reachable",
    "small signed class negative context is reachable",
    "opposite class positive context is reachable",
    "opposite class negative context is reachable",
    "opposite class minimum positive dividend is reachable",
    "opposite class maximum negative dividend is reachable",
    "opposite class maximum magnitude is reachable",
    "opposite class mixed source signs is reachable",
    "multiplication class minimum signed width is reachable",
    "multiplication class maximum unsigned width is reachable",
    "multiplication class mixed source signs is reachable",
    "multiplication class factor one is reachable",
    "multiplication class word crossing is reachable",
    "multiplication class truncated high product is reachable",
    "multiplication class native native sources is reachable",
    "multiplication class native big sources is reachable",
    "multiplication class big native sources is reachable",
    "multiplication class big big sources is reachable",
    "division class big big sources is reachable",
    "division class big native sources is reachable",
    "division class divisor one is reachable",
    "division class maximum width is reachable",
    "division class minimum signed self is reachable",
    "division class minimum width is reachable",
    "division class mixed source signs is reachable",
    "division class native big sources is reachable",
    "division class native native sources is reachable",
    "division class top divisor bit is reachable",
    "division class unsigned nonunit self is reachable",
    "division class upper smaller divisor is reachable",
    "division class word crossing is reachable",
    "division class zero smaller numerator is reachable",
    // Const-generic arithmetic emits diagnostic covers for inactive partitions.
    "special all ones divisor is reachable",
    "special big big sources is reachable",
    "special big native sources is reachable",
    "special exponent wider than result is reachable",
    "special maximum width is reachable",
    "special minimum signed dividend is reachable",
    "special minimum width is reachable",
    "special mixed source signs is reachable",
    "special native big sources is reachable",
    "special native native sources is reachable",
    "special one bit positive exponent is reachable",
    "special power minimum signed base is reachable",
    "special power one base is reachable",
    "special power positive even exponent is reachable",
    "special power positive odd exponent is reachable",
    "special power upper exponent bit is reachable",
    "special power upper unsigned base is reachable",
    "special power zero base is reachable",
    "special signed one bit base is reachable",
    "special signed one bit minus one is reachable",
    "special signed zero division is reachable",
    "special unit base zero extension is reachable",
    "special unsigned zero division is reachable",
    "special upper divisor bit is reachable",
    "special wide signed positive exponent is reachable",
    "special zero dividend is reachable",
    "wiring ordering signed operands are reachable",
    "wide ordering upper unknown is reachable",
    "wide ordering only lhs unknown is reachable",
    "wide ordering only rhs unknown is reachable",
    "wide ordering both unknown is reachable",
    "wide ordering known mismatch under unknown is reachable",
    "wide ordering known true is reachable",
    "wide ordering known false is reachable",
    "wide ordering opposite high bits are reachable",
    "wide ordering equal operands are reachable",
    "wide ordering both high bits set are reachable",
    "wiring mixed storage and signs are reachable",
    "wiring unknown wide amount is reachable",
    "wiring known wide amount is reachable",
    "wiring comparison ignores outer sign",
    "wiring comparison separates result width",
    "nonlinear fixed width natural call is reachable",
    "nonlinear fixed width widened call is reachable",
    "public less equal width4 narrow RHS is reachable",
    "public less equal width4 equal widths are reachable",
    "public less equal width4 narrow LHS is reachable",
    "public less equal width4 three bit LHS is reachable",
    "public logical binary full continuation is reachable",
    "public logical binary witness zero result is reachable",
    "public logical binary witness one result is reachable",
    "public logical binary witness X result is reachable",
];

// Named covers are mandatory for this profile; the shared runner validates them.
// Only these new public proofs omit Kani's automatic per-assertion diagnostics.
pub fn uses_explicit_covers(contract: &Contract) -> bool {
    matches!(
        contract.group,
        "public-nonlinear"
            | "public-big-values"
            | "public-wide-resize"
            | "public-getters"
            | "public-native-sequences"
            | "public-comparison-sequences"
            | "public-gate-sequences"
    )
}

// Full logical/conditional continuations and variable-width wide operators
// exceed the normal60s cap on this toolchain. Keep180s scoped to these names;
// the separate word64 logical mutation-witness contracts retain60s.
// Mixed original widths, two signed conversions and variable shifted products
// need a larger solver budget than the prepared arithmetic kernels.
pub fn uses_signed_quotient_context_timeout(contract: &Contract) -> bool {
    contract.name.starts_with("signedq256-") && contract.name.ends_with("-context")
}

pub fn uses_extended_logical_timeout(contract: &Contract) -> bool {
    contract.name.starts_with("signedq256-")
        || contract.name.starts_with("powerq256-")
        || contract.name.starts_with("unitq256-")
        || contract.name.starts_with("powerhot256-")
        || contract.name.starts_with("powerones256-")
        || contract.name.starts_with("ones256-")
        || contract.name.starts_with("signedpow256-")
        || contract.name.starts_with("signedsmall256-")
        || contract.name.starts_with("opposite256-")
        || contract.name.starts_with("multiply256-")
        || contract.name.starts_with("division256-")
        || contract.name.starts_with("special256-")
        || matches!(
            contract.name,
            "unit256-div-one-reference"
                | "identity256-mul-right"
                | "identity256-mul-right-context"
                | "identity256-mul-left"
                | "identity256-mul-left-context"
                | "identity256-div-one"
                | "identity256-div-one-context"
                | "identity256-rem-one"
                | "identity256-rem-one-context"
                | "power256-unknown"
                | "power256-unknown-context"
                | "power256-negative"
                | "power256-negative-context"
                | "power256-zero"
                | "power256-zero-context"
                | "exception256-mul-context"
                | "exception256-div-context"
                | "exception256-rem-context"
                | "logical256-and-native-big"
                | "logical256-and-big-native"
                | "logical256-and-big-big"
                | "logical256-or-native-big"
                | "logical256-or-big-native"
                | "logical256-or-big-big"
                | "conversion256-bigint"
                | "numeric256-order-api"
                | "ordering256-less-unsigned"
                | "ordering256-less-signed"
                | "ordering256-less-unknown"
                | "ordering256-less-eq-unsigned"
                | "ordering256-less-eq-signed"
                | "ordering256-less-eq-unknown"
                | "ordering256-greater-unsigned"
                | "ordering256-greater-signed"
                | "ordering256-greater-unknown"
                | "ordering256-greater-eq-unsigned"
                | "ordering256-greater-eq-signed"
                | "ordering256-greater-eq-unknown"
                | "equality256-eq"
                | "equality256-ne"
                | "equality256-wild-eq"
                | "equality256-wild-ne"
                | "arithmetic256-add"
                | "arithmetic256-sub"
                | "arithmetic256-neg"
                | "assignment256-big-native"
                | "assignment256-big-big"
                | "bitwise256-and"
                | "bitwise256-or"
                | "bitwise256-xor"
                | "bitwise256-xnor"
                | "shift256-left"
                | "shift256-right"
                | "shift256-arith-left"
                | "shift256-arith-right-signed"
                | "shift256-arith-right-unsigned"
                | "public-logical-and-width65"
                | "public-logical-or-native-continuation"
                | "public-logical-or-width65"
                | "public-conditional-wide-native-native"
                | "public-conditional-wide-native-big"
                | "public-conditional-wide-big-native"
                | "public-conditional-wide-big-big"
        )
}

pub fn mutations(repo: &Path, selected: &[&'static Contract]) -> Vec<Mutation> {
    let mut mutations = Vec::new();
    wide_operator_mutations(repo, selected, &mut mutations);
    mutations.extend(followup::mutations(repo, selected));
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-big-expansion-zero")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "big-expansion-retains-unsigned-sign",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "if !use_sign {\n                        ret.signed = false;",
                "if false && !use_sign {\n                        ret.signed = false;",
            ),
        });
    }
    if let Some(&contract) = selected.iter().find(|c| c.name == "public-big-truncation") {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "big-truncation-loses-payload-clipping",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "*self.payload &= &mask;",
                "// payload clipping removed",
            ),
        });
        mutations.push(Mutation {
            name: "big-truncation-loses-XZ-clipping",
            contract,
            path: CORE,
            source: replace_once(&source, "*self.mask_xz &= &mask;", "// XZ clipping removed"),
        });
        mutations.push(Mutation {
            name: "big-truncation-wrong-width",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "*self.mask_xz &= &mask;\n        self.width = width as u32;",
                "*self.mask_xz &= &mask;\n        self.width = 128;",
            ),
        });
        mutations.push(Mutation { name: "big-truncation-skips-native-conversion", contract, path: CORE, source: replace_once(&source, "if let Some(y) = x.to_value_u64() {\n                        *self = Self::U64(y);", "if let Some(y) = x.to_value_u64() {\n                        let _ = y;") });
    }
    if let Some(&contract) = selected.iter().find(|c| c.name == "public-big-expansion") {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation { name: "big-expansion-skips-sign-fill", contract, path: CORE, source: replace_once(&source, "if x.signed && use_sign {\n                        let msb = ret.payload.bit", "if false && x.signed && use_sign {\n                        let msb = ret.payload.bit") });
        mutations.push(Mutation { name: "big-expansion-wrong-width", contract, path: CORE, source: replace_once(&source, "ret.width = width as u32;\n                    ret\n                }\n            };\n            Cow::Owned(Value::BigUint(ret))", "ret.width = 65;\n                    ret\n                }\n            };\n            Cow::Owned(Value::BigUint(ret))") });
        mutations.push(Mutation {
            name: "big-expansion-skips-XZ-fill",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "if msb_xz {\n                                *ret.mask_xz |= &mask;",
                "if false && msb_xz {\n                                *ret.mask_xz |= &mask;",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-big-selection-wide")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "big-wide-selection-loses-payload",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "*ret.payload >>= end;",
                "*ret.payload = BigUint::zero();",
            ),
        });
        mutations.push(Mutation {
            name: "big-wide-selection-loses-XZ",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "*ret.mask_xz = (&*ret.mask_xz & &valid_mask) | (&mask ^ &valid_mask);",
                "*ret.mask_xz = BigUint::zero();",
            ),
        });
        mutations.push(Mutation { name: "big-wide-selection-missing-bits-zero", contract, path: CORE, source: replace_once(&source, "            if end >= self.width as usize {\n                return Self::new_x(width, false);\n            }\n            let valid_width = (self.width as usize - end).min(width);\n            let valid_mask = Self::gen_mask(valid_width);\n            let mut ret = self.clone();", "            if end >= self.width as usize {\n                return Self { payload: Box::new(BigUint::zero()), mask_xz: Box::new(BigUint::zero()), width: width as u32, signed: false };\n            }\n            let valid_width = (self.width as usize - end).min(width);\n            let valid_mask = Self::gen_mask(valid_width);\n            let mut ret = self.clone();") });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-conditional-wide-native-native")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "wide-native-native-ignores-branch-XZ",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "let different = (a.payload() ^ b.payload()) | a.mask_xz() | b.mask_xz();",
                "let different = a.payload() ^ b.payload();",
            ),
        });
        mutations.push(Mutation {
            name: "wide-native-native-clears-result-sign",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "                ret.signed = signed;\n                Self::BigUint(ret)",
                "                ret.signed = false;\n                Self::BigUint(ret)",
            ),
        });
        mutations.push(Mutation {
            name: "wide-native-native-swaps-definite-branches",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "Some(true) => a.clone(),\n                    Some(false) => b.clone(),",
                "Some(true) => b.clone(),\n                    Some(false) => a.clone(),",
            ),
        });
        mutations.push(Mutation {
            name: "wide-native-native-skips-false-branch-sign",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "let on_false = on_false.expand(width, signed);",
                "let on_false = on_false.expand(width, false);",
            ),
        });
        mutations.push(Mutation {
            name: "wide-native-native-ignores-context",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "let width = context_width.max(on_true.width()).max(on_false.width());",
                "let width = on_true.width().max(on_false.width());",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-conditional-wide-native-big")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "wide-native-big-ignores-branch-XZ",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "let different = (a.payload() ^ b.payload()) | a.mask_xz() | b.mask_xz();",
                "let different = a.payload() ^ b.payload();",
            ),
        });
        mutations.push(Mutation {
            name: "wide-native-big-clears-result-sign",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "                ret.signed = signed;\n                Self::BigUint(ret)",
                "                ret.signed = false;\n                Self::BigUint(ret)",
            ),
        });
        mutations.push(Mutation {
            name: "wide-native-big-swaps-definite-branches",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "Some(true) => a.clone(),\n                    Some(false) => b.clone(),",
                "Some(true) => b.clone(),\n                    Some(false) => a.clone(),",
            ),
        });
        mutations.push(Mutation {
            name: "wide-native-big-skips-false-branch-sign",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "let on_false = on_false.expand(width, signed);",
                "let on_false = on_false.expand(width, false);",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-conditional-wide-big-native")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "wide-big-native-ignores-branch-XZ",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "let different = (a.payload() ^ b.payload()) | a.mask_xz() | b.mask_xz();",
                "let different = a.payload() ^ b.payload();",
            ),
        });
        mutations.push(Mutation {
            name: "wide-big-native-clears-result-sign",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "                ret.signed = signed;\n                Self::BigUint(ret)",
                "                ret.signed = false;\n                Self::BigUint(ret)",
            ),
        });
        mutations.push(Mutation {
            name: "wide-big-native-swaps-definite-branches",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "Some(true) => a.clone(),\n                    Some(false) => b.clone(),",
                "Some(true) => b.clone(),\n                    Some(false) => a.clone(),",
            ),
        });
        mutations.push(Mutation {
            name: "wide-big-native-skips-false-branch-sign",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "let on_false = on_false.expand(width, signed);",
                "let on_false = on_false.expand(width, false);",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-conditional-wide-big-big")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "wide-big-big-ignores-branch-XZ",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "let different = (a.payload() ^ b.payload()) | a.mask_xz() | b.mask_xz();",
                "let different = a.payload() ^ b.payload();",
            ),
        });
        mutations.push(Mutation {
            name: "wide-big-big-clears-result-sign",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "                ret.signed = signed;\n                Self::BigUint(ret)",
                "                ret.signed = false;\n                Self::BigUint(ret)",
            ),
        });
        mutations.push(Mutation {
            name: "wide-big-big-swaps-definite-branches",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "Some(true) => a.clone(),\n                    Some(false) => b.clone(),",
                "Some(true) => b.clone(),\n                    Some(false) => a.clone(),",
            ),
        });
        mutations.push(Mutation {
            name: "wide-big-big-skips-false-branch-sign",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "let on_false = on_false.expand(width, signed);",
                "let on_false = on_false.expand(width, false);",
            ),
        });
    }
    if let Some(&contract) = selected.iter().find(|c| c.name == "public-big-truth") {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "big-truth-ignores-upper-one",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "(!known.is_zero(), !x.mask_xz.is_zero())",
                "(!known.is_zero() && x.payload.bits() <= 64, !x.mask_xz.is_zero())",
            ),
        });
        mutations.push(Mutation {
            name: "big-truth-ignores-unknown",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "(!known.is_zero(), !x.mask_xz.is_zero())",
                "(!known.is_zero(), false)",
            ),
        });
        mutations.push(Mutation {
            name: "big-truth-X-overrides-known-one",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "(!known.is_zero(), !x.mask_xz.is_zero())",
                "(!known.is_zero() && x.mask_xz.is_zero(), !x.mask_xz.is_zero())",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-big-conditional-native")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "big-condition-ignores-truth",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "let condition = self.truth();",
                "let condition = Some(false);",
            ),
        });
        mutations.push(Mutation {
            name: "big-condition-forces-unsigned",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "a.conditional(b, condition, context_width, signed)",
                "a.conditional(b, condition, context_width, false)",
            ),
        });
        mutations.push(Mutation {
            name: "big-condition-swaps-branches",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "a.conditional(b, condition, context_width, signed)",
                "b.conditional(a, condition, context_width, signed)",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-native-sign-extension-wide")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "big-sign-extension-skips-fill",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "if x.signed && use_sign {\n                        let msb = payload.bit((x.width - 1) as u64);",
                "if false && x.signed && use_sign {\n                        let msb = payload.bit((x.width - 1) as u64);",
            ),
        });
        mutations.push(Mutation {
            name: "big-sign-extension-drops-XZ",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "if msb_xz {\n                                *mask_xz |= &mask;",
                "if false && msb_xz {\n                                *mask_xz |= &mask;",
            ),
        });
        mutations.push(Mutation {
            name: "big-sign-extension-clears-sign",
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
        .find(|c| c.name == "public-big-selection-native")
    {
        let source = fs::read_to_string(repo.join(CORE)).unwrap();
        mutations.push(Mutation {
            name: "big-selection-loses-payload",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "*ret.payload >>= end;",
                "*ret.payload = BigUint::zero();",
            ),
        });
        mutations.push(Mutation {
            name: "big-selection-loses-XZ",
            contract,
            path: CORE,
            source: replace_once(
                &source,
                "*ret.mask_xz = (&*ret.mask_xz & &valid_mask) | (&mask ^ &valid_mask);",
                "*ret.mask_xz = BigUint::zero();",
            ),
        });
        mutations.push(Mutation {
            name: "big-selection-missing-bits-zero", contract, path: CORE,
            source: replace_once(&source, "            if end >= self.width as usize {\n                return Self::new_x(width, false);\n            }\n            let valid_width = (self.width as usize - end).min(width);\n            let valid_mask = Self::gen_mask(valid_width);\n            let mut ret = self.clone();", "            if end >= self.width as usize {\n                return Self { payload: Box::new(BigUint::zero()), mask_xz: Box::new(BigUint::zero()), width: width as u32, signed: false };\n            }\n            let valid_width = (self.width as usize - end).min(width);\n            let valid_mask = Self::gen_mask(valid_width);\n            let mut ret = self.clone();"),
        });
    }
    if selected
        .iter()
        .any(|c| c.group == "values" || c.name == "public-conditional-native")
    {
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
            "public-conditional-ignores-upper-word",
            "let condition = self.truth();",
            "let mut condition_value = self.clone(); condition_value.trunc(32); let condition = condition_value.truth();",
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
    for (contract_name, name, path, method, old, new) in [
        (
            "public-small-wildcard-not-equal-width-1-by-4",
            "public-wildcard-not-equal-width4-swaps-operands",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.equal(y, true, true)",
            "if xy_width == 4 { y.equal(x, true, true) } else { x.equal(y, true, true) }",
        ),
        (
            "public-small-wildcard-not-equal-width-1-by-4",
            "public-wildcard-not-equal-width4-ignores-wildcards",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.equal(y, true, true)",
            "x.equal(y, xy_width != 4, true)",
        ),
        (
            "public-small-wildcard-not-equal-width-1-by-4",
            "public-wildcard-not-equal-width4-not-negated",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.equal(y, true, true)",
            "x.equal(y, true, xy_width != 4)",
        ),
        (
            "public-small-wildcard-not-equal-width-1-by-4",
            "public-wildcard-not-equal-width4-zero-sign-extension",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "Op::NeWildcard => {\n                let xy_width = x.width().max(y.width());\n                let eq_signed = x.signed() && y.signed();",
            "Op::NeWildcard => {\n                let xy_width = x.width().max(y.width());\n                let eq_signed = x.signed() && y.signed() && xy_width != 4;",
        ),
        (
            "public-small-wildcard-not-equal-width-1-by-4",
            "public-wildcard-not-equal-width4-LHS-wildcards",
            "veryl/crates/analyzer/src/value.rs",
            "equal",
            "let compare_mask = if wildcard { !rhs.mask_xz } else { u64::MAX };",
            "let compare_mask = if wildcard { if self.width == 4 { !(self.mask_xz | rhs.mask_xz) } else { !rhs.mask_xz } } else { u64::MAX };",
        ),
        (
            "public-small-wildcard-not-equal-width-1-by-4",
            "public-wildcard-not-equal-width4-unknown-overrides-mismatch",
            "veryl/crates/analyzer/src/value.rs",
            "equal",
            "Self::new_bit_1x(mismatch, unknown)",
            "Self::new_bit_1x(mismatch && !(self.width == 4 && wildcard && unknown), unknown)",
        ),
        (
            "public-small-wildcard-not-equal-width-1-by-4",
            "public-wildcard-not-equal-width4-signed-result",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.equal(y, true, true)",
            "{ let mut result = x.equal(y, true, true); if xy_width == 4 { result.signed = true; } result }",
        ),
        (
            "public-small-wildcard-equal-width-1-by-4",
            "public-wildcard-equal-width4-swaps-operands",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.equal(y, true, false)",
            "if xy_width == 4 { y.equal(x, true, false) } else { x.equal(y, true, false) }",
        ),
        (
            "public-small-wildcard-equal-width-1-by-4",
            "public-wildcard-equal-width4-ignores-wildcards",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.equal(y, true, false)",
            "x.equal(y, xy_width != 4, false)",
        ),
        (
            "public-small-wildcard-equal-width-1-by-4",
            "public-wildcard-equal-width4-negated",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.equal(y, true, false)",
            "x.equal(y, true, xy_width == 4)",
        ),
        (
            "public-small-wildcard-equal-width-1-by-4",
            "public-wildcard-equal-width4-zero-sign-extension",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "Op::EqWildcard => {\n                let xy_width = x.width().max(y.width());\n                let eq_signed = x.signed() && y.signed();",
            "Op::EqWildcard => {\n                let xy_width = x.width().max(y.width());\n                let eq_signed = x.signed() && y.signed() && xy_width != 4;",
        ),
        (
            "public-small-wildcard-equal-width-1-by-4",
            "public-wildcard-equal-width4-LHS-wildcards",
            "veryl/crates/analyzer/src/value.rs",
            "equal",
            "let compare_mask = if wildcard { !rhs.mask_xz } else { u64::MAX };",
            "let compare_mask = if wildcard { if self.width == 4 { !(self.mask_xz | rhs.mask_xz) } else { !rhs.mask_xz } } else { u64::MAX };",
        ),
        (
            "public-small-wildcard-equal-width-1-by-4",
            "public-wildcard-equal-width4-unknown-overrides-mismatch",
            "veryl/crates/analyzer/src/value.rs",
            "equal",
            "Self::new_bit_0x(mismatch, unknown)",
            "Self::new_bit_0x(mismatch && !(self.width == 4 && wildcard && unknown), unknown)",
        ),
        (
            "public-small-wildcard-equal-width-1-by-4",
            "public-wildcard-equal-width4-signed-result",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.equal(y, true, false)",
            "{ let mut result = x.equal(y, true, false); if xy_width == 4 { result.signed = true; } result }",
        ),
        (
            "public-small-less-width-1-by-4",
            "public-less-width4-direction",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, false, false)",
            "x.compare(y, signed, xy_width == 4, false)",
        ),
        (
            "public-small-less-width-1-by-4",
            "public-less-width4-inclusive",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, false, false)",
            "x.compare(y, signed, false, xy_width == 4)",
        ),
        (
            "public-small-less-width-1-by-4",
            "public-less-width4-unsigned-order",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, false, false)",
            "x.compare(y, signed && xy_width != 4, false, false)",
        ),
        (
            "public-small-less-width-1-by-4",
            "public-less-width4-zero-extension",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "Op::Less => {\n                let xy_width = x.width().max(y.width());\n                let x = x.expand(xy_width, signed);",
            "Op::Less => {\n                let xy_width = x.width().max(y.width());\n                let x = x.expand(xy_width, signed && xy_width != 4);",
        ),
        (
            "public-small-less-width-1-by-4",
            "public-less-width4-ignores-unknown",
            "veryl/crates/analyzer/src/value.rs",
            "compare",
            "Self::new_bit_x1(self.is_xz() || rhs.is_xz(), relation)",
            "Self::new_bit_x1(self.width != 4 && (self.is_xz() || rhs.is_xz()), relation)",
        ),
        (
            "public-small-less-width-1-by-4",
            "public-less-width4-signed-result",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, false, false)",
            "{ let mut result = x.compare(y, signed, false, false); if xy_width == 4 { result.signed = true; } result }",
        ),
        (
            "public-small-greater-equal-width-1-by-4",
            "public-greater-equal-width4-direction",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, true, true)",
            "x.compare(y, signed, xy_width != 4, true)",
        ),
        (
            "public-small-greater-equal-width-1-by-4",
            "public-greater-equal-width4-strict",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, true, true)",
            "x.compare(y, signed, true, xy_width != 4)",
        ),
        (
            "public-small-greater-equal-width-1-by-4",
            "public-greater-equal-width4-unsigned-order",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, true, true)",
            "x.compare(y, signed && xy_width != 4, true, true)",
        ),
        (
            "public-small-greater-equal-width-1-by-4",
            "public-greater-equal-width4-zero-extension",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "Op::GreaterEq => {\n                let xy_width = x.width().max(y.width());\n                let x = x.expand(xy_width, signed);",
            "Op::GreaterEq => {\n                let xy_width = x.width().max(y.width());\n                let x = x.expand(xy_width, signed && xy_width != 4);",
        ),
        (
            "public-small-greater-equal-width-1-by-4",
            "public-greater-equal-width4-ignores-unknown",
            "veryl/crates/analyzer/src/value.rs",
            "compare",
            "Self::new_bit_x1(self.is_xz() || rhs.is_xz(), relation)",
            "Self::new_bit_x1(self.width != 4 && (self.is_xz() || rhs.is_xz()), relation)",
        ),
        (
            "public-small-greater-equal-width-1-by-4",
            "public-greater-equal-width4-signed-result",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, true, true)",
            "{ let mut result = x.compare(y, signed, true, true); if xy_width == 4 { result.signed = true; } result }",
        ),
        (
            "public-small-greater-width-1-by-4",
            "public-greater-width4-direction",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, true, false)",
            "x.compare(y, signed, xy_width != 4, false)",
        ),
        (
            "public-small-greater-width-1-by-4",
            "public-greater-width4-inclusive",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, true, false)",
            "x.compare(y, signed, true, xy_width == 4)",
        ),
        (
            "public-small-greater-width-1-by-4",
            "public-greater-width4-unsigned-order",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, true, false)",
            "x.compare(y, signed && xy_width != 4, true, false)",
        ),
        (
            "public-small-greater-width-1-by-4",
            "public-greater-width4-zero-extension",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "Op::Greater => {\n                let xy_width = x.width().max(y.width());\n                let x = x.expand(xy_width, signed);",
            "Op::Greater => {\n                let xy_width = x.width().max(y.width());\n                let x = x.expand(xy_width, signed && xy_width != 4);",
        ),
        (
            "public-small-greater-width-1-by-4",
            "public-greater-width4-ignores-unknown",
            "veryl/crates/analyzer/src/value.rs",
            "compare",
            "Self::new_bit_x1(self.is_xz() || rhs.is_xz(), relation)",
            "Self::new_bit_x1(self.width != 4 && (self.is_xz() || rhs.is_xz()), relation)",
        ),
        (
            "public-small-greater-width-1-by-4",
            "public-greater-width4-signed-result",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, true, false)",
            "{ let mut result = x.compare(y, signed, true, false); if xy_width == 4 { result.signed = true; } result }",
        ),
        (
            "public-less-equal-left-width4",
            "public-less-equal-width4-direction",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, false, true)",
            "x.compare(y, signed, xy_width == 4, true)",
        ),
        (
            "public-less-equal-left-width4",
            "public-less-equal-width4-strict",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, false, true)",
            "x.compare(y, signed, false, xy_width != 4)",
        ),
        (
            "public-less-equal-left-width4",
            "public-less-equal-width4-unsigned-order",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, false, true)",
            "x.compare(y, signed && xy_width != 4, false, true)",
        ),
        (
            "public-less-equal-right-width4",
            "public-less-equal-width4-zero-extension",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "Op::LessEq => {\n                let xy_width = x.width().max(y.width());\n                let x = x.expand(xy_width, signed);",
            "Op::LessEq => {\n                let xy_width = x.width().max(y.width());\n                let x = x.expand(xy_width, signed && xy_width != 4);",
        ),
        (
            "public-less-equal-left-width4",
            "public-less-equal-width4-ignores-unknown",
            "veryl/crates/analyzer/src/value.rs",
            "compare",
            "Self::new_bit_x1(self.is_xz() || rhs.is_xz(), relation)",
            "Self::new_bit_x1(self.width != 4 && (self.is_xz() || rhs.is_xz()), relation)",
        ),
        (
            "public-less-equal-left-width4",
            "public-less-equal-width4-signed-result",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, false, true)",
            "{ let mut result = x.compare(y, signed, false, true); if xy_width == 4 { result.signed = true; } result }",
        ),
        (
            "public-small-greater-equal-width-2-by-2",
            "public-greater-equal-direction",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, true, true)",
            "x.compare(y, signed, false, true)",
        ),
        (
            "public-small-greater-equal-width-1-by-1",
            "public-greater-equal-strict",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, true, true)",
            "x.compare(y, signed, true, false)",
        ),
        (
            "public-small-greater-equal-width-2-by-2",
            "public-greater-equal-unsigned-order",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, true, true)",
            "x.compare(y, false, true, true)",
        ),
        (
            "public-small-greater-equal-width-1-by-2",
            "public-greater-equal-zero-extension",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "Op::GreaterEq => {\n                let xy_width = x.width().max(y.width());\n                let x = x.expand(xy_width, signed);",
            "Op::GreaterEq => {\n                let xy_width = x.width().max(y.width());\n                let x = x.expand(xy_width, false);",
        ),
        (
            "public-small-greater-equal-width-1-by-1",
            "public-greater-equal-ignores-unknown",
            "veryl/crates/analyzer/src/value.rs",
            "compare",
            "Self::new_bit_x1(self.is_xz() || rhs.is_xz(), relation)",
            "Self::new_bit_x1(false, relation)",
        ),
        (
            "public-small-greater-equal-width-1-by-1",
            "public-greater-equal-signed-result",
            "veryl/crates/analyzer/src/value.rs",
            "compare",
            "Self::new_bit_x1(self.is_xz() || rhs.is_xz(), relation)",
            "Self { signed: true, ..Self::new_bit_x1(self.is_xz() || rhs.is_xz(), relation) }",
        ),
        (
            "public-small-greater-width-2-by-2",
            "public-greater-direction",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, true, false)",
            "x.compare(y, signed, false, false)",
        ),
        (
            "public-small-greater-width-1-by-1",
            "public-greater-inclusive",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, true, false)",
            "x.compare(y, signed, true, true)",
        ),
        (
            "public-small-greater-width-2-by-2",
            "public-greater-unsigned-order",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, true, false)",
            "x.compare(y, false, true, false)",
        ),
        (
            "public-small-greater-width-1-by-2",
            "public-greater-zero-extension",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "Op::Greater => {\n                let xy_width = x.width().max(y.width());\n                let x = x.expand(xy_width, signed);",
            "Op::Greater => {\n                let xy_width = x.width().max(y.width());\n                let x = x.expand(xy_width, false);",
        ),
        (
            "public-small-greater-width-1-by-1",
            "public-greater-ignores-unknown",
            "veryl/crates/analyzer/src/value.rs",
            "compare",
            "Self::new_bit_x1(self.is_xz() || rhs.is_xz(), relation)",
            "Self::new_bit_x1(false, relation)",
        ),
        (
            "public-small-greater-width-1-by-1",
            "public-greater-signed-result",
            "veryl/crates/analyzer/src/value.rs",
            "compare",
            "Self::new_bit_x1(self.is_xz() || rhs.is_xz(), relation)",
            "Self { signed: true, ..Self::new_bit_x1(self.is_xz() || rhs.is_xz(), relation) }",
        ),
        (
            "public-small-less-equal-width-2-by-2",
            "public-less-equal-direction",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, false, true)",
            "x.compare(y, signed, true, true)",
        ),
        (
            "public-small-less-equal-width-1-by-1",
            "public-less-equal-strict",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, false, true)",
            "x.compare(y, signed, false, false)",
        ),
        (
            "public-small-less-equal-width-2-by-2",
            "public-less-equal-unsigned-order",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, false, true)",
            "x.compare(y, false, false, true)",
        ),
        (
            "public-small-less-equal-width-1-by-2",
            "public-less-equal-zero-extension",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "Op::LessEq => {\n                let xy_width = x.width().max(y.width());\n                let x = x.expand(xy_width, signed);",
            "Op::LessEq => {\n                let xy_width = x.width().max(y.width());\n                let x = x.expand(xy_width, false);",
        ),
        (
            "public-small-less-equal-width-1-by-1",
            "public-less-equal-ignores-unknown",
            "veryl/crates/analyzer/src/value.rs",
            "compare",
            "Self::new_bit_x1(self.is_xz() || rhs.is_xz(), relation)",
            "Self::new_bit_x1(false, relation)",
        ),
        (
            "public-small-less-equal-width-1-by-1",
            "public-less-equal-signed-result",
            "veryl/crates/analyzer/src/value.rs",
            "compare",
            "Self::new_bit_x1(self.is_xz() || rhs.is_xz(), relation)",
            "Self { signed: true, ..Self::new_bit_x1(self.is_xz() || rhs.is_xz(), relation) }",
        ),
        (
            "public-small-less-unsigned-continuation",
            "public-less-unsigned-direction",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, false, false)",
            "x.compare(y, signed, true, false)",
        ),
        (
            "public-small-less-unsigned-continuation",
            "public-less-unsigned-inclusive",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, false, false)",
            "x.compare(y, signed, false, true)",
        ),
        (
            "public-small-less-signed-continuation",
            "public-less-signed-unsigned-order",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.compare(y, signed, false, false)",
            "x.compare(y, false, false, false)",
        ),
        (
            "public-small-less-signed-continuation",
            "public-less-signed-zero-extension",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "Op::Less => {\n                let xy_width = x.width().max(y.width());\n                let x = x.expand(xy_width, signed);",
            "Op::Less => {\n                let xy_width = x.width().max(y.width());\n                let x = x.expand(xy_width, false);",
        ),
        (
            "public-small-less-unsigned-continuation",
            "public-less-unsigned-ignores-unknown",
            "veryl/crates/analyzer/src/value.rs",
            "compare",
            "Self::new_bit_x1(self.is_xz() || rhs.is_xz(), relation)",
            "Self::new_bit_x1(false, relation)",
        ),
        (
            "public-small-less-unsigned-continuation",
            "public-less-unsigned-signed-result",
            "veryl/crates/analyzer/src/value.rs",
            "compare",
            "Self::new_bit_x1(self.is_xz() || rhs.is_xz(), relation)",
            "Self { signed: true, ..Self::new_bit_x1(self.is_xz() || rhs.is_xz(), relation) }",
        ),
        (
            "public-inequality-native-unsigned-continuation",
            "public-inequality-continuation-not-negated",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.equal(y, false, true)",
            "x.equal(y, false, false)",
        ),
        (
            "public-inequality-native-unsigned-continuation",
            "public-inequality-continuation-uses-wildcard",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.equal(y, false, true)",
            "x.equal(y, true, true)",
        ),
        (
            "public-inequality-native-signed-continuation",
            "public-inequality-continuation-zero-sign-extension",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "Op::Ne => {\n                let xy_width = x.width().max(y.width());\n                let eq_signed = x.signed() && y.signed();",
            "Op::Ne => {\n                let xy_width = x.width().max(y.width());\n                let eq_signed = false;",
        ),
        (
            "public-small-wildcard-equality-continuation",
            "public-wildcard-continuation-swaps-operands",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.equal(y, true, false)",
            "y.equal(x, true, false)",
        ),
        (
            "public-inequality-native-unsigned-continuation",
            "public-inequality-continuation-signed-result",
            "veryl/crates/analyzer/src/value.rs",
            "equal",
            "Self::new_bit_1x(mismatch, unknown)",
            "Self { signed: true, ..Self::new_bit_1x(mismatch, unknown) }",
        ),
        (
            "public-small-wildcard-equality-continuation",
            "public-wildcard-continuation-ignores-wildcards",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.equal(y, true, false)",
            "x.equal(y, false, false)",
        ),
        (
            "public-small-wildcard-inequality-unsigned-continuation",
            "public-wildcard-inequality-unsigned-not-negated",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.equal(y, true, true)",
            "x.equal(y, true, false)",
        ),
        (
            "public-small-wildcard-inequality-unsigned-continuation",
            "public-wildcard-inequality-unsigned-swaps-operands",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.equal(y, true, true)",
            "y.equal(x, true, true)",
        ),
        (
            "public-small-wildcard-inequality-signed-continuation",
            "public-wildcard-inequality-signed-ignores-wildcards",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.equal(y, true, true)",
            "x.equal(y, false, true)",
        ),
        (
            "public-small-wildcard-inequality-unsigned-continuation",
            "public-wildcard-inequality-unsigned-signed-result",
            "veryl/crates/analyzer/src/value.rs",
            "equal",
            "Self::new_bit_1x(mismatch, unknown)",
            "Self { signed: true, ..Self::new_bit_1x(mismatch, unknown) }",
        ),
        (
            "public-small-wildcard-inequality-signed-continuation",
            "public-wildcard-inequality-signed-zero-extension",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "Op::NeWildcard => {\n                let xy_width = x.width().max(y.width());\n                let eq_signed = x.signed() && y.signed();",
            "Op::NeWildcard => {\n                let xy_width = x.width().max(y.width());\n                let eq_signed = false;",
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
    // Logical-operation mutations preserve the exact earlier call trace. Wide
    // variants leave native result calls intact; LogicNot leaves NOR unchanged.
    for (contract_name, name, method, old, new) in [
        (
            "public-logical-not-native-continuation",
            "public-logical-not-native-as-or",
            "eval_value_unary",
            "Value::U64(x) => x.reduce_or(true),",
            "Value::U64(x) => if matches!(self, Op::LogicNot) { x.reduce_or(false) } else { x.reduce_or(true) },",
        ),
        (
            "public-logical-not-native-continuation",
            "public-logical-not-native-as-nand",
            "eval_value_unary",
            "Value::U64(x) => x.reduce_or(true),",
            "Value::U64(x) => if matches!(self, Op::LogicNot) { x.reduce_and(true) } else { x.reduce_or(true) },",
        ),
        (
            "public-logical-not-native-continuation",
            "public-logical-not-native-ignores-unknown",
            "eval_value_unary",
            "Value::U64(x) => x.reduce_or(true),",
            "Value::U64(x) => if matches!(self, Op::LogicNot) { { let mut result = x.reduce_or(true); result.mask_xz = 0; result } } else { x.reduce_or(true) },",
        ),
        (
            "public-logical-not-native-continuation",
            "public-logical-not-native-ignores-upper-word",
            "eval_value_unary",
            "Value::U64(x) => x.reduce_or(true),",
            "Value::U64(x) => if matches!(self, Op::LogicNot) { { let mut source = x.clone(); source.trunc(32); source.reduce_or(true) } } else { x.reduce_or(true) },",
        ),
        (
            "public-logical-not-native-continuation",
            "public-logical-not-native-upper-padding-leak",
            "eval_value_unary",
            "Value::U64(x) => x.reduce_or(true),",
            "Value::U64(x) => if matches!(self, Op::LogicNot) { { let mut result = x.reduce_or(true); result.payload |= 1u64 << 63; result } } else { x.reduce_or(true) },",
        ),
        (
            "public-logical-not-width65",
            "public-logical-not-wide-as-or",
            "eval_value_unary",
            "Value::U64(x) => x.reduce_or(true),",
            "Value::U64(x) => if matches!(self, Op::LogicNot) && width >= 65 { x.reduce_or(false) } else { x.reduce_or(true) },",
        ),
        (
            "public-logical-not-width65",
            "public-logical-not-wide-as-nand",
            "eval_value_unary",
            "Value::U64(x) => x.reduce_or(true),",
            "Value::U64(x) => if matches!(self, Op::LogicNot) && width >= 65 { x.reduce_and(true) } else { x.reduce_or(true) },",
        ),
        (
            "public-logical-not-width65",
            "public-logical-not-wide-ignores-unknown",
            "eval_value_unary",
            "Value::U64(x) => x.reduce_or(true),",
            "Value::U64(x) => if matches!(self, Op::LogicNot) && width >= 65 { { let mut result = x.reduce_or(true); result.mask_xz = 0; result } } else { x.reduce_or(true) },",
        ),
        (
            "public-logical-not-width65",
            "public-logical-not-wide-ignores-upper-word",
            "eval_value_unary",
            "Value::U64(x) => x.reduce_or(true),",
            "Value::U64(x) => if matches!(self, Op::LogicNot) && width >= 65 { { let mut source = x.clone(); source.trunc(32); source.reduce_or(true) } } else { x.reduce_or(true) },",
        ),
        (
            "public-logical-not-width65",
            "public-logical-not-wide-upper-padding-leak",
            "eval_value_unary",
            "Value::U64(x) => x.reduce_or(true),",
            "Value::U64(x) => if matches!(self, Op::LogicNot) && width >= 65 { { let mut result = x.reduce_or(true); result.payload |= 1u64 << 63; result } } else { x.reduce_or(true) },",
        ),
        (
            "public-logical-and-width65-witness",
            "public-logical-and-wide-as-or",
            "eval_value_binary",
            "(Value::U64(x), Value::U64(y)) => x.logical_binary(y, true),",
            "(Value::U64(x), Value::U64(y)) => if width >= 65 { x.logical_binary(y, false) } else { x.logical_binary(y, true) },",
        ),
        (
            "public-logical-and-width65-witness",
            "public-logical-and-wide-unknown-overrides-zero",
            "eval_value_binary",
            "(Value::U64(x), Value::U64(y)) => x.logical_binary(y, true),",
            "(Value::U64(x), Value::U64(y)) => if width >= 65 { { let mut result = x.logical_binary(y, true); if x.mask_xz != 0 || y.mask_xz != 0 { result.payload = 0; result.mask_xz = 1; } result } } else { x.logical_binary(y, true) },",
        ),
        (
            "public-logical-and-width65-witness",
            "public-logical-and-wide-ignores-unknown",
            "eval_value_binary",
            "(Value::U64(x), Value::U64(y)) => x.logical_binary(y, true),",
            "(Value::U64(x), Value::U64(y)) => if width >= 65 { { let mut result = x.logical_binary(y, true); result.mask_xz = 0; result } } else { x.logical_binary(y, true) },",
        ),
        (
            "public-logical-and-width65-witness",
            "public-logical-and-wide-ignores-rhs",
            "eval_value_binary",
            "(Value::U64(x), Value::U64(y)) => x.logical_binary(y, true),",
            "(Value::U64(x), Value::U64(y)) => if width >= 65 { x.logical_binary(x, true) } else { x.logical_binary(y, true) },",
        ),
        (
            "public-logical-and-width65-witness",
            "public-logical-and-wide-upper-padding-leak",
            "eval_value_binary",
            "(Value::U64(x), Value::U64(y)) => x.logical_binary(y, true),",
            "(Value::U64(x), Value::U64(y)) => if width >= 65 { { let mut result = x.logical_binary(y, true); result.payload |= 1u64 << 63; result } } else { x.logical_binary(y, true) },",
        ),
        (
            "public-logical-or-native-witness",
            "public-logical-or-native-as-and",
            "eval_value_binary",
            "(Value::U64(x), Value::U64(y)) => x.logical_binary(y, false),",
            "(Value::U64(x), Value::U64(y)) => x.logical_binary(y, true),",
        ),
        (
            "public-logical-or-native-witness",
            "public-logical-or-native-unknown-overrides-one",
            "eval_value_binary",
            "(Value::U64(x), Value::U64(y)) => x.logical_binary(y, false),",
            "(Value::U64(x), Value::U64(y)) => { let mut result = x.logical_binary(y, false); if x.mask_xz != 0 || y.mask_xz != 0 { result.payload = 0; result.mask_xz = 1; } result },",
        ),
        (
            "public-logical-or-native-witness",
            "public-logical-or-native-ignores-unknown",
            "eval_value_binary",
            "(Value::U64(x), Value::U64(y)) => x.logical_binary(y, false),",
            "(Value::U64(x), Value::U64(y)) => { let mut result = x.logical_binary(y, false); result.mask_xz = 0; result },",
        ),
        (
            "public-logical-or-native-witness",
            "public-logical-or-native-ignores-rhs",
            "eval_value_binary",
            "(Value::U64(x), Value::U64(y)) => x.logical_binary(y, false),",
            "(Value::U64(x), Value::U64(y)) => x.logical_binary(x, false),",
        ),
        (
            "public-logical-or-native-witness",
            "public-logical-or-native-upper-padding-leak",
            "eval_value_binary",
            "(Value::U64(x), Value::U64(y)) => x.logical_binary(y, false),",
            "(Value::U64(x), Value::U64(y)) => { let mut result = x.logical_binary(y, false); result.payload |= 1u64 << 63; result },",
        ),
        (
            "public-logical-or-width65-witness",
            "public-logical-or-wide-as-and",
            "eval_value_binary",
            "(Value::U64(x), Value::U64(y)) => x.logical_binary(y, false),",
            "(Value::U64(x), Value::U64(y)) => if width >= 65 { x.logical_binary(y, true) } else { x.logical_binary(y, false) },",
        ),
        (
            "public-logical-or-width65-witness",
            "public-logical-or-wide-unknown-overrides-one",
            "eval_value_binary",
            "(Value::U64(x), Value::U64(y)) => x.logical_binary(y, false),",
            "(Value::U64(x), Value::U64(y)) => if width >= 65 { { let mut result = x.logical_binary(y, false); if x.mask_xz != 0 || y.mask_xz != 0 { result.payload = 0; result.mask_xz = 1; } result } } else { x.logical_binary(y, false) },",
        ),
        (
            "public-logical-or-width65-witness",
            "public-logical-or-wide-ignores-unknown",
            "eval_value_binary",
            "(Value::U64(x), Value::U64(y)) => x.logical_binary(y, false),",
            "(Value::U64(x), Value::U64(y)) => if width >= 65 { { let mut result = x.logical_binary(y, false); result.mask_xz = 0; result } } else { x.logical_binary(y, false) },",
        ),
        (
            "public-logical-or-width65-witness",
            "public-logical-or-wide-ignores-rhs",
            "eval_value_binary",
            "(Value::U64(x), Value::U64(y)) => x.logical_binary(y, false),",
            "(Value::U64(x), Value::U64(y)) => if width >= 65 { x.logical_binary(x, false) } else { x.logical_binary(y, false) },",
        ),
        (
            "public-logical-or-width65-witness",
            "public-logical-or-wide-upper-padding-leak",
            "eval_value_binary",
            "(Value::U64(x), Value::U64(y)) => x.logical_binary(y, false),",
            "(Value::U64(x), Value::U64(y)) => if width >= 65 { { let mut result = x.logical_binary(y, false); result.payload |= 1u64 << 63; result } } else { x.logical_binary(y, false) },",
        ),
    ] {
        if let Some(&contract) = selected.iter().find(|c| c.name == contract_name) {
            let source =
                fs::read_to_string(repo.join("veryl/crates/analyzer/src/ir/op.rs")).unwrap();
            mutations.push(Mutation {
                name,
                contract,
                path: "veryl/crates/analyzer/src/ir/op.rs",
                source: mutate_method(&source, method, old, new),
            });
        }
    }
    for (contract_name, name, method_name, start_marker, end_marker, replacement) in [
        (
            "public-logical-not-native-continuation",
            "public-logical-not-native-wrong-width",
            "eval_value_unary",
            "            Op::BitNor | Op::LogicNot => {",
            "            Op::BitXor => {",
            "ret.expand(if matches!(self, Op::LogicNot) && width == 8 { 1 } else { width }, false).into_owned()",
        ),
        (
            "public-logical-not-native-continuation",
            "public-logical-not-native-signed-result",
            "eval_value_unary",
            "            Op::BitNor | Op::LogicNot => {",
            "            Op::BitXor => {",
            "{ let mut output = ret.expand(width, false).into_owned(); if matches!(self, Op::LogicNot) { if let Value::U64(ref mut native) = output { native.signed = true; } } output }",
        ),
        (
            "public-logical-not-width65",
            "public-logical-not-wide-wrong-width",
            "eval_value_unary",
            "            Op::BitNor | Op::LogicNot => {",
            "            Op::BitXor => {",
            "ret.expand(if matches!(self, Op::LogicNot) && width == 65 { 66 } else { width }, false).into_owned()",
        ),
        (
            "public-logical-not-width65",
            "public-logical-not-wide-signed-result",
            "eval_value_unary",
            "            Op::BitNor | Op::LogicNot => {",
            "            Op::BitXor => {",
            "{ let mut output = ret.expand(width, false).into_owned(); if matches!(self, Op::LogicNot) { if let Value::BigUint(ref mut big) = output { big.signed = true; } } output }",
        ),
        (
            "public-logical-and-width65-witness",
            "public-logical-and-wide-wrong-width",
            "eval_value_binary",
            "            Op::LogicAnd => {",
            "            Op::LogicOr => {",
            "ret.expand(if true && width == 65 { 66 } else { width }, false).into_owned()",
        ),
        (
            "public-logical-and-width65-witness",
            "public-logical-and-wide-signed-result",
            "eval_value_binary",
            "            Op::LogicAnd => {",
            "            Op::LogicOr => {",
            "{ let mut output = ret.expand(width, false).into_owned(); if true { if let Value::BigUint(ref mut big) = output { big.signed = true; } } output }",
        ),
        (
            "public-logical-or-native-witness",
            "public-logical-or-native-wrong-width",
            "eval_value_binary",
            "            Op::LogicOr => {",
            "            Op::LogicShiftR => {",
            "ret.expand(if true && width == 8 { 1 } else { width }, false).into_owned()",
        ),
        (
            "public-logical-or-native-witness",
            "public-logical-or-native-signed-result",
            "eval_value_binary",
            "            Op::LogicOr => {",
            "            Op::LogicShiftR => {",
            "{ let mut output = ret.expand(width, false).into_owned(); if true { if let Value::U64(ref mut native) = output { native.signed = true; } } output }",
        ),
        (
            "public-logical-or-width65-witness",
            "public-logical-or-wide-wrong-width",
            "eval_value_binary",
            "            Op::LogicOr => {",
            "            Op::LogicShiftR => {",
            "ret.expand(if true && width == 65 { 66 } else { width }, false).into_owned()",
        ),
        (
            "public-logical-or-width65-witness",
            "public-logical-or-wide-signed-result",
            "eval_value_binary",
            "            Op::LogicOr => {",
            "            Op::LogicShiftR => {",
            "{ let mut output = ret.expand(width, false).into_owned(); if true { if let Value::BigUint(ref mut big) = output { big.signed = true; } } output }",
        ),
    ] {
        if let Some(&contract) = selected.iter().find(|c| c.name == contract_name) {
            let source =
                fs::read_to_string(repo.join("veryl/crates/analyzer/src/ir/op.rs")).unwrap();
            let method = &source[method_range(&source, method_name)];
            let start = method.find(start_marker).unwrap();
            let end = start + method[start..].find(end_marker).unwrap();
            let branch = &method[start..end];
            let replacement =
                replace_once(branch, "ret.expand(width, false).into_owned()", replacement);
            mutations.push(Mutation {
                name,
                contract,
                path: "veryl/crates/analyzer/src/ir/op.rs",
                source: mutate_method(&source, method_name, branch, &replacement),
            });
        }
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-reduction-xor-width65")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        // Mutate only BigUint XOR result calls, preserving the entire prefix.
        for (name, expression) in [
            ("public-wide-reduction-xor-as-xnor", "x.reduce_xor(true)"),
            ("public-wide-reduction-xor-as-or", "x.reduce_or(false)"),
            (
                "public-wide-reduction-xor-ignores-unknown",
                "{ let mut result = x.reduce_xor(false); result.mask_xz = 0; result }",
            ),
            (
                "public-wide-reduction-xor-ignores-upper-word",
                "{ let mut source = x.clone(); source.trunc(32); source.reduce_xor(false) }",
            ),
            (
                "public-wide-reduction-xor-upper-padding-leak",
                "{ let mut result = x.reduce_xor(false); result.payload |= 1u64 << 63; result }",
            ),
        ] {
            let replacement = format!(
                "Value::U64(x) => if width >= 65 {{ {expression} }} else {{ x.reduce_xor(false) }},"
            );
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(
                    &source,
                    "eval_value_unary",
                    "Value::U64(x) => x.reduce_xor(false),",
                    &replacement,
                ),
            });
        }
        let method = &source[method_range(&source, "eval_value_unary")];
        let start = method.find("            Op::BitXor => {").unwrap();
        let end = start
            + method[start..]
                .find("            Op::BitXnor => {")
                .unwrap();
        let branch = &method[start..end];
        for (name, replacement) in [
            (
                "public-wide-reduction-xor-wrong-width",
                "Value::U64(ret).expand(if width == 65 { 66 } else { width }, false).into_owned()",
            ),
            (
                "public-wide-reduction-xor-signed-result",
                "{ let mut output = Value::U64(ret).expand(width, false).into_owned(); if let Value::BigUint(ref mut big) = output { big.signed = true; } output }",
            ),
        ] {
            let replacement = replace_once(
                branch,
                "Value::U64(ret).expand(width, false).into_owned()",
                replacement,
            );
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_unary", branch, &replacement),
            });
        }
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-reduction-xnor-width65")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        // Mutate only BigUint XNOR result calls, preserving the entire prefix.
        for (name, expression) in [
            ("public-wide-reduction-xnor-as-xor", "x.reduce_xor(false)"),
            ("public-wide-reduction-xnor-as-nor", "x.reduce_or(true)"),
            (
                "public-wide-reduction-xnor-ignores-unknown",
                "{ let mut result = x.reduce_xor(true); result.mask_xz = 0; result }",
            ),
            (
                "public-wide-reduction-xnor-ignores-upper-word",
                "{ let mut source = x.clone(); source.trunc(32); source.reduce_xor(true) }",
            ),
            (
                "public-wide-reduction-xnor-upper-padding-leak",
                "{ let mut result = x.reduce_xor(true); result.payload |= 1u64 << 63; result }",
            ),
        ] {
            let replacement = format!(
                "Value::U64(x) => if width >= 65 {{ {expression} }} else {{ x.reduce_xor(true) }},"
            );
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(
                    &source,
                    "eval_value_unary",
                    "Value::U64(x) => x.reduce_xor(true),",
                    &replacement,
                ),
            });
        }
        let method = &source[method_range(&source, "eval_value_unary")];
        let start = method.find("            Op::BitXnor => {").unwrap();
        let end = start
            + method[start..]
                .find("            _ => unimplemented!(),")
                .unwrap();
        let branch = &method[start..end];
        for (name, replacement) in [
            (
                "public-wide-reduction-xnor-wrong-width",
                "Value::U64(ret).expand(if width == 65 { 66 } else { width }, false).into_owned()",
            ),
            (
                "public-wide-reduction-xnor-signed-result",
                "{ let mut output = Value::U64(ret).expand(width, false).into_owned(); if let Value::BigUint(ref mut big) = output { big.signed = true; } output }",
            ),
        ] {
            let replacement = replace_once(
                branch,
                "Value::U64(ret).expand(width, false).into_owned()",
                replacement,
            );
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_unary", branch, &replacement),
            });
        }
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-reduction-xnor-native-continuation")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        // Mutate only XNOR, leaving every NOT and earlier reduction call intact.
        for (name, expression) in [
            ("public-native-reduction-xnor-as-xor", "x.reduce_xor(false)"),
            ("public-native-reduction-xnor-as-nor", "x.reduce_or(true)"),
            (
                "public-native-reduction-xnor-ignores-unknown",
                "{ let mut result = x.reduce_xor(true); result.mask_xz = 0; result }",
            ),
            (
                "public-native-reduction-xnor-ignores-upper-word",
                "{ let mut source = x.clone(); source.trunc(32); source.reduce_xor(true) }",
            ),
            (
                "public-native-reduction-xnor-upper-padding-leak",
                "{ let mut result = x.reduce_xor(true); result.payload |= 1u64 << 63; result }",
            ),
        ] {
            let replacement = format!("Value::U64(x) => {expression},");
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(
                    &source,
                    "eval_value_unary",
                    "Value::U64(x) => x.reduce_xor(true),",
                    &replacement,
                ),
            });
        }
        let method = &source[method_range(&source, "eval_value_unary")];
        let start = method.find("            Op::BitXnor => {").unwrap();
        let end = start
            + method[start..]
                .find("            _ => unimplemented!(),")
                .unwrap();
        let branch = &method[start..end];
        for (name, replacement) in [
            (
                "public-native-reduction-xnor-wrong-width",
                "Value::U64(ret).expand(if width == 8 { 1 } else { width }, false).into_owned()",
            ),
            (
                "public-native-reduction-xnor-signed-result",
                "{ let mut output = Value::U64(ret).expand(width, false).into_owned(); if let Value::U64(ref mut native) = output { native.signed = true; } output }",
            ),
        ] {
            let replacement = replace_once(
                branch,
                "Value::U64(ret).expand(width, false).into_owned()",
                replacement,
            );
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_unary", branch, &replacement),
            });
        }
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-reduction-xor-native-continuation")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        // Mutate only XOR, leaving every NOT/AND/NAND/OR/NOR prefix call intact.
        for (name, expression) in [
            ("public-native-reduction-xor-as-xnor", "x.reduce_xor(true)"),
            ("public-native-reduction-xor-as-or", "x.reduce_or(false)"),
            (
                "public-native-reduction-xor-ignores-unknown",
                "{ let mut result = x.reduce_xor(false); result.mask_xz = 0; result }",
            ),
            (
                "public-native-reduction-xor-ignores-upper-word",
                "{ let mut source = x.clone(); source.trunc(32); source.reduce_xor(false) }",
            ),
            (
                "public-native-reduction-xor-upper-padding-leak",
                "{ let mut result = x.reduce_xor(false); result.payload |= 1u64 << 63; result }",
            ),
        ] {
            let replacement = format!("Value::U64(x) => {expression},");
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(
                    &source,
                    "eval_value_unary",
                    "Value::U64(x) => x.reduce_xor(false),",
                    &replacement,
                ),
            });
        }
        let method = &source[method_range(&source, "eval_value_unary")];
        let start = method.find("            Op::BitXor => {").unwrap();
        let end = start
            + method[start..]
                .find("            Op::BitXnor => {")
                .unwrap();
        let branch = &method[start..end];
        for (name, replacement) in [
            (
                "public-native-reduction-xor-wrong-width",
                "Value::U64(ret).expand(if width == 8 { 1 } else { width }, false).into_owned()",
            ),
            (
                "public-native-reduction-xor-signed-result",
                "{ let mut output = Value::U64(ret).expand(width, false).into_owned(); if let Value::U64(ref mut native) = output { native.signed = true; } output }",
            ),
        ] {
            let replacement = replace_once(
                branch,
                "Value::U64(ret).expand(width, false).into_owned()",
                replacement,
            );
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_unary", branch, &replacement),
            });
        }
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-reduction-nor-width65")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        // Preserve the full prefix, including NOR 1/8 and the shared LogicNot arm.
        for (name, expression) in [
            ("public-wide-reduction-nor-as-or", "x.reduce_or(false)"),
            ("public-wide-reduction-nor-as-nand", "x.reduce_and(true)"),
            (
                "public-wide-reduction-nor-ignores-unknown",
                "{ let mut result = x.reduce_or(true); result.mask_xz = 0; result }",
            ),
            (
                "public-wide-reduction-nor-unknown-overrides-one",
                "{ let mut result = x.reduce_or(true); if x.mask_xz != 0 { result.payload = 0; result.mask_xz = 1; } result }",
            ),
            (
                "public-wide-reduction-nor-upper-padding-leak",
                "{ let mut result = x.reduce_or(true); result.payload |= 1u64 << 63; result }",
            ),
        ] {
            let replacement = format!(
                "Value::U64(x) => if matches!(self, Op::BitNor) && width >= 65 {{ {expression} }} else {{ x.reduce_or(true) }},"
            );
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(
                    &source,
                    "eval_value_unary",
                    "Value::U64(x) => x.reduce_or(true),",
                    &replacement,
                ),
            });
        }
        let method = &source[method_range(&source, "eval_value_unary")];
        let start = method
            .find("            Op::BitNor | Op::LogicNot => {")
            .unwrap();
        let end = start + method[start..].find("            Op::BitXor => {").unwrap();
        let branch = &method[start..end];
        for (name, replacement) in [
            (
                "public-wide-reduction-nor-wrong-width",
                "ret.expand(if matches!(self, Op::BitNor) && width == 65 { 66 } else { width }, false).into_owned()",
            ),
            (
                "public-wide-reduction-nor-signed-result",
                "{ let mut output = ret.expand(width, false).into_owned(); if matches!(self, Op::BitNor) && width >= 65 { if let Value::BigUint(ref mut big) = output { big.signed = true; } } output }",
            ),
        ] {
            let replacement =
                replace_once(branch, "ret.expand(width, false).into_owned()", replacement);
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_unary", branch, &replacement),
            });
        }
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-reduction-nor-native-continuation")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        // Preserve all NOT/AND/NAND/OR prefix calls and the shared LogicNot arm.
        for (name, expression) in [
            ("public-native-reduction-nor-as-or", "x.reduce_or(false)"),
            ("public-native-reduction-nor-as-nand", "x.reduce_and(true)"),
            (
                "public-native-reduction-nor-ignores-unknown",
                "{ let mut result = x.reduce_or(true); result.mask_xz = 0; result }",
            ),
            (
                "public-native-reduction-nor-unknown-overrides-one",
                "{ let mut result = x.reduce_or(true); if x.mask_xz != 0 { result.payload = 0; result.mask_xz = 1; } result }",
            ),
            (
                "public-native-reduction-nor-upper-padding-leak",
                "{ let mut result = x.reduce_or(true); result.payload |= 1u64 << 63; result }",
            ),
        ] {
            let replacement = format!(
                "Value::U64(x) => if matches!(self, Op::BitNor) {{ {expression} }} else {{ x.reduce_or(true) }},"
            );
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(
                    &source,
                    "eval_value_unary",
                    "Value::U64(x) => x.reduce_or(true),",
                    &replacement,
                ),
            });
        }
        let method = &source[method_range(&source, "eval_value_unary")];
        let start = method
            .find("            Op::BitNor | Op::LogicNot => {")
            .unwrap();
        let end = start + method[start..].find("            Op::BitXor => {").unwrap();
        let branch = &method[start..end];
        for (name, replacement) in [
            (
                "public-native-reduction-nor-wrong-width",
                "ret.expand(if matches!(self, Op::BitNor) && width == 8 { 1 } else { width }, false).into_owned()",
            ),
            (
                "public-native-reduction-nor-signed-result",
                "{ let mut output = ret.expand(width, false).into_owned(); if matches!(self, Op::BitNor) { if let Value::U64(ref mut native) = output { native.signed = true; } } output }",
            ),
        ] {
            let replacement =
                replace_once(branch, "ret.expand(width, false).into_owned()", replacement);
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_unary", branch, &replacement),
            });
        }
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-reduction-or-native-continuation")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        // Mutate only OR, preserving every NOT/AND/NAND prefix call.
        for (name, replacement) in [
            ("public-native-reduction-or-as-and", "x.reduce_and(false)"),
            ("public-native-reduction-or-as-nor", "x.reduce_or(true)"),
            (
                "public-native-reduction-or-ignores-unknown",
                "{ let mut result = x.reduce_or(false); result.mask_xz = 0; result }",
            ),
            (
                "public-native-reduction-or-unknown-overrides-one",
                "{ let mut result = x.reduce_or(false); if x.mask_xz != 0 { result.payload = 0; result.mask_xz = 1; } result }",
            ),
            (
                "public-native-reduction-or-upper-padding-leak",
                "{ let mut result = x.reduce_or(false); result.payload |= 1u64 << 63; result }",
            ),
        ] {
            let replacement = format!("Value::U64(x) => {replacement},");
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(
                    &source,
                    "eval_value_unary",
                    "Value::U64(x) => x.reduce_or(false),",
                    &replacement,
                ),
            });
        }
        let method = &source[method_range(&source, "eval_value_unary")];
        let start = method.find("            Op::BitOr => {").unwrap();
        let end = start
            + method[start..]
                .find("            Op::BitNor | Op::LogicNot => {")
                .unwrap();
        let branch = &method[start..end];
        for (name, replacement) in [
            (
                "public-native-reduction-or-wrong-width",
                "ret.expand(if width == 8 { 1 } else { width }, false).into_owned()",
            ),
            (
                "public-native-reduction-or-signed-result",
                "{ let mut output = ret.expand(width, false).into_owned(); if let Value::U64(ref mut native) = output { native.signed = true; } output }",
            ),
        ] {
            let replacement =
                replace_once(branch, "ret.expand(width, false).into_owned()", replacement);
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_unary", branch, &replacement),
            });
        }
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-reduction-nand-native-continuation")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        // Mutate only NAND, preserving every NOT/AND prefix call.
        for (name, replacement) in [
            ("public-native-reduction-nand-as-and", "x.reduce_and(false)"),
            ("public-native-reduction-nand-as-nor", "x.reduce_or(true)"),
            (
                "public-native-reduction-nand-ignores-unknown",
                "{ let mut result = x.reduce_and(true); result.mask_xz = 0; result }",
            ),
            (
                "public-native-reduction-nand-unknown-overrides-zero",
                "{ let mut result = x.reduce_and(true); if x.mask_xz != 0 { result.payload = 0; result.mask_xz = 1; } result }",
            ),
            (
                "public-native-reduction-nand-upper-padding-leak",
                "{ let mut result = x.reduce_and(true); result.payload |= 1u64 << 63; result }",
            ),
        ] {
            let replacement = format!("Value::U64(x) => {replacement},");
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(
                    &source,
                    "eval_value_unary",
                    "Value::U64(x) => x.reduce_and(true),",
                    &replacement,
                ),
            });
        }
        let method = &source[method_range(&source, "eval_value_unary")];
        let start = method.find("            Op::BitNand => {").unwrap();
        let end = start + method[start..].find("            Op::BitOr => {").unwrap();
        let branch = &method[start..end];
        for (name, replacement) in [
            (
                "public-native-reduction-nand-wrong-width",
                "ret.expand(if width == 8 { 1 } else { width }, false).into_owned()",
            ),
            (
                "public-native-reduction-nand-signed-result",
                "{ let mut output = ret.expand(width, false).into_owned(); if let Value::U64(ref mut native) = output { native.signed = true; } output }",
            ),
        ] {
            let replacement =
                replace_once(branch, "ret.expand(width, false).into_owned()", replacement);
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_unary", branch, &replacement),
            });
        }
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-reduction-or-width65")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        // Each mutation leaves the native result prefix unchanged and affects
        // only the BigUint result calls newly covered by this contract.
        for (name, expression) in [
            ("public-wide-reduction-or-as-and", "x.reduce_and(false)"),
            ("public-wide-reduction-or-as-nor", "x.reduce_or(true)"),
            (
                "public-wide-reduction-or-unknown-overrides-one",
                "{ let mut result = x.reduce_or(false); if x.mask_xz != 0 { result.payload = 0; result.mask_xz = 1; } result }",
            ),
            (
                "public-wide-reduction-or-ignores-unknown",
                "{ let mut result = x.reduce_or(false); result.mask_xz = 0; result }",
            ),
            (
                "public-wide-reduction-or-upper-padding-leak",
                "{ let mut result = x.reduce_or(false); result.payload |= 1u64 << 63; result }",
            ),
        ] {
            let replacement = format!(
                "Value::U64(x) => if width >= 65 {{ {expression} }} else {{ x.reduce_or(false) }},"
            );
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(
                    &source,
                    "eval_value_unary",
                    "Value::U64(x) => x.reduce_or(false),",
                    &replacement,
                ),
            });
        }
        let method = &source[method_range(&source, "eval_value_unary")];
        let start = method.find("            Op::BitOr => {").unwrap();
        let end = start
            + method[start..]
                .find("            Op::BitNor | Op::LogicNot => {")
                .unwrap();
        let branch = &method[start..end];
        for (name, replacement) in [
            (
                "public-wide-reduction-or-wrong-width",
                "ret.expand(if width == 65 { 66 } else { width }, false).into_owned()",
            ),
            (
                "public-wide-reduction-or-signed-result",
                "{ let mut output = ret.expand(width, false).into_owned(); if let Value::BigUint(ref mut big) = output { big.signed = true; } output }",
            ),
        ] {
            let replacement =
                replace_once(branch, "ret.expand(width, false).into_owned()", replacement);
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_unary", branch, &replacement),
            });
        }
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-reduction-nand-width65")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        // Each mutation leaves the native result prefix unchanged and affects
        // only the BigUint result calls newly covered by this contract.
        for (name, expression) in [
            ("public-wide-reduction-nand-as-and", "x.reduce_and(false)"),
            ("public-wide-reduction-nand-as-nor", "x.reduce_or(true)"),
            (
                "public-wide-reduction-nand-unknown-overrides-zero",
                "{ let mut result = x.reduce_and(true); if x.mask_xz != 0 { result.payload = 0; result.mask_xz = 1; } result }",
            ),
            (
                "public-wide-reduction-nand-ignores-unknown",
                "{ let mut result = x.reduce_and(true); result.mask_xz = 0; result }",
            ),
            (
                "public-wide-reduction-nand-upper-padding-leak",
                "{ let mut result = x.reduce_and(true); result.payload |= 1u64 << 63; result }",
            ),
        ] {
            let replacement = format!(
                "Value::U64(x) => if width >= 65 {{ {expression} }} else {{ x.reduce_and(true) }},"
            );
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(
                    &source,
                    "eval_value_unary",
                    "Value::U64(x) => x.reduce_and(true),",
                    &replacement,
                ),
            });
        }
        let method = &source[method_range(&source, "eval_value_unary")];
        let start = method.find("            Op::BitNand => {").unwrap();
        let end = start + method[start..].find("            Op::BitOr => {").unwrap();
        let branch = &method[start..end];
        for (name, replacement) in [
            (
                "public-wide-reduction-nand-wrong-width",
                "ret.expand(if width == 65 { 66 } else { width }, false).into_owned()",
            ),
            (
                "public-wide-reduction-nand-signed-result",
                "{ let mut output = ret.expand(width, false).into_owned(); if let Value::BigUint(ref mut big) = output { big.signed = true; } output }",
            ),
        ] {
            let replacement =
                replace_once(branch, "ret.expand(width, false).into_owned()", replacement);
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_unary", branch, &replacement),
            });
        }
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-reduction-and-width65")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        // Each mutation leaves the native result prefix unchanged and affects
        // only the BigUint result calls newly covered by this contract.
        for (name, expression) in [
            ("public-wide-reduction-and-as-nand", "x.reduce_and(true)"),
            ("public-wide-reduction-and-as-or", "x.reduce_or(false)"),
            (
                "public-wide-reduction-and-unknown-overrides-zero",
                "{ let mut result = x.reduce_and(false); if x.mask_xz != 0 { result.payload = 0; result.mask_xz = 1; } result }",
            ),
            (
                "public-wide-reduction-and-ignores-unknown",
                "{ let mut result = x.reduce_and(false); result.mask_xz = 0; result }",
            ),
            (
                "public-wide-reduction-and-upper-padding-leak",
                "{ let mut result = x.reduce_and(false); result.payload |= 1u64 << 63; result }",
            ),
        ] {
            let replacement = format!(
                "Value::U64(x) => if width >= 65 {{ {expression} }} else {{ x.reduce_and(false) }},"
            );
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(
                    &source,
                    "eval_value_unary",
                    "Value::U64(x) => x.reduce_and(false),",
                    &replacement,
                ),
            });
        }
        let method = &source[method_range(&source, "eval_value_unary")];
        let start = method.find("            Op::BitAnd => {").unwrap();
        let end = start
            + method[start..]
                .find("            Op::BitNand => {")
                .unwrap();
        let branch = &method[start..end];
        for (name, replacement) in [
            (
                "public-wide-reduction-and-wrong-width",
                "ret.expand(if width == 65 { 66 } else { width }, false).into_owned()",
            ),
            (
                "public-wide-reduction-and-signed-result",
                "{ let mut output = ret.expand(width, false).into_owned(); if let Value::BigUint(ref mut big) = output { big.signed = true; } output }",
            ),
        ] {
            let replacement =
                replace_once(branch, "ret.expand(width, false).into_owned()", replacement);
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, "eval_value_unary", branch, &replacement),
            });
        }
    }
    for (name, path, method, old, new) in [
        (
            "public-reduction-and-as-nand",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_unary",
            "Value::U64(x) => x.reduce_and(false),",
            "Value::U64(x) => x.reduce_and(true),",
        ),
        (
            "public-reduction-and-as-or",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_unary",
            "Value::U64(x) => x.reduce_and(false),",
            "Value::U64(x) => x.reduce_or(false),",
        ),
        (
            "public-reduction-and-ignores-zero",
            "veryl/crates/analyzer/src/value.rs",
            "reduce_and",
            "let known_zero = self.payload | self.mask_xz != mask;",
            "let known_zero = false;",
        ),
        (
            "public-reduction-and-ignores-unknown",
            "veryl/crates/analyzer/src/value.rs",
            "reduce_and",
            "Self::new_bit_0x(known_zero, self.is_xz())",
            "Self::new_bit_0x(known_zero, false)",
        ),
        (
            "public-reduction-and-signed-result",
            "veryl/crates/analyzer/src/value.rs",
            "reduce_and",
            "Self::new_bit_0x(known_zero, self.is_xz())",
            "Self { signed: true, ..Self::new_bit_0x(known_zero, self.is_xz()) }",
        ),
        (
            "public-reduction-and-upper-padding-leak",
            "veryl/crates/analyzer/src/value.rs",
            "reduce_and",
            "Self::new_bit_0x(known_zero, self.is_xz())",
            "{ let mut result = Self::new_bit_0x(known_zero, self.is_xz()); result.payload |= 1u64 << 63; result }",
        ),
    ] {
        if let Some(&contract) = selected
            .iter()
            .find(|c| c.name == "public-reduction-and-native-continuation")
        {
            let source = fs::read_to_string(repo.join(path)).unwrap();
            mutations.push(Mutation {
                name,
                contract,
                path,
                source: mutate_method(&source, method, old, new),
            });
        }
    }
    for (name, path, method, old, new) in [
        (
            "public-native-logical-and-as-or",
            "veryl/crates/analyzer/src/ir/op.rs",
            "eval_value_binary",
            "x.logical_binary(y, true)",
            "x.logical_binary(y, false)",
        ),
        (
            "public-native-logical-and-ignores-zero",
            "veryl/crates/analyzer/src/value.rs",
            "logical_binary",
            "!known_zero && unknown",
            "unknown",
        ),
        (
            "public-native-logical-and-Z-as-one",
            "veryl/crates/analyzer/src/value.rs",
            "logical_binary",
            "self.payload & !self.mask_xz != 0",
            "self.payload != 0",
        ),
        (
            "public-native-logical-and-ignores-unknown",
            "veryl/crates/analyzer/src/value.rs",
            "logical_binary",
            "let unknown = self.is_xz() || rhs.is_xz();",
            "let unknown = false;",
        ),
        (
            "public-native-logical-and-signed-result",
            "veryl/crates/analyzer/src/value.rs",
            "logical_binary",
            "Self::new_bit_1x(lhs_one && rhs_one, !known_zero && unknown)",
            "Self { signed: true, ..Self::new_bit_1x(lhs_one && rhs_one, !known_zero && unknown) }",
        ),
        (
            "public-native-logical-and-upper-padding-leak",
            "veryl/crates/analyzer/src/value.rs",
            "logical_binary",
            "Self::new_bit_1x(lhs_one && rhs_one, !known_zero && unknown)",
            "{ let mut result = Self::new_bit_1x(lhs_one && rhs_one, !known_zero && unknown); result.payload |= 1u64 << 63; result }",
        ),
    ] {
        if let Some(&contract) = selected
            .iter()
            .find(|c| c.name == "public-logical-and-mutation-witness")
        {
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
            "public-bitwise-widened-mutation-witness",
            "public-widened-bitwise-and-as-or",
            "eval_value_binary",
            "Value::U64(x.bit_and(y, width))",
            "Value::U64(if width == 4 { x.bit_or(y, width) } else { x.bit_and(y, width) })",
        ),
        (
            "public-bitwise-widened-mutation-witness",
            "public-widened-bitwise-or-as-and",
            "eval_value_binary",
            "Value::U64(x.bit_or(y, width))",
            "Value::U64(if width == 4 { x.bit_and(y, width) } else { x.bit_or(y, width) })",
        ),
        (
            "public-bitwise-widened-mutation-witness",
            "public-widened-bitwise-xor-as-xnor",
            "eval_value_binary",
            "Value::U64(x.bit_xor(y, width))",
            "Value::U64(if width == 4 { x.bit_xnor(y, width) } else { x.bit_xor(y, width) })",
        ),
        (
            "public-bitwise-widened-mutation-witness",
            "public-widened-bitwise-xnor-as-xor",
            "eval_value_binary",
            "Value::U64(x.bit_xnor(y, width))",
            "Value::U64(if width == 4 { x.bit_xor(y, width) } else { x.bit_xnor(y, width) })",
        ),
        (
            "public-bitwise-widened-mutation-witness",
            "public-widened-bitwise-and-disables-left-sign",
            "eval_value_binary",
            "let x = resize(x, width, signed);",
            "let x = resize(x, width, signed && width != 4);",
        ),
        (
            "public-bitwise-widened-mutation-witness",
            "public-widened-bitwise-and-upper-padding-leak",
            "eval_value_binary",
            "Value::U64(x.bit_and(y, width))",
            "{ let mut result = x.bit_and(y, width); if width == 4 { result.payload |= 1u64 << 63; } Value::U64(result) }",
        ),
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
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-shift-widening-4-to-8")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-shift-widening-left-shifts-before-sizing",
            contract,
            path,
            source: mutate_method(
                &source,
                "eval_value_binary",
                "Op::LogicShiftL => {\n                let x = x.expand(width, signed);",
                "Op::LogicShiftL => {\n                let x = x.expand(x.width(), signed);",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-shift-widening-4-to-64")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-shift-widening-right-disables-sign",
            contract,
            path,
            source: mutate_method(
                &source,
                "eval_value_binary",
                "Op::ArithShiftR => {\n                let x = x.expand(width, signed);",
                "Op::ArithShiftR => {\n                let x = x.expand(width, false);",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-shift-widening-4-to-8")
    {
        let path = "veryl/crates/analyzer/src/value.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-shift-widening-drops-X-padding",
            contract,
            path,
            source: mutate_method(
                &source,
                "expand",
                "if msb_xz {\n                    mask_xz |= mask;",
                "if msb_xz && msb {\n                    mask_xz |= mask;",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-big-amount-decoding-128")
    {
        let path = "veryl/crates/analyzer/src/value.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-big-amount-overflow-as-none",
            contract,
            path,
            source: mutate_method(
                &source,
                "to_usize_saturating",
                "Self::BigUint(x) => Some(x.payload.to_usize().unwrap_or(usize::MAX)),",
                "Self::BigUint(x) => x.payload.to_usize(),",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-big-amount-decoding-128")
    {
        let path = "veryl/crates/analyzer/src/value.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-big-amount-ignores-upper-XZ",
            contract,
            path,
            source: mutate_method(
                &source,
                "to_usize_saturating",
                "Self::BigUint(x) if x.is_xz() => None,",
                "Self::BigUint(x) if x.mask_xz.iter_u64_digits().next().unwrap_or(0) != 0 => None,",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-big-amount-decoding-128")
    {
        let path = "veryl/crates/analyzer/src/value.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-big-amount-rejects-wide-storage",
            contract,
            path,
            source: mutate_method(
                &source,
                "to_usize_saturating",
                "Self::BigUint(x) => Some(x.payload.to_usize().unwrap_or(usize::MAX)),",
                "Self::BigUint(_) => None,",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-word64-big-amount-small")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-word64-big-amount-caller-uses-strict-decoding",
            contract,
            path,
            source: mutate_method(&source, "eval_value_binary", "Op::LogicShiftL => {\n                let x = x.expand(width, signed);\n                let y = y.to_usize_saturating();", "Op::LogicShiftL => {\n                let x = x.expand(width, signed);\n                let y = y.to_usize();"),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-word64-big-amount-high")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-word64-big-amount-caller-truncates-high-word",
            contract,
            path,
            source: mutate_method(&source, "eval_value_binary", "Op::LogicShiftR => {\n                let x = x.expand(width, signed);\n                let y = y.to_usize_saturating();", "Op::LogicShiftR => {\n                let x = x.expand(width, signed);\n                let y = if y.mask_xz_u128() != 0 { None } else { Some(y.payload_u128() as usize) };"),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-word64-big-amount-upper-x")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-word64-big-amount-caller-ignores-XZ",
            contract,
            path,
            source: mutate_method(&source, "eval_value_binary", "Op::ArithShiftL => {\n                let x = x.expand(width, signed);\n                let y = y.to_usize_saturating();", "Op::ArithShiftL => {\n                let x = x.expand(width, signed);\n                let y = Some(usize::try_from(y.payload_u128()).unwrap_or(usize::MAX));"),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-word64-big-amount-upper-z")
    {
        let path = "veryl/crates/analyzer/src/value.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-word64-big-amount-Z-as-large-known",
            contract,
            path,
            source: mutate_method(
                &source,
                "to_usize_saturating",
                "Self::BigUint(x) if x.is_xz() => None,",
                "Self::BigUint(x) if x.is_xz() => Some(usize::MAX),",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-word32-big-amount-small")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-word32-big-amount-caller-uses-strict-decoding",
            contract,
            path,
            source: mutate_method(&source, "eval_value_binary", "Op::LogicShiftL => {\n                let x = x.expand(width, signed);\n                let y = y.to_usize_saturating();", "Op::LogicShiftL => {\n                let x = x.expand(width, signed);\n                let y = y.to_usize();"),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-word32-big-amount-high")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-word32-big-amount-caller-truncates-high-word",
            contract,
            path,
            source: mutate_method(&source, "eval_value_binary", "Op::LogicShiftR => {\n                let x = x.expand(width, signed);\n                let y = y.to_usize_saturating();", "Op::LogicShiftR => {\n                let x = x.expand(width, signed);\n                let y = if y.mask_xz_u128() != 0 { None } else { Some(y.payload_u128() as usize) };"),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-word32-big-amount-upper-x")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-word32-big-amount-caller-ignores-XZ",
            contract,
            path,
            source: mutate_method(&source, "eval_value_binary", "Op::ArithShiftL => {\n                let x = x.expand(width, signed);\n                let y = y.to_usize_saturating();", "Op::ArithShiftL => {\n                let x = x.expand(width, signed);\n                let y = Some(usize::try_from(y.payload_u128()).unwrap_or(usize::MAX));"),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-word32-big-amount-upper-z")
    {
        let path = "veryl/crates/analyzer/src/value.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-word32-big-amount-Z-as-large-known",
            contract,
            path,
            source: mutate_method(
                &source,
                "to_usize_saturating",
                "Self::BigUint(x) if x.is_xz() => None,",
                "Self::BigUint(x) if x.is_xz() => Some(usize::MAX),",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-word32-big-amount-high")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-word32-big-amount-forces-word64-result",
            contract,
            path,
            source: mutate_method(
                &source,
                "eval_value_binary",
                "let ret = Value::U64(x.shift_right(y, width, signed, false));\n                        ret.expand(width, false).into_owned()",
                "let ret = Value::U64(x.shift_right(y, width, signed, false));\n                        ret.expand(64, false).into_owned()",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-shift-big-amount-width-1-small")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-shift-big-amount-width-1-caller-uses-strict-decoding",
            contract,
            path,
            source: mutate_method(&source, "eval_value_binary", "Op::LogicShiftL => {\n                let x = x.expand(width, signed);\n                let y = y.to_usize_saturating();", "Op::LogicShiftL => {\n                let x = x.expand(width, signed);\n                let y = y.to_usize();"),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-shift-big-amount-width-2-high")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-shift-big-amount-width-2-caller-truncates-high-word",
            contract,
            path,
            source: mutate_method(&source, "eval_value_binary", "Op::LogicShiftR => {\n                let x = x.expand(width, signed);\n                let y = y.to_usize_saturating();", "Op::LogicShiftR => {\n                let x = x.expand(width, signed);\n                let y = if y.mask_xz_u128() != 0 { None } else { Some(y.payload_u128() as usize) };"),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-shift-big-amount-width-31-upper-x")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-shift-big-amount-width-31-caller-ignores-XZ",
            contract,
            path,
            source: mutate_method(&source, "eval_value_binary", "Op::ArithShiftL => {\n                let x = x.expand(width, signed);\n                let y = y.to_usize_saturating();", "Op::ArithShiftL => {\n                let x = x.expand(width, signed);\n                let y = Some(usize::try_from(y.payload_u128()).unwrap_or(usize::MAX));"),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-shift-big-amount-width-63-upper-z")
    {
        let path = "veryl/crates/analyzer/src/value.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-shift-big-amount-width-63-Z-as-large-known",
            contract,
            path,
            source: mutate_method(
                &source,
                "to_usize_saturating",
                "Self::BigUint(x) if x.is_xz() => None,",
                "Self::BigUint(x) if x.is_xz() => Some(usize::MAX),",
            ),
        });
    }
    if let Some(&contract) = selected
        .iter()
        .find(|c| c.name == "public-shift-big-amount-width-31-high")
    {
        let path = "veryl/crates/analyzer/src/ir/op.rs";
        let source = fs::read_to_string(repo.join(path)).unwrap();
        mutations.push(Mutation {
            name: "public-shift-big-amount-width-31-forces-word64-result",
            contract,
            path,
            source: mutate_method(
                &source,
                "eval_value_binary",
                "let ret = Value::U64(x.shift_right(y, width, signed, false));\n                        ret.expand(width, false).into_owned()",
                "let ret = Value::U64(x.shift_right(y, width, signed, false));\n                        ret.expand(64, false).into_owned()",
            ),
        });
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
            // -1 (one signed bit) % -2 (two signed bits) distinguishes the
            // missing LHS extension at width 2; wider positive proofs stay intact.
            "public-small-remainder-width-2-signed",
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

// Mutate only the BigUint arm in the selected operation. Model-domain and
// timeout failures are rejected by the shared runner, never counted as detection.
fn wide_operator_mutations(repo: &Path, selected: &[&'static Contract], out: &mut Vec<Mutation>) {
    let core = fs::read_to_string(repo.join(CORE)).unwrap();
    let op_path = "veryl/crates/analyzer/src/ir/op.rs";
    let ops = fs::read_to_string(repo.join(op_path)).unwrap();
    for &contract in selected {
        let mut add = |name, path, source| {
            out.push(Mutation {
                name,
                contract,
                path,
                source,
            })
        };
        match contract.name {
            "assignment256-high" => {
                add(
                    "assignment256-leaks-high-payload",
                    CORE,
                    replace_once(
                        &core,
                        "(value.payload() & &mask_range)",
                        "(if value.payload.bits() > 128 { value.payload().clone() } else { value.payload() & &mask_range })",
                    ),
                );
                add(
                    "assignment256-leaks-high-xz",
                    CORE,
                    replace_once(
                        &core,
                        "(value.mask_xz() & &mask_range)",
                        "(if value.mask_xz.bits() > 128 { value.mask_xz().clone() } else { value.mask_xz() & &mask_range })",
                    ),
                );
            }
            "assignment256-big-big" => {
                add(
                    "assignment256-loses-rhs-payload",
                    CORE,
                    replace_once(&core, "(value.payload() & &mask_range)", "BigUint::zero()"),
                );
                add(
                    "assignment256-loses-rhs-xz",
                    CORE,
                    replace_once(&core, "(value.mask_xz() & &mask_range)", "BigUint::zero()"),
                );
            }
            _ => {}
        }
        let binary = ops.find("    pub fn eval_value_binary(").unwrap();
        let unary = ops.find("    pub fn eval_value_unary(").unwrap();
        let spec = match contract.name {
            "bitwise256-and" => Some((
                binary,
                "Op::BitAnd =>",
                "*ret.payload &= ret.mask_xz() ^ mask;",
                "*ret.payload &= mask;",
                "bitwise256-and-retains-unknown-payload",
            )),
            "bitwise256-or" => Some((
                binary,
                "Op::BitOr =>",
                "*ret.payload &= ret.mask_xz() ^ mask;",
                "*ret.payload &= mask;",
                "bitwise256-or-retains-unknown-payload",
            )),
            "bitwise256-xor" => Some((
                binary,
                "Op::BitXor =>",
                "*ret.mask_xz = x.mask_xz() | y.mask_xz();",
                "*ret.mask_xz = x.mask_xz() & y.mask_xz();",
                "bitwise256-xor-loses-one-sided-unknown",
            )),
            "bitwise256-xnor" => Some((
                binary,
                "Op::BitXnor =>",
                "let payload = x.payload() ^ y.payload() ^ mask;",
                "let payload = x.payload() ^ y.payload();",
                "bitwise256-xnor-skips-inversion",
            )),
            "not256" => Some((
                unary,
                "Op::BitNot =>",
                "*ret.payload &= ret.mask_xz() ^ mask;",
                "*ret.payload &= mask;",
                "not256-retains-unknown-payload",
            )),
            "shift256-left" => Some((
                binary,
                "Op::LogicShiftL =>",
                "*ret.payload &= mask;",
                "let _ = &mask;",
                "shift256-left-leaks-upper-bits",
            )),
            "shift256-arith-left" => Some((
                binary,
                "Op::ArithShiftL =>",
                "*ret.mask_xz &= mask;",
                "let _ = &mask;",
                "shift256-arith-left-leaks-upper-xz",
            )),
            "shift256-right" => Some((
                binary,
                "Op::LogicShiftR =>",
                "*ret.mask_xz >>= y;",
                "let _ = y;",
                "shift256-right-leaves-xz-unshifted",
            )),
            "shift256-arith-right-unsigned" => Some((
                binary,
                "Op::ArithShiftR =>",
                "let mut ret = x.clone();",
                "let mut ret = x.clone(); ret.signed = true;",
                "shift256-arith-right-unsigned-retains-sign",
            )),
            "shift256-arith-right-signed" => Some((
                binary,
                "Op::ArithShiftR =>",
                "*ret.mask_xz |= ext_mask_xz;",
                "let _ = ext_mask_xz;",
                "shift256-arith-right-loses-xz-fill",
            )),
            "reduction256-and" => Some((
                unary,
                "Op::BitAnd =>",
                "ValueU64::new_bit_0x(is_zero, is_x)",
                "ValueU64::new_bit_0x(false, is_x)",
                "reduction256-and-loses-zero-dominance",
            )),
            "reduction256-nand" => Some((
                unary,
                "Op::BitNand =>",
                "ValueU64::new_bit_1x(is_one, is_x)",
                "ValueU64::new_bit_0x(is_one, is_x)",
                "reduction256-nand-skips-inversion",
            )),
            "reduction256-or" => Some((
                unary,
                "Op::BitOr =>",
                "ValueU64::new_bit_1x(is_one, is_x)",
                "ValueU64::new_bit_1x(false, is_x)",
                "reduction256-or-loses-one-dominance",
            )),
            "reduction256-nor" => Some((
                unary,
                "Op::BitNor | Op::LogicNot =>",
                "ValueU64::new_bit_0x(is_zero, is_x)",
                "ValueU64::new_bit_1x(is_zero, is_x)",
                "reduction256-nor-skips-inversion",
            )),
            "reduction256-logic-not" => Some((
                unary,
                "Op::BitNor | Op::LogicNot =>",
                "ValueU64::new_bit_0x(is_zero, is_x)",
                "ValueU64::new_bit_1x(is_zero, is_x)",
                "reduction256-logic-not-skips-inversion",
            )),
            "reduction256-xor" => Some((
                unary,
                "Op::BitXor =>",
                "x.payload.count_ones() % 2 == 1",
                "x.payload.count_ones() % 2 == 0",
                "reduction256-xor-inverts-parity",
            )),
            "reduction256-xnor" => Some((
                unary,
                "Op::BitXnor =>",
                "x.payload.count_ones() % 2 == 0",
                "x.payload.count_ones() % 2 == 1",
                "reduction256-xnor-inverts-parity",
            )),
            _ => None,
        };
        if let Some((start, arm, old, new, name)) = spec {
            let a = start + ops[start..].find(arm).unwrap();
            let b = a + ops[a..].find("Value::BigUint(x)").unwrap();
            let end = b + ops[b..].find(old).unwrap() + old.len();
            let mut source = ops.clone();
            source.replace_range(b..end, &replace_once(&ops[b..end], old, new));
            add(name, op_path, source);
            if contract.name == "shift256-left" {
                let mut source = ops.clone();
                source.replace_range(b..end,&replace_once(&ops[b..end],old,
                    "if !(width == 128 && y == 128 && x.payload.bits() == 128) { *ret.payload &= mask; }"));
                add("shift256-left-leaks-bit255", op_path, source);
            }
        }
    }
}
