---
id: feat-0078
title: Connectivity Validation
type: feat
status: pending
priority: medium
epics: [cli, networking]
milestone: 35-setup-wizards
---

# Connectivity Validation

Add end-to-end validation after setup to verify configuration works.

## Context

After saving config, users want confidence that everything works. This story adds optional validation: for tunnels, briefly start the tunnel to verify connectivity; for auth, the JWKS fetch (already done in auth wizard) serves as validation.

## Acceptance Criteria

- [ ] After tunnel setup, offer to test connectivity
- [ ] If yes, briefly start tunnel and verify connection registers
- [ ] Show clear success/failure message
- [ ] For auth, JWKS fetch already validates (done in feat-0076)
- [ ] Clear error messages for common failures:
  - Cloudflared not logged in
  - Invalid tunnel name
  - DNS not routed
  - Invalid team/AUD
- [ ] Suggest fixes for each failure type

## Technical Notes

```rust
async fn test_tunnel_connectivity(config: &TunnelConfigSection) -> Result<()> {
    println!("\nTesting tunnel connectivity...");

    // Start tunnel briefly
    let mut child = spawn_tunnel(&mode, DEFAULT_PORT)?;

    // Wait for connection registration (with timeout)
    let timeout = Duration::from_secs(30);
    let connected = wait_for_connection(&mut child, timeout).await?;

    // Cleanup
    child.kill().await?;

    if connected {
        print_success("Tunnel connected successfully!");
    } else {
        print_error("Tunnel failed to connect within 30 seconds");
        println!("\nTroubleshooting:");
        println!("  - Check your internet connection");
        println!("  - Verify cloudflared login: cloudflared tunnel list");
    }

    Ok(())
}
```

## Size

M - Medium (subprocess management, timeout handling, error diagnostics)

## Notes

This story is marked optional in the design. Implement if time permits after core wizard functionality.
