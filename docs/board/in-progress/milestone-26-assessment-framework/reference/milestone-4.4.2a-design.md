# Milestone 4.4.2a: EventLog Migration

> **Status:** ✅ Complete (infrastructure in 4.4.2a, producer wiring in Milestone 18)
> **Parent:** [milestone-4.4-design.md](milestone-4.4-design.md)

## Overview

Migrate from pub/sub `EventBus` to producer/consumer `EventLog` backed by Iggy. This foundational change provides event persistence, crash recovery, and independent consumer progress.

### Goals

1. **Persistence**: All VibesEvents survive daemon restarts
2. **Crash recovery**: Consumers resume from last committed offset
3. **Independent consumers**: Each consumer tracks its own position
4. **Unified infrastructure**: One Iggy instance for all event storage
5. **Structural cleanup**: Move vibes-groove to plugins/ directory

### Non-Goals

- Assessment logic (deferred to 4.4.2b)
- Circuit breaker implementation (deferred to 4.4.2b)
- LLM integration (deferred to 4.4.2b)

---

## Architecture

### Before: Pub/Sub Model

```
                    ┌─────────────────┐
                    │  MemoryEventBus │
                    │   (broadcast)   │
                    └────────┬────────┘
                             │
            ┌────────────────┼────────────────┐
            ▼                ▼                ▼
     ┌──────────┐     ┌──────────┐     ┌──────────┐
     │Subscriber│     │Subscriber│     │Subscriber│
     │    A     │     │    B     │     │    C     │
     └──────────┘     └──────────┘     └──────────┘

Problems:
- Events lost on restart
- Slow subscribers drop events
- Late joiners miss history
- No acknowledgment semantics
```

### After: Producer/Consumer Model

```
     ┌──────────┐
     │ Producer │
     └────┬─────┘
          │ append
          ▼
┌─────────────────────────────────────────┐
│            Iggy Event Log               │
│  [msg0][msg1][msg2][msg3][msg4][msg5]   │
└─────────────────────────────────────────┘
          │
          ├─── Consumer Group A (offset: 3) ──→ SessionCollector
          │
          ├─── Consumer Group B (offset: 5) ──→ AssessmentProcessor
          │
          ├─── Consumer Group C (offset: 5) ──→ WebSocketBroadcaster
          │
          └─── Consumer Group D (offset: 4) ──→ ChatHistoryPersister

Benefits:
- Events persist across restarts
- Each consumer tracks own offset
- Crash recovery via committed offsets
- Late joiners can replay from any point
```

---

## Core Traits

### EventLog

```rust
/// Monotonically increasing position in the log
pub type Offset = u64;

/// The event log - append-only, durable storage
#[async_trait]
pub trait EventLog: Send + Sync {
    /// Append an event to the log, returns its offset
    async fn append(&self, event: VibesEvent) -> Result<Offset>;

    /// Append multiple events atomically
    async fn append_batch(&self, events: Vec<VibesEvent>) -> Result<Offset>;

    /// Create a consumer for a specific consumer group
    /// Each group tracks its own offset independently
    async fn consumer(&self, group: &str) -> Result<Box<dyn EventConsumer>>;

    /// Get the current high-water mark (latest offset)
    fn high_water_mark(&self) -> Offset;
}
```

### EventConsumer

```rust
/// A consumer that reads events and tracks position
#[async_trait]
pub trait EventConsumer: Send {
    /// Poll for the next batch of events
    /// Blocks until events available or timeout
    async fn poll(&mut self, max_count: usize, timeout: Duration) -> Result<EventBatch>;

    /// Commit offset - acknowledges all events up to and including this offset
    async fn commit(&mut self, offset: Offset) -> Result<()>;

    /// Seek to a specific offset (for replay or skip-ahead)
    async fn seek(&mut self, position: SeekPosition) -> Result<()>;

    /// Current committed offset for this consumer
    fn committed_offset(&self) -> Offset;

    /// Consumer group name
    fn group(&self) -> &str;
}

/// Batch of events returned from poll
pub struct EventBatch {
    pub events: Vec<(Offset, VibesEvent)>,
}

impl EventBatch {
    pub fn is_empty(&self) -> bool { self.events.is_empty() }
    pub fn last_offset(&self) -> Option<Offset> {
        self.events.last().map(|(o, _)| *o)
    }
}

/// Where to seek in the log
pub enum SeekPosition {
    /// Start of the log (offset 0)
    Beginning,
    /// End of the log (live tail)
    End,
    /// Specific offset
    Offset(Offset),
}
```

---

## Consumer Groups

Each system component becomes a consumer group with independent offset tracking:

| Consumer Group | Purpose | Start Position | Commit Strategy |
|----------------|---------|----------------|-----------------|
| `session-collector` | Builds transcripts | Beginning | After buffering session |
| `assessment` | Detects signals | Beginning | After processing batch |
| `websocket` | Real-time to web UI | End (live only) | After broadcast |
| `chat-history` | SQLite persistence | Beginning | After DB write |
| `notifications` | Push notifications | End (live only) | After send |

### Replay vs Live Consumers

```rust
// Replay consumer - starts from beginning, catches up to live
let mut collector = log.consumer("session-collector").await?;
// New group starts at offset 0

// Live consumer - starts from end, only new events
let mut websocket = log.consumer("websocket").await?;
websocket.seek(SeekPosition::End).await?;
// Only receives events published after this point
```

---

## Crate Structure

### New: vibes-iggy

```
vibes-iggy/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public exports
│   ├── traits.rs        # EventLog, EventConsumer traits
│   ├── manager.rs       # IggyManager (subprocess lifecycle)
│   ├── config.rs        # IggyConfig
│   ├── log.rs           # IggyEventLog implementation
│   └── consumer.rs      # IggyEventConsumer implementation
```

