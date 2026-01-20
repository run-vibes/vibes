# Planning Board

> [!NOTE]
> Auto-generated from directory structure. Run `just board generate` to update.

## In Progress

| Story | Type | Priority | Scope |
|-------|------|----------|-------|
| [[FEAT][0190]-eval-cli](stages/in-progress/stories/[FEAT][0190]-eval-cli.md) | feat | medium | evals/01-performance-evaluation |

## Backlog

| Story | Type | Priority | Scope |
|-------|------|----------|-------|
| [[FEAT][0115]-eval-web-ui](stages/backlog/stories/[FEAT][0115]-eval-web-ui.md) | feat | medium | evals/01-performance-evaluation |
| [[FEAT][0116]-pty-server-core](stages/backlog/stories/[FEAT][0116]-pty-server-core.md) | feat | medium | tui/06-terminal-server |
| [[FEAT][0117]-session-management](stages/backlog/stories/[FEAT][0117]-session-management.md) | feat | medium | tui/06-terminal-server |
| [[FEAT][0118]-websocket-pty-endpoint](stages/backlog/stories/[FEAT][0118]-websocket-pty-endpoint.md) | feat | medium | tui/06-terminal-server |
| [[FEAT][0119]-xtermjs-web-integration](stages/backlog/stories/[FEAT][0119]-xtermjs-web-integration.md) | feat | medium | tui/06-terminal-server |

<details>
<summary><h2>Icebox</h2></summary>

| Story | Type | Priority | Scope |
|-------|------|----------|-------|
| [[CHORE][0087]-use-cranelift-for-builds](stages/icebox/stories/[CHORE][0087]-use-cranelift-for-builds.md) | chore | low | dev-environment |
| [[CHORE][0110]-add-scheduled-mutation-testing-workflow](stages/icebox/stories/[CHORE][0110]-add-scheduled-mutation-testing-workflow.md) | chore | medium |  |
| [[DOCS][0003]-plugin-api-versioning-migration-plan](stages/icebox/stories/[DOCS][0003]-plugin-api-versioning-migration-plan.md) | docs | low | plugin-system |
| [[DOCS][0004]-event-schema-versioning-strategy](stages/icebox/stories/[DOCS][0004]-event-schema-versioning-strategy.md) | docs | low | plugin-system |
| [[FEAT][0013]-windows-daemon-support](stages/icebox/stories/[FEAT][0013]-windows-daemon-support.md) | feat | low | cli |
| [[FEAT][0101]-header-identity-display](stages/icebox/stories/[FEAT][0101]-header-identity-display.md) | feat | low | design-system |

</details>

## Epics

### [cli](epics/cli/) (active) - 5 milestones, 4 done

| Milestone | Status |
|-----------|--------|
| [01-command-line-control](epics/cli/milestones/01-command-line-control/) | done |
| [02-live-terminal-sync](epics/cli/milestones/02-live-terminal-sync/) | done |
| [03-event-management-commands](epics/cli/milestones/03-event-management-commands/) | done |
| [04-guided-setup](epics/cli/milestones/04-guided-setup/) | done |
| [05-enhanced-cli-experience](epics/cli/milestones/05-enhanced-cli-experience/) | planned |

### [evals](epics/evals/) (planned) - 6 milestones, 0 done

| Milestone | Status |
|-----------|--------|
| [01-performance-evaluation](epics/evals/milestones/01-performance-evaluation/) | in-progress |
| [02-long-term-studies](epics/evals/milestones/02-long-term-studies/) | planned |
| [03-performance-reports](epics/evals/milestones/03-performance-reports/) | planned |
| [04-performance-trends](epics/evals/milestones/04-performance-trends/) | planned |
| [05-benchmark-suite](epics/evals/milestones/05-benchmark-suite/) | planned |
| [06-extended-benchmarks](epics/evals/milestones/06-extended-benchmarks/) | planned |

### [mobile](epics/mobile/) (active) - 2 milestones, 1 done

| Milestone | Status |
|-----------|--------|
| [01-mobile-alerts](epics/mobile/milestones/01-mobile-alerts/) | done |
| [02-ios-mobile-app](epics/mobile/milestones/02-ios-mobile-app/) | planned |

### [plugin-system](epics/plugin-system/) (active) - 3 milestones, 2 done

| Milestone | Status |
|-----------|--------|
| [01-extensible-plugin-system](epics/plugin-system/milestones/01-extensible-plugin-system/) | done |
| [02-rich-plugin-apis](epics/plugin-system/milestones/02-rich-plugin-apis/) | done |
| [03-out-of-box-plugins](epics/plugin-system/milestones/03-out-of-box-plugins/) | planned |

### [tui](epics/tui/) (planned) - 6 milestones, 3 done

| Milestone | Status |
|-----------|--------|
| [01-terminal-ui-framework](epics/tui/milestones/01-terminal-ui-framework/) | done |
| [02-terminal-dashboard](epics/tui/milestones/02-terminal-dashboard/) | done |
| [03-terminal-agent-control](epics/tui/milestones/03-terminal-agent-control/) | done |
| [04-swarm-monitoring](epics/tui/milestones/04-swarm-monitoring/) | planned |
| [05-customizable-themes](epics/tui/milestones/05-customizable-themes/) | in-progress |
| [06-terminal-server](epics/tui/milestones/06-terminal-server/) | planned |

### [web-ui](epics/web-ui/) (active) - 5 milestones, 4 done

