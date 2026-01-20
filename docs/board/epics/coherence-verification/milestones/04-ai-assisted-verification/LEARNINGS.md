# Milestone Learnings: 04-ai-assisted-verification

> Aggregated learnings from completed stories.

## Story Learnings

### From FEAT0199: Implement model router for Ollama and Claude

### L001: Model router abstracts provider differences cleanly

| Field | Value |
|-------|-------|
| **Category** | architecture |
| **Context** | Implementing router to support both Ollama and Claude APIs |
| **Insight** | **What went well:** A unified interface for different model providers makes it easy to switch between local (Ollama) and cloud (Claude) models • **Harder than expected:** Error handling differs significantly between providers - Ollama fails silently while Claude throws detailed errors • **Would do differently:** Add retry logic with exponential backoff from the start |
| **Suggested Action** | When building multi-provider abstractions, standardize error handling and add retry logic early |
| **Applies To** | Any code that wraps multiple AI/LLM providers |
| **Applied** | |

### From FEAT0200: Implement AI report generator

### L001: Report generator produces clean markdown with proper structu

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Story completion reflection |
| **Insight** | **What went well:** Report generator produces clean markdown with proper structure | **Harder than expected:** Formatting verdict tables with proper alignment | **Would do differently:** Add CSS styling for HTML reports from the start |
| **Suggested Action** | Add CSS styling for HTML reports from the start |
| **Applies To** | (to be determined) |
| **Applied** | |

### From FEAT0201: Add just verify ai command

### L001: Just command integrates cleanly with existing verify workflo

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Story completion reflection |
| **Insight** | **What went well:** Just command integrates cleanly with existing verify workflow | **Harder than expected:** Ensuring proper error handling for missing config | **Would do differently:** Add a --dry-run flag to preview what would be verified |
| **Suggested Action** | Add a --dry-run flag to preview what would be verified |
| **Applies To** | (to be determined) |
| **Applied** | |

