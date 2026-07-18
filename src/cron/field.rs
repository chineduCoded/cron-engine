//! Compact bitmap implementation used throughout the cron engine.
//!
//! `BitField` provides constant-time membership checks and efficient
//! navigation between enabled values. It is used by the compiler and
//! scheduler to represent numeric cron fields without heap allocations.

use serde::{Deserialize, Serialize};

use crate::cron::ast::FieldExpr;
use std::fmt;

use proptest::prelude::*;

/// Iterator over the enabled values in a [`BitField`].
///
/// Values are yielded in ascending order. The iterator consumes an internal
/// copy of the bitmap, leaving the original [`BitField`] unchanged.
///
/// This iterator performs allocation-free iteration by repeatedly extracting
/// the least significant set bit.
///
/// Constructed by [`BitField::iter`].
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default, Hash,
)]
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

/// Result of a wrapping bitfield search.
///
/// Returned by methods such as [`BitField::next_wrapping`] to indicate both
/// the matching value and whether the search wrapped around to the beginning
/// of the field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NextBit {
    /// The matching value.
    pub value: u32,

    /// Indicates whether the search wrapped to the beginning of the field.
    pub wrapped: bool,
}

/// A compact fixed-width bitmap for representing a contiguous range of integer
/// values.
///
/// `BitField` stores up to 64 consecutive values inside a single `u64`,
/// making membership checks and navigation operations constant time.
///
/// Each bit corresponds to one integer value beginning at `offset`.
///
/// For example:
///
/// ```text
/// offset = 1
/// width  = 5
///
/// values: 1 2 3 4 5
/// bits:   1 0 1 1 0
/// ```
///
/// This type is used internally by the cron compiler to represent fields
/// such as seconds, minutes, hours, months, and simple day-of-month rules.
///
/// # Complexity
///
/// Most operations are **O(1)**.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default, Hash,
)]
pub struct BitField {
    bits: u64,
    offset: u32,
    width: u32,
}

impl BitField {
    const fn width_mask(width: u32) -> u64 {
        match width {
            0 => 0,
            1..=63 => (1u64 << width) - 1,
            _ => u64::MAX, // covers 64+
        }
    }

    /// Creates a new bitfield.
    ///
    /// # Panics
    ///
    /// Panics if `width` is zero or greater than 64.
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

    /// Creates an empty bitfield.
    ///
    /// All values are initially unset.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cron_engine::cron::field::BitField;
    /// let bf = BitField::empty(1, 31);
    ///
    /// assert!(!bf.contains(10));
    /// ```
    pub const fn empty(offset: u32, width: u32) -> Self {
        Self::from_parts(0, offset, width)
    }

    /// Creates a bitfield with every value enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cron_engine::BitField;
    /// let bf = BitField::full(0, 60);
    ///
    /// assert!(bf.contains(15));
    /// ```
    pub const fn full(offset: u32, width: u32) -> Self {
        Self::from_parts(Self::width_mask(width), offset, width)
    }

    /// Returns an iterator over all enabled values in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cron_engine::BitField;
    /// let mut bf = BitField::empty(1, 5);
    /// bf.set(2);
    /// bf.set(4);
    ///
    /// let values: Vec<_> = bf.iter().collect();
    ///
    /// assert_eq!(values, vec![2, 4]);
    /// ```
    pub fn iter(&self) -> BitFieldIter {
        BitFieldIter {
            bits: self.as_u64(),
            offset: self.offset,
        }
    }

    /// Returns `true` if no values are enabled.
    ///
    /// # Complexity
    ///
    /// O(1)
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    /// Returns `true` if every representable value is enabled.
    ///
    /// # Complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```
    /// # use cron_engine::BitField;
    /// let bf = BitField::full(0, 60);
    /// assert!(bf.is_full());
    /// ```
    #[inline]
    pub fn is_full(&self) -> bool {
        self.bits == Self::width_mask(self.width)
    }

