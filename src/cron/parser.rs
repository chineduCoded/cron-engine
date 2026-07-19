//! Cron expression parser.
//!
//! Responsible for:
//!
//! - tokenization
//! - field parsing
//! - macro expansion
//! - named month resolution
//! - named weekday resolution
//! - syntax validation
//!
//! Output:
//!
//! ```text
//! Expression
//!      │
//!      ▼
//! CronAst
//! ```

#![allow(unused)]

use std::borrow::Cow;

use derive_builder::Builder;
use strum::EnumIs;

use crate::cron::{
    CronError,
    ast::{CronAst, FieldExpr},
    resolver::{month_name, weekday_name},
};

/// Controls whether the seconds field is accepted by the parser.
///
/// This option determines the expected number of fields in a cron
/// expression.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, EnumIs)]
pub enum Seconds {
    /// The seconds field may be present but not required.
    #[default]
    Optional,

    /// The seconds field must be present.
    Required,

    /// The seconds field must not be present.
    Disallowed,
}

/// Controls whether the year field is accepted by the parser.
///
/// This option determines whether a trailing year field is allowed in
/// cron expressions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, EnumIs)]
pub enum Year {
    /// The year field may be present but is not required.
    #[default]
    Optional,

    /// The year field must be present.
    Required,

    /// The year field must not allowed.
    Disallowed,
}

/// Parser for Quartz-compatible cron expressions.
///
/// A `CronParser` converts a textual cron expression into a [`CronAst`].
///
/// By default the parser accepts optional seconds and optional year
///
/// fields. Use [`CronParser::builder`] to customize this behavior.
///
///The parser performs syntax validation only. Semantic validation and
///optimization are performed later by the compiler.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Builder)]
#[builder(default, build_fn(skip), pattern = "owned")]
pub struct CronParser {
    /// Configure how seconds should be handled.
    seconds: Seconds,

    /// Configure how years should be handled.
    year: Year,

    /// Enable the combined Day of Month (DOM) and Day of Week (DOW)
    dom_or_dow: bool,

    /// Use the Quartz-style weekday mode
    alternative_weekdays: bool,

    /// Allow sloppy range syntax (e.g., `0/10` or `/10`) for backward compatibility.
    /// When enabled, patterns like `0/10` (start at 0, step by 10) and `/10` (same as `*/10`)
    /// are accepted. This is not compliant with OCPS/vixie-cron standards.
    sloppy_ranges: bool,
}

impl CronParser {
    /// Create a new parser.
    ///
    /// Prefer [`CronSchedule`]'s implementation of
    /// [`FromStr`][std::str::FromStr] instead of This
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a builder for custom parsin.
    ///
    /// Equivalent to [`CronParserBuilder::default`].
    pub fn builder() -> CronParserBuilder {
        CronParserBuilder::default()
    }

    /// Parses the cron pattern expression.
    pub fn parse(&self, expression: &str) -> Result<CronAst, CronError> {
        let pattern = expression.trim();

        if expression.is_empty() {
            return Err(CronError::InvalidExpression);
        }

        let expanded = Self::expand_macros(
            pattern,
            !self.seconds.is_disallowed(),
            !self.year.is_disallowed(),
        );

        let parts: Vec<&str> = expanded.split_whitespace().collect();

        self.validate_characters(&parts)?;

        let (has_seconds, has_year) = self.field_layout(parts.len())?;

        let mut idx = 0;

        let second = if has_seconds {
            Self::parse_at(&parts, &mut idx, SECOND_SPEC)?
        } else {
            FieldExpr::Value(0)
        };

        let minute = Self::parse_at(&parts, &mut idx, MINUTE_SPEC)?;

        let hour = Self::parse_at(&parts, &mut idx, HOUR_SPEC)?;

        let day = Self::parse_at(&parts, &mut idx, DAY_SPEC)?;

        let month = Self::parse_at(&parts, &mut idx, MONTH_SPEC)?;

        let day_of_week = Self::parse_at(&parts, &mut idx, WEEKDAY_SPEC)?;

        let year = if has_year {
            Self::parse_at(&parts, &mut idx, YEAR_SPEC)?
        } else {
            FieldExpr::Wildcard
        };

        // STRUCTURAL VALIDATION (IMPORTANT)
        if idx != parts.len() {
            return Err(CronError::InvalidFieldCount {
                expected: if has_year {
                    7
                } else if has_seconds {
                    6
                } else {
                    5
                },
                found: parts.len(),
            });
        }

        self.validate_configuration(has_seconds, has_year)?;

        Ok(CronAst {
            second,
            minute,
            hour,
            day_of_month: day,
            month,
            day_of_week,
            year: Some(year),
            dom_dow_or: self.dom_or_dow,
        })
    }

    fn validate_configuration(&self, has_seconds: bool, has_year: bool) -> Result<(), CronError> {
        match self.seconds {
            Seconds::Required if !has_seconds => {
                return Err(CronError::MissingSecondsField);
            }

            Seconds::Disallowed if has_seconds => {
                return Err(CronError::UnexpectedSecondsField);
            }

            _ => {}
        }

        match self.year {
            Year::Required if !has_year => {
                return Err(CronError::MissingYearField);
            }

            Year::Disallowed if has_year => {
                return Err(CronError::UnexpectedYearField);
            }

            _ => {}
        }

        Ok(())
    }

