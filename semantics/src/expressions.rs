//! Finite integral expression grammar using IEEE 1800-2023 6.24.1, 11.6.1,
//! and 11.8.2: context reaches arithmetic/conditional operands, but stops at
//! numeric casts, comparison operands, reduction inputs, and shift amounts.
//! Leaves have resolved integral types. No effects, unsized literals, floats,
//! concatenation, or arbitrary source-language typing are modeled here.
//! TypedCast represents i32/u32: 6.11.2 and 6.24.1 convert X/Z to zero
//! after sizing to the target, before any enclosing expression is evaluated.

use crate::arithmetic::{Arithmetic, arithmetic_planes};
use crate::comparison::{Comparison, Operand, compared_logic};
use crate::conditional::conditional_logic_bit;
use crate::gates::{BitOp, binary, logical_truth};
use crate::logic4::read_logic;
use crate::resize::{extended_logic_bit, reference_planes};

#[derive(Clone, Debug)]
pub enum Expr {
    Input(usize),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Xor(Box<Expr>, Box<Expr>),
    Less(Box<Expr>, Box<Expr>),
    Shift(Box<Expr>, Box<Expr>),
    Reduce(Box<Expr>),
    Cast(Box<Expr>, usize),
    TypedCast(Box<Expr>, bool),
    Choose(Box<Expr>, Box<Expr>, Box<Expr>),
}

impl Expr {
    pub fn shape(&self, inputs: &[Operand]) -> (usize, bool) {
        match self {
            Self::Input(i) => (inputs[*i].width, inputs[*i].signed),
            Self::Less(..) | Self::Reduce(..) => (1, false),
            Self::Shift(a, _) => a.shape(inputs),
            Self::Cast(a, width) => (*width, a.shape(inputs).1),
            Self::TypedCast(_, signed) => (32, *signed),
            Self::Add(a, b) | Self::Sub(a, b) | Self::Xor(a, b) | Self::Choose(_, a, b) => {
                let (aw, sa) = a.shape(inputs);
                let (bw, sb) = b.shape(inputs);
                (aw.max(bw), sa && sb)
            }
        }
    }

    pub fn evaluate(&self, inputs: &[Operand], width: usize) -> Operand {
        assert!((1..128).contains(&width));
        let shape = self.shape(inputs);
        self.in_context(inputs, width.max(shape.0), shape.1)
    }

