# cron-engine

A production-ready Quartz-compatible cron scheduler written in Rust.

## Features

- Quartz cron expressions
- Optional seconds field
- Optional year field
- Timezone-aware scheduling
- DST handling
- Last day (L)
- Last weekday (xL)
- Last business day (LW)
- Nearest weekday (W)
- Nth weekday (#)
- Zero-allocation scheduler
- High-performance BitField execution
- Property tested
- Criterion benchmarked

## Installation

```toml
[dependencies]
cron-engine = "0.1"
```

## Example

```rust
use cron_engine::cron::CronSchedule;

let schedule =
    CronSchedule::parse("0 */5 * * * *")?;

let next =
    schedule.next_after(chrono::Utc::now());

println!("{:?}", next);
```

## Documentation

```
cargo doc --open
```

## Benchmarks

See BENCHMARKS.md.

## License

MIT