    // Validates that the cron pattern only contained allowed characters for each fields
    fn validate_characters(&self, parts: &[&str]) -> Result<(), CronError> {
        for part in parts {
            for ch in part.chars() {
                if ch.is_ascii_alphanumeric() {
                    continue;
                }

                match ch {
                    '*' | ',' | '-' | '/' | '#' | 'L' | 'W' | '?' => {}

                    _ => return Err(CronError::IllegalCharacters { chars: ch.into() }),
                }
            }
        }

        Ok(())
    }

    // Expands named cron pattern macros into their equivalent standard.
    fn expand_macros(expression: &str, with_seconds: bool, with_year: bool) -> Cow<'_, str> {
        match expression.trim() {
            "@yearly" | "@annually" => Self::build_macro("0 0 1 1 *", with_seconds, with_year),

            "@monthly" => Self::build_macro("0 0 1 * *", with_seconds, with_year),

            "@weekly" => Self::build_macro("0 0 * * 0", with_seconds, with_year),

            "@daily" | "@midnight" => Self::build_macro("0 0 * * *", with_seconds, with_year),

            "@hourly" => Self::build_macro("0 * * * *", with_seconds, with_year),

            _ => Cow::Borrowed(expression),
        }
    }

    fn build_macro(base: &str, with_seconds: bool, with_year: bool) -> Cow<'_, str> {
        let mut result = String::new();

        if with_seconds {
            result.push_str("0 ");
        }

        result.push_str(base);

        if with_year {
            result.push_str(" *");
        }

        result.into()
    }

    /// Parses a single cron field.
    ///
    /// The field is interpreted according to the supplied
    /// [`FieldKind`], allowing support for field-specific syntax such
    /// as month names, weekday names, `L`, `W`, and `#`.
    ///
    /// Comma-separated expressions are parsed into /// [`FieldExpr::List`].
    ///
    /// # Errors
    ///
    /// Returns a [`CronError`] if the field is empty or contains invalid
    /// syntax.
    pub fn parse_field(input: &str, kind: FieldKind) -> Result<FieldExpr, CronError> {
        let input = input.trim();

        if input.is_empty() {
            return Err(CronError::EmptyPattern);
        }

        let mut items = Vec::new();

        for segment in input.split(',') {
            items.push(Self::parse_segment(segment.trim(), kind)?);
        }

        Ok(match items.len() {
            1 => items.remove(0),
            _ => FieldExpr::List(items),
        })
    }

    fn parse_segment(input: &str, kind: FieldKind) -> Result<FieldExpr, CronError> {
        if input == "*" {
            return Ok(FieldExpr::Wildcard);
        }

        if input.eq_ignore_ascii_case("LW") {
            return Ok(FieldExpr::LastBusinessDay);
        }

        if input.eq_ignore_ascii_case("L") {
            return Ok(FieldExpr::LastDay);
        }

        if let Some(day) = input.strip_suffix('W') {
            let day = Self::parse_token(day, kind)?;
            return Ok(FieldExpr::NearestWeekday(day));
        }

        if let Some(day) = input.strip_suffix('L') {
            if !day.is_empty() {
                let weekday = Self::parse_token(day, kind)?;
                return Ok(FieldExpr::LastWeekday(weekday));
            }
        }

        if let Some((weekday, nth)) = input.split_once('#') {
            let weekday = Self::parse_token(weekday, kind)?;
            let nth = nth
                .parse::<u32>()
                .map_err(|_| CronError::InvalidValue(input.into()))?;

            return Ok(FieldExpr::NthWeekday { weekday, nth });
        }

        if let Some((base, step)) = input.split_once('/') {
            let base = Self::parse_segment(base, kind)?;

            let step = step
                .parse::<u32>()
                .map_err(|_| CronError::InvalidValue(input.into()))?;

            if step == 0 {
                return Err(CronError::InvalidValue("step cannot be zero".into()));
            }

            return Ok(FieldExpr::Step(Box::new(base), step));
        }

        if let Some((start, end)) = input.split_once('-') {
            let start = Self::parse_token(start, kind)?;
            let end = Self::parse_token(end, kind)?;

            if start > end {
                return Err(CronError::InvalidValue(format!(
                    "invalid range {start}-{end}"
                )));
            }

            return Ok(FieldExpr::Range(start, end));
        }
        Ok(FieldExpr::Value(Self::parse_token(input, kind)?))
    }

    fn parse_token(input: &str, kind: FieldKind) -> Result<u32, CronError> {
        if let Some(value) = Self::resolve_named(input, kind) {
            return Ok(value);
        }

        input
            .parse::<u32>()
            .map_err(|_| CronError::InvalidValue(input.into()))
    }

    fn parse_at(parts: &[&str], idx: &mut usize, spec: FieldSpec) -> Result<FieldExpr, CronError> {
        let value = parts.get(*idx).ok_or(CronError::InvalidExpression)?;

        *idx += 1;

        Self::parse_field(value, spec.kind)
    }

    fn resolve_named(input: &str, kind: FieldKind) -> Option<u32> {
        match kind {
            FieldKind::Month => month_name(input),
            FieldKind::Weekday => weekday_name(input),
            _ => None,
        }
    }

    fn field_layout(&self, count: usize) -> Result<(bool, bool), CronError> {
        match count {
            5 => Ok((false, false)),
            6 => Ok((true, false)),
            7 => Ok((true, true)),
            _ => Err(CronError::InvalidFieldCount {
                expected: 5,
                found: count,
            }),
        }
    }
}

