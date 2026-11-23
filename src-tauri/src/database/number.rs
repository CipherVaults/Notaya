//! Exact rationals and certified enclosures for database numbers.

use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, LazyLock};

const MAX_BITS: usize = 16_384;
const LIMB_BITS: usize = u32::BITS as usize;
const FRACTION_BITS: u32 = 52;
const FRACTION_MASK: u64 = (1 << FRACTION_BITS) - 1;
const MIN_NORMAL_EXPONENT: i32 = -1022;
const MIN_SUBNORMAL_EXPONENT: i32 = -1074;
const POWER_BITS: u32 = 224;
const MAX_REFINE_BITS: u32 = 4096;
const MAX_EXPR_NODES: usize = 128;
const WORK_ERROR: &str = "The formula exceeds its arithmetic work budget";
const PRECISION_ERROR: &str = "Numeric precision is out of range";
const OVERFLOW: &str = "Numeric overflow";
const DIVISION_BY_ZERO: &str = "Division by zero";
const DOMAIN: &str = "Not a real number";
static MAX_VALUE: LazyLock<BigRational> =
    LazyLock::new(|| BigRational::from_float(f64::MAX).unwrap());
static MIN_VALUE: LazyLock<BigRational> = LazyLock::new(|| MAX_VALUE.negated());
static ZERO: LazyLock<BigRational> = LazyLock::new(BigRational::default);
static HALF: LazyLock<BigRational> = LazyLock::new(|| BigRational::dyadic(1u128.into(), -1));
static CUTOFF: LazyLock<BigRational> =
    LazyLock::new(|| BigRational::dyadic(1u128.into(), MIN_SUBNORMAL_EXPONENT - 1));
static LN_2: LazyLock<(BigUint, BigUint)> = LazyLock::new(|| ln2_bounds(POWER_BITS));

pub(crate) type Charge<'a> = dyn FnMut(usize) -> Result<(), String> + 'a;

pub(crate) fn with_budget<T>(
    f: impl FnOnce(&mut Charge<'_>) -> Result<T, String>,
) -> Result<T, String> {
    let mut remaining = Number::MAX_ARITHMETIC_WORK;
    f(&mut |work| {
        remaining = remaining.checked_sub(work).ok_or(WORK_ERROR)?;
        Ok(())
    })
}

fn next_precision(bits: u32) -> Option<u32> {
    (bits < MAX_REFINE_BITS).then(|| (bits * 2).min(MAX_REFINE_BITS))
}

// Little-endian base-2^32 limbs, with no high zero limbs. Zero is empty.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct BigUint(Vec<u32>);

impl Ord for BigUint {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.0
            .len()
            .cmp(&rhs.0.len())
            .then_with(|| self.0.iter().rev().cmp(rhs.0.iter().rev()))
    }
}

impl PartialOrd for BigUint {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl From<u128> for BigUint {
    fn from(mut value: u128) -> Self {
        let mut limbs = Vec::new();
        while value != 0 {
            limbs.push(value as u32);
            value >>= LIMB_BITS;
        }
        Self(limbs)
    }
}

impl BigUint {
    fn from_limbs(mut limbs: Vec<u32>) -> Self {
        while limbs.last() == Some(&0) {
            limbs.pop();
        }
        Self(limbs)
    }

    fn one() -> Self {
        Self(vec![1])
    }

    fn is_zero(&self) -> bool {
        self.0.is_empty()
    }

    fn is_one(&self) -> bool {
        self.0 == [1]
    }

    fn is_odd(&self) -> bool {
        self.0.first().is_some_and(|limb| limb & 1 != 0)
    }

    fn bits(&self) -> usize {
        self.0.last().map_or(0, |limb| {
            self.0.len() * LIMB_BITS - limb.leading_zeros() as usize
        })
    }

    fn to_u64(&self) -> Option<u64> {
        match self.0.as_slice() {
            [] => Some(0),
            [lo] => Some(u64::from(*lo)),
            [lo, hi] => Some(u64::from(*hi) << LIMB_BITS | u64::from(*lo)),
            _ => None,
        }
    }

    fn shl(&self, bits: usize) -> Self {
        if self.is_zero() || bits == 0 {
            return self.clone();
        }
        let mut limbs = vec![0; bits / LIMB_BITS];
        let shift = bits % LIMB_BITS;
        let mut carry = 0;
        for &limb in &self.0 {
            let wide = (u64::from(limb) << shift) | carry;
            limbs.push(wide as u32);
            carry = wide >> LIMB_BITS;
        }
        if carry != 0 {
            limbs.push(carry as u32);
        }
        Self(limbs)
    }

    fn shr_one(&mut self) {
        let mut carry = 0;
        for limb in self.0.iter_mut().rev() {
            let next_carry = *limb << (LIMB_BITS - 1);
            *limb = (*limb >> 1) | carry;
            carry = next_carry;
        }
        if self.0.last() == Some(&0) {
            self.0.pop();
        }
    }

    fn binary_exponent(&self, denominator: &Self) -> i32 {
        let shift = self.bits() as i32 - denominator.bits() as i32;
        let below = if shift >= 0 {
            self < &denominator.shl(shift as usize)
        } else {
            self.shl(shift.unsigned_abs() as usize) < *denominator
        };
        shift - i32::from(below)
    }

    fn shr(&self, bits: usize) -> Self {
        let limbs = self.0.get(bits / LIMB_BITS..).unwrap_or_default();
        let shift = bits % LIMB_BITS;
        if shift == 0 {
            return Self(limbs.to_vec());
        }
        Self::from_limbs(
            limbs
                .iter()
                .enumerate()
                .map(|(i, &limb)| {
                    (limb >> shift)
                        | (limbs.get(i + 1).copied().unwrap_or(0) << (LIMB_BITS - shift))
                })
                .collect(),
        )
    }

    fn fixed_mul(&self, rhs: &Self, bits: u32) -> Self {
        self.mul(rhs).shr(bits as usize)
    }

    fn add(&self, rhs: &Self) -> Self {
        let mut limbs = Vec::with_capacity(self.0.len().max(rhs.0.len()) + 1);
        let mut carry = 0u64;
        for i in 0..self.0.len().max(rhs.0.len()) {
            let sum = u64::from(self.0.get(i).copied().unwrap_or(0))
                + u64::from(rhs.0.get(i).copied().unwrap_or(0))
                + carry;
            limbs.push(sum as u32);
            carry = sum >> LIMB_BITS;
        }
        if carry != 0 {
            limbs.push(carry as u32);
        }
        Self(limbs)
    }

    fn sub_assign(&mut self, rhs: &Self) {
        // Callers establish self >= rhs.
        let mut borrow = false;
        for (i, limb) in self.0.iter_mut().enumerate() {
            let (difference, b1) = limb.overflowing_sub(rhs.0.get(i).copied().unwrap_or(0));
            let (difference, b2) = difference.overflowing_sub(u32::from(borrow));
            *limb = difference;
            borrow = b1 || b2;
        }
        debug_assert!(!borrow);
        while self.0.last() == Some(&0) {
            self.0.pop();
        }
    }

    fn mul(&self, rhs: &Self) -> Self {
        if self.is_zero() || rhs.is_zero() {
            return Self::default();
        }
        if self.is_one() {
            return rhs.clone();
        }
        if rhs.is_one() {
            return self.clone();
        }
        let mut limbs = vec![0; self.0.len() + rhs.0.len()];
        for (i, &left) in self.0.iter().enumerate() {
            let mut carry = 0;
            for (j, &right) in rhs.0.iter().enumerate() {
                // The product plus an existing limb and carry fits exactly in u64.
                let product = u64::from(left) * u64::from(right) + u64::from(limbs[i + j]) + carry;
                limbs[i + j] = product as u32;
                carry = product >> LIMB_BITS;
            }
            limbs[i + rhs.0.len()] = carry as u32;
        }
        Self::from_limbs(limbs)
    }

    fn div_rem_small(&self, divisor: u32) -> (Self, u32) {
        let mut quotient = self.0.clone();
        let mut remainder = 0;
        for limb in quotient.iter_mut().rev() {
            let wide = (remainder << LIMB_BITS) | u64::from(*limb);
            *limb = (wide / u64::from(divisor)) as u32;
            remainder = wide % u64::from(divisor);
        }
        (Self::from_limbs(quotient), remainder as u32)
    }

    fn div_rem(&self, divisor: &Self) -> (Self, Self) {
        assert!(!divisor.is_zero());
        match self.cmp(divisor) {
            Ordering::Less => return (Self::default(), self.clone()),
            Ordering::Equal => return (Self::one(), Self::default()),
            Ordering::Greater => {}
        }
        if let [limb] = divisor.0.as_slice() {
            let (quotient, remainder) = self.div_rem_small(*limb);
            return (quotient, Self::from(u128::from(remainder)));
        }

        // Binary long division. Walk the aligned divisor down one bit at a time.
        let shift = self.bits() - divisor.bits();
        let mut aligned = divisor.shl(shift);
        let mut quotient = vec![0; shift / LIMB_BITS + 1];
        let mut remainder = self.clone();
        for bit in (0..=shift).rev() {
            if remainder >= aligned {
                remainder.sub_assign(&aligned);
                quotient[bit / LIMB_BITS] |= 1 << (bit % LIMB_BITS);
            }
            aligned.shr_one();
        }
        (Self::from_limbs(quotient), remainder)
    }

    fn div_ceil(&self, divisor: &Self) -> Self {
        let (quotient, remainder) = self.div_rem(divisor);
        if remainder.is_zero() {
            quotient
        } else {
            quotient.add(&Self::one())
        }
    }

    fn gcd(mut left: Self, mut right: Self) -> Self {
        while !right.is_zero() {
            if right.is_one() {
                return right;
            }
            let remainder = left.div_rem(&right).1;
            left = right;
            right = remainder;
        }
        left
    }

    fn pow(&self, mut exponent: u32) -> Self {
        let mut result = Self::one();
        let mut base = self.clone();
        while exponent != 0 {
            if exponent & 1 != 0 {
                result = result.mul(&base);
            }
            exponent >>= 1;
            if exponent != 0 {
                base = base.mul(&base);
            }
        }
        result
    }

    fn integer_root(&self, degree: u32) -> Self {
        if self.is_zero() || self.is_one() {
            return self.clone();
        }
        let width = self.bits().div_ceil(degree as usize);
        // Test one bit at a time for high degrees, where Newton's initial
        // descent from the next power of two can take many iterations.
        if degree as usize > width {
            let mut root = Self::default();
            for bit in (0..width).rev() {
                let trial = root.add(&Self::one().shl(bit));
                if trial.pow(degree) <= *self {
                    root = trial;
                }
            }
            return root;
        }
        // Integer Newton iteration, starting above the root.
        let mut root = Self::one().shl(width);
        let weight = Self::from(u128::from(degree - 1));
        loop {
            let quotient = self.div_rem(&root.pow(degree - 1)).0;
            let next = root.mul(&weight).add(&quotient).div_rem_small(degree).0;
            if next >= root {
                return root;
            }
            root = next;
        }
    }

    fn exact_root(&self, degree: u32) -> Option<Self> {
        let root = self.integer_root(degree);
        (root.pow(degree) == *self).then_some(root)
    }

    fn decimal(digits: &str) -> Self {
        let mut result = Self::default();
        for chunk in digits.as_bytes().rchunks(9).rev() {
            let value = chunk
                .iter()
                .fold(0u32, |value, digit| value * 10 + u32::from(digit - b'0'));
            let factor = 10u64.pow(chunk.len() as u32);
            let mut carry = u64::from(value);
            for limb in &mut result.0 {
                let wide = u64::from(*limb) * factor + carry;
                *limb = wide as u32;
                carry = wide >> LIMB_BITS;
            }
            if carry != 0 {
                result.0.push(carry as u32);
            }
        }
        result
    }
}

// ln((1 + z)/(1 - z)) = 2*z*(1 + z²/3 + z⁴/5 + ...), for 0 <= z <= 1/3.
// Keep the leading z exact so a base extremely close to 1 survives multiplication
// by a large rational exponent.
fn log_series(numerator: &BigUint, denominator: &BigUint, bits: u32) -> (BigUint, BigUint) {
    let terms = bits / 3 + 4;
    let z = numerator.shl(bits as usize).div_rem(denominator).0;
    let square = z.fixed_mul(&z, bits);
    let mut term = BigUint::one().shl(bits as usize);
    let mut sum = term.clone();
    for k in 1..terms {
        term = term.fixed_mul(&square, bits);
        if term.is_zero() {
            break;
        }
        sum = sum.add(&term.div_rem_small(2 * k + 1).0);
    }
    // The square loses <2 ulps and subsequent powers lose <4 ulps.
    // For k >= 1 the doubled summand loses <2*(4/3 + 1) <5 ulps.
    // The scaled tail is below one ulp since 9^-terms < 2^-bits / 8.
    let lo = sum.shl(1);
    let hi = lo.add(&BigUint::from(u128::from(5 * terms + 8)));
    (lo, hi)
}

fn ln2_bounds(bits: u32) -> (BigUint, BigUint) {
    let three = BigUint::from(3);
    let (lo, hi) = log_series(&BigUint::one(), &three, bits);
    (lo.div_rem_small(3).0, hi.div_ceil(&three))
}

// For x in [0, 1], each term loses <2 ulps after the exact linear term.
// When the truncated term vanishes, the remaining scaled tail is <4 ulps.
fn exp_series(x: &BigUint, bits: u32, upward: bool) -> BigUint {
    let mut term = BigUint::one().shl(bits as usize);
    let mut sum = term.clone();
    for k in 1.. {
        term = term.fixed_mul(x, bits).div_rem_small(k).0;
        if term.is_zero() {
            return if upward {
                sum.add(&BigUint::from(u128::from(2 * k + 6)))
            } else {
                sum
            };
        }
        sum = sum.add(&term);
    }
    unreachable!()
}

fn integer_power_work(bits: usize) -> usize {
    4usize.saturating_mul(bits.div_ceil(LIMB_BITS).saturating_pow(2))
}

impl fmt::Display for BigUint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            return f.pad("0");
        }
        let mut chunks = Vec::new();
        let mut rest = self.clone();
        while !rest.is_zero() {
            let (quotient, remainder) = rest.div_rem_small(1_000_000_000);
            chunks.push(remainder);
            rest = quotient;
        }
        let mut output = chunks.pop().unwrap().to_string();
        for chunk in chunks.iter().rev() {
            use fmt::Write;
            write!(&mut output, "{chunk:09}")?;
        }
        f.pad(&output)
    }
}

// The sign lives here independently of the magnitude. Zero is never negative.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BigInt {
    negative: bool,
    magnitude: BigUint,
}

impl BigInt {
    fn new(negative: bool, magnitude: BigUint) -> Self {
        Self {
            negative: negative && !magnitude.is_zero(),
            magnitude,
        }
    }

    fn neg(self) -> Self {
        Self::new(!self.negative, self.magnitude)
    }

