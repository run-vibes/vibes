---
id: REFACTOR0141
title: "Refactor: Single Partition with Event ID Seeking"
type: refactor
status: done
priority: medium
scope: groove/36-novelty-monitoring
depends: []
estimate:
created: 2026-01-02
---

# Refactor: Single Partition with Event ID Seeking

> **For Claude:** Use superpowers:executing-plans to implement this story.

## Goal

Migrate from 8 partitions to a single partition, making offsets globally unique and enabling timestamp-based seeking via UUIDv7 event IDs.

## Context

The current 8-partition architecture creates unnecessary complexity:

### Current Problems

1. **Offset semantics are broken** - Offsets are partition-local, so `before_offset: 50` means something different for each partition. The `load_events_before_offset` function seeks ALL partitions to offset 50, which is semantically wrong.

2. **Cross-partition ordering requires workaround** - Events must be sorted by `event_id` (UUIDv7) after loading because partition offsets have no global ordering relationship.

3. **Consumer complexity** - Tracks 8 separate offsets, distributes poll counts (`per_partition = count/8`), handles per-partition seeking.

4. **Testing complexity** - `PartitionedInMemoryEventLog` exists solely to test the multi-partition workarounds.

5. **Frontend acknowledges the problem** - The type explicitly says "Partition-scoped offset (not unique across partitions, use event_id for keying)".

### Why Partitions Don't Help Us

Partitions are useful for:
- **Parallelizing consumers** - We have a single firehose consumer per WebSocket
- **Session affinity** - We don't process events per-session; we show all events
- **Throughput scaling** - We're nowhere near Iggy's single-partition limits

### Benefits of Single Partition

1. **Offset = global position** - Can use offset for seeking directly
2. **Simpler consumer** - One offset to track, no distribution math
3. **Clean pagination** - `before_offset: 50` means "events 0-49"
4. **Event ID seeking still works** - UUIDv7 timestamp enables Iggy's timestamp-based polling
5. **Less code** - Remove partition tracking, remove workarounds

## Design Decision: Event ID vs Offset for Seeking

**Decision: Switch to event_id-based seeking from frontend**

Rationale:
- `event_id` is part of the domain model, not an implementation detail
- UUIDv7 embeds millisecond timestamp - can use Iggy's `PollingStrategy::timestamp()`
- If we ever migrate storage backends, event_ids are stable
- Offset remains useful internally but frontend shouldn't depend on it

Protocol change:
```typescript
// Before
{ type: "fetch_older", before_offset: 50 }

// After
{ type: "fetch_older", before_event_id: "01936f8a-..." }
```

## Tasks

### Task 1: Change partition count to 1

**Files:**
- Modify: `vibes-iggy/src/iggy_log.rs`

**Steps:**
1. Change `PARTITION_COUNT` to 1
2. Update log message in `connect()` to not mention partitions
3. Run tests: `cargo test -p vibes-iggy`
4. Commit: `refactor(iggy): use single partition for events topic`

### Task 2: Simplify IggyEventConsumer

**Files:**
- Modify: `vibes-iggy/src/iggy_log.rs`

**Steps:**
1. Change offset arrays from `[u64; 8]` to single `u64`
2. Remove partition iteration in `poll()` - just poll partition 0
3. Remove partition iteration in `commit()` and `seek()`
4. Simplify `committed_offset()` - return single value
5. Remove `active_partitions` tracking (no longer needed)
6. Run tests: `cargo test -p vibes-iggy`
7. Commit: `refactor(iggy): simplify consumer for single partition`

### Task 3: Add SeekPosition::BeforeEventId

**Files:**
- Modify: `vibes-iggy/src/traits.rs`
- Modify: `vibes-iggy/src/iggy_log.rs` (seek implementation)
- Modify: `vibes-iggy/src/memory.rs` (seek implementation)

**Steps:**
1. Add `BeforeEventId(Uuid)` variant to `SeekPosition` enum
2. For IggyEventConsumer: extract timestamp from UUIDv7, use `PollingStrategy::timestamp()`
3. For InMemoryEventLog: binary search events by event_id
4. Write tests for timestamp seeking
5. Run tests: `cargo test -p vibes-iggy`
6. Commit: `feat(iggy): add SeekPosition::BeforeEventId for timestamp seeking`

