pub fn month_name(input: &str) -> Option<u32> {
    match input.as_bytes() {
        b"JAN" | b"jan" | b"Jan" => Some(1),
        b"FEB" | b"feb" | b"Feb" => Some(2),
        b"MAR" | b"mar" | b"Mar" => Some(3),
        b"APR" | b"apr" | b"Apr" => Some(4),
        b"MAY" | b"may" | b"May" => Some(5),
        b"JUN" | b"jun" | b"Jun" => Some(6),
        b"JUL" | b"jul" | b"Jul" => Some(7),
        b"AUG" | b"aug" | b"Aug" => Some(8),
        b"SEP" | b"sep" | b"Sep" => Some(9),
        b"OCT" | b"oct" | b"Oct" => Some(10),
        b"NOV" | b"nov" | b"Nov" => Some(11),
        b"DEC" | b"dec" | b"Dec" => Some(12),
        _ => None,
    }
}

pub fn weekday_name(input: &str) -> Option<u32> {
    match input.as_bytes() {
        b"SUN" | b"sun" | b"Sun" => Some(0),
        b"MON" | b"mon" | b"Mon" => Some(1),
        b"TUE" | b"tue" | b"Tue" => Some(2),
        b"WED" | b"wed" | b"Wed" => Some(3),
        b"THU" | b"thu" | b"Thu" => Some(4),
        b"FRI" | b"fri" | b"Fri" => Some(5),
        b"SAT" | b"sat" | b"Sat" => Some(6),
        _ => None,
    }
}

pub fn month_lookup(input: &[u8]) -> Option<u32> {
    match input {
        b"JAN" | b"jan" => Some(1),
        b"FEB" | b"feb" => Some(2),
        b"MAR" | b"mar" => Some(3),
        b"APR" | b"apr" => Some(4),
        b"MAY" | b"may" => Some(5),
        b"JUN" | b"jun" => Some(6),
        b"JUL" | b"jul" => Some(7),
        b"AUG" | b"aug" => Some(8),
        b"SEP" | b"sep" => Some(9),
        b"OCT" | b"oct" => Some(10),
        b"NOV" | b"nov" => Some(11),
        b"DEC" | b"dec" => Some(12),
        _ => None,
    }
}

pub fn weekday_lookup(input: &[u8]) -> Option<u32> {
    match input {
        b"SUN" | b"sun" => Some(0),
        b"MON" | b"mon" => Some(1),
        b"TUE" | b"tue" => Some(2),
        b"WED" | b"wed" => Some(3),
        b"THU" | b"thu" => Some(4),
        b"FRI" | b"fri" => Some(5),
        b"SAT" | b"sat" => Some(6),
        _ => None,
    }
}