    fn to_i32(&self) -> Option<i32> {
        let magnitude = i64::try_from(self.magnitude.to_u64()?).ok()?;
        i32::try_from(if self.negative { -magnitude } else { magnitude }).ok()
    }

    fn add(self, mut rhs: Self) -> Self {
        if self.negative == rhs.negative {
            return Self::new(self.negative, self.magnitude.add(&rhs.magnitude));
        }
        let mut left = self;
        if left.magnitude < rhs.magnitude {
            std::mem::swap(&mut left, &mut rhs);
        }
        left.magnitude.sub_assign(&rhs.magnitude);
        Self::new(left.negative, left.magnitude)
    }

    fn cmp(&self, rhs: &Self) -> Ordering {
        rhs.negative.cmp(&self.negative).then_with(|| {
            if self.negative {
                rhs.magnitude.cmp(&self.magnitude)
            } else {
                self.magnitude.cmp(&rhs.magnitude)
            }
        })
    }

    fn scaled(&self, factor: &BigUint) -> Self {
        Self::new(self.negative, self.magnitude.mul(factor))
    }
}

macro_rules! from_integer {
    (signed: $($signed:ty),+; unsigned: $($unsigned:ty),+) => {
        $(impl From<$signed> for BigInt {
            fn from(value: $signed) -> Self {
                let value = value as i128;
                Self::new(value < 0, BigUint::from(value.unsigned_abs()))
            }
        })+
        $(impl From<$unsigned> for BigInt {
            fn from(value: $unsigned) -> Self {
                Self::new(false, BigUint::from(value as u128))
            }
        })+
    };
}

from_integer!(signed: i8, i16, i32, i64, i128, isize; unsigned: u8, u16, u32, u64, u128, usize);

// Reduced numerator and positive denominator. Integer zero has denominator one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BigRational {
    numerator: BigInt,
    denominator: BigUint,
}

impl Default for BigRational {
    fn default() -> Self {
        Self::from_integer(BigInt::default())
    }
}

impl BigRational {
    fn width(&self) -> usize {
        self.numerator.magnitude.bits().max(self.denominator.bits())
    }

    fn sign(&self) -> Ordering {
        if self.numerator.negative {
            Ordering::Less
        } else if self.numerator.magnitude.is_zero() {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }

    fn negated(&self) -> Self {
        Self {
            numerator: self.numerator.clone().neg(),
            denominator: self.denominator.clone(),
        }
    }

    fn dyadic(mantissa: BigUint, shift: i32) -> Self {
        Self::signed_dyadic(BigInt::new(false, mantissa), shift)
    }

    fn signed_dyadic(mut mantissa: BigInt, shift: i32) -> Self {
        if mantissa.magnitude.is_zero() {
            return Self::default();
        }
        if shift >= 0 {
            mantissa.magnitude = mantissa.magnitude.shl(shift as usize);
            return Self::from_integer(mantissa);
        }
        let places = shift.unsigned_abs() as usize;
        let zeros = mantissa
            .magnitude
            .0
            .iter()
            .take_while(|&&limb| limb == 0)
            .count()
            * LIMB_BITS;
        let zeros = zeros + mantissa.magnitude.0[zeros / LIMB_BITS].trailing_zeros() as usize;
        let cancel = places.min(zeros);
        mantissa.magnitude = mantissa.magnitude.shr(cancel);
        Self {
            numerator: mantissa,
            denominator: BigUint::one().shl(places - cancel),
        }
    }

    fn round_dyadic(&self, shift: i32, down: bool) -> Self {
        let shift_top = (-shift).max(0) as usize;
        let shift_bottom = shift.max(0) as usize;
        let scaled = Self {
            numerator: BigInt::new(
                self.numerator.negative,
                self.numerator.magnitude.shl(shift_top),
            ),
            denominator: self.denominator.shl(shift_bottom),
        };
        Self::signed_dyadic(scaled.directed_integer(down), shift)
    }

    fn exact_root(&self, degree: u32) -> Option<Self> {
        let numerator = self.numerator.magnitude.exact_root(degree)?;
        let denominator = self.denominator.exact_root(degree)?;
        Some(Self {
            numerator: BigInt::new(self.numerator.negative, numerator),
            denominator,
        })
    }

    fn from_decimal_parts(n: BigInt, shift: i32) -> Self {
        let power = BigUint::from(10).pow(shift.unsigned_abs());
        if shift < 0 {
            Self::new(n, power)
        } else {
            Self::from_integer(n.scaled(&power))
        }
    }

    fn new(mut numerator: BigInt, mut denominator: BigUint) -> Self {
        assert!(!denominator.is_zero());
        if numerator.magnitude.is_zero() || denominator.is_one() {
            return Self::from_integer(numerator);
        }
        let common = BigUint::gcd(numerator.magnitude.clone(), denominator.clone());
        if !common.is_one() {
            numerator.magnitude = numerator.magnitude.div_rem(&common).0;
            denominator = denominator.div_rem(&common).0;
        }
        Self {
            numerator,
            denominator,
        }
    }

    fn from_integer(numerator: BigInt) -> Self {
        Self {
            numerator,
            denominator: BigUint::one(),
        }
    }

    fn from_float(value: f64) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }
        let bits = value.to_bits();
        let biased = ((bits >> FRACTION_BITS) & 0x7ff) as i32;
        let fraction = bits & FRACTION_MASK;
        let (significand, exponent) = if biased == 0 {
            (fraction, MIN_SUBNORMAL_EXPONENT)
        } else {
            (
                fraction | (1 << FRACTION_BITS),
                biased - 1023 - FRACTION_BITS as i32,
            )
        };
        if significand == 0 {
            return Some(Self::from_integer(BigInt::default()));
        }

        // Strip powers of two before creating the fraction so it is already reduced.
        let zeros = significand.trailing_zeros();
        let significand = BigUint::from(u128::from(significand >> zeros));
        let exponent = exponent + zeros as i32;
        let negative = value.is_sign_negative();
        Some(if exponent >= 0 {
            Self::from_integer(BigInt::new(negative, significand.shl(exponent as usize)))
        } else {
            Self {
                numerator: BigInt::new(negative, significand),
                denominator: BigUint::one().shl(exponent.unsigned_abs() as usize),
            }
        })
    }

    fn to_f64(&self) -> Option<f64> {
        let magnitude = &self.numerator.magnitude;
        if magnitude.is_zero() {
            return Some(0.0);
        }

        let exponent = magnitude.binary_exponent(&self.denominator);

        if exponent > 1023 {
            return None;
        }
        let sign = u64::from(self.numerator.negative) << 63;
        if exponent < MIN_SUBNORMAL_EXPONENT - 1 {
            return Some(f64::from_bits(sign));
        }

        let unit = (exponent - FRACTION_BITS as i32).max(MIN_SUBNORMAL_EXPONENT);
        let (scaled, divisor) = if unit <= 0 {
            (
                magnitude.shl(unit.unsigned_abs() as usize),
                self.denominator.clone(),
            )
        } else {
            (magnitude.clone(), self.denominator.shl(unit as usize))
        };
        let (quotient, remainder) = scaled.div_rem(&divisor);
        let round_up = match remainder.shl(1).cmp(&divisor) {
            Ordering::Less => false,
            Ordering::Equal => quotient.is_odd(),
            Ordering::Greater => true,
        };
        let significand = quotient.to_u64()? + u64::from(round_up);

        // Add the hidden bit for normals. A rounding carry can advance the exponent,
        // including when the largest subnormal rounds to the smallest normal.
        let biased_without_hidden_bit = (exponent - MIN_NORMAL_EXPONENT).max(0) as u64;
        let bits = (biased_without_hidden_bit << FRACTION_BITS) + significand;
        (bits < f64::INFINITY.to_bits()).then(|| f64::from_bits(sign | bits))
    }

    fn combine(&self, rhs: &Self, subtract: bool) -> Self {
        let right = if subtract {
            rhs.numerator.clone().neg()
        } else {
            rhs.numerator.clone()
        };
        if self.denominator == rhs.denominator {
            return Self::new(self.numerator.clone().add(right), self.denominator.clone());
        }
        let left = self.numerator.scaled(&rhs.denominator);
        Self::new(
            left.add(right.scaled(&self.denominator)),
            self.denominator.mul(&rhs.denominator),
        )
    }

    fn mul(&self, rhs: &Self) -> Self {
        let numerator = BigInt::new(
            self.numerator.negative ^ rhs.numerator.negative,
            self.numerator.magnitude.mul(&rhs.numerator.magnitude),
        );
        Self::new(numerator, self.denominator.mul(&rhs.denominator))
    }

    fn div(&self, rhs: &Self) -> Self {
        let numerator = BigInt::new(
            self.numerator.negative ^ rhs.numerator.negative,
            self.numerator.magnitude.mul(&rhs.denominator),
        );
        Self::new(numerator, self.denominator.mul(&rhs.numerator.magnitude))
    }

    fn rem(&self, rhs: &Self) -> Self {
        let left = self.numerator.magnitude.mul(&rhs.denominator);
        let right = rhs.numerator.magnitude.mul(&self.denominator);
        // Like integer %, the exact remainder has the sign of the dividend.
        let remainder = left.div_rem(&right).1;
        let numerator = BigInt::new(self.numerator.negative, remainder);
        Self::new(numerator, self.denominator.mul(&rhs.denominator))
    }

    fn pow(&self, exponent: i32) -> Self {
        let magnitude = self.numerator.magnitude.pow(exponent.unsigned_abs());
        let denominator = self.denominator.pow(exponent.unsigned_abs());
        let (magnitude, denominator) = if exponent < 0 {
            (denominator, magnitude)
        } else {
            (magnitude, denominator)
        };
        let numerator = BigInt::new(self.numerator.negative && exponent & 1 != 0, magnitude);
        Self {
            numerator,
            denominator,
        }
    }

    fn directed_integer(&self, down: bool) -> BigInt {
        let (quotient, remainder) = self.numerator.magnitude.div_rem(&self.denominator);
        let magnitude = if self.numerator.negative == down && !remainder.is_zero() {
            quotient.add(&BigUint::one())
        } else {
            quotient
        };
        BigInt::new(self.numerator.negative, magnitude)
    }

    fn rounded_integer(numerator: &BigInt, denominator: &BigUint) -> BigInt {
        let (quotient, remainder) = numerator.magnitude.div_rem(denominator);
        let magnitude = if &remainder.shl(1) >= denominator {
            quotient.add(&BigUint::one())
        } else {
            quotient
        };
        BigInt::new(numerator.negative, magnitude)
    }
}

impl Ord for BigRational {
    fn cmp(&self, rhs: &Self) -> Ordering {
        if self.denominator == rhs.denominator || self.numerator.negative != rhs.numerator.negative
        {
            return self.numerator.cmp(&rhs.numerator);
        }
        self.numerator
            .scaled(&rhs.denominator)
            .cmp(&rhs.numerator.scaled(&self.denominator))
    }
}

impl PartialOrd for BigRational {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

#[derive(Clone, Debug)]
pub struct Number(NumberKind);

#[derive(Clone, Debug)]
enum NumberKind {
    Exact(BigRational),
    Interval(Box<IntervalNumber>),
}

#[derive(Clone, Debug)]
struct IntervalNumber {
    bounds: Bounds,
    bits: u32,
    expr: Option<Arc<ApproxExpr>>,
    // Retain an eagerly certified direct-root display decision for the formula meter.
    significant: Option<String>,
}

#[derive(Clone, Debug)]
struct Bounds {
    lo: BigRational,
    hi: BigRational,
}

#[derive(Clone, Debug)]
struct ApproxExpr {
    kind: ExprKind,
}

#[derive(Clone, Debug)]
enum ExprKind {
    Exact(BigRational),
    Sqrt(Arc<ApproxExpr>),
    Cbrt(Arc<ApproxExpr>),
    Ln(Arc<ApproxExpr>),
    Log10(Arc<ApproxExpr>),
    Log2(Arc<ApproxExpr>),
    Exp(Arc<ApproxExpr>),
    FracPow(Arc<ApproxExpr>, Arc<ApproxExpr>),
    Neg(Arc<ApproxExpr>),
    Abs(Arc<ApproxExpr>),
    Add(Arc<ApproxExpr>, Arc<ApproxExpr>),
    Sub(Arc<ApproxExpr>, Arc<ApproxExpr>),
    Mul(Arc<ApproxExpr>, Arc<ApproxExpr>),
    Div(Arc<ApproxExpr>, Arc<ApproxExpr>),
    Rem(Arc<ApproxExpr>, Arc<ApproxExpr>),
    IntPow(Arc<ApproxExpr>, i32),
    Pow(Arc<ApproxExpr>, Arc<ApproxExpr>),
}

#[derive(Clone, Copy)]
enum Unary {
    Sqrt,
    Cbrt,
    Log(LogBase),
    Exp,
    Neg,
    Abs,
    IntPow(i32),
}

#[derive(Clone, Copy)]
pub(crate) enum LogBase {
    Natural,
    Ten,
    Two,
}

#[derive(Clone, Copy)]
pub(crate) enum Binary {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    FracPow,
    Pow,
}

impl Number {
    pub(crate) const MAX_ARITHMETIC_WORK: usize = 32_000_000;

    fn from_ratio(n: BigRational) -> Self {
        Self(NumberKind::Exact(n))
    }

    pub fn integer(n: impl Into<BigInt>) -> Self {
        Self::from_ratio(BigRational::from_integer(n.into()))
    }

    fn decimal_parts(s: &str) -> Result<Option<(BigInt, i32)>, String> {
        let s = s.trim();
        let unsigned = s.strip_prefix(['+', '-']).unwrap_or(s);
        if unsigned.eq_ignore_ascii_case("nan") {
            return Err("Not a real number".into());
        }
        if unsigned.eq_ignore_ascii_case("inf") || unsigned.eq_ignore_ascii_case("infinity") {
            return Err("Numeric overflow".into());
        }
        let (mantissa, exp) = unsigned.split_once(['e', 'E']).unwrap_or((unsigned, "0"));
        let (whole, frac) = mantissa.split_once('.').unwrap_or((mantissa, ""));
        let exp_digits = exp.strip_prefix(['+', '-']).unwrap_or(exp);
        if (whole.is_empty() && frac.is_empty())
            || !whole
                .bytes()
                .chain(frac.bytes())
                .all(|c| c.is_ascii_digit())
            || exp_digits.is_empty()
            || !exp_digits.bytes().all(|c| c.is_ascii_digit())
        {
            return Ok(None);
        }
        let digits = format!("{whole}{frac}");
        let digits = digits.trim_start_matches('0');
        if digits.is_empty() {
            return Ok(Some((BigInt::default(), 0)));
        }
        let digits_without_trailing = digits.trim_end_matches('0');
        let shift = exp
            .parse::<i64>()
            .unwrap_or_else(|_| {
                if exp.starts_with('-') {
                    i64::MIN
                } else {
                    i64::MAX
                }
            })
            .saturating_sub(frac.len() as i64)
            .saturating_add((digits.len() - digits_without_trailing.len()) as i64);
        if shift.saturating_add(digits_without_trailing.len() as i64) > 309 {
            return Err("Numeric overflow".into());
        }
        if shift.unsigned_abs() > MAX_BITS as u64 || digits_without_trailing.len() > MAX_BITS {
            return Err("Numeric precision is out of range".into());
        }
        let n = BigInt::new(
            s.starts_with('-'),
            BigUint::decimal(digits_without_trailing),
        );
        Ok(Some((n, shift as i32)))
    }

