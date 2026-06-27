
use crate::cron::{ast::FieldExpr, field::{BitField, BitFieldIter, MAX_YEAR, MIN_YEAR}};
use crate::cron::scheduler::field::Field;

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

    pub fn contains(
        &self,
        value: u32,
    ) -> bool {
        match self {
            CronValue::Bit(bf) => bf.first_set() == Some(value),
            CronValue::Vec(v) => v.binary_search(&value).is_ok(),
        }
    }

    // inclusive search
    pub fn next_or_same(&self, start: u32) -> Option<u32> {
        match self {
            CronValue::Bit(bf) => bf.next_from(start),

            CronValue::Vec(v) => {
                match v.binary_search(&start) {
                    Ok(i) => v.get(i).copied(),
                    Err(i) => v.get(i).copied(),
                }
            }
        }
    }

    pub fn next_from(
        &self,
        start: u32,
    ) -> Option<u32> {
        match self {
            CronValue::Bit(bf) => bf.next_from(start),

            CronValue::Vec(v) => {
                match v.binary_search(&(start + 1)) {
                    Ok(i) => v.get(i).copied(),
                    Err(i) => v.get(i).copied(),
                }
            }
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

                v.get(i)
                    .copied()
                    .or_else(|| v.first().copied())
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
            CronValue::Bit(bf) => {
                CronValueIter::Bit(bf.iter())
            }

            CronValue::Vec(v) => {
                CronValueIter::Vec(v.iter().copied())
            }
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

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum DayRule {
    Any,
    Bits(BitField),
    LastDay,
    LastWeekday(u32),
    NearestWeekday(u32),
    NthWeekday {
        weekday: u32,
        nth: u32,
    },
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

            FieldExpr::LastWeekday(dow) => {
                DayRule::LastWeekday(dow)
            }

            FieldExpr::NearestWeekday(day) => {
                DayRule::NearestWeekday(day)
            }

            FieldExpr::NthWeekday { weekday, nth } => {
                DayRule::NthWeekday { weekday, nth }
            }

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

            FieldExpr::And(a, b) => {
                DayRule::List(vec![
                    DayRule::from(*a),
                    DayRule::from(*b),
                ])
            }
        }
    }
}
