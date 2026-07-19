//! Scheduling algorithms.
//!
//! This module implements navigation through time using a compiled
//! [`CronIr`].
//!
//! The scheduler advances fields in descending significance:
//!
//! Year
//! ↓
//! Month
//! ↓
//! Day
//! ↓
//! Hour
//! ↓
//! Minute
//! ↓
//! Second
//!
//! Candidate normalization guarantees every intermediate datetime
//! remains valid before timezone resolution.

/// Candidate date-time representation used while searching for matching
/// schedule occurrences.
pub mod candidate;

/// Scheduler field utilities.
pub mod field;

/// Lazy iterators over schedule occurrences.
pub mod iterator;

/// Navigation algorithms for locating matching values within individual fields.
pub mod navigator;

/// Forward schedule search implementation.
pub mod next;

/// Candidate normalization utilities.
pub mod normalize;

/// Reverse schedule search implementation.
pub mod prev;

/// High-level scheduling API.
pub mod scheduler;

/// Shared search for forward and backward implementation.
pub mod search;
