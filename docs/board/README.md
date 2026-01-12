Generated README.md
Auto-generated from directory structure. Run `just board generate` to update.

## In Progress

*No stories in progress*

## Backlog

| Story | Type | Priority | Epics |
|-------|------|----------|-------|
| [chore-0087-use-cranelift-for-builds](stages/backlog/stories/chore-0087-use-cranelift-for-builds.md) | chore | medium | dev-environment |
| [chore-0088-add-codecov-ci-integration](stages/backlog/stories/chore-0088-add-codecov-ci-integration.md) | chore | medium | dev-environment |
| [docs-0003-plugin-api-versioning-migration-plan](stages/backlog/stories/docs-0003-plugin-api-versioning-migration-plan.md) | docs | low | plugin-system |
| [docs-0004-event-schema-versioning-strategy](stages/backlog/stories/docs-0004-event-schema-versioning-strategy.md) | docs | low | core,plugin-system |
| [feat-0013-windows-daemon-support](stages/backlog/stories/feat-0013-windows-daemon-support.md) | feat | low | cli |
| [feat-0071-setup-wizard-infrastructure](stages/backlog/stories/feat-0071-setup-wizard-infrastructure.md) | feat | medium | cli |
| [feat-0072-cloudflared-state-detection](stages/backlog/stories/feat-0072-cloudflared-state-detection.md) | feat | medium | cli |
| [feat-0073-tunnel-wizard-quick-mode](stages/backlog/stories/feat-0073-tunnel-wizard-quick-mode.md) | feat | medium | cli |
| [feat-0074-tunnel-wizard-named-mode](stages/backlog/stories/feat-0074-tunnel-wizard-named-mode.md) | feat | medium | cli |
| [feat-0075-tunnel-wizard-no-cloudflared](stages/backlog/stories/feat-0075-tunnel-wizard-no-cloudflared.md) | feat | medium | cli |
| [feat-0076-auth-setup-wizard](stages/backlog/stories/feat-0076-auth-setup-wizard.md) | feat | medium | cli |
| [feat-0077-config-save-load](stages/backlog/stories/feat-0077-config-save-load.md) | feat | medium | cli |
| [feat-0078-connectivity-validation](stages/backlog/stories/feat-0078-connectivity-validation.md) | feat | medium | cli |

## Epics

| Epic | Status | Stories |
|------|--------|---------|
| [agents](epics/agents/) | planned | 0 |
| [cli](epics/cli/) | active | 13 |
| [core](epics/core/) | active | 22 |
| [cross-platform](epics/cross-platform/) | active | 0 |
| [dev-environment](epics/dev-environment/) | active | 0 |
| [evals](epics/evals/) | planned | 0 |
| [groove](epics/groove/) | active | 0 |
| [mobile](epics/mobile/) | active | 0 |
| [models](epics/models/) | planned | 0 |
| [networking](epics/networking/) | active | 0 |
| [observability](epics/observability/) | planned | 0 |
| [plugin-system](epics/plugin-system/) | active | 23 |
| [release](epics/release/) | active | 0 |
| [tui](epics/tui/) | planned | 0 |
| [verification](epics/verification/) | active | 0 |
| [web-ui](epics/web-ui/) | active | 25 |

## Milestones

