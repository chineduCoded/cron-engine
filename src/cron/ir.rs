use crate::cron::scheduler::field::Field;
use crate::cron::{
    ast::FieldExpr,
    field::{BitField, BitFieldIter, MAX_YEAR, MIN_YEAR},
};

/// Optimized intermediate representation (IR) of a compiled cron 
/// expression. 
///
/// `CronIr` is produced by the compiler from a parsed [`CronAst`] and is 
/// consumed by the scheduler during occurrence calculation. 
///
/// Numeric fields are represented by [`FieldMatcher`]s for efficient 
/// membership tests, while day-of-month and day-of-week fields retain 
/// their Quartz-specific semantics through [`DayRule`].
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct CronIr {
    /// Allowed second values.
    pub second: FieldMatcher,

    /// Allowed minute values.
    pub minute: FieldMatcher,

    /// Allowed hour values.
    pub hour: FieldMatcher,

    ///Day-of-month matching rule.
    pub day_of_month: DayRule,

    /// Allowed month values.
    pub month: FieldMatcher,

    /// Day-of-week matching rule.
    pub day_of_week: DayRule,

    /// Allowed years. 
    ///
    /// If `None`, the schedule is valid for all supported years.
    pub year: Option<FieldMatcher>,

    /// Controls how day-of-month and day-of-week rules are combined. 
    ///
    /// - `true` → match if either rule matches (OR) 
    /// - `false` → both rules must match (AND)
    pub dom_dow_or: bool,
}

impl CronIr {
    /// Returns the smallest year accepted by the schedule. 
    ///
    /// If no year constraint is present, [`MIN_YEAR`] is returned.
    pub fn min_year(&self) -> u32 {
        self.year
            .as_ref()
            .and_then(FieldMatcher::min)
            .unwrap_or(MIN_YEAR)
    }

    /// Returns the largest year accepted by the schedule. 
    ///
    /// If no year constraint is present, [`MAX_YEAR`] is returned.
    pub fn max_year(&self) -> u32 {
        self.year
            .as_ref()
            .and_then(FieldMatcher::max)
            .unwrap_or(MAX_YEAR)
    }

    /// Returns the smallest allowed hour. 
    ///
    /// # Panics 
    ///
    /// Panics if the hour matcher is empty.
    pub fn min_hour(&self) -> u32 {
        self.hour.min().expect("hour matcher cannot be empty")
    }

    /// Returns the smallest allowed minute. 
    ///
    /// # Panics 
    ///
    /// Panics if the minute matcher is empty.
    pub fn min_minute(&self) -> u32 {
        self.minute.min().expect("minute matcher cannot be empty")
    }

    /// Returns the smallest allowed second. 
    ///
    /// # Panics 
    ///
    /// Panics if the second matcher is empty.
    pub fn min_second(&self) -> u32 {
        self.second.min().expect("second matcher cannot be empty")
    }

    /// Returns the smallest allowed month. 
    ///
    /// # Panics 
    ///
    /// Panics if the month matcher is empty.
    pub fn min_month(&self) -> u32 {
        self.month.min().expect("month matcher cannot be empty")
    }

    /// Returns the numeric matcher associated with a scheduler field. 
    ///
    /// Returns `None` for [`Field::Day`], since day matching is handled 
    /// by [`DayRule`] rather than a numeric matcher.
    pub fn matcher(&self, field: Field) -> Option<&FieldMatcher> {
        match field {
            Field::Year => self.year.as_ref(),

            Field::Month => Some(&self.month),

            Field::Hour => Some(&self.hour),

            Field::Minute => Some(&self.minute),

            Field::Second => Some(&self.second),

            Field::Day => None,
        }
    }
}

/// Storage representation for the values accepted by a cron field. 
///
/// Small, dense domains (such as seconds, minutes, hours, months, and 
/// weekdays) are represented as a [`BitField`] for constant-time 
/// membership tests. 
///
/// Larger or sparse domains (such as years) are represented as a sorted 
/// vector. 
///
/// Both variants expose the same lookup interface used by the scheduler.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CronValue {
    /// Bitset-backed representation.
    Bit(BitField),

    /// Sorted collection of allowed values.
    Vec(Vec<u32>),
}

