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