impl CronParserBuilder {
    /// Builds a configured [`CronParser`].
    ///
    /// The resulting parser is immutable and may be reused to parse
    /// multiple cron expressions.
    pub fn build(self) -> CronParser {
        let CronParserBuilder {
            seconds,
            year,
            dom_or_dow,
            alternative_weekdays,
            sloppy_ranges,
        } = self;
        CronParser {
            seconds: seconds.unwrap_or_default(),
            year: year.unwrap_or_default(),
            dom_or_dow: dom_or_dow.unwrap_or_default(),
            alternative_weekdays: alternative_weekdays.unwrap_or_default(),
            sloppy_ranges: sloppy_ranges.unwrap_or_default(),
        }
    }
}

/// Identifies the cron field currently being parsed.
///
/// Different field kinds support different syntax and value ranges.
/// For example, months accept month names while day fields support
/// Quartz-specific calendar rules.
#[derive(Debug, Clone, Copy)]
pub enum FieldKind {
    /// Seconds (`0..=59`).
    Second,

    /// Minutes (`0..=59`).
    Minute,

    /// Hours (`0..=23`).
    Hour,

    /// Day of the month.
    Day,

    /// Month of the year.
    Month,

    /// Day of the week.
    Weekday,

    /// Calendar year.
    Year,
}

#[derive(Debug, Clone, Copy)]
struct FieldSpec {
    kind: FieldKind,
    min: u32,
    max: u32,
}

impl FieldSpec {
    const fn new(kind: FieldKind, min: u32, max: u32) -> Self {
        Self { kind, min, max }
    }
}

const SECOND_SPEC: FieldSpec = FieldSpec::new(FieldKind::Second, 0, 59);

const MINUTE_SPEC: FieldSpec = FieldSpec::new(FieldKind::Minute, 0, 59);

const HOUR_SPEC: FieldSpec = FieldSpec::new(FieldKind::Hour, 0, 23);

const DAY_SPEC: FieldSpec = FieldSpec::new(FieldKind::Day, 1, 31);

const MONTH_SPEC: FieldSpec = FieldSpec::new(FieldKind::Month, 1, 12);

const WEEKDAY_SPEC: FieldSpec = FieldSpec::new(FieldKind::Weekday, 0, 6);

const YEAR_SPEC: FieldSpec = FieldSpec::new(FieldKind::Year, 1970, 2099);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cron::parser::CronParser;

    #[test]
    fn step_zero_is_invalid() {
        let expr = CronParser::parse_field("*/0", FieldKind::Minute);

        assert!(expr.is_err());
    }

    #[test]
    fn expands_yearly() {
        let result = CronParser::expand_macros("@yearly", false, false);

        assert_eq!(result, "0 0 1 1 *");
    }

    #[test]
    fn expands_yearly_with_seconds() {
        let result = CronParser::expand_macros("@yearly", true, false);

        assert_eq!(result, "0 0 0 1 1 *");
    }

    #[test]
    fn expands_yearly_with_year() {
        let result = CronParser::expand_macros("@yearly", true, true);

        assert_eq!(result, "0 0 0 1 1 * *");
    }

    #[test]
    fn leaves_regular_pattern_unchanged() {
        let result = CronParser::expand_macros("*/5 * * * *", false, false);

        assert_eq!(result, "*/5 * * * *");
    }

    #[test]
    fn test_invalid_field_count_too_few() {
        let parser = CronParser::new();

        let err = parser.parse("* * *").unwrap_err();

        matches!(err, CronError::InvalidFieldCount { .. });
    }

    #[test]
    fn test_invalid_field_count_too_many() {
        let parser = CronParser::new();

        let err = parser.parse("* * * * * * * *").unwrap_err();

        matches!(err, CronError::InvalidFieldCount { .. });
    }

    #[test]
    fn test_seconds_required_missing() {
        let parser = CronParser::builder().seconds(Seconds::Required).build();

        let err = parser.parse("* * * * *").unwrap_err();

        matches!(err, CronError::MissingSecondsField);
    }

    #[test]
    fn test_seconds_disallowed_but_present() {
        let parser = CronParser::builder().seconds(Seconds::Disallowed).build();

        let err = parser.parse("0 * * * * *").unwrap_err();

        matches!(err, CronError::UnexpectedSecondsField);
    }

    #[test]
    fn test_empty_list_error() {
        let parser = CronParser::new();

        let err = parser.parse("1,,3 * * * * *").unwrap_err();

        matches!(err, CronError::InvalidValue(_));
    }
}
