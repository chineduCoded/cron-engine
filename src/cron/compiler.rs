use crate::cron::{CronError, ast::{CronAst, FieldExpr}, field::{BITFIELD_MAX_WIDTH, BitField}, ir::{CronIr, CronValue, DayRule, FieldMatcher}};

#[derive(Debug, Clone, Copy)]
pub enum DayField {
    DayOfMonth,
    DayOfWeek,
}

pub struct CronCompiler;

impl CronCompiler {
    pub fn compile(
        ast: CronAst,
    ) -> Result<CronIr, CronError> {
        Self::validate_day_rule(
            &ast.day_of_month, 
            DayField::DayOfMonth,
        )?;

        Self::validate_day_rule(
            &ast.day_of_week,
            DayField::DayOfWeek,
        )?;

        Ok(CronIr {
            second: Self::compile_field(
                &ast.second,
                0,
                59,
            ),

            minute: Self::compile_field(
                &ast.minute,
                0,
                59,
            ),

            hour: Self::compile_field(
                &ast.hour,
                0,
                23,
            ),

            day_of_month: Self::compile_day_rule(
                &ast.day_of_month,
                1,
                31,
            ),

            month: Self::compile_field(
                &ast.month,
                1,
                12,
            ),

            day_of_week: Self::compile_day_rule(
                &ast.day_of_week,
                0,
                6,
            ),

            year: ast.year.as_ref().map(|expr| {
                Self::compile_field(
                    expr,
                    1,
                    5000,
                )
            }),

            dom_dow_or: ast.dom_dow_or,
        })
    }

    pub fn compile_field(
        expr: &FieldExpr,
        min: u32,
        max: u32,
    ) -> FieldMatcher {
        FieldMatcher {
            value: Self::compile_value(expr, min, max),
        }
    }

    pub fn compile_day_rule(
        expr: &FieldExpr,
        min: u32,
        max: u32,
    ) -> DayRule {
        match expr {
            FieldExpr::Wildcard => DayRule::Any,

            FieldExpr::LastDay => DayRule::LastDay,

            FieldExpr::LastBusinessDay => DayRule::LastBusinessDay,

            FieldExpr::LastWeekday(day) => {
                DayRule::LastWeekday(*day)
            }

            FieldExpr::NearestWeekday(day) => {
                DayRule::NearestWeekday(*day)
            }

            FieldExpr::NthWeekday { weekday, nth } => {
                DayRule::NthWeekday { 
                    weekday: *weekday, 
                    nth: *nth, 
                }
            }

            FieldExpr::List(items) => {
                DayRule::List(
                    items
                        .iter()
                        .map(|item| {
                            Self::compile_day_rule(
                                item, 
                                min, 
                                max
                            )
                        })
                        .collect(),
                )
            }

            _ => DayRule::Bits(Self::compile_bits(expr, min, max)),
        }
    }

    fn validate_day_rule(
        expr: &FieldExpr,
        field: DayField,
    ) -> Result<(), CronError> {
        match expr {
            FieldExpr::Wildcard
            | FieldExpr::Value(_)
            | FieldExpr::Range(_, _)
            | FieldExpr::Step(_, _) => {}

            FieldExpr::LastDay => {
                if !matches!(field, DayField::DayOfMonth) {
                    return Err(
                        CronError::InvalidDayRule(
                            "L is only valid in day-of-month".into(),
                        )
                    );
                }
            }

            FieldExpr::LastBusinessDay => {
                if !matches!(field, DayField::DayOfMonth) {
                    return Err(
                        CronError::InvalidDayRule(
                            "LW is only valid day-of-month".into(),
                        )
                    );
                }
            }

            FieldExpr::NearestWeekday(day) => {
                if !matches!(field, DayField::DayOfMonth) {
                    return Err(
                        CronError::InvalidDayRule(
                            "W is only valid in day-of-month".into(),
                        )
                    );
                }

                if *day == 0 || * day > 31 {
                    return Err(
                        CronError::InvalidDayRule(
                            format!(
                                "invalid nearest weekday '{}'",
                                day
                           )
                        )
                    );
                }
            }

            FieldExpr::LastWeekday(day) => {
                if !matches!(field, DayField::DayOfWeek) {
                    return Err(
                        CronError::InvalidDayRule(
                            "xL is only valid in day-of-week"
                                .into(),
                        ),
                    );
                }

                if *day > 6 {
                    return Err(
                        CronError::InvalidDayRule(
                            format!(
                                "invalid weekday '{}'",
                                day
                            ),
                        ),
                    );
                }
            }

            FieldExpr::NthWeekday { weekday, nth } => {
                if !matches!(field, DayField::DayOfWeek) {
                    return Err(
                        CronError::InvalidDayRule(
                            "# is only valid in day-of-week".into(),
                        )
                    );
                }

                if *weekday > 6 {
                    return Err(
                        CronError::InvalidDayRule(
                            format!("invalid weekday '{}'", weekday)
                        )
                    );
                }

                if *nth == 0 || *nth > 5 {
                    return Err(
                        CronError::InvalidDayRule(
                            format!("invalid nth value '{}'", nth)
                        )
                    );
                }
            }

            FieldExpr::List(items) => {
                for item in items {
                    Self::validate_day_rule(
                        item,
                        field,
                    )?;
                }
            }

            FieldExpr::And(left, right) => {
                Self::validate_day_rule(left, field)?;
                Self::validate_day_rule(right, field)?;
            }
        }

        Ok(())
    }

