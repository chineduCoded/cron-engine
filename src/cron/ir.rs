use crate::cron::scheduler::field::Field;
use crate::cron::{
    ast::FieldExpr,
    field::{BitField, BitFieldIter, MAX_YEAR, MIN_YEAR},
};

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct CronIr {
    pub second: FieldMatcher,
    pub minute: FieldMatcher,
    pub hour: FieldMatcher,

    pub day_of_month: DayRule,
    pub month: FieldMatcher,
    pub day_of_week: DayRule,

    pub year: Option<FieldMatcher>,

    pub dom_dow_or: bool,
}

impl CronIr {
    pub fn min_year(&self) -> u32 {
        self.year
            .as_ref()
            .and_then(FieldMatcher::min)
            .unwrap_or(MIN_YEAR)
    }

    pub fn max_year(&self) -> u32 {
        self.year
            .as_ref()
            .and_then(FieldMatcher::max)
            .unwrap_or(MAX_YEAR)
    }

    pub fn min_hour(&self) -> u32 {
        self.hour.min().expect("hour matcher cannot be empty")
    }

    pub fn min_minute(&self) -> u32 {
        self.minute.min().expect("minute matcher cannot be empty")
    }

    pub fn min_second(&self) -> u32 {
        self.second.min().expect("second matcher cannot be empty")
    }

    pub fn min_month(&self) -> u32 {
        self.month.min().expect("month matcher cannot be empty")
    }

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CronValue {
    Bit(BitField),
    Vec(Vec<u32>),
}

impl CronValue {
    pub fn normalize_vec(mut v: Vec<u32>) -> Vec<u32> {
        v.sort_unstable();
        v.dedup();
        v
    }

    pub fn contains(&self, value: u32) -> bool {
        match self {
            CronValue::Bit(bf) => bf.contains(value),
            CronValue::Vec(v) => v.binary_search(&value).is_ok(),
        }
    }

    // inclusive search
    pub fn next_or_same(&self, start: u32) -> Option<u32> {
        match self {
            CronValue::Bit(bf) => bf.next_from(start),

            CronValue::Vec(v) => match v.binary_search(&start) {
                Ok(i) => v.get(i).copied(),
                Err(i) => v.get(i).copied(),
            },
        }
    }

    pub fn next_from(&self, start: u32) -> Option<u32> {
        match self {
            CronValue::Bit(bf) => bf.next_from(start),

            CronValue::Vec(v) => match v.binary_search(&(start + 1)) {
                Ok(i) => v.get(i).copied(),
                Err(i) => v.get(i).copied(),
            },
        }
    }

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

    pub fn values(&self) -> &[u32] {
        // small inconsistency avoided by returning slice via enum match
        // (no allocation needed in BitField path)
        match self {
            CronValue::Bit(_) => unreachable!("use iter_bit"),
            CronValue::Vec(v) => v,
        }
    }

    pub fn min(&self) -> Option<u32> {
        match self {
            CronValue::Bit(bf) => bf.first_set(),
            CronValue::Vec(v) => v.first().copied(),
        }
    }

    pub fn max(&self) -> Option<u32> {
        match self {
            CronValue::Bit(bf) => bf.last_set(),
            CronValue::Vec(v) => v.last().copied(),
        }
    }

    pub fn iter(&self) -> CronValueIter<'_> {
        match self {
            CronValue::Bit(bf) => CronValueIter::Bit(bf.iter()),

            CronValue::Vec(v) => CronValueIter::Vec(v.iter().copied()),
        }
    }
}

pub enum CronValueIter<'a> {
    Bit(BitFieldIter),
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

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct FieldMatcher {
    pub value: CronValue,
}

impl FieldMatcher {
    pub fn contains(&self, value: u32) -> bool {
        self.value.contains(value)
    }

    pub fn next_or_same(&self, value: u32) -> Option<u32> {
        self.value.next_or_same(value)
    }

    pub fn next_wrapping(&self, value: u32) -> Option<u32> {
        self.value.next_wrapping(value)
    }

    pub fn next(&self, value: u32) -> Option<u32> {
        self.value.next_from(value)
    }

    pub fn min(&self) -> Option<u32> {
        self.value.min()
    }

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

/// Calendar matching rule.
///
/// Supports Quartz extensions.
///
/// # Supported
///
/// - `L`
/// - `LW`
/// - `15W`
/// - `2#3`
/// - `5L`
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum DayRule {
    Any,
    Bits(BitField),
    LastDay,
    LastWeekday(u32),
    LastBusinessDay,
    NearestWeekday(u32),
    NthWeekday { weekday: u32, nth: u32 },
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
