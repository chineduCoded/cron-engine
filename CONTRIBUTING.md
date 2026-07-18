# CONTRIBUTING.md

````markdown
# Contributing

Contributions are welcome.

## Development

Clone the repository.

```bash
git clone ...
cd cron-engine
```

## Formatting

```bash
cargo fmt --all
```

## Linting

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

## Testing

```bash
cargo test
```

## Benchmarks

```bash
cargo bench
```

## Documentation

```bash
cargo doc --no-deps
```

## Pull Requests

Please ensure:

- tests pass
- documentation is updated
- benchmarks are not regressed
- public APIs are documented
- clippy passes
- formatting passes

## Commit Style

Recommended:

```
feat:
fix:
perf:
docs:
refactor:
test:
bench:
chore:
```
````
