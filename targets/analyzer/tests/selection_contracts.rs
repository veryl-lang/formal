use veryl_formal_runner::Contract;

pub static CONTRACTS: [Contract; 6] = [
    Contract {
        name: "selection-storage",
        group: "selection-layout",
        harness: "selection_storage_boundaries",
        message: "selection layout semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "selection-geometry",
        group: "selection-layout",
        harness: "selection_small_coordinate_geometry",
        message: "selection layout semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "selection-limits",
        group: "selection-layout",
        harness: "selection_limits_and_scalar_offsets",
        message: "selection layout semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "selection-rebased-storage",
        group: "selection-layout",
        harness: "rebased_selection_rejects_empty_storage_and_missing_range_base",
        message: "rebased selection storage semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "selection-metadata",
        group: "selection-layout",
        harness: "selection_metadata_widths_do_not_wrap",
        message: "selection metadata semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "selection-unpacked-storage",
        group: "selection-layout",
        harness: "unpacked_selection_rejects_empty_or_unrepresentable_storage",
        message: "unpacked selection storage semantics mismatch",
        covers: &[],
    },
];
