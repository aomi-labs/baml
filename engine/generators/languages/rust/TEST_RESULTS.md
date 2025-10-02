# Rust Generator Test Results

This document lists all test folders under `generators/languages/rust/generated_tests/` and shows which tests pass with `cargo test --package generators-rust --lib -- <test_name>`.

## Test Results

| Test Folder | Status | Notes |
|-------------|--------|-------|
| array_types | ✅ PASS | Both consistent and evaluate tests pass |
| asserts | ✅ PASS | Both consistent and evaluate tests pass |
| classes | ✅ PASS | Both consistent and evaluate tests pass |
| edge_cases | ✅ PASS | Both consistent and evaluate tests pass |
| enums | ✅ PASS | Both consistent and evaluate tests pass |
| literal_types | ✅ PASS | Both consistent and evaluate tests pass |
| map_types | ✅ PASS | Both consistent and evaluate tests pass |
| media_types | ✅ PASS | Both consistent and evaluate tests pass |
| mixed_complex_types | ✅ PASS | Both consistent and evaluate tests pass |
| nested_structures | ✅ PASS | Both consistent and evaluate tests pass |
| optional_nullable | ✅ PASS | Both consistent and evaluate tests pass |
| primitive_types | ✅ PASS | Both consistent and evaluate tests pass |
| recursive_types | ✅ PASS | Both consistent and evaluate tests pass |
| sample | ✅ PASS | Both consistent and evaluate tests pass |
| semantic_streaming | ✅ PASS | Both consistent and evaluate tests pass |
| unions | ✅ PASS | Both consistent and evaluate tests pass |
| union_types_extended | ✅ PASS | Both consistent and evaluate tests pass |

## Summary

- **Total Tests**: 17 test folders
- **Passing**: 17 tests (100%) ✅
- **Failing**: 0 tests (0%) ❌

## Detailed Results

### All Tests Passing (17/17) 🎉

Excellent news! All tests are now passing successfully. This represents a **100% success rate** for the Rust generator test suite.

#### Test Categories:

- **Basic Types**: `array_types`, `primitive_types`, `literal_types`
- **Complex Types**: `map_types`, `mixed_complex_types`, `nested_structures`
- **Advanced Features**: `enums`, `unions`, `union_types_extended`, `optional_nullable`, `recursive_types`
- **Special Cases**: `edge_cases`, `asserts`, `semantic_streaming`
- **Media Types**: `media_types`
- **Sample**: `sample`

### Recent Improvements

The `union_types_extended` test has been fixed and is now passing! This was the last failing test, bringing the success rate from 94% to 100%.

## Test Command

To run individual tests:
```bash
cd /Users/han/github/baml/engine
cargo test --package generators-rust --lib -- <test_name>
```

## Last Updated

Updated: $(date)