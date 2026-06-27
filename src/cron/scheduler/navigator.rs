use crate::cron::ir::FieldMatcher;

/// Result of navigating a numeric cron field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvanceResult {
    /// Current value already satisfies the matcher.
    Unchanged,

    /// Next valid value within the current parent field.
    Changed(u32),

    /// Wrapped back to the minimum value.
    ///
    /// The scheduler must advance the parent field.
    Wrapped(u32),
}

/// Behaviour required by a numeric cron field.
///
/// Seconds, minutes, hours, months and years all satisfy this.
pub trait FieldSearch {
    /// Returns true if value is accepted.
    fn contains(&self, value: u32) -> bool;

    /// Returns the first allowed value >= current.
    fn next_or_same(&self, current: u32) -> Option<u32>;

    /// Returns the first allowed value > current.
    fn next(&self, current: u32) -> Option<u32>;

    /// Smallest accepted value.
    fn min(&self) -> Option<u32>;

    /// Largest accepted value.
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
    pub fn new(field: &'a T) -> Self {
        Self { field }
    }

    #[inline]
    pub fn contains(&self, value: u32) -> bool {
        self.field.contains(value)
    }

    #[inline]
    pub fn min(&self) -> u32 {
        self.field
            .min()
            .expect("field matcher must not be empty")
    }

    #[inline]
    pub fn max(&self) -> u32 {
        self.field
            .max()
            .expect("field matcher must not be empty")
    }

    #[inline]
    pub fn next_or_same(
        &self,
        value: u32,
    ) -> Option<u32> {
        self.field.next_or_same(value)
    }

    #[inline]
    pub fn next(
        &self,
        value: u32,
    ) -> Option<u32> {
        self.field.next(value)
    }
}

/// Generic navigation behaviour.
pub trait FieldNavigator {
    /// Computes the next valid value relative to `current`.
    ///
    /// This method is pure: it does not mutate any state.
    fn advance(&self, current: u32) -> AdvanceResult;
}

impl<T> FieldNavigator for NumericNavigator<'_, T>
where 
    T: FieldSearch,
{
    fn advance(&self, current: u32) -> AdvanceResult {
        match self.field.next_or_same(current) {
            Some(next) if next == current => {
                AdvanceResult::Unchanged
            }

            Some(next) => AdvanceResult::Changed(next),

            None => AdvanceResult::Wrapped(
                self.field
                    .min()
                    .expect("field matche cannot be empty")
            )
        }
    }
}

/// Bridge between the scheduler and the IR.
impl FieldSearch for FieldMatcher {
    #[inline]
    fn contains(
        &self,
        value: u32,
    ) -> bool {
        self.contains(value)
    }

    #[inline]
    fn next_or_same(
        &self,
        value: u32,
    ) -> Option<u32> {
        self.next_or_same(value)
    }

    #[inline]
    fn next(
        &self,
        value: u32,
    ) -> Option<u32> {
        self.next(value)
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
    use crate::cron::{
        field::BitField,
        ir::FieldMatcher,
    };

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

        assert_eq!(
            nav.advance(10),
            AdvanceResult::Unchanged,
        );
    }

    #[test]
    fn advances_to_next_allowed_value() {
        let matcher = matcher(&[5, 10, 20]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(
            nav.advance(6),
            AdvanceResult::Changed(10),
        );
    }

    #[test]
    fn advances_from_gap() {
        let matcher = matcher(&[1, 4, 8]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(
            nav.advance(2),
            AdvanceResult::Changed(4),
        );

        assert_eq!(
            nav.advance(5),
            AdvanceResult::Changed(8),
        );
    }

    #[test]
    fn wraps_after_maximum() {
        let matcher = matcher(&[5, 10, 20]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(
            nav.advance(21),
            AdvanceResult::Wrapped(5),
        );
    }

    #[test]
    fn wraps_from_last_value_plus_one() {
        let matcher = matcher(&[15]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(
            nav.advance(16),
            AdvanceResult::Wrapped(15),
        );
    }

    #[test]
    fn single_value_matcher_is_unchanged() {
        let matcher = matcher(&[42]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(
            nav.advance(42),
            AdvanceResult::Unchanged,
        );
    }

    #[test]
    fn single_value_matcher_wraps() {
        let matcher = matcher(&[42]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(
            nav.advance(50),
            AdvanceResult::Wrapped(42),
        );
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
            assert_eq!(
                nav.advance(i),
                AdvanceResult::Unchanged,
            );
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
                AdvanceResult::Changed(next) => {
                    assert!(next > current);
                }

                AdvanceResult::Wrapped(next) => {
                    assert_eq!(next, 3);
                }

                AdvanceResult::Unchanged => {}
            }
        }
    }

    #[test]
    fn advance_from_before_first_value() {
        let matcher = matcher(&[10, 20, 30]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(
            nav.advance(0),
            AdvanceResult::Changed(10),
        );
    }

    #[test]
    fn advance_from_between_every_pair() {
        let matcher = matcher(&[10, 20, 30]);

        let nav = NumericNavigator::new(&matcher);

        assert_eq!(
            nav.advance(11),
            AdvanceResult::Changed(20),
        );

        assert_eq!(
            nav.advance(21),
            AdvanceResult::Changed(30),
        );
    }
}