### Dependencies

```toml
[dependencies]
iggy = "0.6"
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
thiserror = "1"
```

### Dependency Graph (After)

```
vibes-iggy (NEW)
    ├── iggy (SDK)
    └── tokio

vibes-core
    └── vibes-iggy (for EventLog trait re-export)

vibes-server
    ├── vibes-core
    └── vibes-iggy (for IggyEventLog)

plugins/vibes-groove
    ├── vibes-core
    └── vibes-iggy (uses EventLog as consumer)
```

---

## Migration Pattern

### Before: Subscriber Pattern

```rust
// Old: broadcast-based subscription
let bus: Arc<dyn EventBus> = Arc::new(MemoryEventBus::new());

let mut rx = bus.subscribe();
tokio::spawn(async move {
    while let Ok((seq, event)) = rx.recv().await {
        handle_event(event);
    }
});
```

### After: Consumer Pattern

```rust
// New: consumer group with offset tracking
let log: Arc<dyn EventLog> = Arc::new(IggyEventLog::connect(&manager).await?);

let mut consumer = log.consumer("component-name").await?;
tokio::spawn(async move {
    loop {
        let batch = consumer.poll(100, Duration::from_secs(1)).await?;

        for (offset, event) in &batch.events {
            handle_event(event);
        }

        if let Some(offset) = batch.last_offset() {
            consumer.commit(offset).await?;
        }
    }
});
```

---

## Structural Changes

### Move vibes-groove to plugins/

```bash
# Before
vibes/
├── vibes-cli/
├── vibes-core/
├── vibes-groove/        # At root level
├── vibes-server/
└── ...

# After
vibes/
├── vibes-cli/
├── vibes-core/
├── vibes-iggy/          # NEW
├── vibes-server/
├── plugins/
│   └── vibes-groove/    # Moved to plugins/
└── ...
```

Update `Cargo.toml`:
```toml
[workspace]
members = [
    "vibes-cli",
    "vibes-core",
    "vibes-iggy",        # NEW
    "vibes-server",
    "vibes-introspection",
    "plugins/vibes-groove",  # MOVED
]
```

---

## Impact on vibes-groove

With EventLog migration, vibes-groove simplifies:

### Before (4.4.1)
- IggyManager in vibes-groove
- IggyAssessmentLog for assessment events
- Separate event storage from core events

### After (4.4.2a)
- IggyManager moved to vibes-iggy
- No IggyAssessmentLog needed (events already in Iggy!)
- AssessmentProcessor is just a consumer of EventLog
- Groove only owns assessment logic, not infrastructure

```rust
// vibes-groove assessment becomes simple consumer
let mut consumer = log.consumer("assessment").await?;

loop {
    let batch = consumer.poll(100, Duration::from_secs(1)).await?;

    for (_, event) in &batch.events {
        let signals = self.detect_signals(event);
        self.update_circuit_breaker(&signals);

        if self.should_checkpoint() {
            self.run_checkpoint_assessment().await;
        }
    }

    consumer.commit(batch.last_offset().unwrap()).await?;
}
```

---

## Deliverables

1. **vibes-iggy crate**
   - [x] EventLog trait definition
   - [x] EventConsumer trait definition
   - [x] IggyManager (moved from vibes-groove)
   - [x] IggyEventLog implementation
   - [x] IggyEventConsumer implementation
   - [x] Unit tests

2. **vibes-server migration**
   - [x] Replace EventBus with EventLog
   - [x] Migrate SessionCollector to consumer
   - [x] Migrate WebSocket broadcaster to consumer
   - [x] Migrate chat history persister to consumer
   - [x] Migrate notification sender to consumer
   - [x] Update daemon startup sequence
   - [x] Wire event producers to append_event() (Milestone 18)

3. **Structural cleanup**
   - [x] Move vibes-groove to plugins/
   - [x] Update Cargo.toml workspace members
   - [x] Remove IggyManager from vibes-groove
   - [x] Remove IggyAssessmentLog from vibes-groove
   - [x] Delete MemoryEventBus
   - [x] Remove old EventBus trait

4. **Documentation**
   - [x] Update PROGRESS.md
   - [x] Update architecture diagrams in PRD

---

## Exit Criteria

- [x] `vibes-iggy` crate compiles with all tests passing
- [x] Iggy server spawns automatically on daemon startup
- [x] All vibes-server subscribers migrated to consumer pattern
- [x] Events persist across daemon restarts (verified by test)
- [x] Late-joiner replay works (web UI reconnect gets history)
- [x] Consumer lag observable (each consumer's offset visible)
- [x] `vibes-groove` lives in `plugins/` directory
- [x] MemoryEventBus and EventBus trait removed
- [x] All existing integration tests pass
- [x] `just pre-commit` passes

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Iggy SDK complexity | Start with minimal API surface, expand as needed |
| Migration breaks existing functionality | Dual-write during migration, feature flag |
| Performance regression | Benchmark before/after, Iggy is designed for high throughput |
| Iggy server fails to start | Graceful degradation to in-memory mode with warning |

---

## Decision Log

| Decision | Choice | Rationale |
|----------|--------|-----------|
| New crate vs in vibes-core | New `vibes-iggy` crate | Clean separation, iggy is heavy dependency |
| Consumer vs subscriber naming | Consumer (Kafka/Iggy terminology) | Matches mental model of log consumption |
| Offset storage | Iggy-managed | Iggy handles consumer group offsets natively |
| Graceful degradation | Warn and continue without persistence | Better UX than hard failure |
| Move groove to plugins/ | Yes | Architectural clarity, groove is a plugin |
