use thiserror::Error;

/// Errors that can occur while parsing, compiling, or evaluating cron
/// expressions.
#[derive(Debug, Error)]
pub enum CronError {
    /// The supplied cron expression is empty.
    #[error("cron pattern is empty")]
    EmptyPattern,

    /// The scheduler produced an invalid calendar date.
    #[error("invalid date produced by pattern")]
    InvalidDate,

    /// The scheduler produced an invalid time.
    #[error("invalid time produced by pattern")]
    InvalidTime,

    /// The scheduler exceeded its search limit without finding a match.
    #[error("time search limit exceeded")]
    TimeSearchLimitExceeded,

    /// The expression is syntactically invalid.
    #[error("invalid pattern: {reason}")]
    InvalidPattern {
        /// Human-readable explanation.
        reason: String 
    },

    /// The expression contains unsupported characters.
    #[error("illegal characters in pattern: {chars}")]
    IllegalCharacters {
        /// Invalid characters that were encountered.
        chars: String
    },

    /// One field failed to parse.
    #[error("component error at position {position}: {reason}")]
    ComponentError {
        /// Zero-based field index.
        position: usize,

        /// Description of the failure.
        reason: String
    },

    /// The overall expression is malformed.
    #[error("invalid cron expression")]
    InvalidExpression,

    /// The expression contains an unexpected number of fields.
    #[error("invalid field count, expected {expected} but found {found}")]
    InvalidFieldCount {
        /// Expected field count.
        expected: usize,

        /// Actual field count.
        found: usize
    },

    /// A field contains an invalid numeric value.
    #[error("invalid value: {0}")]
    InvalidValue(String),

    /// A Quartz day rule was used in an invalid context.
    #[error("invalid day rule")]
    InvalidDayRule(String),

    /// A value falls outside the valid range for its field.
    #[error("value out of bounds")]
    OutOfBounds,

    /// The parser expected a seconds field but none was present.
    #[error("missing seconds field")]
    MissingSecondsField,

    /// A seconds field was provided when it is disabled.
    #[error("unexpected seconds field")]
    UnexpectedSecondsField,

    /// The parser expected a year field but none was present.
    #[error("missing year field")]
    MissingYearField,

    /// A year field was provided when it is disabled.
    #[error("unexpected year field")]
    UnexpectedYearField,
}