| Milestone | Status | Epics |
|-----------|--------|-------|
| [01-core-proxy](milestones/01-core-proxy/) | done | core,networking |
| [02-cli](milestones/02-cli/) | done | cli |
| [03-plugin-foundation](milestones/03-plugin-foundation/) | done | plugin-system |
| [04-server-web-ui](milestones/04-server-web-ui/) | done | web-ui |
| [05-cloudflare-tunnel](milestones/05-cloudflare-tunnel/) | done | networking |
| [06-cloudflare-access](milestones/06-cloudflare-access/) | done | networking |
| [07-push-notifications](milestones/07-push-notifications/) | done | mobile,networking |
| [08-chat-history](milestones/08-chat-history/) | done | core |
| [09-multi-session](milestones/09-multi-session/) | done | core |
| [10-cli-web-mirroring](milestones/10-cli-web-mirroring/) | done | cli |
| [11-test-coverage](milestones/11-test-coverage/) | done | core |
| [12-pty-backend](milestones/12-pty-backend/) | done | core |
| [13-scrollback-simplification](milestones/13-scrollback-simplification/) | done | core |
| [14-kanban-board](milestones/14-kanban-board/) | done | core |
| [15-harness-introspection](milestones/15-harness-introspection/) | done | core |
| [16-iggy-bundling](milestones/16-iggy-bundling/) | done | core |
| [17-web-ui-modernization](milestones/17-web-ui-modernization/) | done | web-ui |
| [18-iggy-sdk-integration](milestones/18-iggy-sdk-integration/) | done | core |
| [19-eventlog-wiring](milestones/19-eventlog-wiring/) | done | core |
| [20-event-cli](milestones/20-event-cli/) | done | cli |
| [21-storage-foundation](milestones/21-storage-foundation/) | done | core |
| [22-security-foundation](milestones/22-security-foundation/) | done | networking |
| [23-plugin-api-extension](milestones/23-plugin-api-extension/) | done | plugin-system |
| [24-capture-inject](milestones/24-capture-inject/) | done | core |
| [25-assessment-types](milestones/25-assessment-types/) | done | core,cli,plugin-system |
| [26-firehose-infinite-scroll](milestones/26-firehose-infinite-scroll/) | done | web-ui |
| [27-crt-design-system](milestones/27-crt-design-system/) | done | web-ui |
| [28-board-restructure](milestones/28-board-restructure/) | done | core |
| [29-assessment-framework](milestones/29-assessment-framework/) | done | core,cli,plugin-system |
| [30-learning-extraction](milestones/30-learning-extraction/) | done | plugin-system |
| [31-attribution-engine](milestones/31-attribution-engine/) | done | plugin-system |
| [32-adaptive-strategies](milestones/32-adaptive-strategies/) | done | plugin-system |
| [33-groove-dashboard](milestones/33-groove-dashboard/) | done | plugin-system |
| [34-open-world-adaptation](milestones/34-open-world-adaptation/) | done | plugin-system |
| [35-setup-wizards](milestones/35-setup-wizards/) | planned | cli |
| [36-openworld-dashboard](milestones/36-openworld-dashboard/) | done | plugin-system |
| [37-models-registry-auth](milestones/37-models-registry-auth/) | planned | models |
| [38-agent-core](milestones/38-agent-core/) | planned | agents |
| [39-eval-core](milestones/39-eval-core/) | planned | evals |
| [40-observability-tracing](milestones/40-observability-tracing/) | planned | observability |
| [41-tui-core](milestones/41-tui-core/) | planned | tui |
| [42-default-plugins](milestones/42-default-plugins/) | planned | plugin-system |
| [43-cli-enhancements](milestones/43-cli-enhancements/) | planned | cli |
| [44-ios-app](milestones/44-ios-app/) | planned | mobile |

## Done

<details>
<summary>View completed stories</summary>