    fn from_parts(n: BigInt, shift: i32) -> Result<Self, String> {
        Self::checked(BigRational::from_decimal_parts(n, shift))
    }

    /// Nonnumeric text may be skipped by a rollup. Numeric overflow must propagate.
    pub fn parse(s: &str) -> Result<Option<Self>, String> {
        Self::decimal_parts(s)?
            .map(|(n, shift)| Self::from_parts(n, shift))
            .transpose()
    }

    pub fn decimal(s: &str) -> Result<Self, String> {
        Self::parse(s)?.ok_or_else(|| "Not a number".into())
    }

    /// Validate an entered decimal and normalize only the syntax JSON requires.
    pub fn json_decimal(s: &str) -> Result<String, String> {
        let s = s.trim();
        Self::decimal(s)?;
        let (mantissa, exponent) = s.find(['e', 'E']).map_or((s, ""), |i| s.split_at(i));
        let sign = if mantissa.starts_with('-') { "-" } else { "" };
        let unsigned = mantissa.strip_prefix(['+', '-']).unwrap_or(mantissa);
        let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
        let whole = whole.trim_start_matches('0');
        let whole = if whole.is_empty() { "0" } else { whole };
        if fraction.is_empty() {
            Ok(format!("{sign}{whole}{exponent}"))
        } else {
            Ok(format!("{sign}{whole}.{fraction}{exponent}"))
        }
    }

    /// Boundary for functions such as sqrt and exp that produce a floating-point result.
    pub fn from_float(n: f64) -> Result<Self, String> {
        BigRational::from_float(n)
            .map(Self::from_ratio)
            .ok_or_else(|| {
                if n.is_nan() {
                    "Not a real number"
                } else {
                    "Numeric overflow"
                }
                .into()
            })
    }

    fn checked(n: BigRational) -> Result<Self, String> {
        let numerator = n.numerator.magnitude.bits();
        let denominator = n.denominator.bits();
        if numerator.saturating_sub(denominator) >= 1023
            && n.numerator.magnitude > MAX_VALUE.numerator.magnitude.mul(&n.denominator)
        {
            return Err("Numeric overflow".into());
        }
        // Bound exact intermediates just as the formula evaluator bounds steps and list lengths.
        if numerator.max(denominator) > MAX_BITS {
            return Err(PRECISION_ERROR.into());
        }
        Ok(Self::from_ratio(n))
    }

    pub(crate) fn lower(&self) -> &BigRational {
        match &self.0 {
            NumberKind::Exact(n) => n,
            NumberKind::Interval(n) => &n.bounds.lo,
        }
    }

    pub(crate) fn upper(&self) -> &BigRational {
        match &self.0 {
            NumberKind::Exact(n) => n,
            NumberKind::Interval(n) => &n.bounds.hi,
        }
    }

    fn exact(&self) -> Option<&BigRational> {
        match &self.0 {
            NumberKind::Exact(n) => Some(n),
            NumberKind::Interval(_) => None,
        }
    }

    pub(crate) fn is_exact(&self) -> bool {
        self.exact().is_some()
    }

    pub fn is_zero(&self) -> bool {
        self.exact().is_some_and(|n| n.sign().is_eq())
    }

    fn width(&self) -> usize {
        match &self.0 {
            NumberKind::Exact(n) => n.width(),
            NumberKind::Interval(n) => n.bounds.width(),
        }
    }

    fn bits(&self) -> u32 {
        match &self.0 {
            NumberKind::Exact(_) => POWER_BITS,
            NumberKind::Interval(n) => n.bits,
        }
    }

    fn expr(&self) -> Option<Arc<ApproxExpr>> {
        match &self.0 {
            NumberKind::Exact(n) => Some(Arc::new(ApproxExpr {
                kind: ExprKind::Exact(n.clone()),
            })),
            NumberKind::Interval(n) => n.expr.clone(),
        }
    }

    pub(crate) fn same_expr(&self, rhs: &Self) -> bool {
        match (&self.0, &rhs.0) {
            (NumberKind::Exact(a), NumberKind::Exact(b)) => a == b,
            (NumberKind::Interval(a), NumberKind::Interval(b)) => a
                .expr
                .as_deref()
                .zip(b.expr.as_deref())
                .is_some_and(|(a, b)| a.same_expr(b)),
            _ => false,
        }
    }

    fn sort_key(&self) -> (Cow<'_, BigRational>, &BigRational, &BigRational) {
        match &self.0 {
            NumberKind::Exact(n) => (Cow::Borrowed(n), n, n),
            NumberKind::Interval(n) => {
                let Bounds { lo, hi } = &n.bounds;
                (Cow::Owned(lo.combine(hi, false).mul(&HALF)), lo, hi)
            }
        }
    }

    /// Try one shared precision tier and return whether a later tier can still help.
    pub(crate) fn refine_for_sort(&mut self, target_bits: u32, remaining: &mut usize) -> bool {
        if *remaining == 0 {
            return false;
        }
        let NumberKind::Interval(n) = &mut self.0 else {
            return false;
        };
        let Some(expr) = &n.expr else {
            return false;
        };
        if n.bits >= MAX_REFINE_BITS {
            return false;
        }
        if target_bits <= n.bits {
            return true;
        }
        let mut charge = |work| {
            match remaining.checked_sub(work) {
                Some(left) => {
                    *remaining = left;
                    Ok(())
                }
                None => {
                    *remaining = 0;
                    Err(WORK_ERROR.into())
                }
            }
        };
        match expr.evaluate(target_bits, &mut charge) {
            Ok(Some(bounds)) => {
                // Both enclosures certify the same value. Keep their intersection.
                n.bounds.lo = bounds.lo.max(n.bounds.lo.clone());
                n.bounds.hi = bounds.hi.min(n.bounds.hi.clone());
                n.bits = target_bits;
            }
            Ok(None) => {}
            Err(_) => return false,
        }
        target_bits < MAX_REFINE_BITS && *remaining > 0
    }

    fn direct_root(&self) -> Option<(&BigRational, &BigRational, bool)> {
        match &self.0 {
            NumberKind::Interval(n) => n.expr.as_ref()?.direct_root(false),
            NumberKind::Exact(_) => None,
        }
    }

    fn bounds_at(&self, bits: u32, charge: &mut Charge<'_>) -> Result<Option<Bounds>, String> {
        match &self.0 {
            NumberKind::Interval(n) if bits > n.bits => match &n.expr {
                Some(expr) => expr.evaluate(bits, charge),
                None => Err(WORK_ERROR.into()),
            },
            _ => {
                charge(self.width().div_ceil(LIMB_BITS) * 4)?;
                Ok(Some(Bounds {
                    lo: self.lower().clone(),
                    hi: self.upper().clone(),
                }))
            }
        }
    }

    fn value_from_bounds(
        bounds: Bounds,
        bits: u32,
        expr: Option<Arc<ApproxExpr>>,
        significant: Option<String>,
    ) -> Self {
        if bounds.lo == bounds.hi {
            Self::from_ratio(bounds.lo)
        } else {
            Self(NumberKind::Interval(Box::new(IntervalNumber {
                bounds,
                bits,
                expr,
                significant,
            })))
        }
    }

    pub(crate) fn arithmetic_work(&self, op: &str, rhs: &Self) -> usize {
        if let (Some(left), Some(right)) = (self.exact(), rhs.exact()) {
            return exact_work(left, op, right);
        }
        0 // Interval operations charge their actual endpoint work.
    }

    pub(crate) fn unary_work(&self, op: &str) -> usize {
        let width = self.width().div_ceil(LIMB_BITS);
        match op {
            "format" => LIMB_BITS * (2 * width + 2).pow(2),
            "ceil" | "floor" => LIMB_BITS * width.pow(2),
            _ => 4 * width,
        }
    }

    pub(crate) fn sqrt(&self, charge: &mut Charge<'_>) -> Result<Self, String> {
        if let Some(n) = self.exact() {
            if n.sign().is_lt() {
                return Err(DOMAIN.into());
            }
            charge(sqrt_work(n.width()))?;
            if let Some(root) = n.exact_root(2) {
                return Ok(Self::from_ratio(root));
            }
            charge(sqrt_work((2 * POWER_BITS) as usize).saturating_sub(sqrt_work(n.width())))?;
            return self.root_value(sqrt_at(n, POWER_BITS), &HALF, true, charge);
        }
        self.unary_interval(Unary::Sqrt, charge)
    }

    fn root_value(
        &self,
        mut bounds: Bounds,
        exponent: &BigRational,
        sqrt: bool,
        charge: &mut Charge<'_>,
    ) -> Result<Self, String> {
        let base = self.exact().expect("direct root input");
        match certify_root_boundary(base, exponent, &bounds.lo, &bounds.hi, &MAX_VALUE, charge)? {
            Ordering::Greater => return Err(OVERFLOW.into()),
            Ordering::Equal => return Ok(Self::from_ratio(MAX_VALUE.clone())),
            Ordering::Less if bounds.hi > *MAX_VALUE => bounds.hi = MAX_VALUE.clone(),
            Ordering::Less => {}
        }
        if !sqrt
            && certify_root_boundary(base, exponent, &bounds.lo, &bounds.hi, &CUTOFF, charge)?
                .is_le()
        {
            return Ok(Self::integer(0));
        }
        // Check the display boundary against the shared formula budget at the
        // same point as the exact-root work. The enclosure survives this check.
        let significant = root_significant(&bounds, base, exponent, false, charge)?;
        let input = self.expr().expect("exact root input");
        let kind = if sqrt {
            ExprKind::Sqrt(input)
        } else {
            let exponent = Arc::new(ApproxExpr { kind: ExprKind::Exact(exponent.clone()) });
            ExprKind::FracPow(input, exponent)
        };
        let expr = Arc::new(ApproxExpr { kind });
        Ok(Self::value_from_bounds(
            bounds,
            POWER_BITS,
            Some(expr),
            significant,
        ))
    }

    pub(crate) fn pow(&self, rhs: &Self, charge: &mut Charge<'_>) -> Result<Self, String> {
        if let Some(exponent) = rhs.exact() {
            if let Some(base) = self.exact() {
                if base.sign().is_eq() && exponent.sign().is_lt() {
                    return Err(DIVISION_BY_ZERO.into());
                }
                if exponent.denominator.is_one() {
                    return integer_pow_exact(base, exponent);
                }
                if base.sign().is_lt() {
                    return Err(DOMAIN.into());
                }
                if base.sign().is_eq() || base.numerator.magnitude == base.denominator {
                    return Ok(self.clone());
                }
                if exponent == &*HALF {
                    // Formula dispatch already charges the power's square-root work.
                    if let Some(root) = base.exact_root(2) {
                        return Ok(Self::from_ratio(root));
                    }
                    charge(
                        sqrt_work((2 * POWER_BITS) as usize)
                            .saturating_sub(sqrt_work(base.width())),
                    )?;
                    return self.root_value(sqrt_at(base, POWER_BITS), exponent, true, charge);
                }
                let bounds = fractional_power_at(base, exponent, POWER_BITS)?.ok_or(OVERFLOW)?;
                if bounds.hi <= *CUTOFF {
                    return Ok(Self::integer(0));
                }
                if let Some((degree, power, _)) = exact_root_power(base, exponent) {
                    if let Some(root) = base.exact_root(degree) {
                        let exact = root.pow(power);
                        return if exact <= *CUTOFF {
                            Ok(Self::integer(0))
                        } else {
                            Self::checked(exact)
                        };
                    }
                }
                return self.root_value(bounds, exponent, false, charge);
            }
            if exponent.denominator.is_one() {
                let power = exponent
                    .numerator
                    .to_i32()
                    .ok_or("Numeric power is out of range")?;
                return self.unary_interval(Unary::IntPow(power), charge);
            }
            if exponent == &*HALF {
                return self.sqrt(charge);
            }
            return self.binary_interval(Binary::FracPow, rhs, charge);
        }
        self.binary_interval(Binary::Pow, rhs, charge)
    }

    fn unary_interval(&self, op: Unary, charge: &mut Charge<'_>) -> Result<Self, String> {
        let bits = self.bits();
        let expr = self.expr().and_then(|input| {
            ApproxExpr::bounded(match op {
                Unary::Sqrt => ExprKind::Sqrt(input),
                Unary::Cbrt => ExprKind::Cbrt(input),
                Unary::Log(LogBase::Natural) => ExprKind::Ln(input),
                Unary::Log(LogBase::Ten) => ExprKind::Log10(input),
                Unary::Log(LogBase::Two) => ExprKind::Log2(input),
                Unary::Exp => ExprKind::Exp(input),
                Unary::Neg => ExprKind::Neg(input),
                Unary::Abs => ExprKind::Abs(input),
                Unary::IntPow(power) => ExprKind::IntPow(input, power),
            })
        });
        let initial = self.bounds_at(bits, charge)?.expect("current bounds");
        Self::finish_interval(initial.unary(op, bits, charge)?, bits, expr, charge)
    }

    fn binary_interval(
        &self,
        op: Binary,
        rhs: &Self,
        charge: &mut Charge<'_>,
    ) -> Result<Self, String> {
        let bits = self.bits().min(rhs.bits());
        let left = self.bounds_at(bits, charge)?.expect("current bounds");
        let right = if matches!(op, Binary::Div | Binary::Rem) {
            rhs.nonzero_bounds(bits, charge)?
        } else {
            rhs.bounds_at(bits, charge)?.expect("current bounds")
        };
        let expr = self.expr().zip(rhs.expr()).and_then(|(left, right)| {
            ApproxExpr::bounded(match op {
                Binary::Add => ExprKind::Add(left, right),
                Binary::Sub => ExprKind::Sub(left, right),
                Binary::Mul => ExprKind::Mul(left, right),
                Binary::Div => ExprKind::Div(left, right),
                Binary::Rem => ExprKind::Rem(left, right),
                Binary::FracPow => ExprKind::FracPow(left, right),
                Binary::Pow => ExprKind::Pow(left, right),
            })
        });
        Self::finish_interval(left.binary(op, &right, bits, charge)?, bits, expr, charge)
    }