impl CronValue {
    /// Sorts and removes duplicate values. 
    ///
    /// This ensures the vector representation remains suitable for 
    /// binary-search-based lookups.
    pub fn normalize_vec(mut v: Vec<u32>) -> Vec<u32> {
        v.sort_unstable();
        v.dedup();
        v
    }

    /// Returns whether the given value is contained in this field.
    pub fn contains(&self, value: u32) -> bool {
        match self {
            CronValue::Bit(bf) => bf.contains(value),
            CronValue::Vec(v) => v.binary_search(&value).is_ok(),
        }
    }

    /// Returns the first matching value greater than or equal to `start`. 
    ///
    /// This is an inclusive search.
    pub fn next_or_same(&self, start: u32) -> Option<u32> {
        match self {
            CronValue::Bit(bf) => bf.next_from(start),

            CronValue::Vec(v) => match v.binary_search(&start) {
                Ok(i) => v.get(i).copied(),
                Err(i) => v.get(i).copied(),
            },
        }
    }

    /// Returns the first matching value strictly greater than `start`.
    pub fn next_from(&self, start: u32) -> Option<u32> {
        match self {
            CronValue::Bit(bf) => bf.next_from(start),

            CronValue::Vec(v) => match v.binary_search(&(start + 1)) {
                Ok(i) => v.get(i).copied(),
                Err(i) => v.get(i).copied(),
            },
        }
    }

    /// Returns the next matching value, wrapping to the minimum value if 
    /// necessary. 
    ///
    /// This is primarily used when advancing numeric scheduler fields.
    pub fn next_wrapping(&self, start: u32) -> Option<u32> {
        match self {
            Self::Bit(bf) => bf.next_wrapping(start),
            Self::Vec(v) => {
                let i = match v.binary_search(&start) {
                    Ok(i) => i,
                    Err(i) => i,
                };

                v.get(i).copied().or_else(|| v.first().copied())
            }
        }
    }

    /// Returns the underlying values for the vector representation. 
    ///
    /// # Panics 
    ///
    /// Panics if this value is backed by a [`BitField`]. Use 
    /// [`CronValue::iter`] instead when the storage representation is 
    /// unknown.
    pub fn values(&self) -> &[u32] {
        // small inconsistency avoided by returning slice via enum match
        // (no allocation needed in BitField path)
        match self {
            CronValue::Bit(_) => unreachable!("use iter_bit"),
            CronValue::Vec(v) => v,
        }
    }

    /// Returns the minimum allowed value. 
    ///
    /// Returns `None` if the field contains no values.
    pub fn min(&self) -> Option<u32> {
        match self {
            CronValue::Bit(bf) => bf.first_set(),
            CronValue::Vec(v) => v.first().copied(),
        }
    }

    /// Returns the maximum allowed value. 
    ///
    /// Returns `None` if the field contains no values.
    pub fn max(&self) -> Option<u32> {
        match self {
            CronValue::Bit(bf) => bf.last_set(),
            CronValue::Vec(v) => v.last().copied(),
        }
    }

    /// Returns an iterator over the allowed values. 
    ///
    /// Values are yielded in ascending order regardless of the internal 
    /// storage representation.
    pub fn iter(&self) -> CronValueIter<'_> {
        match self {
            CronValue::Bit(bf) => CronValueIter::Bit(bf.iter()),

            CronValue::Vec(v) => CronValueIter::Vec(v.iter().copied()),
        }
    }
}

/// Iterator over the values contained in a [`CronValue`]. 
///
/// This enum provides a unified iterator interface for both bitset- 
/// backed and vector-backed representations.
pub enum CronValueIter<'a> {
    /// Iterator over values stored in a [`BitField`].
    Bit(BitFieldIter),

    /// Iterator over values stored in a sorted vector.
    Vec(std::iter::Copied<std::slice::Iter<'a, u32>>),
}

