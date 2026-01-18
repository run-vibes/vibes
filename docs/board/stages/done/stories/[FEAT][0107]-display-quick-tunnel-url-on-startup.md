---
id: FEAT0107
title: Display quick tunnel URL on startup
type: feat
status: done
priority: high
epics: [networking]
depends: []
estimate: S
created: 2026-01-13
updated: 2026-01-14
---

# Display quick tunnel URL on startup

## Summary

When `vibes serve --quick-tunnel` is run, the tunnel URL (e.g., `https://random-words.trycloudflare.com`) should be displayed in the terminal output so users can access their vibes instance remotely.

Currently, the TunnelManager spawns cloudflared but doesn't monitor its output to extract and display the URL. The parsing logic exists in `vibes-core/src/tunnel/cloudflared.rs` (`parse_quick_tunnel_url`) but isn't connected.

## Acceptance Criteria

- [ ] Quick tunnel URL is printed to stdout when tunnel connects
- [ ] URL is also available via `GET /api/tunnel/status` endpoint
- [ ] Web UI status page shows the tunnel URL
- [ ] Tunnel state transitions are logged (starting → connected)

## Implementation Notes

1. **Output monitoring task**: In `TunnelManager::spawn_process`, spawn a tokio task that reads cloudflared's stderr line by line
2. **URL detection**: Use existing `parse_quick_tunnel_url()` to extract the URL
3. **State update**: When URL is found, update `TunnelState::Connected { url, connected_at }`
4. **Event emission**: Emit `TunnelEvent::Connected { url }` for WebSocket broadcast
5. **CLI display**: Print URL prominently in serve command output

Key files:
- `vibes-core/src/tunnel/manager.rs` - Add output monitoring
- `vibes-core/src/tunnel/cloudflared.rs` - URL parsing already exists
- `vibes-cli/src/commands/serve.rs` - Display URL after tunnel connects