    fn nonzero_bounds(&self, mut bits: u32, charge: &mut Charge<'_>) -> Result<Bounds, String> {
        loop {
            if let Some(bounds) = self.bounds_at(bits, charge)? {
                if !bounds.contains_zero() {
                    return Ok(bounds);
                }
                if bounds.lo == bounds.hi {
                    return Err(DIVISION_BY_ZERO.into());
                }
            }
            bits = next_precision(bits).ok_or(WORK_ERROR)?;
        }
    }

    fn finish_interval(
        initial: Option<Bounds>,
        mut bits: u32,
        expr: Option<Arc<ApproxExpr>>,
        charge: &mut Charge<'_>,
    ) -> Result<Self, String> {
        let mut candidate = initial;
        loop {
            if let Some(bounds) = candidate {
                if bounds.in_range(charge)? {
                    return Ok(Self::value_from_bounds(bounds, bits, expr, None));
                }
            }
            bits = next_precision(bits).ok_or(WORK_ERROR)?;
            candidate = expr.as_ref().ok_or(WORK_ERROR)?.evaluate(bits, charge)?;
        }
    }

    fn observe<T>(
        &self,
        charge: &mut Charge<'_>,
        mut project: impl FnMut(
            &BigRational,
            &BigRational,
            &mut Charge<'_>,
        ) -> Result<Option<T>, String>,
    ) -> Result<T, String> {
        if let Some(n) = self.exact() {
            return project(n, n, charge)?.ok_or_else(|| WORK_ERROR.into());
        }
        let mut bits = self.bits();
        let mut first = true;
        loop {
            if first {
                if let Some(value) = project(self.lower(), self.upper(), charge)? {
                    return Ok(value);
                }
                first = false;
            } else if let Some(bounds) = self.bounds_at(bits, charge)? {
                if let Some(value) = project(&bounds.lo, &bounds.hi, charge)? {
                    return Ok(value);
                }
            }
            bits = next_precision(bits).ok_or(WORK_ERROR)?;
        }
    }

    fn compare_root(
        &self,
        lo: &BigRational,
        hi: &BigRational,
        boundary: &BigRational,
        charge: &mut Charge<'_>,
    ) -> Result<Option<Ordering>, String> {
        let Some((base, exponent, negative)) = self.direct_root() else {
            return Ok(None);
        };
        if negative {
            if boundary.sign().is_ge() {
                return Ok(Some(Ordering::Less));
            }
            return Ok(Some(
                certify_root_boundary(
                    base,
                    exponent,
                    &hi.negated(),
                    &lo.negated(),
                    &boundary.negated(),
                    charge,
                )?
                .reverse(),
            ));
        }
        if boundary.sign().is_le() {
            return Ok(Some(Ordering::Greater));
        }
        certify_root_boundary(base, exponent, lo, hi, boundary, charge).map(Some)
    }

    pub(crate) fn cmp_checked(
        &self,
        rhs: &Self,
        charge: &mut Charge<'_>,
    ) -> Result<Ordering, String> {
        if let (Some(left), Some(right)) = (self.exact(), rhs.exact()) {
            charge(exact_work(left, "<", right))?;
            return Ok(left.cmp(right));
        }
        if let (Some((a, power, an)), Some((b, other_power, bn))) =
            (self.direct_root(), rhs.direct_root())
        {
            if an != bn {
                return Ok(if an {
                    Ordering::Less
                } else {
                    Ordering::Greater
                });
            }
            if power == other_power {
                charge(exact_work(a, "<", b))?;
                let side = a.cmp(b);
                return Ok(if an != power.sign().is_lt() {
                    side.reverse()
                } else {
                    side
                });
            }
        }
        let mut bits = self.bits().min(rhs.bits());
        loop {
            if let (Some(a), Some(b)) =
                (self.bounds_at(bits, charge)?, rhs.bounds_at(bits, charge)?)
            {
                charge(pair_work(a.width(), b.width()))?;
                if a.hi < b.lo {
                    return Ok(Ordering::Less);
                }
                if a.lo > b.hi {
                    return Ok(Ordering::Greater);
                }
                if b.lo == b.hi {
                    if a.lo == a.hi {
                        return Ok(a.lo.cmp(&b.lo));
                    }
                    if let Some(side) = self.compare_root(&a.lo, &a.hi, &b.lo, charge)? {
                        return Ok(side);
                    }
                }
                if a.lo == a.hi {
                    if let Some(side) = rhs.compare_root(&b.lo, &b.hi, &a.lo, charge)? {
                        return Ok(side.reverse());
                    }
                }
            }
            bits = next_precision(bits).ok_or(WORK_ERROR)?;
        }
    }

    pub fn try_is_negative(&self) -> Result<bool, String> {
        if let Some(n) = self.exact() {
            return Ok(n.sign().is_lt());
        }
        with_budget(|charge| Ok(self.cmp_checked(&Self::integer(0), charge)?.is_lt()))
    }

    pub fn try_as_f64(&self) -> Result<f64, String> {
        with_budget(|charge| self.float_checked(charge))
    }

    pub(crate) fn float_checked(&self, charge: &mut Charge<'_>) -> Result<f64, String> {
        if let Some(n) = self.exact() {
            charge(projection_work(n, n))?;
            return n.to_f64().ok_or_else(|| OVERFLOW.into());
        }
        self.observe(charge, |lo, hi, charge| {
            charge(projection_work(lo, hi))?;
            let (Some(low), Some(high)) = (lo.to_f64(), hi.to_f64()) else {
                return Ok(None);
            };
            if low.to_bits() == high.to_bits() {
                return Ok(Some(low));
            }
            if self.direct_root().is_some() && low.to_bits().abs_diff(high.to_bits()) == 1 {
                let lo_float = BigRational::from_float(low).expect("finite");
                let hi_float = BigRational::from_float(high).expect("finite");
                let midpoint = lo_float.combine(&hi_float, false).mul(&HALF);
                if let Some(side) = self.compare_root(lo, hi, &midpoint, charge)? {
                    return Ok(Some(match side {
                        Ordering::Less => low,
                        Ordering::Greater => high,
                        Ordering::Equal if low.to_bits() & 1 == 0 => low,
                        Ordering::Equal => high,
                    }));
                }
            }
            Ok(None)
        })
    }

    pub(crate) fn logarithm(&self, base: LogBase, charge: &mut Charge<'_>) -> Result<Self, String> {
        if let Some(n) = self.exact() {
            if n.sign().is_le() {
                return Err(DOMAIN.into());
            }
            let top = &n.numerator.magnitude;
            let bottom = &n.denominator;
            if top.is_one() && bottom.is_one() {
                return Ok(Self::integer(0));
            }
            if !matches!(base, LogBase::Natural) && (top.is_one() || bottom.is_one()) {
                let reciprocal = top.is_one();
                let magnitude = if reciprocal { bottom } else { top };
                charge(integer_power_work(magnitude.bits()))?;
                // The exponent of two dividing 2^k or 10^k is k in both cases.
                let (limb, digit) = magnitude.0.iter().enumerate().find(|(_, digit)| **digit != 0)
                    .expect("positive logarithm input");
                let exponent = (limb * LIMB_BITS) as u32 + digit.trailing_zeros();
                let matches_power = match base {
                    LogBase::Two => *magnitude == BigUint::one().shl(exponent as usize),
                    LogBase::Ten => {
                        3 * (exponent as usize) < magnitude.bits()
                            && *magnitude == BigUint::from(10).pow(exponent)
                    }
                    LogBase::Natural => unreachable!(),
                };
                if matches_power {
                    let exponent = exponent as i32;
                    return Ok(Self::integer(if reciprocal { -exponent } else { exponent }));
                }
            }
        }
        self.unary_interval(Unary::Log(base), charge)
    }

    pub(crate) fn exp(&self, charge: &mut Charge<'_>) -> Result<Self, String> {
        self.unary_interval(Unary::Exp, charge)
    }

    pub(crate) fn cbrt(&self, charge: &mut Charge<'_>) -> Result<Self, String> {
        if let Some(n) = self.exact() {
            charge(sqrt_work(n.width()))?;
            if let Some(root) = n.exact_root(3) {
                return Ok(Self::from_ratio(root));
            }
        }
        self.unary_interval(Unary::Cbrt, charge)
    }

    pub(crate) fn neg_checked(self, charge: &mut Charge<'_>) -> Result<Self, String> {
        match self.0 {
            NumberKind::Exact(mut n) => {
                n.numerator = n.numerator.neg();
                Ok(Self::from_ratio(n))
            }
            NumberKind::Interval(_) => self.unary_interval(Unary::Neg, charge),
        }
    }

    pub fn abs(&self, charge: &mut Charge<'_>) -> Result<Self, String> {
        if let Some(n) = self.exact() {
            let mut n = n.clone();
            n.numerator.negative = false;
            return Ok(Self::from_ratio(n));
        }
        self.unary_interval(Unary::Abs, charge)
    }

    pub fn floor(&self, charge: &mut Charge<'_>) -> Result<Self, String> {
        self.directed_integer(true, charge)
    }

    pub fn ceil(&self, charge: &mut Charge<'_>) -> Result<Self, String> {
        self.directed_integer(false, charge)
    }

    fn directed_integer(&self, down: bool, charge: &mut Charge<'_>) -> Result<Self, String> {
        self.observe(charge, |lo, hi, charge| {
            if lo != hi {
                charge(projection_work(lo, hi))?;
            }
            let low = lo.directed_integer(down);
            let high = hi.directed_integer(down);
            if low == high {
                return Ok(Some(Self::integer(low)));
            }
            if self.direct_root().is_some() && low.clone().add(1.into()) == high {
                let boundary =
                    BigRational::from_integer(if down { high.clone() } else { low.clone() });
                if let Some(side) = self.compare_root(lo, hi, &boundary, charge)? {
                    return Ok(Some(Self::integer(
                        if (down && side.is_lt()) || (!down && side.is_le()) {
                            low
                        } else {
                            high
                        },
                    )));
                }
            }
            Ok(None)
        })
    }

