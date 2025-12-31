# Milestone 4.4.2b Implementation Plan: Assessment Logic

This plan covers completing the EventLog migration (remaining 4.4.2a tasks) and implementing the assessment logic (4.4.2b).

## Prerequisites

- vibes-iggy crate exists with `EventLog`, `EventConsumer` traits
- `InMemoryEventLog` implementation complete
- `IggyManager` integrated into vibes-groove

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                     Iggy EventLog                                 │
│  [VibesEvent0][VibesEvent1][VibesEvent2]...[VibesEventN]         │
└───────────────────────────┬──────────────────────────────────────┘
                            │
     ┌──────────────────────┼───────────────────────────┐
     │                      │                           │
     ▼                      ▼                           ▼
┌─────────────────┐   ┌─────────────────┐      ┌────────────────────┐
│ websocket       │   │ chat-history    │      │ assessment         │
│ (SeekPosition:: │   │ (SeekPosition:: │      │ (SeekPosition::    │
│  End, live)     │   │  Beginning)     │      │  Beginning)        │
└─────────────────┘   └─────────────────┘      └─────────┬──────────┘
                                                         │
                                              ┌──────────▼──────────┐
                                              │AssessmentProcessor  │
                                              │ ├─LightweightDetector
                                              │ ├─CircuitBreaker    │
                                              │ └─CheckpointManager │
                                              └─────────────────────┘
```

Assessment is "just another consumer" of the unified EventLog.

---

## Phase A: Complete EventLog Migration (4.4.2a Remaining)

### Task A1: Create Consumer Infrastructure

**File:** `vibes-server/src/consumers/mod.rs`

Create a ConsumerManager that spawns and manages consumer tasks:

```rust
pub struct ConsumerManager {
    event_log: Arc<dyn EventLog<VibesEvent>>,
    handles: Vec<JoinHandle<()>>,
}

impl ConsumerManager {
    pub fn new(event_log: Arc<dyn EventLog<VibesEvent>>) -> Self;

    pub async fn spawn_consumer<F>(
        &mut self,
        group: &str,
        start_position: SeekPosition,
        handler: F,
    ) -> Result<()>
    where
        F: Fn(VibesEvent) -> BoxFuture<'static, ()> + Send + Sync + 'static;

    pub async fn shutdown(&mut self);
}
```

**Tests:**
- [ ] `test_consumer_manager_spawns_task`
- [ ] `test_consumer_manager_shutdown_graceful`

### Task A2: Migrate WebSocket Broadcaster

**Current flow:**
```
EventBus::publish() → broadcast::Sender → WebSocket handlers → Web UI
```

**New flow:**
```
EventLog::append() → "websocket" consumer → broadcast to WebSocket clients → Web UI
```

The consumer polls with short timeout for low latency:
```rust
consumer.poll(100, Duration::from_millis(50)).await
```

Still uses internal broadcast channel for fan-out to multiple WebSocket connections.

**Configuration:**
- Consumer group: `"websocket"`
- SeekPosition: `End` (only live events)

**Verification:**
- [ ] Start server, connect Web UI
- [ ] Run Claude session via CLI
- [ ] Verify events appear in Web UI in real-time (<100ms)
- [ ] Test multiple browser tabs receive updates
- [ ] Test reconnection works (catches up via REST API)

**Tests:**
- [ ] `test_websocket_consumer_broadcasts_events`
- [ ] `test_websocket_consumer_low_latency`

### Task A3: Migrate Chat History Writer

**Configuration:**
- Consumer group: `"chat-history"`
- SeekPosition: `Beginning` (replay on restart for durability)

**Implementation:**
```rust
// Commits offset after successful SQLite write
for (offset, event) in batch {
    chat_history.write(event).await?;
    consumer.commit(offset).await?;
}
```

**Tests:**
- [ ] `test_chat_history_consumer_persists_events`
- [ ] `test_chat_history_resumes_from_committed_offset`

### Task A4: Migrate Notification Dispatcher

**Configuration:**
- Consumer group: `"notifications"`
- SeekPosition: `End` (live notifications only)

**Tests:**
- [ ] `test_notification_consumer_dispatches_events`

### Task A5: Remove MemoryEventBus

Once all subscribers migrated:

1. Delete `vibes-core/src/events/bus.rs` (or equivalent)
2. Remove `EventBus` from `AppState`
3. Update all `event_bus.publish()` calls to `event_log.append()`
4. Remove broadcast channel usage for events

**Tests:**
- [ ] Verify no compilation errors after removal
- [ ] All existing tests still pass

---

## Phase B: Assessment Logic (4.4.2b)

### Task B0: Assessment Consumer Loop

**File:** `plugins/vibes-groove/src/assessment/consumer.rs`

The core consumer loop that dispatches to components:

```rust
pub async fn assessment_consumer_loop(
    mut consumer: Box<dyn EventConsumer<VibesEvent>>,
    processor: Arc<AssessmentProcessor>,
    shutdown: CancellationToken,
) -> Result<()> {
    loop {
        tokio::select! {
            _ = shutdown.cancelled() => break,
            result = consumer.poll(100, Duration::from_secs(1)) => {
                let batch = result?;
                for (offset, event) in batch {
                    processor.process(event).await;
                }
                if let Some(last_offset) = batch.last_offset() {
                    consumer.commit(last_offset).await?;
                }
            }
        }
    }
    Ok(())
}
```

**Configuration:**
- Consumer group: `"assessment"`
- SeekPosition: `Beginning` (need full session history for patterns)

**Tests:**
- [ ] `test_assessment_consumer_processes_events`
- [ ] `test_assessment_consumer_commits_after_batch`
- [ ] `test_assessment_consumer_respects_shutdown`

### Task B1: LightweightDetector

**File:** `plugins/vibes-groove/src/assessment/lightweight.rs`

Pattern matching and EMA computation:

```rust
pub struct LightweightDetector {
    config: PatternConfig,
    ema_state: EmaState,
}

