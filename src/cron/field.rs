use serde::{Deserialize, Serialize};

use crate::cron::ast::FieldExpr;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default, Hash)]
pub struct BitField {
    bits: u64,
    offset: u32,
    width: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default, Hash)]
pub struct BitFieldIter {
    bits: u64,
    offset: u32,
}

impl Iterator for BitFieldIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        }

        let pos = self.bits.trailing_zeros();

        self.bits &= self.bits - 1;

        Some(self.offset + pos)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NextBit {
    pub value: u32,
    pub wrapped: bool,
}

impl BitField {
    const fn width_mask(width: u32) -> u64 {
        match width {
            0 => 0,
            1..=63 => (1u64 << width) - 1,
            _ => u64::MAX, // covers 64+
        }
    }

    pub const fn from_parts(bits: u64, offset: u32, width: u32) -> Self {
        assert!(width > 0);
        assert!(width <= 64, "BitField width must be <= 64");

        let masked = if width == 64 {
            bits
        } else {
            bits & ((1u64 << width) - 1)
        };

        Self {
            bits: masked,
            offset,
            width,
        }
    }

    pub const fn empty(offset: u32, width: u32) -> Self {
        Self::from_parts(0, offset, width)
    }

    pub const fn full(offset: u32, width: u32) -> Self {
        Self::from_parts(Self::width_mask(width), offset, width)
    }

    pub fn iter(&self) -> BitFieldIter {
        BitFieldIter { bits: self.as_u64(), offset: self.offset }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.bits == Self::width_mask(self.width)
    }

    #[inline]
    pub fn len(&self) -> u32 {
        self.bits.count_ones()
    }

    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.bits
    }

    #[inline]
    pub fn offset(&self) -> u32 {
        self.offset
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn min_value(&self) -> u32 {
        self.offset
    }

    #[inline]
    pub fn max_value(&self) -> u32 {
        self.offset + self.width - 1
    }

    #[inline]
    pub fn pos(&self, value: u32) -> Option<u32> {
        if value < self.offset || value >= self.offset + self.width {
            None
        } else {
            Some(value - self.offset)
        }
    }

    pub fn first_set(&self) -> Option<u32> {
        if self.bits == 0 {
            return None;
        }
        Some(self.offset + self.bits.trailing_zeros())
    }

    pub fn last_set(&self) -> Option<u32> {
        if self.bits == 0 {
            return None;
        }

        let pos = 63 - self.bits.trailing_zeros();
        Some(self.offset + pos)
    }

    #[inline]
    pub fn next_from(&self, start: u32) -> Option<u32> {
        if self.bits == 0 {
            return None;
        }

        let start = start.saturating_sub(self.offset);

        if start >= self.width {
            return None;
        }

        let shifted = self.bits >> start;

        if shifted == 0 {
            return None;
        }

        let pos = start + shifted.trailing_zeros();
        Some(self.offset + pos)
    }

    pub fn next_wrapping(&self, start: u32) -> Option<u32> {
        if self.bits == 0 {
            return None;
        }
        
        let start = start.saturating_sub(self.offset);

        let shifted = self.bits >> start;

        if shifted != 0 {
            let pos = start + shifted.trailing_zeros();
            return Some(self.offset + pos);
        }

        self.first_set()
    }

    pub fn set(&mut self, value: u32) -> bool {
        match self.pos(value) {
            Some(pos) => {
                self.bits |= 1u64 << pos;
                true
            }
            None => false,
        }
    }

    pub fn clear(&mut self, value: u32) -> bool {
        match self.pos(value) {
            Some(pos) => {
                self.bits &= !(1u64 << pos);
                true
            }
            None => false,
        }
    }

    pub fn contains(&self, value: u32) -> bool {
        self.pos(value)
            .is_some_and(|pos| (self.bits & (1u64 << pos)) != 0)
    }

    pub fn union_inplace(&mut self, other: &Self) {
        debug_assert_eq!(self.offset, other.offset);
        debug_assert_eq!(self.width, other.width);

        self.bits |= other.bits;
    }

    pub fn values(&self) -> Vec<u32> {
        let mut bits = self.bits;
        let mut values = Vec::with_capacity(self.len() as usize);

        while bits != 0 {
            let pos = bits.trailing_zeros();
            values.push(self.offset + pos);
            bits &= bits - 1;
        }

        values
    }

    /// Returns the next set value >= start.
    /// If none exists, wraps around and returns the first set value.
    pub fn next_set_bit_wrapping(&self, start: u32) -> Option<NextBit> {
        if self.bits == 0 {
            return None;
        }

        let start_pos = match self.pos(start) {
            Some(pos) => pos,
            None => {
                let first = self.bits.trailing_zeros();

                return Some(NextBit {
                    value: self.offset + first,
                    wrapped: true,
                });
            }
        };

        let shifted = self.bits >> start_pos;

        if shifted != 0 {
            let pos = start_pos + shifted.trailing_zeros();

            return Some(NextBit {
                value: self.offset + pos,
                wrapped: false,
            });
        }

        let first = self.bits.trailing_zeros();

        Some(NextBit {
            value: self.offset + first,
            wrapped: true,
        })
    }

    pub fn from_expr(expr: &FieldExpr, offset: u32, width: u32) -> Self {
        match expr {
            FieldExpr::Wildcard => BitField::full(offset, width),

            FieldExpr::Value(v) => {
                let mut bf = BitField::empty(offset, width);
                bf.set(*v);
                bf
            }

            FieldExpr::Range(start, end) => {
                let mut bf = BitField::empty(offset, width);
                for v in *start..=*end {
                    bf.set(v);
                }
                bf
            }

            FieldExpr::List(items) => {
                let mut bf = BitField::empty(offset, width);
                for item in items {
                    bf.union_inplace(&BitField::from_expr(item, offset, width));
                }
                bf
            }

            FieldExpr::Step(inner, step) => {
                let base = BitField::from_expr(inner, offset, width);
                let mut out = BitField::empty(offset, width);

                let min = base.min_value();
                let max = base.max_value();

                let mut v = min;
                while v <= max {
                    if base.contains(v) {
                        out.set(v);
                    }
                    v += step;
                }

                out
            }

            FieldExpr::And(a, b) => {
                let mut left = BitField::from_expr(a, offset, width);
                let right = BitField::from_expr(b, offset, width);

                left.union_inplace(&right); // OR semantics (adjust if AND is real intersection)
                left
            }

            // semantic constructs must NOT be forced into BitField
            FieldExpr::LastDay
            | FieldExpr::LastWeekday(_)
            | FieldExpr::NearestWeekday(_)
            | FieldExpr::NthWeekday { .. } => {
                BitField::empty(offset, width)
            }
        }
    }
}

