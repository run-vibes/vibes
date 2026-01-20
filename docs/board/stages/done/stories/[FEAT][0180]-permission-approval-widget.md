---
id: FEAT0180
title: Permission approval widget
type: feat
status: done
priority: high
scope: tui/03-terminal-agent-control
depends: [m43-feat-01]
estimate: 4h
---

# Permission Approval Widget

## Summary

Implement the permission request handling widget for the agent detail view. When an agent needs approval for an action (file write, command execution, etc.), this widget displays the request and provides approval controls.

## Features

### Permission Request Display

```
├─────────────────────────────────────────────────────┤
│ ⚠ Permission Request: Write to src/auth/login.rs    │
│ [y] Approve  [n] Deny  [v] View diff  [e] Edit      │
├─────────────────────────────────────────────────────┤
```

When no permission pending:
```
├─────────────────────────────────────────────────────┤
│ No pending permissions                              │
├─────────────────────────────────────────────────────┤
```

### Permission Types

Display appropriate context for each type:
- **File write**: Show file path, offer diff view
- **Command execution**: Show command, highlight risky commands
- **File read**: Show file path
- **Web request**: Show URL

### Actions

| Key | Action | Description |
|-----|--------|-------------|
| `y` | Approve | Accept the permission request |
| `n` | Deny | Reject the permission request |
| `v` | View | Show diff/details in modal |
| `e` | Edit | Edit the proposed content (file writes only) |

### Diff Modal

When pressing `v` for file writes:
- Show current file content vs proposed changes
- Side-by-side or unified diff view
- Syntax highlighting if possible
- Close with Esc

### Permission State

```rust
pub struct PermissionWidget {
    pending: Option<PermissionRequest>,
}

pub struct PermissionRequest {
    id: PermissionId,
    request_type: PermissionType,
    description: String,
    details: PermissionDetails,
    timestamp: DateTime<Utc>,
}

pub enum PermissionDetails {
    FileWrite { path: PathBuf, content: String, original: Option<String> },
    Command { command: String, working_dir: PathBuf },
    FileRead { path: PathBuf },
    WebRequest { url: String, method: String },
}
```

## Implementation

1. Create `src/widgets/permission.rs` with PermissionWidget
2. Define PermissionRequest and related types
3. Implement render method for permission area
4. Add WebSocket subscription for permission request events
5. Implement approve/deny command handlers
6. Create diff view modal component
7. Wire keyboard shortcuts (y/n/v/e)
8. Send approval/denial responses via WebSocket
9. Handle "no pending permissions" state

## Acceptance Criteria

- [ ] Permission widget renders in designated area
- [ ] Pending permission shows type and description
- [ ] `y` key sends approval response
- [ ] `n` key sends denial response
- [ ] `v` key opens diff view for file writes
- [ ] Diff view shows before/after content
- [ ] Esc closes diff modal
- [ ] No permission state displays appropriately
- [ ] Permission requests received via WebSocket
- [ ] Approval/denial sent via WebSocket