### Task 4: Update firehose protocol

**Files:**
- Modify: `vibes-server/src/ws/firehose.rs`

**Steps:**
1. Change `FirehoseClientMessage::FetchOlder` to use `before_event_id: Uuid`
2. Rename `load_events_before_offset` to `load_events_before_event_id`
3. Use new `SeekPosition::BeforeEventId` for pagination
4. Update `FirehoseEventsBatch` to include `oldest_event_id` instead of `oldest_offset`
5. Keep offset in individual events (useful for debugging)
6. Update tests
7. Run tests: `cargo test -p vibes-server`
8. Commit: `refactor(firehose): use event_id for pagination seeking`

### Task 5: Update frontend hook

**Files:**
- Modify: `web-ui/src/hooks/useFirehose.ts`

**Steps:**
1. Track `oldestEventId: string | null` instead of `oldestOffset`
2. Change `FetchOlderMessage` to send `before_event_id`
3. Parse `oldest_event_id` from `EventsBatchMessage`
4. Update `fetchOlder()` to use event ID
5. Update comments to reflect new semantics
6. Run tests: `npm test --workspace=web-ui`
7. Commit: `refactor(web-ui): use event_id for firehose pagination`

### Task 6: Remove or simplify PartitionedInMemoryEventLog

**Files:**
- Remove or simplify: `vibes-iggy/src/partitioned_memory.rs`
- Modify: `vibes-iggy/src/lib.rs` (remove export if deleting)
- Modify: `vibes-server/src/ws/firehose.rs` (update test that uses it)

**Steps:**
1. Decide: remove entirely or keep as 1-partition variant?
2. If removing: delete file and update lib.rs exports
3. If keeping: change to single partition to match Iggy behavior
4. Update firehose test `load_historical_events_sorts_by_event_id_across_partitions`
5. Run tests: `cargo test -p vibes-iggy && cargo test -p vibes-server`
6. Commit: `refactor(iggy): remove PartitionedInMemoryEventLog`

### Task 7: Update integration tests

**Files:**
- Modify: `vibes-iggy/tests/integration.rs`

**Steps:**
1. Update any assertions about partition counts
2. Test event_id-based seeking
3. Verify offset is now globally unique
4. Run tests: `just test-all`
5. Commit: `test(iggy): update integration tests for single partition`

### Task 8: End-to-end verification

**Steps:**
1. Start vibes server: `just build && ./target/release/vibes serve`
2. Open web UI firehose
3. Generate events, verify live streaming
4. Scroll up to trigger pagination
5. Verify "Load more" fetches correct older events
6. Check browser console for event_id-based requests
7. Verify no errors in server logs
8. Run full test suite: `just pre-commit`
9. Commit any fixes discovered

## Migration Notes

### Existing Iggy Data

If upgrading an existing vibes installation with data:
- The topic has 8 partitions with existing messages
- Option A: Delete the topic on upgrade (data loss, simple)
- Option B: Create new topic "events_v2" with 1 partition (no data loss, migration path)
- Recommendation: Option A for now (vibes is pre-1.0, data loss acceptable)

### Backward Compatibility

- The wire protocol changes (`before_event_id` vs `before_offset`)
- Old frontend + new server = won't paginate (server ignores old field)
- New frontend + old server = won't paginate (old server ignores new field)
- Both must be updated together (acceptable for internal tool)

## Acceptance Criteria

- [ ] `PARTITION_COUNT` is 1
- [ ] `IggyEventConsumer` tracks single offset (not array)
- [ ] Frontend sends `before_event_id` for pagination
- [ ] Server uses `SeekPosition::BeforeEventId` for pagination
- [ ] Pagination works correctly (fetch older shows earlier events)
- [ ] Live event streaming still works
- [ ] All tests pass
- [ ] No cross-partition sorting workarounds needed

## Out of Scope

- Keeping backward compatibility with old wire protocol
- Migrating existing Iggy data from 8 partitions
- Changing how events are stored (still JSON in Iggy)