    /// Returns the number of enabled values.
    ///
    /// # Complexity
    ///
    /// O(number of set bits)
    #[inline]
    pub fn len(&self) -> u32 {
        self.bits.count_ones()
    }

    /// Returns the underlying bitmap.
    ///
    /// This method is intended primarily for debugging, testing,
    /// and low-level optimizations.
    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.bits
    }

    /// Returns the smallest value represented by this bitfield.
    #[inline]
    pub fn offset(&self) -> u32 {
        self.offset
    }

    /// Returns the number of values represented by this bitfield.
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the smallest representable value.
    #[inline]
    pub fn min_value(&self) -> u32 {
        self.offset
    }

    /// Returns the largest representable value.
    #[inline]
    pub fn max_value(&self) -> u32 {
        self.offset + self.width - 1
    }

    /// Returns the zero-based bit position corresponding to `value`.
    ///
    /// The returned position is relative to the field's offset.
    ///
    /// Returns `None` if `value` lies outside the representable range.
    ///
    /// # Complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```
    /// # use cron_engine::BitField;
    /// let bf = BitField::empty(10, 5);
    ///
    /// assert_eq!(bf.pos(10), Some(0));
    /// assert_eq!(bf.pos(14), Some(4));
    /// assert_eq!(bf.pos(15), None);
    /// ```
    #[inline]
    pub fn pos(&self, value: u32) -> Option<u32> {
        if value < self.offset || value >= self.offset + self.width {
            None
        } else {
            Some(value - self.offset)
        }
    }

    /// Returns the smallest enabled value.
    ///
    /// Returns `None` if the bitfield contains no enabled values.
    ///
    /// # Complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```
    /// # use cron_engine::BitField;
    /// let mut bf = BitField::empty(1, 31);
    /// bf.set(10);
    /// bf.set(20);
    ///
    /// assert_eq!(bf.first_set(), Some(10));
    /// ```
    pub fn first_set(&self) -> Option<u32> {
        if self.bits == 0 {
            return None;
        }
        Some(self.offset + self.bits.trailing_zeros())
    }

    /// Returns the largest enabled value.
    ///
    /// Returns `None` if the bitfield contains no enabled values.
    ///
    /// # Complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```
    /// # use cron_engine::BitField;
    /// let mut bf = BitField::empty(1, 31);
    /// bf.set(10);
    /// bf.set(20);
    ///
    /// assert_eq!(bf.last_set(), Some(20));
    /// ```
    pub fn last_set(&self) -> Option<u32> {
        if self.bits == 0 {
            return None;
        }

        let pos = 63 - self.bits.leading_zeros();
        Some(self.offset + pos)
    }

    /// Returns the first enabled value greater than or equal to `start`.
    ///
    /// Unlike [`BitField::next_wrapping`], this method does not wrap to the
    /// beginning of the field.
    ///
    /// Returns `None` if:
    ///
    /// - the bitfield is empty,
    /// - `start` lies beyond the field's range, or
    /// - no enabled value exists after `start`.
    ///
    /// # Complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```
    /// # use cron_engine::BitField;
    /// let mut bf = BitField::empty(0, 60);
    /// bf.set(5);
    /// bf.set(20);
    ///
    /// assert_eq!(bf.next_from(6), Some(20));
    /// assert_eq!(bf.next_from(21), None);
    /// ```
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

    /// Returns the next enabled value, wrapping to the beginning if necessary.
    ///
    /// Unlike [`BitField::next_from`], this method never stops at the upper bound.
    /// If no later value exists, the search continues from the minimum value.
    ///
    /// Returns `None` only if the bitfield is empty.
    ///
    /// # Complexity
    ///
    /// O(1)
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

    /// Enables the specified value.
    ///
    /// Values outside the configured range are ignored.
    ///
    /// # Complexity
    ///
    /// O(1)
    pub fn set(&mut self, value: u32) -> bool {
        match self.pos(value) {
            Some(pos) => {
                self.bits |= 1u64 << pos;
                true
            }
            None => false,
        }
    }

    /// Disables the specified value.
    ///
    /// # Complexity
    ///
    /// O(1)
    pub fn clear(&mut self, value: u32) -> bool {
        match self.pos(value) {
            Some(pos) => {
                self.bits &= !(1u64 << pos);
                true
            }
            None => false,
        }
    }

    /// Returns `true` if the given value is present.
    ///
    /// Values outside the configured range always return `false`.
    ///
    /// # Complexity
    ///
    /// O(1)
    ///
    /// # Examples
    ///
    /// ```
    /// # use cron_engine::BitField;
    /// let mut bf = BitField::empty(1, 31);
    /// bf.set(15);
    ///
    /// assert!(bf.contains(15));
    /// assert!(!bf.contains(16));
    /// ```
    pub fn contains(&self, value: u32) -> bool {
        self.pos(value)
            .is_some_and(|pos| (self.bits & (1u64 << pos)) != 0)
    }

    /// Returns the union of two bitfields.
    pub fn union_inplace(&mut self, other: &Self) {
        debug_assert_eq!(self.offset, other.offset);
        debug_assert_eq!(self.width, other.width);

        self.bits |= other.bits;
    }

    /// Returns all enabled values in ascending order.
    ///
    /// This is primarily intended for debugging, testing, serialization,
    /// and interoperability. For allocation-free traversal, prefer
    /// [`BitField::iter`].
    ///
    /// # Complexity
    ///
    /// O(n), where *n* is the number of enabled values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cron_engine::BitField;
    /// let mut bf = BitField::empty(1, 10);
    /// bf.set(2);
    /// bf.set(5);
    /// bf.set(8);
    ///
    /// assert_eq!(bf.values(), vec![2, 5, 8]);
    /// ```
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

    /// Builds a bitfield from a parsed cron field expression.
    ///
    /// This is primarily used by the cron compiler during IR generation.
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
            | FieldExpr::LastBusinessDay
            | FieldExpr::NearestWeekday(_)
            | FieldExpr::NthWeekday { .. } => BitField::empty(offset, width),
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
    /// Returns the underlying bitmap.
    ///
    /// This method is intended primarily for debugging, testing,
    /// and low-level optimizations.
    pub const fn mask() -> u64 {
        if WIDTH >= 64 {
            u64::MAX
        } else {
            (1u64 << WIDTH) - 1
        }
    }

    /// Creates a new flag set from the provided raw bits.
    ///
    /// Any bits outside the valid range are automatically cleared.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cron_engine::CronFlags;
    /// let flags = CronFlags::new(0b1010);
    /// ```
    pub const fn new(bits: u64) -> Self {
        Self(bits & Self::mask())
    }

    /// Creates an empty flag set.
    ///
    /// No flags are enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cron_engine::CronFlags;
    /// let flags = CronFlags::empty();
    /// assert_eq!(flags.to_u64(), 0);
    /// ```
    pub const fn empty() -> Self {
        Self::new(0)
    }

    /// Returns the bit mask for the given bit position.
    ///
    /// This is a convenience helper for defining compile-time flag constants.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cron_engine::CronFlags;
    /// assert_eq!(CronFlags::bit(3), 1 << 3);
    /// ```
    pub const fn bit(p: u32) -> u64 {
        1u64 << p
    }

    /// Returns a new flag set with the specified bits enabled.
    ///
    /// This method does not modify the original value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cron_engine::{CronFlags, LAST_BIT};
    /// let flags = CronFlags::empty().with(LAST_BIT);
    /// assert!(flags.contains(LAST_BIT));
    /// ```
    pub const fn with(self, mask: u64) -> Self {
        Self::new(self.0 | mask)
    }

    /// Returns a new flag set with the specified bits cleared.
    ///
    /// This method does not modify the original value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cron_engine::{CronFlags, LAST_BIT};
    /// let flags = CronFlags::empty()
    ///     .with(LAST_BIT)
    ///     .without(LAST_BIT);
    ///
    /// assert!(!flags.contains(LAST_BIT));
    /// ```
    pub const fn without(self, mask: u64) -> Self {
        Self::new(self.0 & !mask)
    }

    /// Returns `true` if all bits in `bit` are enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cron_engine::{CronFlags, LAST_BIT};
    /// let flags = CronFlags::empty().with(LAST_BIT);
    ///
    /// assert!(flags.contains(LAST_BIT));
    /// ```
    pub const fn contains(self, bit: u64) -> bool {
        (self.0 & bit) != 0
    }

    /// Returns the raw bit representation of the flag set.
    ///
    /// This is primarily intended for serialization, debugging,
    /// and interoperability.
    pub const fn to_u64(self) -> u64 {
        self.0
    }
}