    pub fn sign(&self, charge: &mut Charge<'_>) -> Result<Self, String> {
        Ok(Self::integer(
            match self.cmp_checked(&Self::integer(0), charge)? {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            },
        ))
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn sub(&self, rhs: &Self) -> Result<Self, String> {
        with_budget(|charge| self.binary_checked(Binary::Sub, rhs, charge))
    }

    pub fn add(&self, rhs: &Self) -> Result<Self, String> {
        with_budget(|charge| self.binary_checked(Binary::Add, rhs, charge))
    }

    pub fn mul(&self, rhs: &Self) -> Result<Self, String> {
        with_budget(|charge| self.binary_checked(Binary::Mul, rhs, charge))
    }

    pub fn div(&self, rhs: &Self) -> Result<Self, String> {
        with_budget(|charge| self.binary_checked(Binary::Div, rhs, charge))
    }

    pub(crate) fn binary_checked(
        &self,
        op: Binary,
        rhs: &Self,
        charge: &mut Charge<'_>,
    ) -> Result<Self, String> {
        if let (Some(a), Some(b)) = (self.exact(), rhs.exact()) {
            if matches!(op, Binary::Rem) {
                charge(exact_work(a, "%", b))?;
            }
            return Self::checked(match op {
                Binary::Add => a.combine(b, false),
                Binary::Sub => a.combine(b, true),
                Binary::Mul => a.mul(b),
                Binary::Div | Binary::Rem if b.sign().is_eq() => {
                    return Err(DIVISION_BY_ZERO.into())
                }
                Binary::Div => a.div(b),
                Binary::Rem => a.rem(b),
                _ => unreachable!(),
            });
        }
        self.binary_interval(op, rhs, charge)
    }

    pub fn rem(&self, rhs: &Self, charge: &mut Charge<'_>) -> Result<Self, String> {
        self.binary_checked(Binary::Rem, rhs, charge)
    }

    fn significant_parts(ratio: &BigRational) -> (BigUint, BigUint, i32) {
        let n = &ratio.numerator.magnitude;
        let d = &ratio.denominator;
        let ten = BigUint::from(10);
        let mut exponent = n.to_string().len() as i32 - d.to_string().len() as i32;
        let power = ten.pow(exponent.unsigned_abs());
        let below = if exponent < 0 {
            n.mul(&power) < *d
        } else {
            *n < d.mul(&power)
        };
        exponent -= i32::from(below);
        let shift = 11 - exponent;
        let factor = ten.pow(shift.unsigned_abs());
        let (numerator, denominator) = if shift < 0 {
            (n.clone(), d.mul(&factor))
        } else {
            (n.mul(&factor), d.clone())
        };
        (numerator, denominator, exponent)
    }

    pub fn significant_text(&self) -> Result<String, String> {
        with_budget(|charge| self.significant_checked(charge))
    }

    pub(crate) fn significant_checked(&self, charge: &mut Charge<'_>) -> Result<String, String> {
        charge(self.unary_work("format"))?;
        if let Some(n) = self.exact() {
            return Ok(significant_text(n));
        }
        if let NumberKind::Interval(n) = &self.0 {
            if let Some(text) = &n.significant {
                return Ok(text.clone());
            }
        }
        self.observe(charge, |lo, hi, charge| {
            charge(projection_work(lo, hi))?;
            if lo.sign() != hi.sign() {
                return Ok(None);
            }
            let low = significant_text(lo);
            if low == significant_text(hi) {
                return Ok(Some(low));
            }
            if let Some((base, exponent, negative)) = self.direct_root() {
                let bounds = if negative {
                    Bounds {
                        lo: hi.negated(),
                        hi: lo.negated(),
                    }
                } else {
                    Bounds {
                        lo: lo.clone(),
                        hi: hi.clone(),
                    }
                };
                return root_significant(&bounds, base, exponent, negative, charge);
            }
            Ok(None)
        })
    }

    fn rounded_units(
        &self,
        places: i32,
        factor: &BigUint,
        charge: &mut Charge<'_>,
    ) -> Result<BigInt, String> {
        let units = |n: &BigRational| {
            if places < 0 {
                BigRational::rounded_integer(&n.numerator, &n.denominator.mul(factor))
            } else {
                BigRational::rounded_integer(&n.numerator.scaled(factor), &n.denominator)
            }
        };
        self.observe(charge, |lo, hi, charge| {
            if lo != hi {
                charge(projection_work(lo, hi) + integer_power_work(factor.bits()))?;
            }
            let low = units(lo);
            let high = units(hi);
            if low == high {
                return Ok(Some(low));
            }
            if self.direct_root().is_some() && low.clone().add(1.into()) == high {
                let midpoint_units = BigRational::from_integer(low.clone()).combine(&HALF, false);
                let base = BigRational::from_integer(BigInt::new(false, factor.clone()));
                let midpoint = if places < 0 {
                    midpoint_units.mul(&base)
                } else {
                    midpoint_units.div(&base)
                };
                if let Some(side) = self.compare_root(lo, hi, &midpoint, charge)? {
                    let use_low = side.is_lt() || (side.is_eq() && midpoint.sign().is_lt());
                    return Ok(Some(if use_low { low } else { high }));
                }
            }
            Ok(None)
        })
    }

    pub fn round(&self, places: i32, charge: &mut Charge<'_>) -> Result<Self, String> {
        if places.unsigned_abs() > 308 {
            return Err("Rounding precision is out of range".into());
        }
        charge(self.rounding_work(places))?;
        let factor = BigUint::from(10).pow(places.unsigned_abs());
        let units = self.rounded_units(places, &factor, charge)?;
        Self::checked(if places < 0 {
            BigRational::from_integer(units.scaled(&factor))
        } else {
            BigRational::new(units, factor)
        })
    }

    fn rounding_work(&self, places: i32) -> usize {
        let bits = self.width() + 4 * places.unsigned_abs() as usize;
        LIMB_BITS * bits.div_ceil(LIMB_BITS).pow(2)
    }

    /// Currency display rounds once to minor units, half away from zero.
    pub fn fixed(&self, places: u32) -> Result<String, String> {
        with_budget(|charge| self.fixed_checked(places, charge))
    }

    fn fixed_checked(&self, places: u32, charge: &mut Charge<'_>) -> Result<String, String> {
        if places > 308 {
            return Err("Rounding precision is out of range".into());
        }
        charge(self.rounding_work(places as i32))?;
        let factor = BigUint::from(10).pow(places);
        let units = self.rounded_units(places as i32, &factor, charge)?;
        let (whole, frac) = units.magnitude.div_rem(&factor);
        let sign = if units.negative { "-" } else { "" };
        if frac.is_zero() {
            Ok(format!("{sign}{whole}"))
        } else {
            let fraction = format!("{frac:0>width$}", width = places as usize);
            Ok(format!("{sign}{whole}.{}", fraction.trim_end_matches('0')))
        }
    }
}

fn significant_text(n: &BigRational) -> String {
    if n.sign().is_eq() {
        return "0".into();
    }
    let (numerator, denominator, mut exponent) = Number::significant_parts(n);
    let numerator = BigInt::new(n.sign().is_lt(), numerator);
    let rounded = BigRational::rounded_integer(&numerator, &denominator)
        .magnitude
        .to_string();
    exponent += rounded.len() as i32 - 12;
    let digits = rounded.trim_end_matches('0');
    let sign = if n.sign().is_lt() { "-" } else { "" };
    if !(-9..15).contains(&exponent) {
        let (head, tail) = digits.split_at(1);
        return if tail.is_empty() {
            format!("{sign}{head}e{exponent}")
        } else {
            format!("{sign}{head}.{tail}e{exponent}")
        };
    }
    if exponent < 0 {
        return format!("{sign}0.{}{digits}", "0".repeat((-exponent - 1) as usize));
    }
    let point = exponent as usize + 1;
    if point >= digits.len() {
        format!("{sign}{digits}{}", "0".repeat(point - digits.len()))
    } else {
        let (whole, fraction) = digits.split_at(point);
        format!("{sign}{whole}.{fraction}")
    }
}

impl Ord for Number {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.sort_key().cmp(&rhs.sort_key())
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl PartialEq for Number {
    fn eq(&self, rhs: &Self) -> bool {
        self.cmp(rhs).is_eq()
    }
}

impl Eq for Number {}

fn sqrt_work(width: usize) -> usize {
    4 * LIMB_BITS * width.div_ceil(LIMB_BITS).pow(2)
}

fn pair_work(a: usize, b: usize) -> usize {
    LIMB_BITS * (a.div_ceil(LIMB_BITS) + b.div_ceil(LIMB_BITS)).pow(2)
}

fn projection_work(lo: &BigRational, hi: &BigRational) -> usize {
    // Rational division, decimal digit counts and the projection's scale factor.
    8 * (lo.width().max(hi.width()).div_ceil(LIMB_BITS) + 2).pow(2)
}

fn exact_root_power(base: &BigRational, rhs: &BigRational) -> Option<(u32, i32, usize)> {
    let degree = u32::try_from(rhs.denominator.to_u64()?).ok()?;
    let width = base.width();
    // A component other than 1 needs at least degree + 1 bits to have an exact root.
    if [&base.numerator.magnitude, &base.denominator]
        .iter()
        .any(|n| !n.is_one() && n.bits() <= degree as usize)
    {
        return None;
    }
    let power = rhs.numerator.to_i32()?;
    let root_width = width.div_ceil(degree as usize);
    let bits = root_width.saturating_mul(power.unsigned_abs() as usize);
    if bits > MAX_BITS {
        return None;
    }
    let trial_work = integer_power_work(width + degree as usize);
    let root_work = sqrt_work(width).max(2 * root_width.min(degree as usize) * trial_work);
    let work = root_work + integer_power_work(bits);
    (work <= Number::MAX_ARITHMETIC_WORK).then_some((degree, power, work))
}

fn exact_work(a: &BigRational, op: &str, b: &BigRational) -> usize {
    let left = a.width().div_ceil(LIMB_BITS);
    let right = b.width().div_ceil(LIMB_BITS);
    let width = left + right;
    let integers = a.denominator.is_one() && b.denominator.is_one();
    match op {
        "+" | "-" if integers => width,
        "*" if integers => left * right,
        "==" | "!=" | "<" | ">" | "<=" | ">=" => width + 2 * left * right,
        "^" if b == &*HALF => sqrt_work(a.width()),
        "^" if !b.denominator.is_one() => {
            let root_work = exact_root_power(a, b).map_or(0, |(_, _, work)| work);
            32_768 + LIMB_BITS * (left * width + right * right) + root_work
        }
        "^" => {
            if let Some(exponent) = b.numerator.to_i32() {
                let bits = a.width().saturating_mul(exponent.unsigned_abs() as usize);
                if bits <= MAX_BITS {
                    return integer_power_work(bits);
                }
            }
            width
        }
        _ => pair_work(a.width(), b.width()),
    }
}

fn integer_pow_exact(base: &BigRational, exponent: &BigRational) -> Result<Number, String> {
    let width = base.width();
    if let Some(power) = exponent
        .numerator
        .to_i32()
        .filter(|power| width.saturating_mul(power.unsigned_abs() as usize) <= MAX_BITS)
    {
        return Number::checked(base.pow(power));
    }
    if width > 1 {
        return Err("Numeric power is out of range".into());
    }
    if base.numerator.negative && exponent.numerator.magnitude.is_odd() {
        Ok(Number::integer(-1))
    } else {
        Ok(Number::integer(i32::from(!base.sign().is_eq())))
    }
}

fn sqrt_at(n: &BigRational, bits: u32) -> Bounds {
    root_at(n, 2, bits)
}

fn root_at(n: &BigRational, degree: u32, bits: u32) -> Bounds {
    if n.sign().is_eq() {
        return Bounds::point(n.clone());
    }
    let (top, bottom) = (&n.numerator.magnitude, &n.denominator);
    let shift = top.binary_exponent(bottom).div_euclid(degree as i32) - bits as i32;
    let scale = degree as usize * shift.unsigned_abs() as usize;
    // Flooring before the integer root preserves the floor on this dyadic grid.
    let (scaled, remainder) = if shift < 0 {
        top.shl(scale).div_rem(bottom)
    } else {
        top.div_rem(&bottom.shl(scale))
    };
    let mantissa = scaled.integer_root(degree);
    let exact = remainder.is_zero() && mantissa.pow(degree) == scaled;
    let upper = if exact {
        mantissa.clone()
    } else {
        mantissa.add(&BigUint::one())
    };
    let lo = BigRational::dyadic(mantissa, shift);
    let hi = BigRational::dyadic(upper, shift);
    if n.sign().is_lt() {
        Bounds { lo: hi.negated(), hi: lo.negated() }
    } else {
        Bounds { lo, hi }
    }
}

fn certify_root_boundary(
    base: &BigRational,
    exponent: &BigRational,
    lo: &BigRational,
    hi: &BigRational,
    boundary: &BigRational,
    charge: &mut Charge<'_>,
) -> Result<Ordering, String> {
    if boundary < lo {
        return Ok(Ordering::Greater);
    }
    if boundary > hi {
        return Ok(Ordering::Less);
    }
    let (a, c) = (&base.numerator.magnitude, &base.denominator);
    let (a, c) = if exponent.sign().is_lt() {
        (c, a)
    } else {
        (a, c)
    };
    let bounded_exponent = |n: &BigUint| {
        n.to_u64()
            .and_then(|v| u32::try_from(v).ok())
            .ok_or(WORK_ERROR)
    };
    let power = bounded_exponent(&exponent.numerator.magnitude)?;
    let degree = bounded_exponent(&exponent.denominator)?;
    let (n, d) = (&boundary.numerator.magnitude, &boundary.denominator);
    let product_width = |base: &BigUint, boundary: &BigUint| {
        base.bits()
            .saturating_mul(power as usize)
            .saturating_add(boundary.bits().saturating_mul(degree as usize))
    };
    let bits = product_width(a, d).max(product_width(c, n));
    let work = integer_power_work(bits);
    if work > Number::MAX_ARITHMETIC_WORK {
        return Err(WORK_ERROR.into());
    }
    charge(work)?;
    Ok(a.pow(power)
        .mul(&d.pow(degree))
        .cmp(&c.pow(power).mul(&n.pow(degree))))
}

// First positive 12-significant-digit midpoint at or above the argument.
fn display_midpoint(lo: &BigRational, strict: bool) -> BigRational {
    let (scaled, denominator, mut exponent) = Number::significant_parts(lo);
    let (whole, remainder) = scaled.div_rem(&denominator);
    let mut whole = whole.to_u64().expect("12 significant digits");
    let side = remainder.shl(1).cmp(&denominator);
    if side.is_gt() || (strict && side.is_eq()) {
        whole += 1;
        if whole == 1_000_000_000_000 {
            whole /= 10;
            exponent += 1;
        }
    }
    BigRational::from_decimal_parts(BigInt::from(u128::from(whole) * 10 + 5), exponent - 12)
}

fn root_significant(
    bounds: &Bounds,
    base: &BigRational,
    exponent: &BigRational,
    negative: bool,
    charge: &mut Charge<'_>,
) -> Result<Option<String>, String> {
    let (lo, hi) = (&bounds.lo, &bounds.hi); // positive magnitudes
    if lo.sign().is_eq() {
        return Ok(None);
    }
    let midpoint = display_midpoint(lo, false);
    if midpoint > *hi || display_midpoint(&midpoint, true) <= *hi {
        return Ok(None);
    }
    let value = match certify_root_boundary(base, exponent, lo, hi, &midpoint, charge)? {
        Ordering::Less => lo,
        // If hi is the midpoint, the midpoint itself supplies the tie display.
        Ordering::Equal => &midpoint,
        Ordering::Greater => hi,
    };
    let text = significant_text(value);
    Ok(Some(if negative { format!("-{text}") } else { text }))
}

// None indicates the magnitude is already known to exceed 2^1024.
fn log_bounds(n: &BigRational, bits: u32, ln2: &(BigUint, BigUint)) -> Bounds {
    let (top, bottom) = (&n.numerator.magnitude, &n.denominator);
    let reciprocal = top < bottom;
    let (top, bottom) = if reciprocal { (bottom, top) } else { (top, bottom) };
    let exponent = top.binary_exponent(bottom) as usize;
    let scaled = bottom.shl(exponent);
    let mut z_top = top.clone();
    z_top.sub_assign(&scaled);
    let z_bottom = top.add(&scaled);
    let (series_lo, series_hi) = log_series(&z_top, &z_bottom, bits);
    let scale = z_bottom.shl(bits as usize);
    let scaled_exponent = z_bottom.mul(&BigUint::from(exponent as u128));
    // Keep z exact. Taking a reciprocal below one avoids cancellation against ln(2).
    let bound = |ln2: &BigUint, series: &BigUint| {
        BigRational::new(
            BigInt::new(false, ln2.mul(&scaled_exponent).add(&z_top.mul(series))),
            scale.clone(),
        )
    };
    let lo = bound(&ln2.0, &series_lo);
    let hi = bound(&ln2.1, &series_hi);
    if reciprocal {
        Bounds { lo: hi.negated(), hi: lo.negated() }
    } else {
        Bounds { lo, hi }
    }
}

fn exp_at(n: &BigRational, bits: u32, ln2: &(BigUint, BigUint)) -> Result<Option<Bounds>, String> {
    if n.sign().is_eq() {
        return Ok(Some(Bounds::point(BigRational::from_integer(1.into()))));
    }
    let scaled = n.numerator.magnitude.shl(bits as usize);
    let (lo, remainder) = scaled.div_rem(&n.denominator);
    if lo.is_zero() {
        // For -1 < x < 1, 1 + x <= exp(x) <= 1/(1 - x).
        // Keeping the exact linear term preserves cancellation below the fixed-point grid.
        let one = BigRational::from_integer(1.into());
        return Ok(Some(Bounds {
            lo: one.combine(n, false),
            hi: one.div(&one.combine(n, true)),
        }));
    }
    let hi = if remainder.is_zero() { lo.clone() } else { lo.add(&BigUint::one()) };
    exp_bounds(lo, hi, n.numerator.negative, bits, ln2, ExpMode::Exponential)
}

fn fractional_power_at(
    base: &BigRational,
    rhs: &BigRational,
    bits: u32,
) -> Result<Option<Bounds>, String> {
    let n = &base.numerator.magnitude;
    let d = &base.denominator;
    if n == d || rhs.sign().is_eq() {
        return Ok(Some(Bounds::point(BigRational::from_integer(1.into()))));
    }
    if n.is_zero() {
        return if rhs.sign().is_lt() {
            Err(DIVISION_BY_ZERO.into())
        } else {
            Ok(Some(Bounds::point(ZERO.clone())))
        };
    }
    let reciprocal = n < d;
    let (top, bottom) = if reciprocal { (d, n) } else { (n, d) };
    let exponent = top.binary_exponent(bottom);
    let scaled = bottom.shl(exponent as usize);
    let mut z_top = top.clone();
    z_top.sub_assign(&scaled);
    let z_bottom = top.add(&scaled);
    let (series_lo, series_hi) = log_series(&z_top, &z_bottom, bits);
    let ln2 = if bits == POWER_BITS { &*LN_2 } else { &ln2_bounds(bits) };
    let (ln2_lo, ln2_hi) = ln2;
    let scaled_exponent = z_bottom.mul(&BigUint::from(exponent as u128));
    let log_product = |ln2: &BigUint, series: &BigUint| {
        ln2.mul(&scaled_exponent)
            .add(&z_top.mul(series))
            .mul(&rhs.numerator.magnitude)
    };
    let denominator = z_bottom.mul(&rhs.denominator);
    let powered_lo = log_product(ln2_lo, &series_lo).div_rem(&denominator).0;
    let powered_hi = log_product(ln2_hi, &series_hi).div_ceil(&denominator);
    exp_bounds(
        powered_lo,
        powered_hi,
        reciprocal != rhs.sign().is_lt(),
        bits,
        ln2,
        ExpMode::FractionalPower,
    )
}

enum ExpMode {
    Exponential,
    FractionalPower,
}

fn exp_bounds(
    scaled_lo: BigUint,
    scaled_hi: BigUint,
    negative: bool,
    bits: u32,
    (ln2_lo, ln2_hi): &(BigUint, BigUint),
    mode: ExpMode,
) -> Result<Option<Bounds>, String> {
    // Reduce both endpoints by the same integer multiple of ln(2).
    let (whole, remainder_lo) = scaled_lo.div_rem(ln2_hi);
    let whole = whole.to_u64().unwrap_or(u64::MAX);
    if negative {
        match mode {
            ExpMode::FractionalPower if whole >= 1075 => {
                return Ok(Some(Bounds { lo: ZERO.clone(), hi: CUTOFF.clone() }));
            }
            // A denominator of 2^(whole + bits) needs whole + bits + 1 bits.
            ExpMode::Exponential if whole >= (MAX_BITS - bits as usize) as u64 => {
                return Err(PRECISION_ERROR.into());
            }
            _ => {}
        }
    } else if whole >= 1024 {
        return Ok(None);
    }
    let mut remainder_hi = scaled_hi;
    remainder_hi.sub_assign(&ln2_lo.mul(&BigUint::from(whole as u128)));
    let unit = BigUint::one().shl(bits as usize);
    if remainder_hi > unit {
        return Err(WORK_ERROR.into());
    }
    let exp_lo = exp_series(&remainder_lo, bits, false);
    let exp_hi = exp_series(&remainder_hi, bits, true);
    let (lo, hi) = if negative {
        let unit_squared = unit.shl(bits as usize);
        let shift = -(whole as i32) - bits as i32;
        (
            BigRational::dyadic(unit_squared.div_rem(&exp_hi).0, shift),
            BigRational::dyadic(unit_squared.div_ceil(&exp_lo), shift),
        )
    } else {
        let shift = whole as i32 - bits as i32;
        (
            BigRational::dyadic(exp_lo, shift),
            BigRational::dyadic(exp_hi, shift),
        )
    };
    Ok(Some(Bounds { lo, hi }))
}

impl Bounds {
    fn point(n: BigRational) -> Self {
        Self {
            lo: n.clone(),
            hi: n,
        }
    }

