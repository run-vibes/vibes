---
id: FEAT0106
title: Add gzip and brotli compression for HTTP responses
type: feat
status: done
priority: medium
epics: [web-ui]
depends: []
estimate: small
created: 2026-01-13
updated: 2026-01-13
---

# Add gzip and brotli compression for HTTP responses

## Summary

Enable response compression for vibes-server using tower-http's CompressionLayer. This reduces bandwidth for both embedded web-ui assets and API responses. The build already outputs gzip file sizes for reference, but we're not actually serving compressed responses.

## Acceptance Criteria

- [ ] tower-http `compression-gzip` and `compression-br` features enabled
- [ ] CompressionLayer added to axum router middleware stack
- [ ] Responses include appropriate `Content-Encoding` header when compressed
- [ ] Compression respects client `Accept-Encoding` header
- [ ] Verify reduced transfer sizes in browser dev tools

## Implementation Notes

### Changes required

1. **Update Cargo.toml** (`vibes-server/Cargo.toml`):
   ```toml
   tower-http = { version = "0.6", features = ["cors", "compression-gzip", "compression-br"] }
   ```

2. **Add middleware** (`vibes-server/src/http/mod.rs`):
   ```rust
   use tower_http::compression::CompressionLayer;

   // In create_router():
   .layer(CompressionLayer::new())
   ```

### Future consideration

Pre-compressed assets at build time (emit `.gz` and `.br` files from vite) would eliminate runtime CPU cost, but requires more complex serving logic to detect `Accept-Encoding` and serve the right file. Server-side compression is the right starting point.
