//! Evaluation of numeric cron fields.
//!
//! This module provides helper functions for matching values against
//! compiled field matchers used by the scheduler.

use crate::cron::ir::FieldMatcher;

/// Returns `true` if a field matcher accepts the specified value.
///
/// This is the fundamental predicate used to evaluate numeric cron
/// fields such as seconds, minutes, hours, months, and years.
pub fn matches(field: &FieldMatcher, value: u32) -> bool {
    field.contains(value)
}
