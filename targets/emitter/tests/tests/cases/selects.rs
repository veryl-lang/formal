//! Finite emitted-SV reads with independent per-bit clipping and X/Z expectations.
//! Icarus executes generated SV at test time. Neither simulator acceptance nor
//! these selected values establish general standards conformance.

use super::common;

fn text(bits: &[char]) -> String {
    bits.iter().rev().collect()
}
fn expected(data: &[char], start: Option<i128>, window: usize) -> String {
    (0..window)
        .rev()
        .map(|b| {
            start
                .and_then(|n| usize::try_from(n + b as i128).ok())
                .and_then(|i| data.get(i))
                .copied()
                .unwrap_or('x')
        })
        .collect()
}
#[test]
fn emitted_indexed_selects_preserve_xz_and_boundaries() {
    let mut comparisons = 0;
    let mut configs = 0;
    for width in [8usize, 17, 65] {
        for window in [3usize, 8] {
            for signed in [false, true] {
                let sign = if signed { "signed " } else { "" };
                let source = format!(
                    "module Top(data:input logic<{width}>,index:input {sign}logic<8>,ascending:output logic<{window}>,descending:output logic<{window}>,stepped:output logic<{window}>,mirror:output logic<{width}>){{assign ascending = data[index+:{window}];assign descending = data[index-:{window}];assign stepped = data[index step {window}];assign mirror = data;}}"
                );
                let mut bench = format!(
                    "module testbench;logic [{}:0] data;logic {sign}[7:0] index;wire [{}:0] ascending,descending,stepped;wire [{}:0] mirror;prj_Top dut(.*);initial begin\n",
                    width - 1,
                    window - 1,
                    width - 1
                );
                let min = if signed { -128 } else { 0 };
                let max = if signed { 127 } else { 255 };
                let mut numbers = vec![
                    min,
                    -8,
                    -3,
                    -1,
                    0,
                    1,
                    width as i128 - 1,
                    width as i128,
                    width as i128 + 1,
                    127,
                    128,
                    255,
                ];
                numbers.retain(|&n| n >= min && n <= max);
                numbers.sort();
                numbers.dedup();
                let mut cases: Vec<_> = numbers
                    .into_iter()
                    .map(|n| (format!("{:08b}", n as u8), Some(n)))
                    .collect();
                for base in [0, width - 1, 255] {
                    for bit in [0usize, 3, 7] {
                        for state in ['x', 'z'] {
                            let mut digits: Vec<_> = format!("{base:08b}").chars().collect();
                            digits[7 - bit] = state;
                            cases.push((digits.into_iter().collect(), None));
                        }
                    }
                }
                let mut want = Vec::new();
                for mode in 0..4 {
                    let data: Vec<_> = (0..width)
                        .map(|b| match mode {
                            0 => '0',
                            1 => '1',
                            2 => ['0', '1', 'x', 'z'][(b + b / 32) % 4],
                            _ => ['z', 'x', '1', '0'][(b + b / 32) % 4],
                        })
                        .collect();
                    for (index, n) in &cases {
                        bench += &format!(
                            "data={width}'b{};index=8'b{index};#1;$display(\"case %b %b %b %b\",ascending,descending,stepped,mirror);\n",
                            text(&data)
                        );
                        want.push(
                            [
                                expected(&data, *n, window),
                                expected(&data, n.map(|n| n - window as i128 + 1), window),
                                expected(&data, n.map(|n| n * window as i128), window),
                                text(&data),
                            ]
                            .join(" "),
                        );
                    }
                }
                bench += "$display(\"complete\");$finish;end endmodule\n";
                let got = common::execute(&source, &bench);
                assert_eq!(got.len(), want.len(), "SV vector count changed");
                for (i, (got, want)) in got.iter().zip(want).enumerate() {
                    assert_eq!(
                        *got, want,
                        "emitter selection output mismatch: width={width} window={window} signed={signed} case={i}"
                    );
                    comparisons += 4;
                }
                configs += 1;
            }
        }
    }
    eprintln!("emitter select complete configurations={configs} comparisons={comparisons}");
}
