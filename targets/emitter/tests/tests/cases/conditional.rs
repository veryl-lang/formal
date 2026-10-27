//! The analyzer is compared against IEEE 1800-2023 Table 11-20 for every case.
//! Icarus is a secondary check on the subset without ambiguous common Z: it
//! disagrees with the normative Z/Z -> X entry. Those vectors still exercise
//! the analyzer and reference; simulator behavior never defines the oracle.

use super::common;
use veryl_analyzer::value::Value;
use veryl_formal_semantics::conditional::merged_logic;
use veryl_formal_semantics::logic4::Logic4;
fn bits(mut state: usize, width: usize) -> String {
    (0..width)
        .map(|_| {
            let c = ['0', '1', 'x', 'z'][state % 4];
            state /= 4;
            c
        })
        .collect()
}
fn logic(c: char) -> Logic4 {
    match c {
        '0' => Logic4::Zero,
        '1' => Logic4::One,
        'x' => Logic4::X,
        'z' => Logic4::Z,
        _ => unreachable!(),
    }
}
fn planes(text: &str) -> (u128, u128) {
    let mut p = 0;
    let mut m = 0;
    for c in text.chars() {
        p = p * 2 + u128::from(matches!(c, '1' | 'z'));
        m = m * 2 + u128::from(matches!(c, 'x' | 'z'));
    }
    (p, m)
}
#[test]
fn conditional_four_state_consensus_matches_external_sv_and_values() {
    let mut comparisons = 0;
    let mut excluded_external = 0;
    for width in [1, 2, 65] {
        let source = format!(
            "module Top(c:input logic<{width}>,a:input logic<{width}>,b:input logic<{width}>,y:output logic<{width}>){{assign y=if |c ? a : b;}}"
        );
        let patterns: Vec<String> = if width <= 2 {
            (0..4usize.pow(width as u32))
                .map(|s| bits(s, width))
                .collect()
        } else {
            ['0', '1', 'x', 'z']
                .into_iter()
                .map(|c| c.to_string().repeat(width))
                .chain([
                    "10xz".repeat(17)[..width].to_owned(),
                    "z1x0".repeat(17)[..width].to_owned(),
                ])
                .collect()
        };
        let mut bench = format!(
            "module testbench;logic [{}:0] c,a,b;wire [{}:0] y;prj_Top dut(.*);initial begin\n",
            width - 1,
            width - 1
        );
        let mut expected = Vec::new();
        for c in &patterns {
            for a in &patterns {
                for b in &patterns {
                    let truth = if c.contains('1') {
                        Logic4::One
                    } else if c.contains(['x', 'z']) {
                        Logic4::X
                    } else {
                        Logic4::Zero
                    };
                    let want: String = a
                        .chars()
                        .zip(b.chars())
                        .map(|(a, b)| match merged_logic(truth, logic(a), logic(b)) {
                            Logic4::Zero => '0',
                            Logic4::One => '1',
                            Logic4::X => 'x',
                            Logic4::Z => 'z',
                        })
                        .collect();
                    let val = |s: &str| {
                        let (p, m) = planes(s);
                        Value::from_u128(p, m, width, false)
                    };
                    let got = val(c).conditional(&val(a), &val(b), width, false);
                    assert_eq!(
                        (got.payload_u128(), got.mask_xz_u128()),
                        planes(&want),
                        "external conditional output mismatch: analyzer width={width}"
                    );
                    if matches!(truth, Logic4::X | Logic4::Z)
                        && a.chars().zip(b.chars()).any(|(a, b)| a == 'z' && b == 'z')
                    {
                        excluded_external += 1;
                        continue;
                    }
                    bench += &format!(
                        "c={width}'b{c};a={width}'b{a};b={width}'b{b};#1;$display(\"case %b\",y);\n"
                    );
                    expected.push(want);
                }
            }
        }
        bench += "$display(\"complete\");$finish;end endmodule\n";
        let got = common::execute(&source, &bench);
        assert_eq!(
            got.len(),
            expected.len(),
            "external conditional vector count"
        );
        for (got, want) in got.iter().zip(&expected) {
            assert_eq!(
                got, want,
                "external conditional output mismatch: SV width={width}"
            );
        }
        comparisons += expected.len();
    }
    assert!(excluded_external > 0);
    eprintln!(
        "conditional external complete comparisons={comparisons} unsupported_icarus_vectors={excluded_external}"
    );
}
