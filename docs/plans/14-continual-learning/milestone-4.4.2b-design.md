# Milestone 4.4.2b: Assessment Logic

> **Status:** Design approved
> **Depends on:** [milestone-4.4.2a-design.md](milestone-4.4.2a-design.md) (EventLog Migration)
> **Parent:** [milestone-4.4-design.md](milestone-4.4-design.md)

## Overview

Implement the assessment intelligence layer: signal detection, circuit breaker, LLM-powered analysis, and CLI commands. This builds on the EventLog foundation from 4.4.2a.

### Goals

1. **Lightweight detection**: Real-time frustration/success signal detection (<10ms)
2. **Circuit breaker**: Intervene when sessions go bad
3. **Checkpoint summarization**: LLM-powered segment analysis
4. **Session analysis**: Full session outcome classification
5. **CLI commands**: `assess status` and `assess history`

### Prerequisites

- 4.4.2a complete (EventLog Migration)
- vibes-groove in `plugins/` directory
- IggyEventLog operational

---

## Architecture

### Assessment as EventLog Consumer

```
┌─────────────────────────────────────────┐
│            Iggy Event Log               │
│  [msg0][msg1][msg2][msg3][msg4][msg5]   │
└─────────────────────────────────────────┘
                    │
                    │ consumer("assessment")
                    ▼
         ┌─────────────────────┐
         │ AssessmentProcessor │
         │                     │
         │  ┌───────────────┐  │
         │  │  Lightweight  │  │  ← <10ms per event
         │  │   Detector    │  │
         │  └───────┬───────┘  │
         │          │          │
         │  ┌───────▼───────┐  │
         │  │Circuit Breaker│  │  ← Intervention decisions
         │  └───────┬───────┘  │
         │          │          │
         │  ┌───────▼───────┐  │
         │  │  Checkpoint   │  │  ← Triggers on boundaries
         │  │   Manager     │  │
         │  └───────────────┘  │
         └─────────────────────┘
                    │
                    ▼ (intervention)
         ┌─────────────────────┐
         │   Hook Injection    │
         └─────────────────────┘
```

---

## Component Design

### 1. Lightweight Detector

Fast, synchronous signal detection on every message.

```rust
pub struct LightweightDetector {
    patterns: PatternMatcher,
    frustration_ema: EmaTracker,
    success_ema: EmaTracker,
}

impl LightweightDetector {
    /// Detect signals in a message (<10ms)
    pub fn detect(&mut self, event: &VibesEvent) -> Vec<LightweightSignal> {
        let mut signals = vec![];

        if let Some(content) = event.user_content() {
            // Linguistic patterns
            signals.extend(self.patterns.match_frustration(content));
            signals.extend(self.patterns.match_success(content));

            // Correction detection
            if self.is_correction(content) {
                signals.push(LightweightSignal::UserCorrection);
            }
        }

        if let Some(tool_output) = event.tool_output() {
            // Tool failure detection
            signals.extend(self.detect_tool_signals(tool_output));
        }

        // Update EMAs
        self.update_emas(&signals);

        signals
    }

    pub fn frustration_ema(&self) -> f64 { self.frustration_ema.value() }
    pub fn success_ema(&self) -> f64 { self.success_ema.value() }
}
```

#### Default Patterns

```rust
pub struct PatternMatcher {
    frustration: Vec<CompiledPattern>,
    success: Vec<CompiledPattern>,
}

// Default frustration patterns
const FRUSTRATION_PATTERNS: &[&str] = &[
    r"(?i)\bno\b,?\s*(that'?s?\s*)?(not|wrong)",
    r"(?i)\bi\s+(already|just)\s+(said|told|mentioned)",
    r"(?i)\btry\s+again\b",
    r"(?i)\bstill\s+(not|doesn'?t|won'?t)\b",
    r"(?i)\bwhy\s+(are|do|did)\s+you\b",
    r"(?i)\bi\s+don'?t\s+want\b",
    r"(?i)\bactually\b.*\bnot\b",
];

// Default success patterns
const SUCCESS_PATTERNS: &[&str] = &[
    r"(?i)\b(perfect|excellent|great|awesome)\b",
    r"(?i)\bthat'?s?\s+(exactly|precisely)\b",
    r"(?i)\bthank(s|\s+you)\b",
    r"(?i)\bworks?\s+(great|perfectly|well)\b",
    r"(?i)\bgood\s+job\b",
];
```

#### Tool Signal Detection