    fn width(&self) -> usize {
        self.lo.width().max(self.hi.width())
    }

    fn contains_zero(&self) -> bool {
        self.lo.sign().is_le() && self.hi.sign().is_ge()
    }

    fn in_range(&self, charge: &mut Charge<'_>) -> Result<bool, String> {
        let near_limit = |x: &BigRational| {
            x.numerator
                .magnitude
                .bits()
                .saturating_sub(x.denominator.bits())
                >= 1023
        };
        if !near_limit(&self.lo) && !near_limit(&self.hi) {
            return Ok(true);
        }
        charge(pair_work(self.width(), MAX_VALUE.width()))?;
        if self.lo > *MAX_VALUE || self.hi < *MIN_VALUE {
            return Err(OVERFLOW.into());
        }
        Ok(self.hi <= *MAX_VALUE && self.lo >= *MIN_VALUE)
    }

    fn compact(self, bits: u32, charge: &mut Charge<'_>) -> Result<Self, String> {
        if self.width() <= (bits + 64) as usize || self.lo == self.hi {
            return Ok(self);
        }
        // Keep a common dyadic grid relative to the largest magnitude.
        // A fixed scale around zero would erase very small valid results.
        charge(pair_work(
            self.width(),
            self.width().max((bits + 32) as usize),
        ))?;
        let scale = |x: &BigRational| {
            (!x.sign().is_eq()).then(|| x.numerator.magnitude.binary_exponent(&x.denominator))
        };
        let exponent = scale(&self.lo)
            .into_iter()
            .chain(scale(&self.hi))
            .max()
            .unwrap_or(0);
        let shift = exponent - (bits + 32) as i32;
        Ok(Self {
            lo: self.lo.round_dyadic(shift, true),
            hi: self.hi.round_dyadic(shift, false),
        })
    }

    fn unary(self, op: Unary, bits: u32, charge: &mut Charge<'_>) -> Result<Option<Self>, String> {
        let result = match op {
            Unary::Neg => Self {
                lo: self.hi.negated(),
                hi: self.lo.negated(),
            },
            Unary::Abs if self.lo.sign().is_ge() => self,
            Unary::Abs if self.hi.sign().is_le() => Self {
                lo: self.hi.negated(),
                hi: self.lo.negated(),
            },
            Unary::Abs => Self {
                lo: ZERO.clone(),
                hi: self.lo.negated().max(self.hi),
            },
            Unary::Sqrt => {
                if self.hi.sign().is_lt() {
                    return Err(DOMAIN.into());
                }
                if self.lo.sign().is_lt() {
                    return Ok(None);
                }
                charge(2 * sqrt_work(self.width().max(2 * bits as usize)))?;
                return Ok(Some(Self {
                    lo: sqrt_at(&self.lo, bits).lo,
                    hi: sqrt_at(&self.hi, bits).hi,
                }));
            }
            Unary::Cbrt => {
                charge(2 * sqrt_work(self.width().max(3 * bits as usize)))?;
                return Ok(Some(Self {
                    lo: root_at(&self.lo, 3, bits).lo,
                    hi: root_at(&self.hi, 3, bits).hi,
                }));
            }
            Unary::Log(base) => return self.logarithm(base, bits, charge),
            Unary::Exp => return self.exp(bits, charge),
            Unary::IntPow(power) => return self.int_pow(power, bits, charge),
        };
        Ok(Some(result))
    }

    fn logarithm(self, base: LogBase, bits: u32, charge: &mut Charge<'_>) -> Result<Option<Self>, String> {
        if self.hi.sign().is_le() {
            return Err(DOMAIN.into());
        }
        if self.lo.sign().is_le() {
            return Ok(None);
        }
        charge((3 + usize::from(matches!(base, LogBase::Ten))) * series_work(bits)
            + 2 * pair_work(self.width(), bits as usize))?;
        let ln2 = if bits == POWER_BITS { &*LN_2 } else { &ln2_bounds(bits) };
        let lower = log_bounds(&self.lo, bits, ln2);
        let hi = if self.lo == self.hi { lower.hi } else { log_bounds(&self.hi, bits, ln2).hi };
        let bounds = Self { lo: lower.lo, hi };
        let divisor = match base {
            LogBase::Natural => return Ok(Some(bounds.compact(bits, charge)?)),
            LogBase::Ten => log_bounds(&BigRational::from_integer(10.into()), bits, ln2),
            LogBase::Two => Self {
                lo: BigRational::dyadic(ln2.0.clone(), -(bits as i32)),
                hi: BigRational::dyadic(ln2.1.clone(), -(bits as i32)),
            },
        };
        bounds.binary(Binary::Div, &divisor, bits, charge)
    }

    fn exp(self, bits: u32, charge: &mut Charge<'_>) -> Result<Option<Self>, String> {
        if self.lo.sign().is_eq() && self.hi.sign().is_eq() {
            return Ok(Some(Self::point(BigRational::from_integer(1.into()))));
        }
        charge(5 * series_work(bits) + 2 * pair_work(self.width(), bits as usize))?;
        let ln2 = if bits == POWER_BITS { &*LN_2 } else { &ln2_bounds(bits) };
        let lower = exp_at(&self.lo, bits, ln2)?.ok_or(OVERFLOW)?;
        let hi = if self.lo == self.hi {
            lower.hi
        } else {
            let Some(upper) = exp_at(&self.hi, bits, ln2)? else {
                return Ok(None);
            };
            upper.hi
        };
        let bounds = Self { lo: lower.lo, hi };
        if bounds.width() > MAX_BITS {
            return Err(PRECISION_ERROR.into());
        }
        Ok(Some(bounds))
    }

    fn int_pow(
        mut self,
        power: i32,
        bits: u32,
        charge: &mut Charge<'_>,
    ) -> Result<Option<Self>, String> {
        if power == 0 {
            return Ok(Some(Self::point(BigRational::from_integer(1.into()))));
        }
        if power < 0 {
            if self.contains_zero() {
                return if self.lo == self.hi {
                    Err(DIVISION_BY_ZERO.into())
                } else {
                    Ok(None)
                };
            }
            charge(pair_work(self.width(), 1))?;
            let one = BigRational::from_integer(1.into());
            self = Self {
                lo: one.div(&self.hi),
                hi: one.div(&self.lo),
            };
        }
        let exponent = power.unsigned_abs();
        if exponent & 1 == 0 {
            self = self
                .unary(Unary::Abs, bits, charge)?
                .expect("absolute value");
        }
        // The positive power is monotone on this domain. Preserve that
        // correlation by powering the two ends separately.
        let power_endpoint = |n: BigRational, charge: &mut Charge<'_>| -> Result<Self, String> {
            let mut count = exponent;
            let mut base = Self::point(n);
            let mut result = Self::point(BigRational::from_integer(1.into()));
            while count != 0 {
                if count & 1 != 0 {
                    result = result
                        .products(&base, false, charge)?
                        .compact(bits, charge)?;
                }
                count >>= 1;
                if count != 0 {
                    base = base.products(&base, false, charge)?.compact(bits, charge)?;
                }
            }
            Ok(result)
        };
        // Reject excessive projected work before forming the first product.
        let width = self.width().saturating_mul(exponent as usize);
        if integer_power_work(width) > Number::MAX_ARITHMETIC_WORK {
            return Err(WORK_ERROR.into());
        }
        charge(2 * integer_power_work(width))?;
        Ok(Some(
            Self {
                lo: power_endpoint(self.lo, charge)?.lo,
                hi: power_endpoint(self.hi, charge)?.hi,
            }
            .compact(bits, charge)?,
        ))
    }

    fn products(&self, rhs: &Self, divide: bool, charge: &mut Charge<'_>) -> Result<Self, String> {
        charge(pair_work(self.width(), rhs.width()))?;
        let op = if divide {
            BigRational::div
        } else {
            BigRational::mul
        };
        let a = op(&self.lo, &rhs.lo);
        let b = op(&self.lo, &rhs.hi);
        let c = op(&self.hi, &rhs.lo);
        let d = op(&self.hi, &rhs.hi);
        let mut values = [a, b, c, d];
        values.sort_unstable();
        let [lo, _, _, hi] = values;
        Ok(Self { lo, hi })
    }

    fn binary(
        &self,
        op: Binary,
        rhs: &Self,
        bits: u32,
        charge: &mut Charge<'_>,
    ) -> Result<Option<Self>, String> {
        let result = match op {
            Binary::Add | Binary::Sub => {
                charge(pair_work(self.width(), rhs.width()))?;
                if matches!(op, Binary::Add) {
                    Self {
                        lo: self.lo.combine(&rhs.lo, false),
                        hi: self.hi.combine(&rhs.hi, false),
                    }
                } else {
                    Self {
                        lo: self.lo.combine(&rhs.hi, true),
                        hi: self.hi.combine(&rhs.lo, true),
                    }
                }
            }
            Binary::Mul => self.products(rhs, false, charge)?,
            Binary::Div | Binary::Rem => {
                if rhs.contains_zero() {
                    return if rhs.lo == rhs.hi {
                        Err(DIVISION_BY_ZERO.into())
                    } else {
                        Ok(None)
                    };
                }
                let quotient = self.products(rhs, true, charge)?;
                if matches!(op, Binary::Div) {
                    quotient
                } else {
                    charge(projection_work(&quotient.lo, &quotient.hi))?;
                    let truncate = |n: &BigRational| n.directed_integer(n.sign().is_ge());
                    let units = truncate(&quotient.lo);
                    if units != truncate(&quotient.hi) {
                        return Ok(None);
                    }
                    let product = rhs.products(
                        &Self::point(BigRational::from_integer(units)),
                        false,
                        charge,
                    )?;
                    charge(pair_work(self.width(), product.width()))?;
                    Self {
                        lo: self.lo.combine(&product.hi, true),
                        hi: self.hi.combine(&product.lo, true),
                    }
                }
            }
            Binary::FracPow | Binary::Pow => return self.real_pow(rhs, bits, charge),
        };
        Ok(Some(result.compact(bits, charge)?))
    }

    fn real_pow(
        &self,
        rhs: &Self,
        bits: u32,
        charge: &mut Charge<'_>,
    ) -> Result<Option<Self>, String> {
        if rhs.lo == rhs.hi && rhs.lo.denominator.is_one() {
            let power = rhs
                .lo
                .numerator
                .to_i32()
                .ok_or("Numeric power is out of range")?;
            return self.clone().int_pow(power, bits, charge);
        }
        if self.hi.sign().is_lt() {
            charge(projection_work(&rhs.lo, &rhs.hi))?;
            if rhs
                .lo
                .directed_integer(false)
                .cmp(&rhs.hi.directed_integer(true))
                .is_gt()
            {
                return Err(DOMAIN.into());
            }
            return Ok(None);
        }
        if self.lo.sign().is_lt() {
            return Ok(None);
        }
        if self.lo.sign().is_eq() && rhs.lo.sign().is_le() {
            if self.lo == self.hi && rhs.hi.sign().is_lt() {
                return Err(DIVISION_BY_ZERO.into());
            }
            return Ok(None);
        }
        // log(x)*y attains both extrema at corners of the rectangle.
        let count = if rhs.lo == rhs.hi { 2 } else { 4 };
        charge(fractional_work(self.width(), rhs.width(), bits) * count)?;
        let mut result: Option<Self> = None;
        let mut overflow = 0;
        for (base, power) in [
            (&self.lo, &rhs.lo),
            (&self.hi, &rhs.lo),
            (&self.lo, &rhs.hi),
            (&self.hi, &rhs.hi),
        ]
        .into_iter()
        .take(count)
        {
            if let Some(bounds) = fractional_power_at(base, power, bits)? {
                result = Some(match result {
                    None => bounds,
                    Some(prev) => Self {
                        lo: prev.lo.min(bounds.lo),
                        hi: prev.hi.max(bounds.hi),
                    },
                });
            } else {
                overflow += 1;
            }
        }
        if overflow == count {
            return Err(OVERFLOW.into());
        }
        if overflow != 0 {
            return Ok(None);
        }
        let result = result.expect("finite corner");
        if result.hi <= *CUTOFF {
            return Ok(Some(Self::point(ZERO.clone())));
        }
        Ok((result.lo > *CUTOFF).then_some(result))
    }
}

fn series_work(bits: u32) -> usize {
    let limbs = (bits as usize).div_ceil(LIMB_BITS);
    (bits as usize / 3 + 4) * (limbs + 1).pow(2)
}

fn fractional_work(base: usize, exponent: usize, bits: u32) -> usize {
    // Two log series and two exponentials, which terminate sooner for x <= 1.
    4 * series_work(bits) + 4 * pair_work(base + bits as usize, exponent)
}

impl ApproxExpr {
    fn same_expr(&self, rhs: &Self) -> bool {
        // Both DAGs have at most MAX_EXPR_NODES nodes. Compare each pair only
        // once, even when multiple paths reach the same shared subexpressions.
        let mut seen = HashSet::new();
        let mut pending = vec![(self, rhs)];
        while let Some((a, b)) = pending.pop() {
            if std::ptr::eq(a, b) || !seen.insert((a as *const Self, b as *const Self)) {
                continue;
            }
            match (&a.kind, &b.kind) {
                (ExprKind::Exact(a), ExprKind::Exact(b)) if a == b => {}
                (ExprKind::Sqrt(a), ExprKind::Sqrt(b))
                | (ExprKind::Cbrt(a), ExprKind::Cbrt(b))
                | (ExprKind::Ln(a), ExprKind::Ln(b))
                | (ExprKind::Log10(a), ExprKind::Log10(b))
                | (ExprKind::Log2(a), ExprKind::Log2(b))
                | (ExprKind::Exp(a), ExprKind::Exp(b))
                | (ExprKind::Neg(a), ExprKind::Neg(b))
                | (ExprKind::Abs(a), ExprKind::Abs(b)) => pending.push((a, b)),
                (ExprKind::IntPow(a, n), ExprKind::IntPow(b, m)) if n == m => {
                    pending.push((a, b));
                }
                (ExprKind::FracPow(a, b), ExprKind::FracPow(c, d))
                | (ExprKind::Add(a, b), ExprKind::Add(c, d))
                | (ExprKind::Sub(a, b), ExprKind::Sub(c, d))
                | (ExprKind::Mul(a, b), ExprKind::Mul(c, d))
                | (ExprKind::Div(a, b), ExprKind::Div(c, d))
                | (ExprKind::Rem(a, b), ExprKind::Rem(c, d))
                | (ExprKind::Pow(a, b), ExprKind::Pow(c, d)) => {
                    pending.push((a, c));
                    pending.push((b, d));
                }
                _ => return false,
            }
        }
        true
    }

