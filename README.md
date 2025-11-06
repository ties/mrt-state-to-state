# MRT state-to-state

**This is an extremely rough and quick prototype.**

Build future BGP state from a full table dump (bview) and a set of MRT updates.

## Features

- Parse MRT BGP table dumps (bview files)
- Process MRT update files with BGP routing changes
- Track BGP connection states and hold timers
- Maintain routing table state across multiple update files

## Building

```bash
cargo build --release
```

## Running

Create a configuration file (see `config.yml` for an example) and run:

```bash
cargo run -- --config config.yml
```

## Testing

The project includes integration tests that parse real-world MRT files from RIPE NCC.

```bash
# Run basic unit tests
cargo test

# Run integration tests (requires network access to download test data)
cargo test -- --ignored
```

See `tests/README.md` for more details about the test suite.