```rust
fn detect_tool_signals(&self, output: &str) -> Vec<LightweightSignal> {
    let mut signals = vec![];

    // Build failures
    if output.contains("error:") || output.contains("FAILED") {
        signals.push(LightweightSignal::BuildFailure);
    }

    // Test failures
    if output.contains("test result: FAILED") || output.contains("failures:") {
        signals.push(LightweightSignal::TestFailure);
    }

    // Lint errors
    if output.contains("warning:") && output.contains("error:") {
        signals.push(LightweightSignal::LintError);
    }

    // Build success
    if output.contains("Finished") && output.contains("release") {
        signals.push(LightweightSignal::BuildSuccess);
    }

    // Test success
    if output.contains("test result: ok") {
        signals.push(LightweightSignal::TestSuccess);
    }

    signals
}
```

### 2. Circuit Breaker

Decides when to intervene based on accumulated signals.

```rust
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitBreakerState,
}

pub struct CircuitBreakerState {
    frustration_ema: f64,
    consecutive_failures: u32,
    corrections_in_window: VecDeque<Instant>,
    last_intervention: Option<Instant>,
    intervention_count: u32,
}

impl CircuitBreaker {
    /// Update state with new signals
    pub fn update(&mut self, signals: &[LightweightSignal]) {
        for signal in signals {
            match signal {
                LightweightSignal::UserCorrection => {
                    self.state.corrections_in_window.push_back(Instant::now());
                    self.state.consecutive_failures += 1;
                }
                LightweightSignal::BuildFailure | LightweightSignal::TestFailure => {
                    self.state.consecutive_failures += 1;
                }
                LightweightSignal::BuildSuccess | LightweightSignal::TestSuccess => {
                    self.state.consecutive_failures = 0;
                }
                _ => {}
            }
        }

        // Prune old corrections from window
        let window = Duration::from_secs(300); // 5 minute window
        while let Some(front) = self.state.corrections_in_window.front() {
            if front.elapsed() > window {
                self.state.corrections_in_window.pop_front();
            } else {
                break;
            }
        }
    }

    /// Check if intervention is needed
    pub fn should_intervene(&self) -> Option<InterventionType> {
        // Respect cooldown
        if let Some(last) = self.state.last_intervention {
            if last.elapsed() < self.config.cooldown {
                return None;
            }
        }

        // Correction storm: 3+ corrections in 5 minutes
        if self.state.corrections_in_window.len() >= 3 {
            return Some(InterventionType::ClarificationPause);
        }

        // Consecutive failures
        if self.state.consecutive_failures >= self.config.failure_threshold {
            return Some(InterventionType::ApproachPivot);
        }

        // High frustration EMA
        if self.state.frustration_ema >= self.config.frustration_threshold {
            return Some(InterventionType::ExplicitCheckIn);
        }

        None
    }

    /// Record that intervention happened
    pub fn record_intervention(&mut self) {
        self.state.last_intervention = Some(Instant::now());
        self.state.intervention_count += 1;
    }
}
```

### 3. Intervention Mechanism

Injects intervention messages via hooks.

```rust
pub enum InterventionType {
    /// "Let me pause - I want to make sure I understand correctly..."
    ClarificationPause,
    /// "This approach isn't working. Let me try a different strategy..."
    ApproachPivot,
    /// "I notice we've hit some bumps. Want to step back and reassess?"
    ExplicitCheckIn,
}

pub struct HookIntervention {
    tx: mpsc::Sender<InterventionMessage>,
}

impl HookIntervention {
    pub async fn intervene(&self, intervention: InterventionType) -> Result<()> {
        let message = match intervention {
            InterventionType::ClarificationPause => {
                "Before we continue: I want to make sure I understand \
                 what you're looking for. Could you help me understand \
                 which part isn't matching your expectations?"
            }
            InterventionType::ApproachPivot => {
                "I notice my current approach isn't working well. \
                 Let me step back and try a different strategy. \
                 What would be most helpful right now?"
            }
            InterventionType::ExplicitCheckIn => {
                "I sense some friction in our session. Would you like to \
                 pause and reassess our approach, or should I continue \
                 with adjustments?"
            }
        };

        self.tx.send(InterventionMessage {
            content: message.to_string(),
            intervention_type: intervention,
        }).await?;

        Ok(())
    }
}
```

### 4. Checkpoint Manager

Triggers medium assessments at logical boundaries.

