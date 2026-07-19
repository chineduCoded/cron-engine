# BENCHMARKS.md

````markdown
# Benchmarks

Benchmarks are implemented with Criterion.

Run:

```bash
cargo bench
```

## Categories

### Parser

- wildcard
- lists
- ranges
- names
- L
- W
- LW
- #
- year

### Compiler

- simple
- complex

### Scheduler

- every second
- every 5 minutes
- hourly
- daily
- monthly
- last weekday

### Calendar

- weekday
- leap year
- days in month
- nearest weekday
- nth weekday
- last weekday
- last business day

### Candidate

- reset
- normalize
- field access

### Navigator

- min
- max
- next

### BitField

- contains
- next
- iteration

### Pathological

- Feb 29
- 31st
- Last weekday
- Fifth Monday

### Impossible schedules

Benchmarks worst-case search behaviour.

### Throughput

- 1,000
- 10,000
- 100,000
- 1,000,000

## Heap profiling

Memory allocations can be profiled using DHAT.

Example:

```
Total allocations:
486 MB
513k allocations
Peak:
8.7 MB
```

## Philosophy

Benchmarks focus on:

- latency
- throughput
- allocation behaviour
- pathological schedules
- realistic workloads
````
