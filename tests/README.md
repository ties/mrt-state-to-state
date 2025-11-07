# MRT Parser Integration Tests

This directory contains integration tests for the MRT state-to-state parser.

## Test Overview

The integration tests verify that the parser can correctly process real-world MRT files from RIPE NCC's RIS (Routing Information Service) data.

### Test Files

1. **test_processor_basic_functionality** - Unit test for basic processor instantiation (runs without network)
2. **test_parse_ripe_mrt_bview** - Tests parsing a BGP table dump (bview file)
3. **test_parse_single_update_file** - Tests parsing a single MRT update file
4. **test_parse_ripe_mrt_updates** - Full integration test with bview + multiple update files

## Running the Tests

### Quick Test (No Network Required)
```bash
cargo test test_processor_basic_functionality
```

### Integration Tests (Network Required)

The integration tests are marked with `#[ignore]` because they download files from the internet. To run them:

```bash
# Run all tests including ignored ones
cargo test -- --ignored

# Run a specific integration test
cargo test test_parse_ripe_mrt_bview -- --ignored

# Run the full integration test with bview + updates
cargo test test_parse_ripe_mrt_updates -- --ignored
```

### Test Data

The tests automatically download MRT files from:
- **Initial state**: https://data.ris.ripe.net/rrc18/2023.05/bview.20230501.0000.gz
- **Updates**:
  - https://data.ris.ripe.net/rrc18/2023.05/updates.20230501.0000.gz
  - https://data.ris.ripe.net/rrc18/2023.05/updates.20230501.0005.gz
  - https://data.ris.ripe.net/rrc18/2023.05/updates.20230501.0010.gz

Downloaded files are cached in the `test_data/` directory to avoid re-downloading on subsequent test runs. Files are stored in their compressed format (.gz) and decompressed automatically by bgpkit-parser's oneio feature when parsed.

## Test Data Cache

Test files are cached in `test_data/` (in the project root) to speed up repeated test runs. This directory is excluded from git via `.gitignore`.

To clear the cache:
```bash
rm -rf test_data/
```

## What the Tests Verify

1. **Parsing Validity**: Files can be parsed without errors
2. **Data Extraction**: BGP peers and routes are extracted from the files
3. **State Management**: The processor correctly maintains BGP state across multiple update files
4. **Hold Timer Behavior**: BGP hold timers are correctly enforced
5. **Connection State Tracking**: BGP connection state changes are properly tracked

## Test Data Source

The test data comes from RIPE NCC's RRC18 collector, which is located in London, UK. The data is from May 1, 2023.

- RRC18 is a route collector that peers with multiple ISPs and networks
- The bview file contains a full BGP routing table snapshot
- The update files contain incremental BGP routing changes

For more information about RIPE RIS data: https://www.ripe.net/analyse/internet-measurements/routing-information-service-ris