```rust
pub struct CheckpointManager {
    config: CheckpointConfig,
    messages_since_checkpoint: u32,
    last_checkpoint: Instant,
}

impl CheckpointManager {
    /// Check if we should trigger a checkpoint
    pub fn should_checkpoint(&mut self, event: &VibesEvent) -> bool {
        self.messages_since_checkpoint += 1;

        // Message count trigger
        if self.messages_since_checkpoint >= self.config.message_threshold {
            return true;
        }

        // Task boundary keywords
        if let Some(content) = event.user_content() {
            if self.is_task_boundary(content) {
                return true;
            }
        }

        // Git commit detected
        if let Some(output) = event.tool_output() {
            if output.contains("commit") && output.contains("[") {
                return true;
            }
        }

        // Time-based trigger
        if self.last_checkpoint.elapsed() > self.config.time_threshold {
            return true;
        }

        false
    }

    fn is_task_boundary(&self, content: &str) -> bool {
        let patterns = [
            "done", "finished", "complete",
            "next", "now let's", "moving on",
            "that works", "perfect",
        ];
        patterns.iter().any(|p| content.to_lowercase().contains(p))
    }

    pub fn record_checkpoint(&mut self) {
        self.messages_since_checkpoint = 0;
        self.last_checkpoint = Instant::now();
    }
}
```

### 5. LLM Backend (HarnessLLM)

Uses Claude subprocess for summarization.

```rust
#[async_trait]
pub trait AssessmentLLM: Send + Sync {
    async fn summarize_segment(&self, messages: &[Message]) -> Result<SegmentSummary>;
    async fn analyze_session(&self, transcript: &SessionTranscript) -> Result<SessionAnalysis>;
}

pub struct HarnessLLM {
    binary_path: PathBuf,
}

#[async_trait]
impl AssessmentLLM for HarnessLLM {
    async fn summarize_segment(&self, messages: &[Message]) -> Result<SegmentSummary> {
        let prompt = format!(
            "Summarize this conversation segment in 2-3 sentences. \
             Focus on: what was attempted, what succeeded/failed, \
             and any friction points.\n\n{}",
            Self::format_messages(messages)
        );

        let output = Command::new(&self.binary_path)
            .arg("--print")
            .arg(&prompt)
            .output()
            .await?;

        let summary = String::from_utf8(output.stdout)?;

        Ok(SegmentSummary {
            text: summary.trim().to_string(),
            token_metrics: self.compute_metrics(messages),
        })
    }

    async fn analyze_session(&self, transcript: &SessionTranscript) -> Result<SessionAnalysis> {
        let prompt = format!(
            "Analyze this coding session and classify the outcome. \
             Consider: goal achievement, efficiency, friction points.\n\n\
             Respond with JSON: {{\"outcome\": \"success|partial|failure|abandoned\", \
             \"confidence\": 0.0-1.0, \"summary\": \"...\"}}\n\n{}",
            transcript.format()
        );

        let output = Command::new(&self.binary_path)
            .arg("--print")
            .arg(&prompt)
            .output()
            .await?;

        let json: SessionAnalysisResponse = serde_json::from_slice(&output.stdout)?;

        Ok(SessionAnalysis {
            outcome: json.outcome.parse()?,
            confidence: json.confidence,
            summary: json.summary,
        })
    }
}
```

### 6. Session End Detection

Detects when sessions complete for heavy assessment.

```rust
pub trait SessionEndDetector: Send + Sync {
    fn is_session_end(&self, event: &VibesEvent) -> bool;
}

/// Detects session end via Claude Stop hook
pub struct HookSessionEndDetector;

impl SessionEndDetector for HookSessionEndDetector {
    fn is_session_end(&self, event: &VibesEvent) -> bool {
        matches!(event, VibesEvent::Hook(hook) if hook.name == "Stop")
    }
}

/// Fallback: timeout-based detection
pub struct TimeoutSessionEndDetector {
    last_activity: RwLock<HashMap<SessionId, Instant>>,
    timeout: Duration,
}

impl SessionEndDetector for TimeoutSessionEndDetector {
    fn is_session_end(&self, event: &VibesEvent) -> bool {
        // Check if previous session timed out
        // (Implementation tracks activity per session)
        false
    }
}
```

### 7. Sampling Strategy

Decides which sessions get heavy assessment.

