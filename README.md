# cron-engine 

[![Crates.io](https://img.shields.io/crates/v/cron-engine.svg)](https://crates.io/crates/cron-engine) 
[![Documentation](https://docs.rs/cron-engine/badge.svg)](https://docs.rs/cron-engine) 
[![License](https://img.shields.io/crates/l/cron-engine.svg)](LICENSE) 

A high-performance, Quartz-compatible cron parser and scheduler written in Rust. 

`cron-engine` parses cron expressions into an optimized intermediate representation (IR) and computes future or previous occurrences using efficient bitset-based evaluation with minimal allocations. 

## Features 

- Quartz-compatible syntax 
- Optional seconds field 
- Optional year field 
- Time zone aware scheduling 
- DST-aware occurrence calculation 
- Immutable schedules 
- Forward and backward navigation 
- Lazy iterators 
- Efficient bitfield matching 
- Property tested 
- Benchmarked with Criterion 

### Supported syntax 

| Feature | Supported |
|----------|-----------|
| Wildcard `*` | ✓ |
| Lists `1,2,3` | ✓ |
| Ranges `1-5` | ✓ |
| Steps `*/5` | ✓ |
| Month names | ✓ |
| Weekday names | ✓ |
| `L` | ✓ |
| `LW` | ✓ |
| `W` | ✓ |
| `#` | ✓ |
| `5L` | ✓ | 

## Installation 

```toml
[dependencies]
cron-engine = "0.1" 
``` 

## Example 
```rust 
use chrono::{TimeZone, Timelike; 
use chrono_tz::UTC; 
use cron_engine::CronSchedule;

let schedule = CronSchedule::parse("0 */15 * * * *")?; 

let now = UTC
    .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
    .unwrap();

let next = schedule.next_after(now).unwrap(); 

println!("{next}");

# Ok::<(), cron_engine::CronError>(())
```

## Architecture See [ARCHITECTURE.md](ARCHITECTURE.md).

## Performance 

Benchmarks are maintained using Criterion. 

Typical results:

- BitField contains: ~350 ps 
- BitField next: ~340 ps 
- Scheduler next occurrence: <1 µs 
- 1,000,000 occurrences: ~0.8 s 

See [BENCHMARKS.md](BENCHMARKS.md). 

## Documentation 

```bash
cargo doc --open
``` 

## Testing 
```bash
cargo test
cargo bench
```

## Contributing See [CONTRIBUTING.md](CONTRIBUTING.md). 

## License MIT OR Apache-2.0
