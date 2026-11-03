use veryl_formal_runner::Contract;

pub static CONTRACTS: [Contract; 4] = [
    Contract {
        name: "aggregate-width",
        group: "aggregate",
        harness: "struct_width_native_boundaries",
        message: "aggregate width semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "aggregate-spans",
        group: "aggregate",
        harness: "flat_struct_and_union_spans",
        message: "aggregate member span semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "aggregate-nested",
        group: "aggregate",
        harness: "nested_struct_union_packed_and_unpacked_coordinates",
        message: "aggregate nested coordinate semantics mismatch",
        covers: &[],
    },
    Contract {
        name: "aggregate-invalid-storage",
        group: "aggregate",
        harness: "aggregate_invalid_storage_does_not_wrap",
        message: "aggregate invalid storage semantics mismatch",
        covers: &[],
    },
];