impl Iterator for CronValueIter<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CronValueIter::Bit(iter) => iter.next(),
            CronValueIter::Vec(iter) => iter.next(),
        }
    }
}

/// Efficient matcher for a numeric cron field. 
///
/// `FieldMatcher` provides constant-time or logarithmic-time lookup, 
/// depending on the underlying representation.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct FieldMatcher {
    /// Allowed values for the field.
    pub value: CronValue,
}

impl FieldMatcher {
    /// Returns whether the given value is accepted by this field.
    pub fn contains(&self, value: u32) -> bool {
        self.value.contains(value)
    }

    /// Returns the first matching value greater than or equal to 
    /// `value`.
    pub fn next_or_same(&self, value: u32) -> Option<u32> {
        self.value.next_or_same(value)
    }

    /// Returns the next matching value, wrapping to the minimum value if 
    /// necessary.
    pub fn next_wrapping(&self, value: u32) -> Option<u32> {
        self.value.next_wrapping(value)
    }

    /// Returns the first matching value strictly greater than `value`.
    pub fn next(&self, value: u32) -> Option<u32> {
        self.value.next_from(value)
    }

    /// Returns the minimum value accepted by this field.
    pub fn min(&self) -> Option<u32> {
        self.value.min()
    }

    /// Returns the maximum value accepted by this field.
    pub fn max(&self) -> Option<u32> {
        self.value.max()
    }
}

/// Backward-compatible constructor
impl From<BitField> for FieldMatcher {
    fn from(bf: BitField) -> Self {
        Self {
            value: CronValue::Bit(bf),
        }
    }
}

/// Quartz-compatible matching rule for day-of-month and day-of-week 
/// fields. 
///
/// Unlike other cron fields, day fields support calendar-aware 
/// expressions such as `L`, `LW`, `W`, and `#`.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum DayRule {
    /// Matches every day.
    Any,

    /// Matches a fixed set of numeric day values.
    Bits(BitField),

    /// Matches the last day of the month (`L`).
    LastDay,

    /// Matches the last occurrence of the specified weekday in the month 
    /// (for example, `5L`).
    LastWeekday(u32),

    /// Matches the last business day (Monday–Friday) of the month 
    /// (`LW`).
    LastBusinessDay,

    /// Matches the weekday nearest the specified day of the month 
    /// (for example, `15W`).
    NearestWeekday(u32),

    /// Matches the *n*th occurrence of a weekday within the month 
    /// (for example, `1#3` for the third Monday).
    NthWeekday {
        /// Weekday (`0 = Sunday` through `6 = Saturday`).
        weekday: u32,

        /// One-based occurrence within the month.
        nth: u32 
    },

    /// Matches if any contained rule matches. 
    ///
    /// Used for comma-separated day expressions.
    List(Vec<DayRule>),
}

