---
id: CHORE0085
title: Fix web UI chunk size warnings
type: chore
status: done
priority: low
epics: [dev-environment, web-ui]
depends: []
estimate: 1h
created: 2026-01-12
---

# Fix web UI chunk size warnings

## Summary

Fix the Vite build warning about large chunks in the web UI:

```
(!) Some chunks are larger than 500 kB after minification.
```

## Context

The web UI build produces chunks that exceed the recommended 500 kB limit. This can impact initial page load performance.

## Tasks

### Task 1: Analyze and implement code splitting

**Steps:**
1. Run `npm run build --workspace=web-ui` and identify large chunks
2. Implement dynamic imports for heavy dependencies (react-query, charts)
3. Configure `build.rollupOptions.output.manualChunks` if needed
4. Verify chunks are under 500 kB
5. Test that lazy loading works correctly
6. Commit: `chore(web-ui): implement code splitting for smaller chunks`

## Acceptance Criteria

- [ ] No chunk size warnings during build
- [ ] Page still loads and functions correctly
- [ ] Lazy-loaded components load on demand