/// Flag set used by the cron compiler and evaluator.
///
/// Eight bits are currently reserved for representing special cron
/// semantics such as `L`, `W`, and `#`.
pub type CronFlags = Flags<0, 8>;

/// No flags enabled.
pub const NONE_BIT: u64 = 0;

/// Indicates that the field represents all possible values (`*`).
pub const ALL_BIT: u64 = CronFlags::bit(0);

/// Indicates use of the `L` (last) modifier.
pub const LAST_BIT: u64 = CronFlags::bit(1);

/// Indicates use of the `W` (nearest weekday) modifier.
pub const CLOSEST_WEEKDAY_BIT: u64 = CronFlags::bit(2);

/// Indicates the first occurrence of a weekday (`#1`).
pub const NTH_1ST_BIT: u64 = CronFlags::bit(3);

/// Indicates the second occurrence of a weekday (`#2`).
pub const NTH_2ND_BIT: u64 = CronFlags::bit(4);

/// Indicates the third occurrence of a weekday (`#3`).
pub const NTH_3RD_BIT: u64 = CronFlags::bit(5);

/// Indicates the fourth occurrence of a weekday (`#4`).
pub const NTH_4TH_BIT: u64 = CronFlags::bit(6);

/// Indicates the fifth occurrence of a weekday (`#5`).
pub const NTH_5TH_BIT: u64 = CronFlags::bit(7);

