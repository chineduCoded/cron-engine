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