impl LightweightDetector {
    pub fn process(&mut self, event: &VibesEvent) -> Option<LightweightEvent>;
}

struct EmaState {
    token_rate: f64,
    error_rate: f64,
    tool_call_rate: f64,
}
```

Emits `LightweightEvent` (fire-and-forget to Iggy).

**Tests:**
- [ ] `test_detector_matches_error_patterns`
- [ ] `test_detector_ema_computation`
- [ ] `test_detector_emits_lightweight_events`

### Task B2: CircuitBreaker

**File:** `plugins/vibes-groove/src/assessment/circuit_breaker.rs`

State machine for intervention decisions:

```rust
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitState,
    failure_count: u32,
    last_state_change: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,      // Normal operation
    Open,        // Intervention needed
    HalfOpen,    // Testing if recovered
}

impl CircuitBreaker {
    pub fn record_event(&mut self, event: &LightweightEvent) -> Option<CircuitTransition>;
    pub fn state(&self) -> CircuitState;
}
```

**Tests:**
- [ ] `test_circuit_breaker_opens_on_threshold`
- [ ] `test_circuit_breaker_half_open_after_timeout`
- [ ] `test_circuit_breaker_closes_on_success`

### Task B3: SessionBuffer

**File:** `plugins/vibes-groove/src/assessment/session_buffer.rs`

Collects events per session for batch processing:

```rust
pub struct SessionBuffer {
    buffers: HashMap<SessionId, VecDeque<VibesEvent>>,
    max_events_per_session: usize,
    max_sessions: usize,
}

impl SessionBuffer {
    pub fn push(&mut self, session_id: SessionId, event: VibesEvent);
    pub fn get(&self, session_id: &SessionId) -> Option<&VecDeque<VibesEvent>>;
    pub fn drain(&mut self, session_id: &SessionId) -> Vec<VibesEvent>;
}
```

Uses LRU eviction when max_sessions exceeded.

**Tests:**
- [ ] `test_session_buffer_collects_events`
- [ ] `test_session_buffer_lru_eviction`
- [ ] `test_session_buffer_drain`

### Task B4: CheckpointManager

**File:** `plugins/vibes-groove/src/assessment/checkpoint.rs`

Detects checkpoint triggers:

```rust
pub struct CheckpointManager {
    config: CheckpointConfig,
    last_checkpoint: HashMap<SessionId, Instant>,
}

impl CheckpointManager {
    pub fn should_checkpoint(
        &mut self,
        session_id: &SessionId,
        detector_output: &LightweightEvent,
        buffer: &SessionBuffer,
    ) -> Option<CheckpointTrigger>;
}

pub enum CheckpointTrigger {
    PatternMatch { pattern: String },
    TimeInterval,
    ThresholdExceeded { metric: String, value: f64 },
}
```

**Tests:**
- [ ] `test_checkpoint_on_pattern_match`
- [ ] `test_checkpoint_on_time_interval`
- [ ] `test_checkpoint_on_threshold`

### Task B5: HarnessLLM

**File:** `plugins/vibes-groove/src/assessment/harness_llm.rs`

Subprocess-based LLM calls:

```rust
pub struct HarnessLLM {
    config: LlmConfig,
    circuit_breaker: CircuitBreaker,
}

impl HarnessLLM {
    pub async fn analyze(&self, context: AnalysisContext) -> Result<AnalysisResult>;
}
```

Runs LLM in subprocess to avoid blocking main runtime.

**Tests:**
- [ ] `test_harness_llm_subprocess_isolation`
- [ ] `test_harness_llm_timeout`
- [ ] `test_harness_llm_circuit_breaker_integration`

### Task B6: SessionEndDetector

**File:** `plugins/vibes-groove/src/assessment/session_end.rs`

Detects session end via event or timeout:

```rust
pub struct SessionEndDetector {
    config: SessionEndConfig,
    last_activity: HashMap<SessionId, Instant>,
}

impl SessionEndDetector {
    pub fn process(&mut self, event: &VibesEvent) -> Option<SessionEnd>;
    pub fn check_timeouts(&mut self) -> Vec<SessionEnd>;
}

pub struct SessionEnd {
    pub session_id: SessionId,
    pub reason: SessionEndReason,
}