/// Mask containing all supported `#` occurrence flags.
///
/// Equivalent to the bitwise OR of
/// [`NTH_1ST_BIT`], [`NTH_2ND_BIT`], [`NTH_3RD_BIT`],
/// [`NTH_4TH_BIT`], and [`NTH_5TH_BIT`].
pub const NTH_ALL: u64 = NTH_1ST_BIT 
    | NTH_2ND_BIT 
    | NTH_3RD_BIT 
    | NTH_4TH_BIT 
    | NTH_5TH_BIT;

/// Maximum number of values representable by a [`BitField`].
///
/// This limit corresponds to the width of a `u64`.
pub const BITFIELD_MAX_WIDTH: u32 = 64;

/// Smallest supported year in cron expressions.
pub const MIN_YEAR: u32 = 1;

/// Largest supported year in cron expressions.
///
/// Schedules cannot generate occurrences beyond this year.
pub const MAX_YEAR: u32 = 5000;

#[cfg(test)]
mod tests {
    use super::*;

    fn field() -> BitField {
        BitField::empty(0, 64)
    }

    #[test]
    fn with_64() {
        let bits = BitField::full(0, 64);

        assert!(bits.contains(63));
    }

    #[test]
    fn empty_bitfield_has_no_bits() {
        let bf = BitField::empty(0, 64);

        assert!(bf.is_empty());
        assert_eq!(bf.len(), 0);
        assert_eq!(bf.first_set(), None);
        assert_eq!(bf.last_set(), None)
    }

    #[test]
    fn full_bitfield_has_all_bits() {
        let bf = BitField::full(0, 64);

        assert!(bf.is_full());
        assert_eq!(bf.len(), 64);
        assert_eq!(bf.first_set(), Some(0));
        assert_eq!(bf.last_set(), Some(63));
    }