- [bug-0001-cwd-propagation](stages/done/stories/bug-0001-cwd-propagation.md)
- [bug-0002-firehose-not-loading-historical-events](stages/done/stories/bug-0002-firehose-not-loading-historical-events.md)
- [bug-0003-pty-created-with-hardcoded-dimensions-ignoring-client-size](stages/done/stories/bug-0003-pty-created-with-hardcoded-dimensions-ignoring-client-size.md)
- [bug-0004-fix-flaky-pty-integration-tests](stages/done/stories/bug-0004-fix-flaky-pty-integration-tests.md)
- [bug-0005-dashboard-trends-route-not-found](stages/done/stories/bug-0005-dashboard-trends-route-not-found.md)
- [chore-0001-nav-cleanup](stages/done/stories/chore-0001-nav-cleanup.md)
- [chore-0002-firehose-filter-cleanup](stages/done/stories/chore-0002-firehose-filter-cleanup.md)
- [chore-0003-assessment-tier-buttons](stages/done/stories/chore-0003-assessment-tier-buttons.md)
- [chore-0010-align-sessions-page-layout-with-firehose](stages/done/stories/chore-0010-align-sessions-page-layout-with-firehose.md)
- [chore-0011-align-groove-pages-layout-with-firehose](stages/done/stories/chore-0011-align-groove-pages-layout-with-firehose.md)
- [chore-0013-add-test-coverage-metrics](stages/done/stories/chore-0013-add-test-coverage-metrics.md)
- [chore-0014-resolve-remaining-code-todos](stages/done/stories/chore-0014-resolve-remaining-code-todos.md)
- [chore-0015-cli-help-text-audit](stages/done/stories/chore-0015-cli-help-text-audit.md)
- [chore-0016-proper-shutdown-signal-coordination-for-plugin-manager](stages/done/stories/chore-0016-proper-shutdown-signal-coordination-for-plugin-manager.md)
- [chore-0017-remove-assess-button-icon](stages/done/stories/chore-0017-remove-assess-button-icon.md)
- [chore-0018-change-default-iggy-port](stages/done/stories/chore-0018-change-default-iggy-port.md)
- [chore-0067-test-organization](stages/done/stories/chore-0067-test-organization.md)
- [chore-0085-fix-web-ui-chunk-warnings](stages/done/stories/chore-0085-fix-web-ui-chunk-warnings.md)
- [chore-0086-fix-nix-mac-cargo-llvm-cov](stages/done/stories/chore-0086-fix-nix-mac-cargo-llvm-cov.md)
- [docs-0001-fix-broken-documentation-links](stages/done/stories/docs-0001-fix-broken-documentation-links.md)
- [docs-0002-update-prd-for-current-architecture](stages/done/stories/docs-0002-update-prd-for-current-architecture.md)
- [docs-0068-documentation-review](stages/done/stories/docs-0068-documentation-review.md)
- [feat-0003-web-ui-navigation-hierarchy](stages/done/stories/feat-0003-web-ui-navigation-hierarchy.md)
- [feat-0004-subnav](stages/done/stories/feat-0004-subnav.md)
- [feat-0005-session-terminal-outline](stages/done/stories/feat-0005-session-terminal-outline.md)
- [feat-0014-production-iggy-polling-for-assessment-queries](stages/done/stories/feat-0014-production-iggy-polling-for-assessment-queries.md)
- [feat-0015-paginate-assessment-history](stages/done/stories/feat-0015-paginate-assessment-history.md)
- [feat-0016-precompute-tier-distribution-stats](stages/done/stories/feat-0016-precompute-tier-distribution-stats.md)
- [feat-0017-link-more-to-history-page](stages/done/stories/feat-0017-link-more-to-history-page.md)
- [feat-0019-learning-types-storage](stages/done/stories/feat-0019-learning-types-storage.md)
- [feat-0020-local-embedder](stages/done/stories/feat-0020-local-embedder.md)
- [feat-0021-semantic-deduplication](stages/done/stories/feat-0021-semantic-deduplication.md)
- [feat-0022-correction-detector](stages/done/stories/feat-0022-correction-detector.md)
- [feat-0023-error-recovery-detector](stages/done/stories/feat-0023-error-recovery-detector.md)
- [feat-0024-extraction-consumer](stages/done/stories/feat-0024-extraction-consumer.md)
- [feat-0025-extraction-cli](stages/done/stories/feat-0025-extraction-cli.md)
- [feat-0026-attribution-types-storage](stages/done/stories/feat-0026-attribution-types-storage.md)
- [feat-0027-activation-detection](stages/done/stories/feat-0027-activation-detection.md)
- [feat-0028-temporal-correlation](stages/done/stories/feat-0028-temporal-correlation.md)
- [feat-0029-ablation-manager](stages/done/stories/feat-0029-ablation-manager.md)
- [feat-0030-value-aggregation](stages/done/stories/feat-0030-value-aggregation.md)
- [feat-0031-attribution-consumer](stages/done/stories/feat-0031-attribution-consumer.md)
- [feat-0032-auto-deprecation](stages/done/stories/feat-0032-auto-deprecation.md)
- [feat-0033-attribution-cli](stages/done/stories/feat-0033-attribution-cli.md)
- [feat-0034-strategy-types-storage](stages/done/stories/feat-0034-strategy-types-storage.md)
- [feat-0035-strategy-distribution](stages/done/stories/feat-0035-strategy-distribution.md)
- [feat-0036-thompson-sampling](stages/done/stories/feat-0036-thompson-sampling.md)
- [feat-0037-outcome-router](stages/done/stories/feat-0037-outcome-router.md)
- [feat-0038-distribution-updater](stages/done/stories/feat-0038-distribution-updater.md)
- [feat-0039-strategy-consumer](stages/done/stories/feat-0039-strategy-consumer.md)
- [feat-0040-novelty-hook](stages/done/stories/feat-0040-novelty-hook.md)
- [feat-0041-strategy-cli](stages/done/stories/feat-0041-strategy-cli.md)
- [feat-0042-dashboard-layout](stages/done/stories/feat-0042-dashboard-layout.md)
- [feat-0043-websocket-dashboard](stages/done/stories/feat-0043-websocket-dashboard.md)
- [feat-0044-overview-page](stages/done/stories/feat-0044-overview-page.md)
- [feat-0045-learnings-page](stages/done/stories/feat-0045-learnings-page.md)
- [feat-0046-learning-actions](stages/done/stories/feat-0046-learning-actions.md)
- [feat-0047-attribution-page](stages/done/stories/feat-0047-attribution-page.md)
- [feat-0048-strategy-page](stages/done/stories/feat-0048-strategy-page.md)
- [feat-0049-health-page](stages/done/stories/feat-0049-health-page.md)
- [feat-0050-chart-components](stages/done/stories/feat-0050-chart-components.md)
- [feat-0051-learning-indicator](stages/done/stories/feat-0051-learning-indicator.md)
- [feat-0052-openworld-types](stages/done/stories/feat-0052-openworld-types.md)
- [feat-0053-novelty-detector](stages/done/stories/feat-0053-novelty-detector.md)
- [feat-0054-dbscan-clustering](stages/done/stories/feat-0054-dbscan-clustering.md)
- [feat-0055-gap-detector](stages/done/stories/feat-0055-gap-detector.md)
- [feat-0056-graduated-response](stages/done/stories/feat-0056-graduated-response.md)
- [feat-0057-solution-generator](stages/done/stories/feat-0057-solution-generator.md)
- [feat-0058-openworld-hook](stages/done/stories/feat-0058-openworld-hook.md)
- [feat-0059-openworld-store](stages/done/stories/feat-0059-openworld-store.md)
- [feat-0060-openworld-consumer](stages/done/stories/feat-0060-openworld-consumer.md)
- [feat-0061-cli-novelty](stages/done/stories/feat-0061-cli-novelty.md)
- [feat-0062-cli-gaps](stages/done/stories/feat-0062-cli-gaps.md)
- [feat-0063-openworld-config](stages/done/stories/feat-0063-openworld-config.md)
- [feat-0064-visual-regression](stages/done/stories/feat-0064-visual-regression.md)
- [feat-0065-workflow-videos](stages/done/stories/feat-0065-workflow-videos.md)
- [feat-0066-cli-recording](stages/done/stories/feat-0066-cli-recording.md)
- [feat-0079-openworld-page-and-routing](stages/done/stories/feat-0079-openworld-page-and-routing.md)
- [feat-0080-openworld-backend-data-providers](stages/done/stories/feat-0080-openworld-backend-data-providers.md)
- [feat-0081-novelty-tab-components](stages/done/stories/feat-0081-novelty-tab-components.md)
- [feat-0082-gaps-tab-with-split-view](stages/done/stories/feat-0082-gaps-tab-with-split-view.md)
- [feat-0083-solutions-tab-with-actions](stages/done/stories/feat-0083-solutions-tab-with-actions.md)
- [feat-0084-activity-tab-with-live-updates](stages/done/stories/feat-0084-activity-tab-with-live-updates.md)
- [m14-chore-01-design](stages/done/stories/m14-chore-01-design.md)
- [m14-chore-02-implementation](stages/done/stories/m14-chore-02-implementation.md)
- [m26-chore-01-eventbus-cleanup](stages/done/stories/m26-chore-01-eventbus-cleanup.md)
- [m26-chore-03-integration-testing](stages/done/stories/m26-chore-03-integration-testing.md)
- [m26-feat-02-processor-wiring](stages/done/stories/m26-feat-02-processor-wiring.md)
- [m26-feat-06-iggy-assessment-log](stages/done/stories/m26-feat-06-iggy-assessment-log.md)
- [m26-feat-09-complete-hook-support](stages/done/stories/m26-feat-09-complete-hook-support.md)
- [m26-feat-10-cli-assess-queries](stages/done/stories/m26-feat-10-cli-assess-queries.md)
- [m26-fix-04-plugin-route-mounting](stages/done/stories/m26-fix-04-plugin-route-mounting.md)
- [m26-fix-05-event-flow-to-firehose](stages/done/stories/m26-fix-05-event-flow-to-firehose.md)
- [m26-fix-08-assessment-multiselect](stages/done/stories/m26-fix-08-assessment-multiselect.md)
- [m26-refactor-07-plugin-lifecycle](stages/done/stories/m26-refactor-07-plugin-lifecycle.md)
- [m27-chore-0012-align-settings-page-with-crt-design-system](stages/done/stories/m27-chore-0012-align-settings-page-with-crt-design-system.md)
- [m29-feat-0012-wire-circuit-breaker-intervention](stages/done/stories/m29-feat-0012-wire-circuit-breaker-intervention.md)
- [m29-feat-11-web-ui-assess-queries](stages/done/stories/m29-feat-11-web-ui-assess-queries.md)
- [m36-feat-01-backend-pagination](stages/done/stories/m36-feat-01-backend-pagination.md)
- [m36-feat-02-backend-filters](stages/done/stories/m36-feat-02-backend-filters.md)
- [m36-feat-03-frontend-hook](stages/done/stories/m36-feat-03-frontend-hook.md)
- [m36-feat-04-virtual-scroll](stages/done/stories/m36-feat-04-virtual-scroll.md)
- [m36-feat-05-ui-polish](stages/done/stories/m36-feat-05-ui-polish.md)
- [m36-feat-06-uuidv7-sequencing](stages/done/stories/m36-feat-06-uuidv7-sequencing.md)
- [m36-refactor-01-single-partition](stages/done/stories/m36-refactor-01-single-partition.md)
- [m37-chore-09-cleanup](stages/done/stories/m37-chore-09-cleanup.md)
- [m37-feat-01-design-tokens](stages/done/stories/m37-feat-01-design-tokens.md)
- [m37-feat-02-theme-toggle](stages/done/stories/m37-feat-02-theme-toggle.md)
- [m37-feat-03-crt-effects](stages/done/stories/m37-feat-03-crt-effects.md)
- [m37-feat-04-typography](stages/done/stories/m37-feat-04-typography.md)
- [m37-feat-05-core-components](stages/done/stories/m37-feat-05-core-components.md)
- [m37-feat-06-navigation](stages/done/stories/m37-feat-06-navigation.md)
- [m37-feat-07-firehose](stages/done/stories/m37-feat-07-firehose.md)
- [m37-feat-08-session-cards](stages/done/stories/m37-feat-08-session-cards.md)
- [refactor-0008-consolidate-assessment-types-module](stages/done/stories/refactor-0008-consolidate-assessment-types-module.md)
- [refactor-0069-dashboard-visual-consistency](stages/done/stories/refactor-0069-dashboard-visual-consistency.md)
- [refactor-0070-design-system-extraction](stages/done/stories/refactor-0070-design-system-extraction.md)

</details>