pub enum SessionEndReason {
    Explicit,         // SessionEnded event received
    InactivityTimeout,
}
```

**Tests:**
- [ ] `test_session_end_explicit`
- [ ] `test_session_end_timeout`

### Task B7: SamplingStrategy

**File:** `plugins/vibes-groove/src/assessment/sampling.rs`

Decides what reaches Medium/Heavy tiers:

```rust
pub struct SamplingStrategy {
    config: SamplingConfig,
    rng: StdRng,
}

impl SamplingStrategy {
    pub fn should_sample(&mut self, trigger: &CheckpointTrigger) -> SamplingDecision;
}

pub enum SamplingDecision {
    Skip,
    Medium,
    Heavy,
}
```

**Tests:**
- [ ] `test_sampling_respects_rates`
- [ ] `test_sampling_deterministic_with_seed`

### Task B8: HookIntervention

**File:** `plugins/vibes-groove/src/assessment/intervention.rs`

Injects learning when circuit breaker opens:

```rust
pub struct HookIntervention {
    config: InterventionConfig,
}

impl HookIntervention {
    pub async fn intervene(&self, session_id: &SessionId, learning: &str) -> Result<()>;
}
```

Writes to `.claude/hooks/` or similar mechanism.

**Tests:**
- [ ] `test_intervention_writes_hook`
- [ ] `test_intervention_graceful_degradation`

### Task B9: CLI Commands

**File:** `plugins/vibes-groove/src/cli.rs`

Add assessment-related commands:

```bash
vibes groove assess status    # Current circuit state, recent events
vibes groove assess history   # Past assessments for session
```

**Tests:**
- [ ] `test_cli_assess_status`
- [ ] `test_cli_assess_history`

---

## Phase C: Integration Tests

### Test Infrastructure

**File:** `tests/e2e/helpers.rs`

```rust
pub struct E2ETestHarness {
    server: VibesServer,
    event_log: Arc<InMemoryEventLog<VibesEvent>>,
    ws_client: WebSocketClient,
}

impl E2ETestHarness {
    pub async fn new() -> Self;
    pub async fn wait_for_event<P>(&self, predicate: P, timeout: Duration) -> Option<VibesEvent>;
    pub async fn run_claude_session(&self, prompt: &str) -> SessionId;
    pub async fn get_consumer_offset(&self, group: &str) -> Offset;
}
```

### E2E Test Scenarios

**E2E-1: Full Event Flow**
```
CLI Session → EventLog → All Consumers → Verify outputs
```
- [ ] Start vibes server
- [ ] Run `vibes claude` with a simple prompt
- [ ] Verify: WebSocket client receives events
- [ ] Verify: Chat history persisted
- [ ] Verify: Assessment events emitted

**E2E-2: Web UI Live Updates**
```
Browser → WebSocket → Server → CLI Session → Events flow back
```
- [ ] Open Web UI in browser (or headless test)
- [ ] Start session from CLI
- [ ] Verify: Events appear in Web UI in real-time (<100ms latency)
- [ ] Verify: Multiple browser tabs all receive updates

**E2E-3: Consumer Restart Recovery**
```
Process events → Kill server → Restart → Resume from committed offset
```
- [ ] Process some events, commit offsets
- [ ] Kill and restart server
- [ ] Verify: No duplicate processing
- [ ] Verify: No missed events

**E2E-4: Assessment Pipeline**
```
Session with patterns → LightweightDetector → CircuitBreaker → Intervention
```
- [ ] Run session that triggers configured patterns (e.g., repeated errors)
- [ ] Verify: LightweightEvents emitted
- [ ] Verify: CircuitBreaker state transitions correctly
- [ ] Verify: Intervention triggered when threshold crossed

**E2E-5: Session End Handling**
```
Session → End session → Final checkpoint → Summary generated
```
- [ ] Complete a full session
- [ ] Verify: SessionEndDetector fires
- [ ] Verify: Final assessment checkpoint created

---

## Execution Order

1. **A1** → Consumer infrastructure (foundation)
2. **A2** → WebSocket migration (most visible, verify Web UI works)
3. **A3** → Chat history migration
4. **A4** → Notification migration
5. **A5** → Remove EventBus (cleanup)
6. **B0** → Assessment consumer loop
7. **B1** → LightweightDetector (foundation for assessment)
8. **B2** → CircuitBreaker
9. **B3** → SessionBuffer
10. **B4** → CheckpointManager
11. **B5** → HarnessLLM
12. **B6** → SessionEndDetector
13. **B7** → SamplingStrategy
14. **B8** → HookIntervention
15. **B9** → CLI commands
16. **C** → E2E tests throughout

---

## Success Criteria

- [ ] All consumers migrated to EventLog pattern
- [ ] MemoryEventBus completely removed
- [ ] Web UI receives real-time updates via consumer
- [ ] Assessment pipeline processes events end-to-end
- [ ] Circuit breaker triggers interventions correctly
- [ ] All E2E tests pass
- [ ] `just test` passes
- [ ] `just pre-commit` passes