| Milestone | Status |
|-----------|--------|
| [01-web-dashboard](epics/web-ui/milestones/01-web-dashboard/) | done |
| [02-modern-web-interface](epics/web-ui/milestones/02-modern-web-interface/) | done |
| [03-infinite-event-stream](epics/web-ui/milestones/03-infinite-event-stream/) | done |
| [04-crt-visual-design](epics/web-ui/milestones/04-crt-visual-design/) | done |
| [05-image-understanding](epics/web-ui/milestones/05-image-understanding/) | in-progress |

## Done

<details>
<summary>Completed Epics & Milestones</summary>

### [agents](epics/agents/) (done)

- [01-autonomous-agents](epics/agents/milestones/01-autonomous-agents/)

### [cli](epics/cli/) - Completed Milestones

- [01-command-line-control](epics/cli/milestones/01-command-line-control/)
- [02-live-terminal-sync](epics/cli/milestones/02-live-terminal-sync/)
- [03-event-management-commands](epics/cli/milestones/03-event-management-commands/)
- [04-guided-setup](epics/cli/milestones/04-guided-setup/)

### [coherence-verification](epics/coherence-verification/) (done)

- [01-verification-artifact-pipeline](epics/coherence-verification/milestones/01-verification-artifact-pipeline/)
- [02-epic-based-project-hierarchy](epics/coherence-verification/milestones/02-epic-based-project-hierarchy/)
- [03-formal-planning-process](epics/coherence-verification/milestones/03-formal-planning-process/)
- [04-ai-assisted-verification](epics/coherence-verification/milestones/04-ai-assisted-verification/)
- [05-learnings-capture](epics/coherence-verification/milestones/05-learnings-capture/)

### [core](epics/core/) (done)

- [01-remote-session-access](epics/core/milestones/01-remote-session-access/)
- [02-persistent-conversations](epics/core/milestones/02-persistent-conversations/)
- [03-parallel-workspaces](epics/core/milestones/03-parallel-workspaces/)
- [04-reliable-test-suite](epics/core/milestones/04-reliable-test-suite/)
- [05-full-terminal-emulation](epics/core/milestones/05-full-terminal-emulation/)
- [06-efficient-scrollback](epics/core/milestones/06-efficient-scrollback/)
- [07-visual-project-planning](epics/core/milestones/07-visual-project-planning/)
- [08-bundled-event-store](epics/core/milestones/08-bundled-event-store/)
- [09-native-event-storage](epics/core/milestones/09-native-event-storage/)
- [10-event-driven-architecture](epics/core/milestones/10-event-driven-architecture/)
- [11-organized-project-tracking](epics/core/milestones/11-organized-project-tracking/)

### [groove](epics/groove/) (done)

- [01-ai-harness-discovery](epics/groove/milestones/01-ai-harness-discovery/)
- [02-persistent-learning-storage](epics/groove/milestones/02-persistent-learning-storage/)
- [03-secure-data-handling](epics/groove/milestones/03-secure-data-handling/)
- [04-context-injection](epics/groove/milestones/04-context-injection/)
- [05-structured-assessments](epics/groove/milestones/05-structured-assessments/)
- [06-assessment-pipeline](epics/groove/milestones/06-assessment-pipeline/)
- [07-automatic-learning-capture](epics/groove/milestones/07-automatic-learning-capture/)
- [08-learning-impact-tracking](epics/groove/milestones/08-learning-impact-tracking/)
- [09-smart-recommendations](epics/groove/milestones/09-smart-recommendations/)
- [10-learning-insights-dashboard](epics/groove/milestones/10-learning-insights-dashboard/)
- [11-handling-new-situations](epics/groove/milestones/11-handling-new-situations/)
- [12-novelty-monitoring](epics/groove/milestones/12-novelty-monitoring/)

### [mobile](epics/mobile/) - Completed Milestones

- [01-mobile-alerts](epics/mobile/milestones/01-mobile-alerts/)

### [models](epics/models/) (done)

- [01-model-management](epics/models/milestones/01-model-management/)

### [networking](epics/networking/) (done)

- [01-secure-remote-access](epics/networking/milestones/01-secure-remote-access/)
- [02-team-authentication](epics/networking/milestones/02-team-authentication/)

### [observability](epics/observability/) (done)

- [01-distributed-tracing](epics/observability/milestones/01-distributed-tracing/)

### [plugin-system](epics/plugin-system/) - Completed Milestones

- [01-extensible-plugin-system](epics/plugin-system/milestones/01-extensible-plugin-system/)
- [02-rich-plugin-apis](epics/plugin-system/milestones/02-rich-plugin-apis/)

### [tui](epics/tui/) - Completed Milestones

- [01-terminal-ui-framework](epics/tui/milestones/01-terminal-ui-framework/)
- [02-terminal-dashboard](epics/tui/milestones/02-terminal-dashboard/)
- [03-terminal-agent-control](epics/tui/milestones/03-terminal-agent-control/)

### [web-ui](epics/web-ui/) - Completed Milestones

- [01-web-dashboard](epics/web-ui/milestones/01-web-dashboard/)
- [02-modern-web-interface](epics/web-ui/milestones/02-modern-web-interface/)
- [03-infinite-event-stream](epics/web-ui/milestones/03-infinite-event-stream/)
- [04-crt-visual-design](epics/web-ui/milestones/04-crt-visual-design/)

</details>