    fn bounded(kind: ExprKind) -> Option<Arc<Self>> {
        let root = Arc::new(Self { kind });
        let mut seen = HashSet::new();
        let mut pending = vec![&root];
        while let Some(node) = pending.pop() {
            if !seen.insert(Arc::as_ptr(node)) {
                continue;
            }
            if seen.len() > MAX_EXPR_NODES {
                return None;
            }
            match &node.kind {
                ExprKind::Exact(_) => {}
                ExprKind::Sqrt(input)
                | ExprKind::Cbrt(input)
                | ExprKind::Ln(input)
                | ExprKind::Log10(input)
                | ExprKind::Log2(input)
                | ExprKind::Exp(input)
                | ExprKind::Neg(input)
                | ExprKind::Abs(input)
                | ExprKind::IntPow(input, _) => pending.push(input),
                ExprKind::FracPow(left, right)
                | ExprKind::Add(left, right)
                | ExprKind::Sub(left, right)
                | ExprKind::Mul(left, right)
                | ExprKind::Div(left, right)
                | ExprKind::Rem(left, right)
                | ExprKind::Pow(left, right) => pending.extend([left, right]),
            }
        }
        Some(root)
    }

    fn direct_root(&self, negative: bool) -> Option<(&BigRational, &BigRational, bool)> {
        let exact = |node: &ApproxExpr| matches!(node.kind, ExprKind::Exact(_));
        match &self.kind {
            ExprKind::Sqrt(base) if exact(base) => {
                let ExprKind::Exact(base) = &base.kind else {
                    unreachable!()
                };
                Some((base, &HALF, negative))
            }
            ExprKind::FracPow(base, exponent) => match (&base.kind, &exponent.kind) {
                (ExprKind::Exact(base), ExprKind::Exact(exponent)) => {
                    Some((base, exponent, negative))
                }
                _ => None,
            },
            ExprKind::Neg(inner) => inner.direct_root(!negative),
            ExprKind::Abs(inner) => inner
                .direct_root(false)
                .map(|(base, power, _)| (base, power, negative)),
            _ => None,
        }
    }

    fn evaluate(self: &Arc<Self>, bits: u32, charge: &mut Charge<'_>) -> Result<Option<Bounds>, String> {
        self.eval_at(bits, charge, &mut HashMap::new())
    }

    fn eval_at(
        self: &Arc<Self>,
        bits: u32,
        charge: &mut Charge<'_>,
        memo: &mut HashMap<usize, Option<Bounds>>,
    ) -> Result<Option<Bounds>, String> {
        let key = Arc::as_ptr(self) as usize;
        if let Some(bounds) = memo.get(&key) {
            return Ok(bounds.clone());
        }
        let bounds = self.evaluate_node(bits, charge, memo)?;
        memo.insert(key, bounds.clone());
        Ok(bounds)
    }