impl fmt::Display for BitField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = self
            .values()
            .into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",");

        write!(f, "BitField{{{}}}", values)
    }
}


/// Const-generic Flags for semantic cron bits (small width).
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Flags<const OFFSET: u32, const WIDTH: u32>(pub u64);

impl<const OFFSET: u32, const WIDTH: u32> Flags<OFFSET, WIDTH> {
    pub const fn mask() -> u64 { if WIDTH >= 64 { u64::MAX } else { (1u64 << WIDTH) - 1 } }
    pub const fn new(bits: u64) -> Self { Self(bits & Self::mask()) }
    pub const fn empty() -> Self { Self::new(0) }
    pub const fn bit(p: u32) -> u64 { 1u64 << p }
    pub const fn with(self, mask: u64) -> Self { Self::new(self.0 | mask) }
    pub const fn without(self, mask: u64) -> Self { Self::new(self.0 & !mask) }
    pub const fn contains(self, bit: u64) -> bool { (self.0 & bit) != 0 }
    pub const fn to_u64(self) -> u64 { self.0 }
}

pub type CronFlags = Flags<0, 8>;
pub const NONE_BIT: u64 = 0;
pub const ALL_BIT: u64 = CronFlags::bit(0);
pub const LAST_BIT: u64 = CronFlags::bit(1);
pub const CLOSEST_WEEKDAY_BIT: u64 = CronFlags::bit(2);
pub const NTH_1ST_BIT: u64 = CronFlags::bit(3);
pub const NTH_2ND_BIT: u64 = CronFlags::bit(4);
pub const NTH_3RD_BIT: u64 = CronFlags::bit(5);
pub const NTH_4TH_BIT: u64 = CronFlags::bit(6);
pub const NTH_5TH_BIT: u64 = CronFlags::bit(7);
pub const NTH_ALL: u64 = NTH_1ST_BIT | NTH_2ND_BIT | NTH_3RD_BIT | NTH_4TH_BIT | NTH_5TH_BIT;

pub const BITFIELD_MAX_WIDTH: u32 = 64;
pub const MIN_YEAR: u32 = 1;
pub const MAX_YEAR: u32 = 5000;

#[cfg(test)]
mod tests {
    use crate::cron::field::BitField;

    #[test]
    fn with_64() {
        let bits = BitField::full(0, 64);

        assert!(bits.contains(63));
    }

    #[test]
    fn empty_bitfield() {
        let bits = BitField::empty(0, 64);

        assert!(bits.is_empty());
    }

    #[test]
    fn union() {
        let mut a = BitField::empty(0, 64);
        let mut b = BitField::empty(0, 64);

        a.set(1);
        b.set(5);

        a.union_inplace(&b);

        assert!(a.contains(1));
        assert!(a.contains(5));
    }

    #[test]
    fn next_bit_wraps() {
        let mut bits = BitField::empty(0, 60);

        bits.set(5);

        let next = bits
            .next_set_bit_wrapping(10)
            .unwrap();

        assert_eq!(next.value, 5);
        assert!(next.wrapped);
    }
}

