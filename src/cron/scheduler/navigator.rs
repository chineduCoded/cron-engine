use crate::cron::{ir::FieldMatcher, scheduler::scheduler::Direction};


/// Result of navigating a scheduler field.
///
/// This value describes how the scheduler should proceed after attempting
/// to move a field to a matching value.
///
/// Unlike the raw search methods on [`FieldMatcher`], this type encodes
/// whether the current value already matched, a new value was selected
/// within the current parent field, or the search wrapped around and the
/// parent field must be adjusted.
///
/// Used by both forward and backward schedulers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationResult {
    /// The current value already satisfies the field matcher.
    ///
    /// No changes are required.
    Unchanged,

    /// A different matching value was found within the current parent field.
    ///
    /// The contained value should replace the current field value while
    /// leaving higher-order fields unchanged.
    Changed(u32),

    /// Navigation wrapped past the field boundary.
    ///
    /// For forward navigation this is the minimum allowed value after
    /// advancing the parent field.
    ///
    /// For backward navigation this is the maximum allowed value after
    /// decrementing the parent field.
    Wrapped(u32),
}

/// Behaviour required by a numeric cron field.
///
/// Seconds, minutes, hours, months and years all satisfy this.
pub trait FieldSearch {
    fn contains(&self, value: u32) -> bool;

    fn next_or_same(&self, current: u32) -> Option<u32>;
    fn next(&self, current: u32) -> Option<u32>;

    fn prev_or_same(&self, current: u32) -> Option<u32>;
    fn prev(&self, current: u32) -> Option<u32>;

    fn min(&self) -> Option<u32>;
    fn max(&self) -> Option<u32>;
}

/// Generic navigator for numeric cron fields.
///
/// This type is completely independent of BitField,
/// FieldMatcher, DateTime or the scheduler.
#[derive(Debug, Clone, Copy)]
pub struct NumericNavigator<'a, T>
where
    T: FieldSearch,
{
    field: &'a T,
}

impl<'a, T> NumericNavigator<'a, T>
where
    T: FieldSearch,
{
    /// Creates a navigator over a numeric field matcher.
    pub fn new(field: &'a T) -> Self {
        Self { field }
    }

    /// Returns whether the specified value matches the field.
    #[inline]
    pub fn contains(&self, value: u32) -> bool {
        self.field.contains(value)
    }

    /// Returns the minimum value accepted by the field.
    ///
    /// # Panics
    ///
    /// Panics if the field matcher contains no values.
    #[inline]
    pub fn min(&self) -> u32 {
        self.field.min().expect("field matcher must not be empty")
    }

    /// Returns the maximum value accepted by the field.
    ///
    /// # Panics
    ///
    /// Panics if the field matcher contains no values.
    #[inline]
    pub fn max(&self) -> u32 {
        self.field.max().expect("field matcher must not be empty")
    }

    /// Returns the next matching value greater than or equal to `value`.
    #[inline]
    pub fn next_or_same(&self, value: u32) -> Option<u32> {
        self.field.next_or_same(value)
    }

    /// Returns the next matching value strictly greater than `value`.
    #[inline]
    pub fn next(&self, value: u32) -> Option<u32> {
        self.field.next(value)
    }

    #[inline]
    pub fn prev_or_same(&self, value: u32) -> Option<u32> {
        self.field.prev_or_same(value)
    }

    #[inline]
    pub fn prev(&self, value: u32) -> Option<u32> {
        self.field.prev(value)
    }
}

/// Generic navigation behaviour.
pub trait FieldNavigator {
    fn advance(&self, current: u32) -> NavigationResult;
    fn retreat(&self, current: u32) -> NavigationResult;
    fn navigate(
        &self,
        current: u32,
        direction: Direction,
    ) -> NavigationResult;
}

impl<T> FieldNavigator for NumericNavigator<'_, T>
where
    T: FieldSearch,
{
    fn advance(&self, current: u32) -> NavigationResult {
        match self.field.next_or_same(current) {
            Some(next) if next == current => NavigationResult::Unchanged,

            Some(next) => NavigationResult::Changed(next),

            None => NavigationResult::Wrapped(self.field.min().expect("field matche cannot be empty")),
        }
    }

    fn retreat(&self, current: u32) -> NavigationResult {
        match self.field.prev_or_same(current) {
            Some(prev) if prev == current => NavigationResult::Unchanged,

            Some(prev) => NavigationResult::Changed(prev),

            None => NavigationResult::Wrapped(
                self.field
                    .max()
                    .expect("field matcher cannot be empty"),
            ),
        }
    }

    fn navigate(
        &self,
        current: u32,
        direction: Direction,
    ) ->NavigationResult {
        match direction {
            Direction::Forward => self.advance(current),
            Direction::Backward => self.retreat(current),
        }
    }
}