impl From<FieldExpr> for DayRule {
    fn from(expr: FieldExpr) -> Self {
        match expr {
            FieldExpr::Wildcard => DayRule::Any,

            FieldExpr::Value(v) => {
                let mut bf = BitField::empty(1, 31);
                bf.set(v);
                DayRule::Bits(bf)
            }

            FieldExpr::Range(start, end) => {
                let mut bf = BitField::empty(1, 31);

                for v in start..=end {
                    bf.set(v);
                }

                DayRule::Bits(bf)
            }

            FieldExpr::List(items) => {
                let rules = items.into_iter().map(DayRule::from).collect();
                DayRule::List(rules)
            }

            FieldExpr::LastDay => DayRule::LastDay,

            FieldExpr::LastWeekday(dow) => DayRule::LastWeekday(dow),

            FieldExpr::LastBusinessDay => DayRule::LastBusinessDay,

            FieldExpr::NearestWeekday(day) => DayRule::NearestWeekday(day),

            FieldExpr::NthWeekday { weekday, nth } => DayRule::NthWeekday { weekday, nth },

            FieldExpr::Step(inner, step) => {
                let base = DayRule::from(*inner);

                match base {
                    DayRule::Bits(bf) => {
                        let mut out = BitField::empty(bf.offset(), bf.width());

                        let mut i = bf.min_value();
                        while i <= bf.max_value() {
                            if bf.contains(i) {
                                out.set(i);
                            }
                            i += step;
                        }

                        DayRule::Bits(out)
                    }
                    other => other,
                }
            }

            FieldExpr::And(a, b) => DayRule::List(vec![DayRule::from(*a), DayRule::from(*b)]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::cron::{
        evaluator::{
            calendar::{Calendar, Weekday},
            day::{evaluate_dom_rule, evaluate_dow_rule, matches_day},
        },
        field::BitField,
        ir::{CronIr, DayRule},
    };

    fn bits(values: &[u32]) -> BitField {
        let mut bf = BitField::empty(1, 31);

        for &v in values {
            bf.set(v);
        }

        bf
    }

    fn cal(year: i32, month: u32, day: u32) -> Calendar {
        Calendar::new(year, month, day)
    }

    fn matcher(values: &[u32]) -> FieldMatcher {
        FieldMatcher::from(bits(values))
    }

    fn ir(dom: DayRule, dow: DayRule, dom_dow_or: bool) -> CronIr {
        CronIr {
            second: matcher(&[0]),
            minute: matcher(&[0]),
            hour: matcher(&[0]),
            month: matcher(&[1]),

            day_of_month: dom,
            day_of_week: dow,

            year: None,

            dom_dow_or,
        }
    }

    #[test]
    fn matches_day_last_business_day() {
        let ir = ir(DayRule::LastBusinessDay, DayRule::Any, true);

        let cal = Calendar::new(2025, 5, 30);

        assert!(matches_day(&ir, &cal));
    }

    #[test]
    fn dom_any_matches_everyday() {
        let cal = cal(2025, 6, 17);

        assert!(evaluate_dom_rule(&DayRule::Any, &cal,));
    }

    #[test]
    fn dom_bits_matches() {
        let cal = cal(2025, 6, 15);

        assert!(evaluate_dom_rule(&DayRule::Bits(bits(&[5, 10, 15])), &cal,));
    }

    #[test]
    fn dom_bits_rejects() {
        let cal = cal(2025, 6, 14);

        assert!(!evaluate_dom_rule(&DayRule::Bits(bits(&[5, 10, 15])), &cal,));
    }

    #[test]
    fn dom_last_day_matches() {
        let cal = cal(2025, 2, 28);

        assert!(evaluate_dom_rule(&DayRule::LastDay, &cal,));
    }

    #[test]
    fn dom_last_day_leap_year() {
        let cal = cal(2024, 2, 29);

        assert!(evaluate_dom_rule(&DayRule::LastDay, &cal,));
    }

    #[test]
    fn dom_last_day_rejects() {
        let cal = cal(2025, 2, 27);

        assert!(!evaluate_dom_rule(&DayRule::LastDay, &cal,));
    }

    #[test]
    fn dom_nearest_weekday_matches() {
        // June 1, 2024 was Saturday.
        // Nearest weekday = Monday June 3.
        let cal = cal(2024, 6, 3);

        assert!(evaluate_dom_rule(&DayRule::NearestWeekday(1), &cal,));
    }

    #[test]
    fn dom_nearest_weekday_rejects_other_days() {
        let cal = cal(2024, 6, 2);

        assert!(!evaluate_dom_rule(&DayRule::NearestWeekday(1), &cal,));
    }

    #[test]
    fn dom_list_matches() {
        let rule = DayRule::List(vec![DayRule::Bits(bits(&[5])), DayRule::LastDay]);

        let cal = cal(2025, 6, 5);

        assert!(evaluate_dom_rule(&rule, &cal,));
    }

    #[test]
    fn dow_any_matches() {
        let cal = cal(2025, 6, 17);

        assert!(evaluate_dow_rule(&DayRule::Any, &cal,));
    }

    #[test]
    fn dow_bits_matches() {
        let cal = cal(2025, 6, 16); // Monday

        assert!(evaluate_dow_rule(&DayRule::Bits(bits(&[1])), &cal,));
    }

    #[test]
    fn dow_bits_rejects() {
        let cal = cal(2025, 6, 16);

        assert!(!evaluate_dow_rule(&DayRule::Bits(bits(&[2])), &cal,));
    }

    #[test]
    fn dow_last_weekday_matches() {
        // Last Friday of May 2025 = 30
        let cal = cal(2025, 5, 30);

        assert!(evaluate_dow_rule(
            &DayRule::LastWeekday(Weekday::new(5).into(),),
            &cal,
        ));
    }

    #[test]
    fn dow_last_weekday_rejects() {
        let cal = cal(2025, 5, 23);

        assert!(!evaluate_dow_rule(
            &DayRule::LastWeekday(Weekday::new(5).into(),),
            &cal,
        ));
    }

    #[test]
    fn dow_nth_weekday_matches() {
        // Third Monday of June 2025 = 16
        let cal = cal(2025, 6, 16);

        assert!(evaluate_dow_rule(
            &DayRule::NthWeekday {
                weekday: Weekday::new(1).into(),
                nth: 3,
            },
            &cal,
        ));
    }

    #[test]
    fn dow_nth_weekday_rejects() {
        let cal = cal(2025, 6, 9);

        assert!(!evaluate_dow_rule(
            &DayRule::NthWeekday {
                weekday: Weekday::new(1).into(),
                nth: 3,
            },
            &cal,
        ));
    }

    #[test]
    fn dow_list_matches() {
        let rule = DayRule::List(vec![DayRule::LastWeekday(Weekday::new(5).into())]);

        let cal = cal(2025, 5, 30);

        assert!(evaluate_dow_rule(&rule, &cal,));
    }

    #[test]
    fn matches_day_any_any() {
        let cal = cal(2025, 6, 17);

        assert!(matches_day(&ir(DayRule::Any, DayRule::Any, true,), &cal,));
    }

    #[test]
    fn matches_day_dom_only() {
        let cal = cal(2025, 6, 15);

        assert!(matches_day(
            &ir(DayRule::Bits(bits(&[15])), DayRule::Any, true,),
            &cal,
        ));
    }

    #[test]
    fn matches_day_dow_only() {
        let cal = cal(2025, 6, 16);

        assert!(matches_day(
            &ir(DayRule::Any, DayRule::Bits(bits(&[1])), true,),
            &cal,
        ));
    }

    #[test]
    fn matches_day_or_logic() {
        let cal = cal(2025, 6, 16);

        let ir = ir(DayRule::Bits(bits(&[15])), DayRule::Bits(bits(&[1])), true);

        assert!(matches_day(&ir, &cal));
    }

    #[test]
    fn matches_day_and_logic() {
        let cal = cal(2025, 6, 16);

        let ir = ir(DayRule::Bits(bits(&[16])), DayRule::Bits(bits(&[1])), false);

        assert!(matches_day(&ir, &cal));
    }

    #[test]
    fn matches_day_and_fails() {
        let cal = cal(2025, 6, 16);

        let ir = ir(DayRule::Bits(bits(&[15])), DayRule::Bits(bits(&[1])), false);

        assert!(!matches_day(&ir, &cal));
    }

    #[test]
    fn dom_nested_list_matches() {
        let rule = DayRule::List(vec![
            DayRule::Bits(bits(&[5])),
            DayRule::List(vec![DayRule::Bits(bits(&[10])), DayRule::Bits(bits(&[15]))]),
        ]);

        let cal = cal(2025, 6, 15);

        assert!(evaluate_dom_rule(&rule, &cal));
    }

    #[test]
    fn dow_nested_list_matches() {
        let rule = DayRule::List(vec![
            DayRule::Bits(bits(&[2])),
            DayRule::List(vec![DayRule::Bits(bits(&[1]))]),
        ]);

        let cal = cal(2025, 6, 16); // Monday

        assert!(evaluate_dow_rule(&rule, &cal));
    }

    #[test]
    fn dom_list_rejects() {
        let rule = DayRule::List(vec![DayRule::Bits(bits(&[5])), DayRule::Bits(bits(&[10]))]);

        let cal = cal(2025, 6, 15);

        assert!(!evaluate_dom_rule(&rule, &cal));
    }

    #[test]
    fn dow_list_rejects() {
        let rule = DayRule::List(vec![DayRule::Bits(bits(&[2])), DayRule::Bits(bits(&[3]))]);

        let cal = cal(2025, 6, 16); // Monday

        assert!(!evaluate_dow_rule(&rule, &cal));
    }

    #[test]
    fn last_weekday_all_weekdays() {
        for weekday in 0..7 {
            let day = Calendar::last_weekday(2025, 12, Weekday::try_from(weekday).unwrap());

            let cal = cal(2025, 12, day);

            assert!(evaluate_dow_rule(&DayRule::LastWeekday(weekday), &cal,));
        }
    }

    #[test]
    fn nth_weekday_all_occurrences() {
        for nth in 1..=5 {
            if let Some(day) = Calendar::nth_weekday(2025, 6, 1, nth) {
                let cal = cal(2025, 6, day);

                assert!(evaluate_dow_rule(
                    &DayRule::NthWeekday { weekday: 1, nth },
                    &cal,
                ));
            }
        }
    }

    #[test]
    fn impossible_nth_weekday_returns_false() {
        let cal = cal(2025, 2, 28);

        assert!(!evaluate_dow_rule(
            &DayRule::NthWeekday { weekday: 1, nth: 5 },
            &cal,
        ));
    }

    #[test]
    fn nearest_weekday_from_sunday() {
        // June 1 2025 = Sunday
        // nearest weekday = Monday June 2

        let cal = cal(2025, 6, 2);

        assert!(evaluate_dom_rule(&DayRule::NearestWeekday(1), &cal,));
    }

    #[test]
    fn last_day_30_day_month() {
        let cal = cal(2025, 4, 30);

        assert!(evaluate_dom_rule(&DayRule::LastDay, &cal,));
    }

    #[test]
    fn last_day_31_day_month() {
        let cal = cal(2025, 1, 31);

        assert!(evaluate_dom_rule(&DayRule::LastDay, &cal,));
    }

    #[test]
    fn matches_day_or_both_fail() {
        let cal = cal(2025, 6, 16);

        let ir = ir(DayRule::Bits(bits(&[15])), DayRule::Bits(bits(&[2])), true);

        assert!(!matches_day(&ir, &cal));
    }

    #[test]
    fn matches_day_or_both_match() {
        let cal = cal(2025, 6, 16);

        let ir = ir(DayRule::Bits(bits(&[16])), DayRule::Bits(bits(&[1])), true);

        assert!(matches_day(&ir, &cal));
    }

    #[test]
    fn matches_day_and_dom_only_fails() {
        let cal = cal(2025, 6, 16);

        let ir = ir(DayRule::Bits(bits(&[16])), DayRule::Bits(bits(&[2])), false);

        assert!(!matches_day(&ir, &cal));
    }

    #[test]
    fn matches_day_and_dow_only_fails() {
        let cal = cal(2025, 6, 16);

        let ir = ir(DayRule::Bits(bits(&[15])), DayRule::Bits(bits(&[1])), false);

        assert!(!matches_day(&ir, &cal));
    }

    #[test]
    fn any_ignores_dom_dow_operator() {
        let cal = cal(2025, 6, 16);

        let ir = ir(DayRule::Any, DayRule::Any, false);

        assert!(matches_day(&ir, &cal));
    }
}
