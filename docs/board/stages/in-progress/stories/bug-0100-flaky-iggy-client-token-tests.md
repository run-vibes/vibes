---
title: Fix flaky iggy_client token cache tests
type: bug
status: in-progress
created: 2026-01-13
---

## Problem

The iggy_client token cache tests are flaky:
- `iggy_client::tests::load_cached_token_returns_none_when_no_file`
- `iggy_client::tests::cache_and_load_token_roundtrip`

These tests pass in isolation but sometimes fail when run with the full test suite due to shared state or file system race conditions.

## Cause

Likely causes:
1. Tests sharing the same cache file path without proper isolation
2. Tests not cleaning up temp files before/after
3. Race condition between parallel test execution

## Solution

- Ensure each test uses a unique temp directory
- Add proper cleanup in test teardown
- Consider using `tempfile` crate for automatic cleanup