    fn in_context(&self, inputs: &[Operand], width: usize, signed: bool) -> Operand {
        let value = |planes| Operand {
            planes,
            width,
            signed,
        };
        let own = |x: &Self| {
            let (width, signed) = x.shape(inputs);
            x.in_context(inputs, width, signed)
        };
        match self {
            Self::Input(i) => resize(inputs[*i], width, signed),
            Self::Cast(a, _) | Self::TypedCast(a, _) => {
                let (target, target_signed) = self.shape(inputs);
                let (natural, source_signed) = a.shape(inputs);
                let operand = a.in_context(inputs, natural.max(target), source_signed);
                let mut cast = resize(operand, target, source_signed);
                if matches!(self, Self::TypedCast(..)) {
                    cast.planes.0 &= !cast.planes.1;
                    cast.planes.1 = 0;
                }
                let cast = Operand {
                    signed: target_signed,
                    ..cast
                };
                resize(cast, width, signed)
            }
            Self::Reduce(a) => {
                let a = own(a);
                let truth = logical_truth(a.planes, a.width);
                resize(
                    Operand {
                        planes: reference_planes(1, |_| truth),
                        width: 1,
                        signed: false,
                    },
                    width,
                    false,
                )
            }
            Self::Less(a, b) => {
                let (aw, sa) = a.shape(inputs);
                let (bw, sb) = b.shape(inputs);
                let a = a.in_context(inputs, aw.max(bw), sa && sb);
                let b = b.in_context(inputs, aw.max(bw), sa && sb);
                let bit = compared_logic(a, b, Comparison::Less);
                resize(
                    Operand {
                        planes: reference_planes(1, |_| bit),
                        width: 1,
                        signed: false,
                    },
                    width,
                    false,
                )
            }
            Self::Choose(c, a, b) => {
                let c = own(c);
                let a = a.in_context(inputs, width, signed);
                let b = b.in_context(inputs, width, signed);
                value(reference_planes(width, |p| {
                    conditional_logic_bit(
                        logical_truth(c.planes, c.width),
                        (a.planes, a.width, a.signed),
                        (b.planes, b.width, b.signed),
                        width,
                        signed,
                        p,
                    )
                }))
            }
            Self::Shift(a, b) => {
                let a = a.in_context(inputs, width, signed);
                let b = own(b);
                let planes = if b.planes.1 != 0 {
                    (0, (1u128 << width) - 1)
                } else {
                    let shift = b.planes.0;
                    reference_planes(width, |p| {
                        if shift <= p as u128 {
                            read_logic(a.planes.0, a.planes.1, p - shift as usize)
                        } else {
                            crate::logic4::Logic4::Zero
                        }
                    })
                };
                value(planes)
            }
            Self::Add(a, b) | Self::Sub(a, b) | Self::Xor(a, b) => {
                let a = a.in_context(inputs, width, signed);
                let b = b.in_context(inputs, width, signed);
                value(match self {
                    Self::Add(..) => arithmetic_planes(a.planes, b.planes, width, Arithmetic::Add),
                    Self::Sub(..) => {
                        arithmetic_planes(a.planes, b.planes, width, Arithmetic::Subtract)
                    }
                    _ => reference_planes(width, |p| {
                        binary(
                            read_logic(a.planes.0, a.planes.1, p),
                            read_logic(b.planes.0, b.planes.1, p),
                            BitOp::Xor,
                        )
                    }),
                })
            }
        }
    }
}

fn resize(a: Operand, width: usize, signed: bool) -> Operand {
    Operand {
        planes: reference_planes(width, |p| {
            extended_logic_bit(a.planes, a.width, width, a.signed, signed, p)
        }),
        width,
        signed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cast_boundary_prevents_outer_width_from_recovering_carry() {
        let inputs = [
            Operand {
                planes: (3, 0),
                width: 2,
                signed: false,
            },
            Operand {
                planes: (1, 0),
                width: 2,
                signed: false,
            },
        ];
        let sum = Expr::Add(Box::new(Expr::Input(0)), Box::new(Expr::Input(1)));
        assert_eq!(sum.evaluate(&inputs, 3).planes, (4, 0));
        assert_eq!(
            Expr::Cast(Box::new(sum), 2).evaluate(&inputs, 3).planes,
            (0, 0)
        );
    }
    #[test]
    fn signed_cast_extends_input_before_changing_result_sign() {
        let inputs = [Operand {
            planes: (3, 0),
            width: 2,
            signed: true,
        }];
        let cast = Expr::TypedCast(Box::new(Expr::Input(0)), false);
        assert_eq!(cast.evaluate(&inputs, 33).planes, (u32::MAX as u128, 0));
    }

    #[test]
    fn typed_cast_clears_xz_after_sizing() {
        for (planes, source_signed, target_signed, expected) in [
            ((1, 1), false, false, 0),
            ((0, 1), false, true, 0),
            ((13, 9), true, false, 4),
            ((13, 1), true, false, 0xffff_fffc),
            ((13, 1), true, true, 0x1_ffff_fffc),
        ] {
            let inputs = [Operand {
                planes,
                width: 4,
                signed: source_signed,
            }];
            let cast = Expr::TypedCast(Box::new(Expr::Input(0)), target_signed);
            assert_eq!(cast.evaluate(&inputs, 33).planes, (expected, 0));
        }
        let inputs = [Operand {
            planes: (1, 1),
            width: 4,
            signed: false,
        }];
        assert_eq!(
            Expr::Cast(Box::new(Expr::Input(0)), 32)
                .evaluate(&inputs, 33)
                .planes,
            (1, 1)
        );
    }
}