```rust
pub struct SamplingStrategy {
    config: SamplingConfig,
    session_count: AtomicU64,
}

impl SamplingStrategy {
    pub fn should_assess(&self, context: &SamplingContext) -> bool {
        // Always assess during burn-in
        let count = self.session_count.load(Ordering::SeqCst);
        if count < self.config.burn_in_sessions {
            return true;
        }

        // Always assess if circuit breaker triggered
        if context.circuit_breaker_triggered {
            return true;
        }

        // Always assess if explicit feedback given
        if context.has_explicit_feedback {
            return true;
        }

        // Always assess unusually long sessions
        if context.message_count > self.config.long_session_threshold {
            return true;
        }

        // Base rate sampling
        rand::random::<f64>() < self.config.base_rate
    }
}
```

---

## AssessmentProcessor (Complete)

```rust
pub struct AssessmentProcessor {
    config: AssessmentConfig,
    detector: LightweightDetector,
    circuit_breaker: CircuitBreaker,
    checkpoint_manager: CheckpointManager,
    intervention: HookIntervention,
    llm: Arc<dyn AssessmentLLM>,
    sampling: SamplingStrategy,
    session_buffers: HashMap<SessionId, SessionBuffer>,
}

impl AssessmentProcessor {
    /// Main processing loop - runs as EventLog consumer
    pub async fn run(mut self, log: Arc<dyn EventLog>) -> Result<()> {
        let mut consumer = log.consumer("assessment").await?;

        loop {
            let batch = consumer.poll(100, Duration::from_secs(1)).await?;

            for (offset, event) in &batch.events {
                self.process_event(event).await?;
            }

            if let Some(offset) = batch.last_offset() {
                consumer.commit(offset).await?;
            }
        }
    }

    async fn process_event(&mut self, event: &VibesEvent) -> Result<()> {
        let session_id = match event.session_id() {
            Some(id) => id.to_string(),
            None => return Ok(()), // Non-session event
        };

        // 1. Lightweight detection (sync, <10ms)
        let signals = self.detector.detect(event);

        // 2. Update circuit breaker
        self.circuit_breaker.update(&signals);

        // 3. Check for intervention
        if let Some(intervention_type) = self.circuit_breaker.should_intervene() {
            self.intervention.intervene(intervention_type).await?;
            self.circuit_breaker.record_intervention();
        }

        // 4. Buffer for checkpoint/session analysis
        let buffer = self.session_buffers
            .entry(session_id.clone().into())
            .or_insert_with(SessionBuffer::new);
        buffer.add_event(event.clone(), signals.clone());

        // 5. Check for checkpoint (medium assessment)
        if self.checkpoint_manager.should_checkpoint(event) {
            self.run_checkpoint_assessment(&session_id, buffer).await?;
            self.checkpoint_manager.record_checkpoint();
        }

        // 6. Check for session end (heavy assessment)
        if self.is_session_end(event) {
            self.run_session_assessment(&session_id, buffer).await?;
            self.session_buffers.remove(&session_id.into());
        }

        Ok(())
    }

    async fn run_checkpoint_assessment(
        &self,
        session_id: &str,
        buffer: &SessionBuffer,
    ) -> Result<()> {
        let messages = buffer.messages_since_checkpoint();
        let summary = self.llm.summarize_segment(&messages).await?;

        tracing::info!(
            session_id,
            summary = %summary.text,
            "Checkpoint assessment complete"
        );

        Ok(())
    }

    async fn run_session_assessment(
        &self,
        session_id: &str,
        buffer: &SessionBuffer,
    ) -> Result<()> {
        // Check sampling
        let context = SamplingContext {
            circuit_breaker_triggered: buffer.had_intervention(),
            has_explicit_feedback: buffer.has_feedback(),
            message_count: buffer.message_count(),
        };

        if !self.sampling.should_assess(&context) {
            tracing::debug!(session_id, "Skipping heavy assessment (sampling)");
            return Ok(());
        }

        let transcript = buffer.to_transcript();
        let analysis = self.llm.analyze_session(&transcript).await?;

        tracing::info!(
            session_id,
            outcome = ?analysis.outcome,
            confidence = analysis.confidence,
            "Session assessment complete"
        );

        Ok(())
    }
}
```

---

## CLI Commands

### `vibes groove assess status`

```
Assessment Status
─────────────────
Enabled: true
Circuit Breaker: OK (frustration: 0.23, threshold: 0.70)

Active Session
──────────────
Session: abc-123
Messages: 47
Frustration EMA: 0.23
Success EMA: 0.81
Checkpoints: 4
Interventions: 0

Active Learnings
────────────────
• L-abc123: "Prefer Result over unwrap..." (user)
• L-def456: "Run just pre-commit before..." (project)

Consumer Status
───────────────
assessment: offset 1247/1250 (lag: 3)
```