/// Bridge between the scheduler and the IR.
impl FieldSearch for FieldMatcher {
    #[inline]
    fn contains(&self, value: u32) -> bool {
        self.contains(value)
    }

    #[inline]
    fn next_or_same(&self, value: u32) -> Option<u32> {
        self.next_or_same(value)
    }

    #[inline]
    fn next(&self, value: u32) -> Option<u32> {
        self.next(value)
    }

    #[inline]
    fn prev_or_same(&self, value: u32) -> Option<u32> {
        self.prev_or_same(value)
    }

    #[inline]
    fn prev(&self, value: u32) -> Option<u32> {
        self.prev(value)
    }

    #[inline]
    fn min(&self) -> Option<u32> {
        self.min()
    }

    #[inline]
    fn max(&self) -> Option<u32> {
        self.max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cron::{field::BitField, ir::FieldMatcher};

    fn matcher(values: &[u32]) -> FieldMatcher {
        let mut bits = BitField::empty(0, 60);

        for &v in values {
            bits.set(v);
        }

        FieldMatcher::from(bits)
    }

    #[test]
    fn unchanged_when_value_is_present() {
        let matcher = matcher(&[5, 10, 20]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(nav.advance(10), NavigationResult::Unchanged,);
    }

    #[test]
    fn advances_to_next_allowed_value() {
        let matcher = matcher(&[5, 10, 20]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(nav.advance(6), NavigationResult::Changed(10),);
    }

    #[test]
    fn advances_from_gap() {
        let matcher = matcher(&[1, 4, 8]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(nav.advance(2), NavigationResult::Changed(4),);

        assert_eq!(nav.advance(5), NavigationResult::Changed(8),);
    }

    #[test]
    fn wraps_after_maximum() {
        let matcher = matcher(&[5, 10, 20]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(nav.advance(21), NavigationResult::Wrapped(5),);
    }

    #[test]
    fn wraps_from_last_value_plus_one() {
        let matcher = matcher(&[15]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(nav.advance(16), NavigationResult::Wrapped(15),);
    }

    #[test]
    fn single_value_matcher_is_unchanged() {
        let matcher = matcher(&[42]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(nav.advance(42), NavigationResult::Unchanged,);
    }

    #[test]
    fn single_value_matcher_wraps() {
        let matcher = matcher(&[42]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(nav.advance(50), NavigationResult::Wrapped(42),);
    }

    #[test]
    fn full_range_never_changes() {
        let mut bits = BitField::empty(0, 60);

        for i in 0..60 {
            bits.set(i);
        }

        let matcher = FieldMatcher::from(bits);

        let nav = NumericNavigator::new(&matcher);

        for i in 0..60 {
            assert_eq!(nav.advance(i), NavigationResult::Unchanged,);
        }
    }

    #[test]
    fn min_and_max_are_correct() {
        let matcher = matcher(&[7, 11, 15, 30]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(nav.min(), 7);
        assert_eq!(nav.max(), 30);
    }

    #[test]
    #[should_panic(expected = "field matcher must not be empty")]
    fn empty_matcher_panics_on_min() {
        let bits = BitField::empty(0, 60);

        let matcher = FieldMatcher::from(bits);

        let nav = NumericNavigator::new(&matcher);

        nav.min();
    }

    #[test]
    fn advance_is_monotonic_until_wrap() {
        let matcher = matcher(&[3, 7, 11, 20]);

        let nav = NumericNavigator::new(&matcher);

        for current in 0..20 {
            match nav.advance(current) {
                NavigationResult::Changed(next) => {
                    assert!(next > current);
                }

                NavigationResult::Wrapped(next) => {
                    assert_eq!(next, 3);
                }

                NavigationResult::Unchanged => {}
            }
        }
    }

    #[test]
    fn advance_from_before_first_value() {
        let matcher = matcher(&[10, 20, 30]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(nav.advance(0), NavigationResult::Changed(10),);
    }

    #[test]
    fn advance_from_between_every_pair() {
        let matcher = matcher(&[10, 20, 30]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(nav.advance(11), NavigationResult::Changed(20),);

        assert_eq!(nav.advance(21), NavigationResult::Changed(30),);
    }
}
