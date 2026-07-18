# ARCHITECTURE.md

````markdown
# Architecture

## Overview

The library follows a staged compilation pipeline.

```
Expression
        ▼
    Parser
        ▼
      AST
        ▼
   Compiler
        ▼
        IR
        ▼
   Scheduler
        ▼
Occurrences
```

## Parser

Responsible for:

- tokenization
- syntax validation
- Quartz extensions
- AST construction

Produces:

```
CronAst
```

---

## Compiler

Transforms the AST into an optimized Intermediate Representation.

Responsibilities:

- compile numeric fields
- compile day rules
- optimize storage
- build bitfields

Produces:

```
CronIr
```

---

## Intermediate Representation

The IR is immutable.

Numeric fields use:

- BitField
- CronValue
- FieldMatcher

Day fields use:

- DayRule

---

## Scheduler

Consumes the IR to compute occurrences.

Responsibilities:

- next occurrence
- previous occurrence
- iterator support
- timezone handling
- DST correctness

---

## Calendar Evaluator

Provides Quartz day semantics:

- L
- LW
- W
- #
- 5L

---

## Design Goals

- immutable
- thread-safe
- allocation-free hot path
- predictable performance
- Quartz compatible
- timezone aware
````

---

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
