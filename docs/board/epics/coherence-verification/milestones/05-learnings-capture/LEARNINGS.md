# Milestone Learnings: 05-learnings-capture

> Aggregated learnings from completed stories.

## Story Learnings

### From CHORE0202: Add learnings template to story template

### L001: Template provides structured format for learnings

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Story completion reflection |
| **Insight** | **What went well:** Template provides structured format for learnings | **Harder than expected:** Deciding which fields to include in the template |
| **Suggested Action** | (none suggested) |
| **Applies To** | (to be determined) |
| **Applied** | |

### From FEAT0203: Implement learning capture on story completion

### L001: Interactive prompts integrate smoothly with done workflow

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Story completion reflection |
| **Insight** | **What went well:** Interactive prompts integrate smoothly with done workflow | **Harder than expected:** Counting existing learnings to generate sequential IDs | **Would do differently:** Use a more robust ID generation approach |
| **Suggested Action** | Use a more robust ID generation approach |
| **Applies To** | (to be determined) |
| **Applied** | |

### From FEAT0204: Implement milestone learnings aggregation

### L001: Aggregation extracts and combines learnings from multiple st

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Story completion reflection |
| **Insight** | **What went well:** Aggregation extracts and combines learnings from multiple stories | **Harder than expected:** Finding the right sed patterns for extraction | **Would do differently:** Consider using a dedicated parsing library |
| **Suggested Action** | Consider using a dedicated parsing library |
| **Applies To** | (to be determined) |
| **Applied** | |

### From FEAT0205: Add just learn reflect command

### L001: Reflect command provides a clean way to review learnings

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Story completion reflection |
| **Insight** | **What went well:** Reflect command provides a clean way to review learnings | **Harder than expected:** Formatting output for terminal readability |
| **Suggested Action** | (none suggested) |
| **Applies To** | (to be determined) |
| **Applied** | |

### From FEAT0206: Add just learn apply propagation engine

### L001: Apply command provides a clean interface for propagating lea

| Field | Value |
|-------|-------|
| **Category** | process |
| **Context** | Story completion reflection |
| **Insight** | **What went well:** Apply command provides a clean interface for propagating learnings | **Harder than expected:** Matching learning IDs with their target files | **Would do differently:** Add a preview mode to see what would change |
| **Suggested Action** | Add a preview mode to see what would change |
| **Applies To** | (to be determined) |
| **Applied** | |

<!-- No learnings captured for this story -->

### From FEAT0208: Add CLI options for non-interactive learning capture

### L001: Just variadic args dont preserve quotes

| Field | Value |
|-------|-------|
| **Category** | tooling |
| **Context** | Implementing non-interactive CLI options for just recipes |
| **Insight** | **What went well:** Environment variables provide a clean way to pass multi-word values to bash scripts in just recipes • **Harder than expected:** Just variadic args (`*ARGS`) split on whitespace even with quotes, making `--flag "multi word"` impossible to parse correctly • **Would do differently:** Start with environment variables from the beginning when values might contain spaces |
| **Suggested Action** | Use environment variables (not CLI args) when passing multi-word values to just recipes |
| **Applies To** | All just recipes that need to accept string values with spaces |
| **Applied** | |

