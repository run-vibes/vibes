//! End-to-end integration tests for groove capture → inject pipeline
//!
//! These tests validate the pipeline infrastructure. The LearningExtractor
//! is a stub until Milestone 4.5 (AI extraction), so we test with manually
//! created learnings to verify the flow.

use tempfile::TempDir;
use vibes_groove::{
    capture::{ExtractedLearning, LearningCategory, LearningExtractor, TranscriptParser},
    inject::{ClaudeCodeInjector, InjectionMethod, LearningFormatter},
};

/// Test transcript parsing with realistic Claude Code output
#[test]
fn test_transcript_parser_with_realistic_content() {
    let parser = TranscriptParser::new();

    // Parser uses "role" field for messages, "tool_name" for tool uses
    let transcript = r#"{"version":1}
{"role":"user","content":"help me write a rust function"}
{"role":"assistant","content":"I'll help you write a Rust function."}
{"tool_name":"Write","input":{"file_path":"/tmp/test.rs","content":"fn main() {}"},"output":"File written","success":true}
{"role":"user","content":"thanks, that works!"}
{"role":"assistant","content":"You're welcome!"}"#;

    let parsed = parser.parse(transcript, "test-session").unwrap();

    assert_eq!(parsed.session_id, "test-session");
    assert_eq!(parsed.messages.len(), 4); // 2 user + 2 assistant
    assert_eq!(parsed.tool_uses.len(), 1);
    assert_eq!(parsed.tool_uses[0].tool_name, "Write");
    assert!(parsed.tool_uses[0].success);
}

/// Test that the extractor stub returns empty results (to be replaced in 4.5)
#[test]
fn test_extractor_stub_returns_empty() {
    let parser = TranscriptParser::new();
    let extractor = LearningExtractor::new();

    let transcript = r#"{"version":1}
{"role":"user","content":"always use pytest"}
{"role":"assistant","content":"I'll use pytest."}"#;

    let parsed = parser.parse(transcript, "session").unwrap();
    let learnings = extractor.extract(&parsed);

    // Stub returns empty - this test documents current behavior
    // and will be updated when AI extraction is added in 4.5
    assert!(learnings.is_empty());
}

/// Test the formatter with manually created learnings
#[test]
fn test_formatter_creates_valid_markdown() {
    let formatter = LearningFormatter::new();

    let learnings = vec![
        ExtractedLearning {
            content: "Use pytest instead of unittest".to_string(),
            category: LearningCategory::Preference,
            confidence: 0.9,
            source_tool: Some("Bash".to_string()),
        },
        ExtractedLearning {
            content: "Project uses async/await patterns".to_string(),
            category: LearningCategory::Pattern,
            confidence: 0.85,
            source_tool: None,
        },
    ];

    let formatted = formatter.format_for_injection(&learnings);

    // Formatter creates sections, not the main header
    assert!(formatted.contains("Preferences"));
    assert!(formatted.contains("pytest"));
    assert!(formatted.contains("Patterns"));
    assert!(formatted.contains("async/await"));
}

/// Test formatter handles empty learnings gracefully
#[test]
fn test_formatter_handles_empty_learnings() {
    let formatter = LearningFormatter::new();
    let formatted = formatter.format_for_injection(&[]);
    assert!(formatted.is_empty());
}

/// Test injector creates valid hook response
#[test]
fn test_injector_creates_hook_response() {
    let injector = ClaudeCodeInjector::new();

    let content = "### Preferences\n- Use pytest";
    let response = injector.inject_via_hook(content);

    assert!(response.additional_context.is_some());
    let ctx = response.additional_context.unwrap();
    assert!(ctx.contains("pytest"));
}

/// Test injector writes to CLAUDE.md
#[test]
fn test_injector_writes_to_claude_md() {
    let temp_dir = TempDir::new().unwrap();
    let claude_md_path = temp_dir.path().join("CLAUDE.md");

    // Create existing CLAUDE.md content
    std::fs::write(&claude_md_path, "# Existing Content\n\nSome docs here.\n").unwrap();

    let injector = ClaudeCodeInjector::new();
    let content = "### Preferences\n- Use pytest";

    let result = injector
        .inject_to_claude_md(content, &claude_md_path)
        .unwrap();

    assert!(matches!(result.method, InjectionMethod::ClaudeMdAppend));
    assert!(result.content_length > 0);

    let updated = std::fs::read_to_string(&claude_md_path).unwrap();
    assert!(updated.contains("# Existing Content"));
    assert!(updated.contains("pytest"));
    assert!(updated.contains("<!-- vibes-groove-learnings -->"));
}

