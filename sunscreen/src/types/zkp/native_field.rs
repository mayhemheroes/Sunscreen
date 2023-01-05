use std::marker::PhantomData;

use subtle::{Choice, ConditionallySelectable};
use sunscreen_compiler_macros::TypeName;
use sunscreen_runtime::ZkpProgramInputTrait;
use sunscreen_zkp_backend::{BackendField, BigInt};

use crate::{
    invoke_gadget,
    types::zkp::{AddVar, ProgramNode},
    zkp::{with_zkp_ctx, ZkpContextOps},
};

use crate::types::zkp::{
    gadgets::ToUInt, ConstrainEqVarVar, IntoProgramNode, MulVar, NegVar, NumFieldElements,
    ToNativeFields, ZkpType,
};

use crate as sunscreen;

// Shouldn't need Clone + Copy, but there appears to be a bug in the Rust
// compiler that prevents ProgramNode from being Copy if we don't.
// https://github.com/rust-lang/rust/issues/104264
#[derive(Debug, Clone, TypeName)]
/**
 * The native field type in the underlying backend proof system. For
 * example, in Bulletproofs, this is [`Scalar`](https://docs.rs/curve25519-dalek-ng/4.1.1/curve25519_dalek_ng/scalar/struct.Scalar.html).
 */
pub struct NativeField<F: BackendField> {
    /**
     * The native field's value.
     */
    pub val: BigInt,

    _phantom: PhantomData<F>,
}

// Can't #[derive()] due to PhantomData.
impl<F: BackendField> Copy for NativeField<F> {}

impl<F: BackendField> NativeField<F> {
    /**
     * Converts a big-endian hex string into a native field.
     */
    pub fn from_be_hex(hex_str: &str) -> Self {
        Self {
            val: BigInt::from_be_hex(hex_str),
            _phantom: PhantomData,
        }
    }
}

impl<F: BackendField> From<u8> for NativeField<F> {
    fn from(x: u8) -> Self {
        (u64::from(x)).into()
    }
}

impl<F: BackendField> From<u16> for NativeField<F> {
    fn from(x: u16) -> Self {
        (u64::from(x)).into()
    }
}

impl<F: BackendField> From<u32> for NativeField<F> {
    fn from(x: u32) -> Self {
        (u64::from(x)).into()
    }
}

impl<F: BackendField> From<u64> for NativeField<F> {
    fn from(x: u64) -> Self {
        assert!(F::FIELD_MODULUS != BigInt::ZERO);

        // unwrap is okay here as we've ensured FIELD_MODULUS is
        // non-zero.
        Self {
            val: BigInt::from(BigInt::from(x).reduce(&F::FIELD_MODULUS).unwrap()),
            _phantom: PhantomData,
        }
    }
}

impl<F: BackendField> From<i8> for NativeField<F> {
    fn from(x: i8) -> Self {
        (i64::from(x)).into()
    }
}

impl<F: BackendField> From<i16> for NativeField<F> {
    fn from(x: i16) -> Self {
        (i64::from(x)).into()
    }
}

impl<F: BackendField> From<i32> for NativeField<F> {
    fn from(x: i32) -> Self {
        (i64::from(x)).into()
    }
}

impl<F: BackendField> From<i64> for NativeField<F> {
    fn from(x: i64) -> Self {
        assert!(F::FIELD_MODULUS != BigInt::ZERO);

        // Shr on i64 is an arithmetic shift, so we need to mask
        // the LSB so we don't get 255 for negative values.
        let is_negative = Choice::from(((x >> 63) & 0x1) as u8);

        let abs_val = BigInt::from(i64::conditional_select(&x, &-x, is_negative) as u64);

        // unwrap is okay here as we've ensured FIELD_MODULUS is
        // non-zero.
        let abs_val = BigInt::from(abs_val.reduce(&F::FIELD_MODULUS).unwrap());

        let neg = BigInt::from(F::FIELD_MODULUS.wrapping_sub(&abs_val));

        Self {
            val: BigInt::conditional_select(&abs_val, &neg, is_negative),
            _phantom: PhantomData,
        }
    }
}

impl<F: BackendField> NumFieldElements for NativeField<F> {
    const NUM_NATIVE_FIELD_ELEMENTS: usize = 1;
}

impl<F: BackendField> ToNativeFields for NativeField<F> {
    fn to_native_fields(&self) -> Vec<BigInt> {
        vec![self.val]
    }
}

impl<F: BackendField> ZkpType for NativeField<F> {}
impl<F: BackendField> ZkpProgramInputTrait for NativeField<F> {}

impl<F: BackendField> AddVar for NativeField<F> {
    fn add(lhs: ProgramNode<Self>, rhs: ProgramNode<Self>) -> ProgramNode<Self> {
        with_zkp_ctx(|ctx| {
            let o = ctx.add_addition(lhs.ids[0], rhs.ids[0]);

            ProgramNode::new(&[o])
        })
    }
}