    pub fn compile_bits(
        expr: &FieldExpr,
        min: u32,
        max: u32,
    ) -> BitField {
        assert!(min <= max, "invalid range: min > max ({min}, {max})");

        let width = max - min + 1;
        assert!(width <= 64, "BitField overflow: width={width}");

        let mut bits = BitField::empty(min, width);
        Self::compile_into(expr, &mut bits, min, max);

        bits
    }

    fn compile_into(
        expr: &FieldExpr,
        bits: &mut BitField,
        min: u32,
        max: u32,
    ) {
        match expr {
            FieldExpr::Wildcard => {
                *bits = BitField::full(min, max - min + 1);
            }

            FieldExpr::Value(v) => {
                if *v >= min && *v <= max {
                    bits.set(*v);
                }
            }

            FieldExpr::Range(start, end) => {
                let start = (*start).max(min);
                let end = (*end).min(max);

                for value in start..=end {
                    bits.set(value);
                }
            }

            FieldExpr::List(items) => {
                for item in items {
                    Self::compile_into(
                        item, 
                        bits, 
                        min, 
                        max
                    );
                }
            }

            FieldExpr::Step(base, step) => {
                Self::compile_step(base, bits, *step, min, max);
            }

            _ => {}
        }
    }

    fn compile_step(
        base: &FieldExpr,
        out: &mut BitField,
        step: u32,
        min: u32,
        max: u32,
    ) {
        match base {
            FieldExpr::Wildcard => {
                let mut value = min;

                while value <= max {
                    out.set(value);
                    value += step;
                }
            }

            FieldExpr::Range(start, end) => {
                let mut value = *start;

                while value <= *end {
                    if value >= min && value <= max {
                        out.set(value);
                    }

                    value += step;
                }
            }

            _ => {
                let bits =
                    Self::compile_bits(base, min, max);

                let mut first = None;

                for value in min..=max {
                    if bits.contains(value) {
                        first = Some(value);
                        break;
                    }
                }

                let Some(origin) = first else {
                    return;
                };

                for value in min..=max {
                    if bits.contains(value)
                        && ((value - origin) % step == 0)
                    {
                        out.set(value);
                    }
                }
            }
        }
    }

    pub fn compile_value(expr: &FieldExpr, min: u32, max: u32) -> CronValue {
        let width = max - min + 1;

        if width <= BITFIELD_MAX_WIDTH {
            let bf = Self::compile_bits(expr, min, max);
            return CronValue::Bit(bf);
        }

        let mut set = Vec::new();
        Self::compile_into_vec(expr, min, max, &mut set);
        CronValue::Vec(set)
    }