    #[test]
    fn from_parts_masks_unused_bits() {
        let bf = BitField::from_parts(u64::MAX, 10, 5);

        assert_eq!(bf.len(), 5);
        assert_eq!(bf.first_set(), Some(10));
        assert_eq!(bf.last_set(), Some(14));
    }

    #[test]
    fn offset_and_width_are_preserved() {
        let bf = BitField::empty(5, 10);

        assert_eq!(bf.offset(), 5);
        assert_eq!(bf.width(), 10);
        assert_eq!(bf.min_value(), 5);
        assert_eq!(bf.max_value(), 14);
    }

    #[test]
    fn pos_maps_values_correctly() {
        let bf = BitField::empty(5, 10);

        assert_eq!(bf.pos(5), Some(0));
        assert_eq!(bf.pos(14), Some(9));
    }

    #[test]
    fn pos_returns_none_outside_range() {
        let bf = BitField::empty(5, 10);

        assert_eq!(bf.pos(4), None);
        assert_eq!(bf.pos(15), None);
    }

    #[test]
    fn set_inside_range() {
        let mut bf = field();

        assert!(bf.set(5));
        assert!(bf.contains(5));
        assert_eq!(bf.len(), 1);
    }

    #[test]
    fn set_outside_range_returns_false() {
        let mut bf = BitField::empty(10, 5);

        assert!(!bf.set(100));
        assert!(bf.is_empty());
    }

    #[test]
    fn setting_same_bit_twice_is_idempotent() {
        let mut bf = field();

        bf.set(7);
        bf.set(7);

        assert_eq!(bf.len(), 1);
    }

    #[test]
    fn clear_existing_bit() {
        let mut bf = field();

        bf.set(12);

        assert!(bf.clear(12));
        assert!(!bf.contains(12));
    }

    #[test]
    fn clear_missing_bit() {
        let mut bf = field();

        assert!(bf.clear(20));
        assert!(!bf.contains(20));
    }

    #[test]
    fn clear_outside_range_returns_false() {
        let mut bf = BitField::empty(10, 5);

        assert!(!bf.clear(100));
    }

    #[test]
    fn contains_only_set_bits() {
        let mut bf = field();

        bf.set(4);
        bf.set(10);

        assert!(bf.contains(4));
        assert!(bf.contains(10));

        assert!(!bf.contains(5));
        assert!(!bf.contains(9));
    }

    #[test]
    fn first_set_returns_smallest_bit() {
        let mut bf = field();

        bf.set(20);
        bf.set(5);
        bf.set(10);

        assert_eq!(bf.first_set(), Some(5));
    }

    #[test]
    fn first_set_empty_returns_none() {
        assert_eq!(field().first_set(), None);
    }

    #[test]
    fn last_set_returns_largest_bit() {
        let mut bf = field();

        bf.set(5);
        bf.set(17);
        bf.set(40);

        assert_eq!(bf.last_set(), Some(40));
    }

    #[test]
    fn last_set_empty_returns_none() {
        assert_eq!(field().last_set(), None);
    }

    #[test]
    fn values_are_sorted() {
        let mut bf = field();

        bf.set(20);
        bf.set(3);
        bf.set(10);

        assert_eq!(bf.values(), vec![3, 10, 20]);
    }

    #[test]
    fn values_empty() {
        assert!(field().values().is_empty());
    }

    #[test]
    fn iterator_matches_values() {
        let mut bf = field();

        bf.set(1);
        bf.set(7);
        bf.set(40);

        let values: Vec<_> = bf.iter().collect();

        assert_eq!(values, vec![1, 7, 40]);
    }

    #[test]
    fn iterator_empty() {
        let values: Vec<_> = field().iter().collect();

        assert!(values.is_empty());
    }

    #[test]
    fn union_combines_sets() {
        let mut a = field();
        let mut b = field();

        a.set(5);
        a.set(10);

        b.set(20);
        b.set(10);

        a.union_inplace(&b);

        assert_eq!(a.values(), vec![5, 10, 20]);
    }

