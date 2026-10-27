//! Finite emitted-SV execution compared with independent widened arithmetic for
//! selected precedence, cast, width, and signedness fixtures. Icarus runs generated
//! SV at test time; analyzer evaluation is not the expected-value oracle.

use super::common;

#[test]
fn emitted_precedence_casts_and_widths() {
    check_integer_cast_clears_unknown_bits();
    let mut comparisons = 0;
    for width in 1..=3usize {
        for signed in [false, true] {
            let sign = if signed { "signed " } else { "" };
            let wide = width * 2;
            let source = format!(
                "module Top(a:input {sign}logic<{width}>,b:input {sign}logic<{width}>,c:input {sign}logic<{width}>,priority_value:output logic<{wide}>,grouped_value:output logic<{wide}>,cast_sum:output logic<8>,cast_product:output logic<8>,cast_negative:output logic,not_sum:output logic<{width}>,shift_sum:output logic<{width}>){{assign priority_value = a + b * c;assign grouped_value = (a + b) * c;assign cast_sum = (a + b) as 8;assign cast_product = (a as 8) * b;assign cast_negative = (a as i32) <: 0;assign not_sum = ~(a+b);assign shift_sum = a << (b+c);}}"
            );
            let mut bench = format!(
                "module testbench;logic {sign}[{}:0] a,b,c;wire [{}:0] priority_value,grouped_value;wire [7:0] cast_sum,cast_product;wire cast_negative;wire [{}:0] not_sum,shift_sum;prj_Top dut(.*);initial begin\n",
                width - 1,
                wide - 1,
                width - 1
            );
            let mut expected = Vec::new();
            let mask = (1u64 << width) - 1;
            let decode = |n: u64| {
                if signed && n & (1 << (width - 1)) != 0 {
                    n as i128 - (1 << width)
                } else {
                    n as i128
                }
            };
            for a in 0..=mask {
                for b in 0..=mask {
                    for c in 0..=mask {
                        bench += &format!(
                            "a={width}'d{a};b={width}'d{b};c={width}'d{c};#1;$display(\"case %b %b %b %b %b %b %b\",priority_value,grouped_value,cast_sum,cast_product,cast_negative,not_sum,shift_sum);\n"
                        );
                        let sa = decode(a);
                        let sb = decode(b);
                        let sc = decode(c);
                        let results = [
                            (sa + sb * sc) as u64,
                            (sa + sb).wrapping_mul(sc) as u64,
                            (sa + sb) as u64,
                            (sa * sb) as u64,
                            u64::from(sa < 0),
                            (!(sa + sb)) as u64,
                            a.checked_shl(((b + c) & mask) as u32).unwrap_or(0),
                        ];
                        let widths = [wide, wide, 8, 8, 1, width, width];
                        expected.push(
                            results
                                .into_iter()
                                .zip(widths)
                                .map(|(n, w)| format!("{:0w$b}", n & ((1 << w) - 1), w = w))
                                .collect::<Vec<_>>()
                                .join(" "),
                        );
                    }
                }
            }
            bench += "$display(\"complete\");$finish;end endmodule\n";
            let got = common::execute(&source, &bench);
            assert_eq!(got.len(), expected.len(), "SV vector count changed");
            for (i, (got, want)) in got.iter().zip(expected).enumerate() {
                assert_eq!(
                    *got, want,
                    "emitter scalar output mismatch: width={width} signed={signed} case={i}"
                );
                comparisons += 7;
            }
        }
    }
    eprintln!("emitter scalar complete configurations=6 comparisons={comparisons}");
}

// IEEE 1800-2023 6.11.2 and 6.24.1: casting to int clears X/Z after sizing.
fn check_integer_cast_clears_unknown_bits() {
    for signed in [false, true] {
        let modifier = if signed { "signed " } else { "" };
        let source = format!(
            "module Top(a:input {modifier}logic<4>,y:output logic<32>){{assign y=a as i32;}}"
        );
        let vectors = ["0000", "1111", "z101", "x101", "1z0x", "zzzz", "xxxx"];
        let mut bench =
            "module testbench;logic [3:0] a;wire [31:0] y;prj_Top dut(.*);initial begin\n"
                .to_owned();
        let mut expected = Vec::new();
        for digits in vectors {
            bench += &format!("a=4'b{digits};#1;$display(\"case %b\",y);\n");
            let fill = if signed && digits.starts_with('1') {
                '1'
            } else {
                '0'
            };
            let known: String = digits
                .chars()
                .map(|c| if c == '1' { '1' } else { '0' })
                .collect();
            expected.push(format!("{}{known}", fill.to_string().repeat(28)));
        }
        bench += "$display(\"complete\");$finish;end endmodule\n";
        assert_eq!(common::execute(&source, &bench), expected);
    }
}
