//! Core implementation of the cron scheduler.
//!
//! This module contains the parser, compiler, intermediate representation,
//! evaluator, and scheduler used by `cron-engine`.
//!
//! Most users should interact only with [`CronSchedule`] and [`CronError`].
//! The remaining modules are primarily intended for library internals or
//! advanced use cases.

/// Abstract syntax tree for parsed cron expressions.
pub mod ast;

/// Compiles parsed expressions into an optimized intermediate representation.
pub mod compiler;

/// Error types returned by parsing, compilation, and scheduling.
pub mod error;

/// Schedule evaluation utilities.
pub mod evaluator;

/// Bitfield and field matcher implementations.
pub mod field;

/// Optimized intermediate representation used by the scheduler.
pub mod ir;

/// Cron expression parser.
pub mod parser;

/// Lookup tables for named months and weekdays.
pub mod resolver;

/// Scheduler implementation and iterators.
pub mod scheduler;

/// Time zone conversion helpers.
pub mod timezone;

/// Error type returned by parsing, compilation, and scheduling.
pub use error::CronError;

/// Immutable compiled cron schedule.
///
/// This is the primary entry point for the library.
pub use scheduler::scheduler::CronSchedule;