    #[test]
    fn union_with_empty() {
        let mut a = field();
        let b = field();

        a.set(7);

        a.union_inplace(&b);

        assert_eq!(a.values(), vec![7]);
    }

    #[test]
    fn next_from_exact_match() {
        let mut bf = field();

        bf.set(5);
        bf.set(10);

        assert_eq!(bf.next_from(5), Some(5));
    }

    #[test]
    fn next_from_between_values() {
        let mut bf = field();

        bf.set(5);
        bf.set(10);

        assert_eq!(bf.next_from(6), Some(10));
    }

    #[test]
    fn next_from_after_last() {
        let mut bf = field();

        bf.set(5);

        assert_eq!(bf.next_from(6), None);
    }

    #[test]
    fn next_from_empty() {
        assert_eq!(field().next_from(5), None);
    }

    #[test]
    fn next_wrapping_without_wrap() {
        let mut bf = field();

        bf.set(5);
        bf.set(20);

        assert_eq!(bf.next_wrapping(6), Some(20));
    }

    #[test]
    fn next_wrapping_wraps() {
        let mut bf = field();

        bf.set(5);
        bf.set(20);

        assert_eq!(bf.next_wrapping(30), Some(5));
    }

    #[test]
    fn next_wrapping_exact_match() {
        let mut bf = field();

        bf.set(20);

        assert_eq!(bf.next_wrapping(20), Some(20));
    }

    #[test]
    fn next_wrapping_empty() {
        assert_eq!(field().next_wrapping(10), None);
    }

    #[test]
    fn next_set_bit_no_wrap() {
        let mut bf = field();

        bf.set(5);
        bf.set(20);

        let next = bf.next_set_bit_wrapping(6).unwrap();

        assert_eq!(next.value, 20);
        assert!(!next.wrapped);
    }

    #[test]
    fn next_set_bit_wrap() {
        let mut bf = field();

        bf.set(5);

        let next = bf.next_set_bit_wrapping(20).unwrap();

        assert_eq!(next.value, 5);
        assert!(next.wrapped);
    }

    #[test]
    fn next_set_bit_empty() {
        assert!(field().next_set_bit_wrapping(5).is_none());
    }

    #[test]
    fn first_set_returns_smallest_value() {
        let mut bits = BitField::empty(0, 64);

        bits.set(7);
        bits.set(30);
        bits.set(55);

        assert_eq!(bits.first_set(), Some(7));
    }

    #[test]
    fn last_set_returns_largest_value() {
        let mut bits = BitField::empty(0, 64);

        bits.set(7);
        bits.set(30);
        bits.set(55);

        assert_eq!(bits.last_set(), Some(55));
    }
}

proptest! {
    #[test]
    fn set_implies_contains(value in 0u32..63) {
        let mut bf = BitField::empty(0, 64);

        bf.set(value);

        prop_assert!(bf.contains(value));
    }
}

proptest! {
    #[test]
    fn clear_removes_membership(value in 0u32..63) {
        let mut bf = BitField::full(0,64);

        bf.clear(value);

        prop_assert!(!bf.contains(value));
    }
}

proptest! {
    #[test]
    fn first_is_not_after_last(values in prop::collection::vec(0u32..63,1..64)) {

        let mut bf = BitField::empty(0,64);

        for v in values {
            bf.set(v);
        }

        prop_assert!(
            bf.first_set().unwrap()
                <=
            bf.last_set().unwrap()
        );
    }
}

proptest! {
    #[test]
    fn iterator_is_sorted(values in prop::collection::vec(0u32..63,1..64)) {

        let mut bf = BitField::empty(0,64);

        for v in values {
            bf.set(v);
        }

        let collected: Vec<_> = bf.iter().collect();

        for pair in collected.windows(2) {
            prop_assert!(pair[0] < pair[1]);
        }
    }
}