### `vibes groove assess history`

```
Session History (last 7 days)
────────────────────────────
SESSION          OUTCOME    MSGS  FRUST  DATE
abc-123          success    47    0.12   2025-12-30 14:32
def-456          partial    23    0.45   2025-12-30 11:15
ghi-789          success    89    0.08   2025-12-29 16:42
jkl-012          failure    12    0.78   2025-12-29 09:23

Use `vibes groove assess history <session-id>` for details.
```

### `vibes groove assess history <session-id>`

```
Session: abc-123
────────────────
Date: 2025-12-30 14:32
Duration: 23 minutes
Messages: 47
Outcome: success (confidence: 0.92)

Frustration Trajectory
──────────────────────
0.0 ▁▁▂▂▁▁▃▂▁▁▁▂▁▁▁▁▁▁▁▁ 1.0
    Start                End

Checkpoints
───────────
1. "Set up project structure" (msgs 1-12)
2. "Implemented auth flow" (msgs 13-28)
3. "Fixed test failures" (msgs 29-41)
4. "Final cleanup" (msgs 42-47)

Signals Detected
────────────────
• 2x UserCorrection (msgs 15, 22)
• 3x BuildSuccess (msgs 28, 35, 47)
• 1x TestFailure (msg 33)

Interventions: None
```

---

## Module Structure

```
plugins/vibes-groove/src/
├── assessment/
│   ├── mod.rs
│   ├── processor.rs      # AssessmentProcessor
│   ├── config.rs         # AssessmentConfig (existing)
│   ├── types.rs          # Event types (existing)
│   ├── detection/
│   │   ├── mod.rs
│   │   ├── lightweight.rs   # LightweightDetector
│   │   ├── patterns.rs      # PatternMatcher
│   │   └── ema.rs           # EmaTracker
│   ├── circuit_breaker/
│   │   ├── mod.rs
│   │   ├── state.rs         # CircuitBreakerState
│   │   ├── breaker.rs       # CircuitBreaker
│   │   └── intervention.rs  # HookIntervention
│   ├── checkpoint/
│   │   ├── mod.rs
│   │   └── manager.rs       # CheckpointManager
│   ├── llm/
│   │   ├── mod.rs
│   │   ├── traits.rs        # AssessmentLLM trait
│   │   └── harness.rs       # HarnessLLM
│   ├── session/
│   │   ├── mod.rs
│   │   ├── buffer.rs        # SessionBuffer
│   │   ├── detector.rs      # SessionEndDetector
│   │   └── sampling.rs      # SamplingStrategy
│   └── cli/
│       ├── mod.rs
│       ├── status.rs        # assess status command
│       └── history.rs       # assess history command
```

---

## Deliverables

1. **Detection**
   - [ ] LightweightDetector with pattern matching
   - [ ] Default frustration/success patterns
   - [ ] EMA computation
   - [ ] Tool output signal detection

2. **Circuit Breaker**
   - [ ] CircuitBreaker state machine
   - [ ] Intervention decision logic
   - [ ] HookIntervention implementation
   - [ ] Cooldown enforcement

3. **Checkpoint**
   - [ ] CheckpointManager with triggers
   - [ ] Message count trigger
   - [ ] Task boundary detection
   - [ ] Time-based trigger

4. **LLM**
   - [ ] AssessmentLLM trait
   - [ ] HarnessLLM implementation
   - [ ] Segment summarization prompt
   - [ ] Session analysis prompt

5. **Session**
   - [ ] SessionBuffer for event collection
   - [ ] SessionEndDetector (hook + timeout)
   - [ ] SamplingStrategy

6. **AssessmentProcessor**
   - [ ] EventLog consumer integration
   - [ ] Full processing pipeline
   - [ ] Offset commit handling

7. **CLI**
   - [ ] `vibes groove assess status`
   - [ ] `vibes groove assess history`
   - [ ] `vibes groove assess history <id>`

---

## Exit Criteria

- [ ] Lightweight detection runs in <10ms per event
- [ ] Circuit breaker triggers intervention on correction storm
- [ ] Checkpoints trigger on message count and task boundaries
- [ ] HarnessLLM produces valid summaries
- [ ] Session assessment classifies outcomes correctly
- [ ] CLI commands display accurate information
- [ ] All unit tests pass
- [ ] Integration tests validate full pipeline
- [ ] `just pre-commit` passes