    fn compile_into_vec(expr: &FieldExpr, min: u32, max: u32, out: &mut Vec<u32>) {
        match expr {
            FieldExpr::Wildcard => {
                for v in min..=max {
                    out.push(v);
                }
            }

            FieldExpr::Value(v) => {
                if *v >= min && *v <= max {
                    out.push(*v);
                }
            }

            FieldExpr::Range(a, b) => {
                for v in (*a).max(min)..=(*b).min(max) {
                    out.push(v);
                }
            }

            FieldExpr::List(items) => {
                for i in items {
                    Self::compile_into_vec(i, min, max, out);
                }
            }

            _ => {
                // semantic constructs handled elsewhere
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cron::compiler::CronCompiler;

    #[test]
    fn wildcard_sets_everything() {
        let bits = CronCompiler::compile_bits(
            &FieldExpr::Wildcard,
            0,
            59,
        );

        for v in 0..=59 {
            assert!(bits.contains(v));
        }
    }

    #[test]
    fn single_value() {
        let bits = CronCompiler::compile_bits(
            &FieldExpr::Value(15),
            0,
            59,
        );

        assert!(bits.contains(15));
        assert!(!bits.contains(14));
        assert!(!bits.contains(16));
    }

    #[test]
    fn value_outside_range() {
        let bits = CronCompiler::compile_bits(
            &FieldExpr::Value(100),
            0,
            59,
        );

        assert!(bits.is_empty());
    }

    #[test]
    fn range() {
        let bits = CronCompiler::compile_bits(
            &FieldExpr::Range(10, 15),
            0,
            59,
        );

        for v in 10..=15 {
            assert!(bits.contains(v));
        }

        assert!(!bits.contains(9));
        assert!(!bits.contains(16));
    }

    #[test]
    fn range_clamps() {
        let bits = CronCompiler::compile_bits(
            &FieldExpr::Range(50, 100),
            0,
            59,
        );

        for v in 50..=59 {
            assert!(bits.contains(v));
        }
    }

    #[test]
    fn list() {
        let bits = CronCompiler::compile_bits(
            &FieldExpr::List(vec![
                FieldExpr::Value(1),
                FieldExpr::Value(5),
                FieldExpr::Value(10),
            ]),
            0,
            59,
        );

        assert!(bits.contains(1));
        assert!(bits.contains(5));
        assert!(bits.contains(10));
    }

    #[test]
    fn duplicate_values() {
        let bits = CronCompiler::compile_bits(
            &FieldExpr::List(vec![
                FieldExpr::Value(5),
                FieldExpr::Value(5),
            ]),
            0,
            59,
        );

        assert!(bits.contains(5));
    }

    #[test]
    fn wildcard_step() {
        let bits = CronCompiler::compile_bits(
            &FieldExpr::Step(
                Box::new(FieldExpr::Wildcard),
                15,
            ),
            0,
            59,
        );

        assert!(bits.contains(0));
        assert!(bits.contains(15));
        assert!(bits.contains(30));
        assert!(bits.contains(45));

        assert!(!bits.contains(14));
    }

    #[test]
    fn range_step() {
        let bits = CronCompiler::compile_bits(
            &FieldExpr::Step(
                Box::new(FieldExpr::Range(10, 20)),
                5,
            ),
            0,
            59,
        );

        assert!(bits.contains(10));
        assert!(bits.contains(15));
        assert!(bits.contains(20));

        assert!(!bits.contains(11));
    }

    #[test]
    fn step_one() {
        let bits = CronCompiler::compile_bits(
            &FieldExpr::Step(
                Box::new(FieldExpr::Wildcard),
                1,
            ),
            0,
            59,
        );

        for v in 0..=59 {
            assert!(bits.contains(v));
        }
    }

    #[test]
    fn huge_step() {
        let bits = CronCompiler::compile_bits(
            &FieldExpr::Step(
                Box::new(FieldExpr::Wildcard),
                100,
            ),
            0,
            59,
        );

        assert!(bits.contains(0));

        for v in 1..=59 {
            assert!(!bits.contains(v));
        }
    }

    #[test]
    fn nested_expressions() {
        let expr = FieldExpr::List(vec![
            FieldExpr::Value(1),
            FieldExpr::Range(5,10),
            FieldExpr::Step(
                Box::new(FieldExpr::Wildcard),
                20,
            ),
        ]);

        let bits = CronCompiler::compile_bits(
            &expr,
            0,
            59,
        );

        assert!(bits.contains(1));
        assert!(bits.contains(5));
        assert!(bits.contains(20));
        assert!(bits.contains(40));
    }
}