/// Test full pipeline: parse → extract (stub) → format → inject
#[test]
fn test_full_pipeline_with_stub_extractor() {
    let temp_dir = TempDir::new().unwrap();

    // 1. Parse transcript (using correct format with "role" field)
    let parser = TranscriptParser::new();
    let transcript = r#"{"version":1}
{"role":"user","content":"use cargo-nextest for tests"}
{"role":"assistant","content":"I'll use cargo-nextest."}"#;

    let parsed = parser.parse(transcript, "pipeline-test").unwrap();
    assert!(!parsed.messages.is_empty());

    // 2. Extract learnings (stub returns empty)
    let extractor = LearningExtractor::new();
    let extracted = extractor.extract(&parsed);
    assert!(extracted.is_empty()); // Stub behavior

    // 3. Manually create learnings to test rest of pipeline
    let learnings = vec![ExtractedLearning {
        content: "Use cargo-nextest for faster tests".to_string(),
        category: LearningCategory::Technique,
        confidence: 0.95,
        source_tool: Some("Bash".to_string()),
    }];

    // 4. Format for injection
    let formatter = LearningFormatter::new();
    let formatted = formatter.format_for_injection(&learnings);
    assert!(formatted.contains("cargo-nextest"));

    // 5. Inject via hook response
    let injector = ClaudeCodeInjector::new();
    let response = injector.inject_via_hook(&formatted);
    assert!(response.additional_context.is_some());

    // 6. Also test CLAUDE.md injection
    let claude_md_path = temp_dir.path().join("CLAUDE.md");
    let result = injector
        .inject_to_claude_md(&formatted, &claude_md_path)
        .unwrap();
    assert!(result.content_length > 0);

    let saved = std::fs::read_to_string(&claude_md_path).unwrap();
    assert!(saved.contains("cargo-nextest"));
}

/// Test injector replaces existing groove section in CLAUDE.md
#[test]
fn test_injector_replaces_existing_groove_section() {
    let temp_dir = TempDir::new().unwrap();
    let claude_md_path = temp_dir.path().join("CLAUDE.md");

    // The injector uses a single marker for start and end
    let marker = "<!-- vibes-groove-learnings -->";

    // Create CLAUDE.md with existing groove section
    let initial_content = format!(
        r#"# My Project

Some documentation.

{}
# Groove Learnings

### Old Section
- Old learning 1
- Old learning 2
{}

More documentation.
"#,
        marker, marker
    );
    std::fs::write(&claude_md_path, &initial_content).unwrap();

    let injector = ClaudeCodeInjector::new();
    let new_content = "### Preferences\n- New learning";

    let result = injector
        .inject_to_claude_md(new_content, &claude_md_path)
        .unwrap();
    assert!(result.content_length > 0);

    let updated = std::fs::read_to_string(&claude_md_path).unwrap();

    // Should NOT contain old learnings
    assert!(!updated.contains("Old learning 1"));

    // SHOULD contain new learnings
    assert!(updated.contains("New learning"));

    // Should preserve surrounding content
    assert!(updated.contains("# My Project"));
    assert!(updated.contains("Some documentation"));
    assert!(updated.contains("More documentation"));
}

/// Test transcript parser handles malformed lines gracefully
#[test]
fn test_parser_handles_malformed_lines() {
    let parser = TranscriptParser::new();

    let transcript = r#"{"version":1}
{"role":"user","content":"valid message"}
not valid json
{"role":"assistant","content":"response"}"#;

    let parsed = parser.parse(transcript, "test").unwrap();

    // Should skip invalid line and parse valid ones
    assert_eq!(parsed.messages.len(), 2);
}

/// Test formatter with all learning categories
#[test]
fn test_formatter_all_categories() {
    let formatter = LearningFormatter::new();

    let learnings = vec![
        ExtractedLearning {
            content: "Pattern example".to_string(),
            category: LearningCategory::Pattern,
            confidence: 0.8,
            source_tool: None,
        },
        ExtractedLearning {
            content: "Technique example".to_string(),
            category: LearningCategory::Technique,
            confidence: 0.8,
            source_tool: None,
        },
        ExtractedLearning {
            content: "Preference example".to_string(),
            category: LearningCategory::Preference,
            confidence: 0.8,
            source_tool: None,
        },
        ExtractedLearning {
            content: "Context example".to_string(),
            category: LearningCategory::Context,
            confidence: 0.8,
            source_tool: None,
        },
    ];

    let formatted = formatter.format_for_injection(&learnings);

    assert!(formatted.contains("Patterns"));
    assert!(formatted.contains("Techniques"));
    assert!(formatted.contains("Preferences"));
    assert!(formatted.contains("Context"));
}