impl<F: BackendField> MulVar for NativeField<F> {
    fn mul(lhs: ProgramNode<Self>, rhs: ProgramNode<Self>) -> ProgramNode<Self> {
        with_zkp_ctx(|ctx| {
            let o = ctx.add_multiplication(lhs.ids[0], rhs.ids[0]);

            ProgramNode::new(&[o])
        })
    }
}

impl<F: BackendField> NegVar for NativeField<F> {
    fn neg(lhs: ProgramNode<Self>) -> ProgramNode<Self> {
        with_zkp_ctx(|ctx| {
            let o = ctx.add_negate(lhs.ids[0]);

            ProgramNode::new(&[o])
        })
    }
}

impl<F: BackendField> ConstrainEqVarVar for NativeField<F> {
    fn constraint_eq(lhs: ProgramNode<Self>, rhs: ProgramNode<Self>) -> ProgramNode<Self> {
        with_zkp_ctx(|ctx| {
            let sub = ctx.add_subtraction(lhs.ids[0], rhs.ids[0]);

            let constraint = ctx.add_constraint(sub, &BigInt::ZERO);

            ProgramNode::new(&[constraint])
        })
    }
}

impl<F: BackendField> IntoProgramNode for NativeField<F> {
    type Output = NativeField<F>;

    fn into_program_node(self) -> ProgramNode<Self> {
        with_zkp_ctx(|ctx| ProgramNode::new(&[ctx.add_constant(&self.val)]))
    }
}

/**
 * Methods for decomposing values into binary.
 */
pub trait ToBinary<F: BackendField> {
    /**
     * Decompose this value into unsigned N-bit binary. If the value
     * is too large, the proof will fail to validate.
     */
    fn to_unsigned<const N: usize>(&self) -> [ProgramNode<NativeField<F>>; N];
}

impl<F: BackendField> ToBinary<F> for ProgramNode<NativeField<F>> {
    fn to_unsigned<const N: usize>(&self) -> [ProgramNode<NativeField<F>>; N] {
        let bits = invoke_gadget(ToUInt::<N>, self.ids);

        let mut vals = [*self; N];

        for (i, bit) in bits.iter().enumerate() {
            vals[i] = Self::new(&[*bit]);
        }

        vals
    }
}

#[cfg(test)]
mod tests {
    use std::ops::{Add, Mul, Neg, Sub};

    use curve25519_dalek::scalar::Scalar;
    use sunscreen_zkp_backend::ZkpInto;

    use super::*;

    #[test]
    fn can_encode_negative_number() {
        let x = NativeField::<Scalar>::from(-1);

        assert_eq!(
            x.val,
            BigInt::from(Scalar::FIELD_MODULUS.wrapping_sub(&BigInt::ONE))
        );

        let x = NativeField::<Scalar>::from(1i64);

        assert_eq!(x.val, BigInt::ONE);
    }

    #[derive(Copy, Clone, PartialEq, Eq)]
    struct TestField;

    impl Neg for TestField {
        type Output = Self;

        fn neg(self) -> Self::Output {
            unreachable!()
        }
    }

    impl TryFrom<BigInt> for TestField {
        type Error = sunscreen_zkp_backend::Error;

        fn try_from(_: BigInt) -> Result<Self, Self::Error> {
            unreachable!()
        }
    }

    impl Mul for TestField {
        type Output = Self;

        fn mul(self, _: Self) -> Self::Output {
            unreachable!()
        }
    }

    impl Sub for TestField {
        type Output = Self;

        fn sub(self, _: Self) -> Self::Output {
            unreachable!()
        }
    }

    impl Add for TestField {
        type Output = Self;

        fn add(self, _: Self) -> Self::Output {
            unreachable!()
        }
    }

    impl ZkpInto<BigInt> for TestField {
        fn into(self) -> BigInt {
            unreachable!()
        }
    }

    impl BackendField for TestField {
        const FIELD_MODULUS: BigInt = BigInt::from_u32(7);
    }

    #[test]
    fn can_wrap_number() {
        let x = NativeField::<TestField>::from(22i64);

        assert_eq!(x.val, BigInt::ONE);

        let x = NativeField::<TestField>::from(-22i64);

        assert_eq!(
            x.val,
            BigInt::from(TestField::FIELD_MODULUS.wrapping_sub(&BigInt::ONE))
        );
    }

    #[test]
    fn can_encode_unsigned_value() {
        let x = NativeField::<TestField>::from(6u64);

        assert_eq!(x.val, BigInt::from(6u64));

        let x = NativeField::<TestField>::from(7u64);

        assert_eq!(x.val, BigInt::ZERO);

        let x = NativeField::<TestField>::from(22u64);

        assert_eq!(x.val, BigInt::ONE);
    }
}