    fn evaluate_node(
        &self,
        bits: u32,
        charge: &mut Charge<'_>,
        memo: &mut HashMap<usize, Option<Bounds>>,
    ) -> Result<Option<Bounds>, String> {
        let unary =
            |input: &Arc<Self>, op, charge: &mut Charge<'_>, memo: &mut _| match input.eval_at(bits, charge, memo)? {
                Some(bounds) => bounds.unary(op, bits, charge),
                None => Ok(None),
            };
        let binary = |left: &Arc<Self>, op, right: &Arc<Self>, charge: &mut Charge<'_>, memo: &mut _| match (
            left.eval_at(bits, charge, memo)?,
            right.eval_at(bits, charge, memo)?,
        ) {
            (Some(a), Some(b)) => a.binary(op, &b, bits, charge),
            _ => Ok(None),
        };
        let candidate = match &self.kind {
            ExprKind::Exact(n) => {
                charge(4 * n.width().div_ceil(LIMB_BITS))?;
                return Ok(Some(Bounds::point(n.clone())));
            }
            ExprKind::Sqrt(n) => unary(n, Unary::Sqrt, charge, memo)?,
            ExprKind::Cbrt(n) => unary(n, Unary::Cbrt, charge, memo)?,
            ExprKind::Ln(n) => unary(n, Unary::Log(LogBase::Natural), charge, memo)?,
            ExprKind::Log10(n) => unary(n, Unary::Log(LogBase::Ten), charge, memo)?,
            ExprKind::Log2(n) => unary(n, Unary::Log(LogBase::Two), charge, memo)?,
            ExprKind::Exp(n) => unary(n, Unary::Exp, charge, memo)?,
            ExprKind::FracPow(_, _) if self.direct_root(false).is_some() => {
                let (base, exponent, _) = self.direct_root(false).expect("direct root");
                charge(fractional_work(base.width(), exponent.width(), bits))?;
                let mut bounds = fractional_power_at(base, exponent, bits)?.ok_or(OVERFLOW)?;
                if certify_root_boundary(base, exponent, &bounds.lo, &bounds.hi, &CUTOFF, charge)?
                    .is_le()
                {
                    return Ok(Some(Bounds::point(ZERO.clone())));
                }
                match certify_root_boundary(
                    base, exponent, &bounds.lo, &bounds.hi, &MAX_VALUE, charge,
                )? {
                    Ordering::Greater => return Err(OVERFLOW.into()),
                    Ordering::Equal => return Ok(Some(Bounds::point(MAX_VALUE.clone()))),
                    Ordering::Less => bounds.hi = bounds.hi.min(MAX_VALUE.clone()),
                }
                return Ok(Some(bounds));
            }
            ExprKind::FracPow(a, b) => binary(a, Binary::FracPow, b, charge, memo)?,
            ExprKind::Neg(n) => unary(n, Unary::Neg, charge, memo)?,
            ExprKind::Abs(n) => unary(n, Unary::Abs, charge, memo)?,
            ExprKind::IntPow(n, power) => unary(n, Unary::IntPow(*power), charge, memo)?,
            ExprKind::Add(a, b) => binary(a, Binary::Add, b, charge, memo)?,
            ExprKind::Sub(a, b) => binary(a, Binary::Sub, b, charge, memo)?,
            ExprKind::Mul(a, b) => binary(a, Binary::Mul, b, charge, memo)?,
            ExprKind::Div(a, b) => binary(a, Binary::Div, b, charge, memo)?,
            ExprKind::Rem(a, b) => binary(a, Binary::Rem, b, charge, memo)?,
            ExprKind::Pow(a, b) => binary(a, Binary::Pow, b, charge, memo)?,
        };
        match candidate {
            Some(bounds) if bounds.in_range(charge)? => Ok(Some(bounds)),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unmetered(_: usize) -> Result<(), String> {
        Ok(())
    }

    #[test]
    fn exact_projections_and_remainder_precharge_arithmetic() {
        let n = Number::decimal("3.14e-1000").unwrap();
        let one = Number::integer(1);
        let mut costs = Vec::new();
        let mut reject = |work| {
            costs.push(work);
            Err("meter rejected work".into())
        };
        assert_eq!(n.significant_checked(&mut reject).unwrap_err(), "meter rejected work");
        assert_eq!(n.float_checked(&mut reject).unwrap_err(), "meter rejected work");
        assert_eq!(n.fixed_checked(2, &mut reject).unwrap_err(), "meter rejected work");
        assert_eq!(n.rem(&one, &mut reject).unwrap_err(), "meter rejected work");
        let root = one.add(&one).unwrap().sqrt(&mut unmetered).unwrap();
        assert_eq!(root.significant_checked(&mut reject).unwrap_err(), "meter rejected work");
        let exact = n.exact().unwrap();
        assert_eq!(costs, [
            n.unary_work("format"),
            projection_work(exact, exact),
            n.rounding_work(2),
            exact_work(exact, "%", one.exact().unwrap()),
            root.unary_work("format"),
        ]);
    }

    #[test]
    fn tiny_exponential_bounds_separate_from_one_before_refinement() {
        let one = Number::integer(1);
        for (literal, side) in [("1e-1000", Ordering::Greater), ("-1e-1000", Ordering::Less)] {
            let input = Number::decimal(literal).unwrap();
            let exp = with_budget(|charge| input.exp(charge)).unwrap();
            assert_eq!(exp.bits(), POWER_BITS);
            assert_eq!(exp.lower().cmp(one.lower()), side);
            assert_eq!(exp.upper().cmp(one.upper()), side);
        }
    }

    #[test]
    fn overlapping_number_order_refines_before_using_endpoints() {
        let root = Number::integer(2).sqrt(&mut unmetered).unwrap();
        let larger = root.mul(&root).unwrap().mul(&root).unwrap()
            .div(&Number::integer(2)).unwrap()
            .add(&Number::decimal("1e-100").unwrap()).unwrap();
        assert!(root.lower() > larger.lower());
        assert_eq!(root.cmp(&larger), Ordering::Less);
        assert_eq!(larger.cmp(&root), Ordering::Greater);
        assert_eq!(root.cmp(&root.clone()), Ordering::Equal);
    }

    #[test]
    fn limb_carries_and_wide_division() {
        let max = BigUint::from(u128::MAX);
        let square = max.mul(&max);
        assert_eq!(
            square.0,
            [1, 0, 0, 0, u32::MAX - 1, u32::MAX, u32::MAX, u32::MAX]
        );
        let dividend = square.add(&BigUint::one());
        assert_eq!(dividend.div_rem(&max), (max, BigUint::one()));
    }

    #[test]
    fn exact_rational_operations_preserve_sign_and_reduce() {
        let left = Number::decimal("-5.5").unwrap();
        let right = Number::decimal("0.75").unwrap();
        assert_eq!(
            left.rem(
                &right.clone().neg_checked(&mut unmetered).unwrap(),
                &mut unmetered
            )
            .unwrap(),
            Number::decimal("-0.25").unwrap()
        );
        assert_eq!(left.floor(&mut unmetered).unwrap(), Number::integer(-6));
        assert_eq!(left.ceil(&mut unmetered).unwrap(), Number::integer(-5));
        assert_eq!(left.round(0, &mut unmetered).unwrap(), Number::integer(-6));
        assert_eq!(
            Number::integer(150).round(-2, &mut unmetered).unwrap(),
            Number::integer(200)
        );

        let numerator = Number::integer(u128::MAX);
        let denominator = Number::decimal("340282366920938463463374607431768211457").unwrap();
        let ratio = numerator.div(&denominator).unwrap();
        assert!(ratio < Number::integer(1));
        assert_eq!(ratio.mul(&denominator).unwrap(), numerator);
        assert_eq!(
            ratio.pow(&Number::integer(-1), &mut unmetered).unwrap(),
            denominator.div(&numerator).unwrap()
        );
    }

    #[test]
    fn fractional_powers_preserve_tiny_bases_and_binary64_underflow() {
        let decimal = |text| Number::decimal(text).unwrap();
        let base = decimal("1e-400");
        assert_eq!(
            base.pow(&decimal("0.5"), &mut unmetered).unwrap(),
            decimal("1e-200")
        );
        assert_eq!(
            base.pow(&decimal("0.5"), &mut unmetered),
            base.sqrt(&mut unmetered)
        );

        let base = decimal("16e-800");
        for (power, expected) in [("0.25", "2e-200"), ("-0.25", "5e199")] {
            assert_eq!(
                base.pow(&decimal(power), &mut unmetered)
                    .unwrap()
                    .significant_text()
                    .unwrap(),
                expected
            );
        }

        let base = decimal("1e-430");
        assert_eq!(
            base.pow(&decimal("0.75"), &mut unmetered)
                .unwrap()
                .try_as_f64()
                .unwrap()
                .to_bits(),
            6
        );
        assert_eq!(
            base.pow(&decimal("-0.75"), &mut unmetered).unwrap_err(),
            "Numeric overflow"
        );
        assert!(decimal("1e-500")
            .pow(&decimal("0.75"), &mut unmetered)
            .unwrap()
            .is_zero());
        assert_eq!(
            decimal("1e-647")
                .pow(&decimal("0.5"), &mut unmetered)
                .unwrap()
                .try_as_f64()
                .unwrap()
                .to_bits(),
            1
        );
    }

    #[test]
    fn fractional_powers_resolve_overflow_and_display_boundaries() {
        let decimal = |text| Number::decimal(text).unwrap();
        for (base, power, expected) in [
            (
                "5.410860736180795545311869365348484425069e-423",
                "-0.73",
                "1.79769313486e308",
            ),
            ("1.00000000001999999999999e-800", "0.25", "1e-200"),
            ("9.999999999799996e-801", "-0.25", "1.00000000001e200"),
        ] {
            let result = decimal(base).pow(&decimal(power), &mut unmetered).unwrap();
            assert_eq!(result.significant_text().unwrap(), expected);
        }
        assert_eq!(
            decimal("4.326339818212956088414698819550503504786e-441")
                .pow(&decimal("-0.7"), &mut unmetered),
            Err("Numeric overflow".into())
        );
        assert_eq!(
            decimal("1e-700").sqrt(&mut unmetered).unwrap(),
            decimal("1e-350")
        );

        let max = Number::from_float(f64::MAX).unwrap();
        let inverse_square = max.pow(&Number::integer(-2), &mut unmetered).unwrap();
        assert_eq!(
            inverse_square
                .pow(&decimal("-0.5"), &mut unmetered)
                .unwrap(),
            max
        );

        let midpoint = decimal("1.000000000005e-200");
        let fourth_power = midpoint.pow(&Number::integer(4), &mut unmetered).unwrap();
        let root = fourth_power.pow(&decimal("0.25"), &mut unmetered).unwrap();
        assert_eq!(root, midpoint);
        assert_eq!(root.significant_text().unwrap(), "1.00000000001e-200");
    }

    #[test]
    fn fractional_power_keeps_small_changes_to_one_before_exponent_multiplication() {
        let base = Number::integer(1)
            .add(&Number::decimal("1e-300").unwrap())
            .unwrap();
        let power = Number::decimal("1e300")
            .unwrap()
            .add(&Number::decimal("0.5").unwrap())
            .unwrap();
        assert_eq!(
            base.pow(&power, &mut unmetered)
                .unwrap()
                .significant_text()
                .unwrap(),
            "2.71828182846"
        );
    }

    #[test]
    fn fractional_power_certifies_boundaries_past_degree_and_power_64() {
        let decimal = |text| Number::decimal(text).unwrap();
        let power = decimal("32.5");
        let below_max = decimal(concat!(
            "30532367336933791742514340671612505127314849033994312715",
            "97473263850344937621217891430236404319521766464136496148291e-105"
        ));
        let result = below_max.pow(&power, &mut unmetered).unwrap();
        assert_eq!(result.significant_text().unwrap(), "1.79769313486e308");
        assert_eq!(result.try_as_f64().unwrap(), f64::MAX);
        let above_max = below_max.add(&decimal("1e-105")).unwrap();
        assert_eq!(
            above_max.pow(&power, &mut unmetered),
            Err("Numeric overflow".into())
        );

        let above_midpoint = decimal(concat!(
            "10065047886021074257997915525324881917220220969117164472",
            "93354855878436635445721833888276777024126582885656e-105"
        ));
        assert_eq!(
            above_midpoint
                .pow(&power, &mut unmetered)
                .unwrap()
                .significant_text()
                .unwrap(),
            "1.23456789013"
        );
        let mut projected_work = 0;
        assert_eq!(
            above_midpoint.pow(&power, &mut |work| {
                projected_work = work;
                Err(WORK_ERROR.into())
            }),
            Err(WORK_ERROR.into())
        );
        assert_eq!(above_midpoint.arithmetic_work("^", &power), 548_128);
        assert_eq!(projected_work, 1_988_100);
        let base = decimal("1.01568603515625");
        for exp in [decimal("1000.5"), decimal("-1000.5")] {
            assert_eq!(base.arithmetic_work("^", &exp), 1_036_996);
        }

        let midpoint = decimal("1.234567890125");
        let raised = midpoint.pow(&Number::integer(65), &mut unmetered).unwrap();
        let root_power = Number::integer(1).div(&Number::integer(65)).unwrap();
        let exact = raised.pow(&root_power, &mut unmetered).unwrap();
        assert_eq!(exact, midpoint);
        assert_eq!(exact.significant_text().unwrap(), "1.23456789013");
        let change = decimal("1e-820");
        for (base, expected) in [
            (raised.sub(&change).unwrap(), "1.23456789012"),
            (raised.add(&change).unwrap(), "1.23456789013"),
        ] {
            assert_eq!(
                base.pow(&root_power, &mut unmetered)
                    .unwrap()
                    .significant_text()
                    .unwrap(),
                expected
            );
        }
    }

    #[test]
    fn fractional_power_fails_closed_when_ambiguous_comparison_is_too_wide() {
        let power = Number::integer(1013).div(&Number::integer(1009)).unwrap();
        let near_midpoint = Number::decimal(concat!(
            "1.2335410739076564699932459548504070399223374719787144347",
            "555590387151253888469164856148859400725906489631119716"
        ))
        .unwrap();
        let near_max = Number::decimal(concat!(
            "1.0902352930376362171174852998230615631612151150466090958",
            "259538460652255075836540357377481368561842514457122180e307"
        ))
        .unwrap();
        for base in [&near_midpoint, &near_max] {
            // The operation caps the comparison even without a caller-supplied budget.
            assert_eq!(base.pow(&power, &mut unmetered), Err(WORK_ERROR.into()));
        }
        let outside_interval = near_midpoint
            .add(&Number::decimal("1e-55").unwrap())
            .unwrap();
        let mut reject_comparison = |_| panic!("a separated midpoint needs no comparison");
        assert_eq!(
            outside_interval
                .pow(&power, &mut reject_comparison)
                .unwrap()
                .significant_text()
                .unwrap(),
            "1.23456789013"
        );
    }

    #[test]
    fn exponential_respects_reduced_endpoint_width() {
        let value = Number::decimal("-11201").unwrap().exp(&mut unmetered).unwrap();
        assert!(!value.is_exact());
        assert!(value.lower().sign().is_gt());
        assert_eq!(value.lower().width().max(value.upper().width()), MAX_BITS);
        assert_eq!(
            Number::decimal("-11201.3").unwrap().exp(&mut unmetered),
            Err(PRECISION_ERROR.into())
        );

        // Below the fixed-point grid, the reduced lower bound 1 + input
        // has a MAX_BITS-bit denominator but a (MAX_BITS + 1)-bit numerator.
        let mut denominator = BigUint::one().shl(MAX_BITS);
        denominator.sub_assign(&BigUint::one());
        let input = Number::from_ratio(BigRational {
            numerator: 1.into(),
            denominator,
        });
        assert_eq!(input.width(), MAX_BITS);
        assert_eq!(input.exp(&mut unmetered), Err(PRECISION_ERROR.into()));
    }

    #[test]
    fn fractional_power_certifies_binary64_subnormal_projection_and_cutoff() {
        let decimal = |text| Number::decimal(text).unwrap();
        let center = decimal("0.5")
            .pow(&Number::integer(645), &mut unmetered)
            .unwrap();
        let delta = decimal("1e-400");
        let exponent = Number::integer(5).div(&Number::integer(3)).unwrap();
        let above = center
            .add(&delta)
            .unwrap()
            .pow(&exponent, &mut unmetered)
            .unwrap();
        assert!(!above.is_exact());
        assert_eq!(above.significant_text().unwrap(), "2.47032822921e-324");
        assert_eq!(above.try_as_f64().unwrap().to_bits(), 1);
        let negative = above.neg_checked(&mut unmetered).unwrap();
        assert_eq!(negative.try_as_f64().unwrap().to_bits(), (1u64 << 63) | 1);
        for base in [center.sub(&delta).unwrap(), center] {
            let result = base.pow(&exponent, &mut unmetered).unwrap();
            assert_eq!(result, Number::integer(0));
            assert_eq!(result.try_as_f64().unwrap().to_bits(), 0);
        }
    }

    #[test]
    fn approximate_subtraction_refines_before_display_and_projection() {
        let decimal = |text| Number::decimal(text).unwrap();
        let a = decimal("1.234567890124");
        let squared = a.mul(&a).unwrap();
        let delta = decimal("1e-100");
        for (input, expected, binary) in [
            (
                squared.add(&delta).unwrap(),
                "4.05000003645e-101",
                4.0500000364482186e-101,
            ),
            (
                squared.sub(&delta).unwrap(),
                "-4.05000003645e-101",
                -4.0500000364482186e-101,
            ),
        ] {
            let root = input.sqrt(&mut unmetered).unwrap();
            let result = root.sub(&a).unwrap();
            assert_eq!(result.significant_text().unwrap(), expected);
            assert_eq!(result.try_as_f64().unwrap(), binary);
            assert_eq!(
                result
                    .cmp_checked(&Number::integer(0), &mut unmetered)
                    .unwrap()
                    .is_gt(),
                binary > 0.0
            );
        }
        let root = Number::integer(2).sqrt(&mut unmetered).unwrap();
        let cancellation = root.sub(&root).unwrap();
        assert_eq!(cancellation.significant_text(), Err(WORK_ERROR.into()));
        assert_eq!(cancellation.try_as_f64(), Err(WORK_ERROR.into()));
        assert_eq!(cancellation.sign(&mut unmetered), Err(WORK_ERROR.into()));
        assert_eq!(
            Number::integer(1).div(&cancellation),
            Err(WORK_ERROR.into())
        );
        assert_eq!(root.rem(&root, &mut unmetered), Err(WORK_ERROR.into()));
    }

    #[test]
    fn roots_certify_fixed_rounding_and_compound_limits() {
        let decimal = |text| Number::decimal(text).unwrap();
        let delta = decimal("1e-90");
        for (center, offset) in [("0.005", 0), ("1.005", 1)] {
            let center = decimal(center);
            let square = center.mul(&center).unwrap();
            let offset = Number::integer(offset);
            for (radicand, expected) in [
                (square.sub(&delta).unwrap(), "0"),
                (square.clone(), "0.01"),
                (square.add(&delta).unwrap(), "0.01"),
            ] {
                let root = radicand.sqrt(&mut unmetered).unwrap().sub(&offset).unwrap();
                assert_eq!(root.fixed(2).unwrap(), expected);
                let negated = root.neg_checked(&mut unmetered).unwrap();
                assert_eq!(
                    negated.fixed(2).unwrap(),
                    if expected == "0" { "0" } else { "-0.01" }
                );
            }
        }
        let root = Number::integer(2).sqrt(&mut unmetered).unwrap();
        let uncertain_tie = root.sub(&root).unwrap().add(&decimal("0.005")).unwrap();
        assert_eq!(uncertain_tie.fixed(2), Err(WORK_ERROR.into()));
        assert_eq!(
            uncertain_tie.round(2, &mut unmetered),
            Err(WORK_ERROR.into())
        );
        let limit = Number::from_float(f64::MAX).unwrap();
        let center = decimal("1.001");
        let radicand = center.mul(&center).unwrap();
        for (input, overflow) in [
            (radicand.add(&delta).unwrap(), true),
            (radicand.sub(&delta).unwrap(), false),
        ] {
            let side = input.sqrt(&mut unmetered).unwrap().sub(&center).unwrap();
            let result = limit.add(&side);
            if overflow {
                assert_eq!(result, Err(OVERFLOW.into()));
            } else {
                assert_eq!(result.unwrap().try_as_f64().unwrap(), f64::MAX);
            }
        }
    }

    #[test]
    fn refinement_witness_cap_fails_closed_on_ambiguous_values() {
        let root = Number::integer(2).sqrt(&mut unmetered).unwrap();
        let mut sum = Number::integer(0);
        for _ in 0..MAX_EXPR_NODES {
            sum = sum.add(&root).unwrap();
        }
        assert!(sum.expr().is_none());
        assert_eq!(sum.significant_text().unwrap(), "181.019335984");
        assert_eq!(
            sum.sub(&sum).unwrap().significant_text(),
            Err(WORK_ERROR.into())
        );
    }

    #[test]
    fn interval_order_is_transitive_without_a_refinement_witness() {
        let two = Number::integer(2);
        let a = two.sqrt(&mut unmetered).unwrap();
        let c = a.mul(&a).unwrap().mul(&a).unwrap().div(&two).unwrap()
            .add(&Number::decimal("1e-100").unwrap()).unwrap();
        let mut b = a.add(&c).unwrap().div(&two).unwrap();
        for _ in 0..64 {
            b = b.add(&Number::integer(1)).unwrap();
        }
        let b = b.sub(&Number::integer(64)).unwrap();
        assert!(b.expr().is_none());
        assert!(a.lower() <= b.upper() && b.lower() <= a.upper());
        assert!(c.lower() <= b.upper() && b.lower() <= c.upper());
        let numbers = [&a, &b, &c];
        for x in numbers {
            for y in numbers {
                for z in numbers {
                    assert!(!(x <= y && y <= z) || x <= z);
                }
            }
        }
    }

    #[test]
    fn shared_interval_witnesses_replay_within_the_arithmetic_budget() {
        let one = Number::integer(1);
        let near_one = one.add(&Number::decimal("1e-100").unwrap()).unwrap();
        let mut delta = near_one.sqrt(&mut unmetered).unwrap().sub(&one).unwrap();
        for _ in 0..12 {
            delta = delta.add(&delta).unwrap();
        }
        let value = delta.div(&Number::integer(4096)).unwrap();
        assert_eq!(value.significant_text().unwrap(), "5e-101");
    }

    #[test]
    fn float_conversion_rounds_ties_to_even_at_binary_boundaries() {
        let half_ulp = Number::from_float(f64::EPSILON / 2.0).unwrap();
        let next = Number::from_float(f64::from_bits(1.0f64.to_bits() + 1)).unwrap();
        let previous = Number::from_float(f64::from_bits(2.0f64.to_bits() - 1)).unwrap();
        assert_eq!(
            Number::integer(1)
                .add(&half_ulp)
                .unwrap()
                .try_as_f64()
                .unwrap(),
            1.0
        );
        assert_eq!(
            next.add(&half_ulp).unwrap().try_as_f64().unwrap().to_bits(),
            1.0f64.to_bits() + 2
        );
        assert_eq!(previous.add(&half_ulp).unwrap().try_as_f64().unwrap(), 2.0);

        let min = Number::from_float(f64::from_bits(1)).unwrap();
        let half_min = min.div(&Number::integer(2)).unwrap();
        assert_eq!(
            half_min
                .clone()
                .neg_checked(&mut unmetered)
                .unwrap()
                .try_as_f64()
                .unwrap()
                .to_bits(),
            (-0.0f64).to_bits()
        );
        assert_eq!(
            min.add(&half_min).unwrap().try_as_f64().unwrap().to_bits(),
            2
        );
        let largest_subnormal =
            Number::from_float(f64::from_bits(f64::MIN_POSITIVE.to_bits() - 1)).unwrap();
        assert_eq!(
            largest_subnormal
                .add(&half_min)
                .unwrap()
                .try_as_f64()
                .unwrap(),
            f64::MIN_POSITIVE
        );
    }

    #[test]
    fn float_inputs_are_exact_and_bounds_are_checked() {
        let decimal =
            Number::decimal("0.1000000000000000055511151231257827021181583404541015625").unwrap();
        assert_eq!(Number::from_float(0.1).unwrap(), decimal);
        let max = Number::from_float(f64::MAX).unwrap();
        assert_eq!(max.try_as_f64().unwrap(), f64::MAX);
        assert_eq!(
            max.add(&Number::integer(1)).unwrap_err(),
            "Numeric overflow"
        );
        assert_eq!(
            Number::from_float(f64::NAN).unwrap_err(),
            "Not a real number"
        );
        assert_eq!(
            Number::decimal("1e-4932").unwrap().try_as_f64().unwrap(),
            0.0
        );
        assert_eq!(
            Number::decimal("1e-4933").unwrap_err(),
            "Numeric precision is out of range"
        );
    }
}
