# Planning Board

> [!NOTE]
> Auto-generated from directory structure. Run `just board generate` to update.

## In Progress

| Story | Type | Priority | Epic |
|-------|------|----------|------|
| [[FEAT][0190]-eval-cli](stages/in-progress/stories/[FEAT][0190]-eval-cli.md) | feat | medium | evals |

## Backlog

| Story | Type | Priority | Epic |
|-------|------|----------|------|
| [[FEAT][0115]-eval-web-ui](stages/backlog/stories/[FEAT][0115]-eval-web-ui.md) | feat | medium | evals |
| [[FEAT][0116]-pty-server-core](stages/backlog/stories/[FEAT][0116]-pty-server-core.md) | feat | medium | tui |
| [[FEAT][0117]-session-management](stages/backlog/stories/[FEAT][0117]-session-management.md) | feat | medium | tui |
| [[FEAT][0118]-websocket-pty-endpoint](stages/backlog/stories/[FEAT][0118]-websocket-pty-endpoint.md) | feat | medium | tui |
| [[FEAT][0119]-xtermjs-web-integration](stages/backlog/stories/[FEAT][0119]-xtermjs-web-integration.md) | feat | medium | tui |

## Icebox

| Story | Type | Priority | Blocked By |
|-------|------|----------|------------|
| [[CHORE][0087]-use-cranelift-for-builds](stages/icebox/stories/[CHORE][0087]-use-cranelift-for-builds.md) | chore | low |  |
| [[CHORE][0110]-add-scheduled-mutation-testing-workflow](stages/icebox/stories/[CHORE][0110]-add-scheduled-mutation-testing-workflow.md) | chore | medium |  |
| [[DOCS][0003]-plugin-api-versioning-migration-plan](stages/icebox/stories/[DOCS][0003]-plugin-api-versioning-migration-plan.md) | docs | low |  |
| [[DOCS][0004]-event-schema-versioning-strategy](stages/icebox/stories/[DOCS][0004]-event-schema-versioning-strategy.md) | docs | low |  |
| [[FEAT][0013]-windows-daemon-support](stages/icebox/stories/[FEAT][0013]-windows-daemon-support.md) | feat | low |  |
| [[FEAT][0101]-header-identity-display](stages/icebox/stories/[FEAT][0101]-header-identity-display.md) | feat | low | refactor-0100 |

## Epics

### [cli](epics/cli/) (active) - 5 milestones, 4 done

| Milestone | Status |
|-----------|--------|
| [02-command-line-control](epics/cli/milestones/02-command-line-control/) | done |
| [10-live-terminal-sync](epics/cli/milestones/10-live-terminal-sync/) | done |
| [20-event-management-commands](epics/cli/milestones/20-event-management-commands/) | done |
| [35-guided-setup](epics/cli/milestones/35-guided-setup/) | done |
| [54-enhanced-cli-experience](epics/cli/milestones/54-enhanced-cli-experience/) | planned |

### [coherence-verification](epics/coherence-verification/) (active) - 2 milestones, 0 done

| Milestone | Status |
|-----------|--------|
| [01-verification-artifact-pipeline](epics/coherence-verification/milestones/01-verification-artifact-pipeline/) | planned |
| [02-epic-based-project-hierarchy](epics/coherence-verification/milestones/02-epic-based-project-hierarchy/) | planned |

### [evals](epics/evals/) (planned) - 6 milestones, 0 done

| Milestone | Status |
|-----------|--------|
| [39-performance-evaluation](epics/evals/milestones/39-performance-evaluation/) | in-progress |
| [48-long-term-studies](epics/evals/milestones/48-long-term-studies/) | planned |
| [49-performance-reports](epics/evals/milestones/49-performance-reports/) | planned |
| [50-performance-trends](epics/evals/milestones/50-performance-trends/) | planned |
| [51-benchmark-suite](epics/evals/milestones/51-benchmark-suite/) | planned |
| [52-extended-benchmarks](epics/evals/milestones/52-extended-benchmarks/) | planned |

### [mobile](epics/mobile/) (active) - 2 milestones, 1 done

| Milestone | Status |
|-----------|--------|
| [07-mobile-alerts](epics/mobile/milestones/07-mobile-alerts/) | done |
| [55-ios-mobile-app](epics/mobile/milestones/55-ios-mobile-app/) | planned |

### [plugin-system](epics/plugin-system/) (active) - 3 milestones, 2 done

| Milestone | Status |
|-----------|--------|
| [03-extensible-plugin-system](epics/plugin-system/milestones/03-extensible-plugin-system/) | done |
| [23-rich-plugin-apis](epics/plugin-system/milestones/23-rich-plugin-apis/) | done |
| [53-out-of-box-plugins](epics/plugin-system/milestones/53-out-of-box-plugins/) | planned |

### [tui](epics/tui/) (planned) - 6 milestones, 3 done

| Milestone | Status |
|-----------|--------|
| [41-terminal-ui-framework](epics/tui/milestones/41-terminal-ui-framework/) | done |
| [42-terminal-dashboard](epics/tui/milestones/42-terminal-dashboard/) | done |
| [43-terminal-agent-control](epics/tui/milestones/43-terminal-agent-control/) | done |
| [44-swarm-monitoring](epics/tui/milestones/44-swarm-monitoring/) | planned |
| [45-customizable-themes](epics/tui/milestones/45-customizable-themes/) | in-progress |
| [46-terminal-server](epics/tui/milestones/46-terminal-server/) | planned |

### [web-ui](epics/web-ui/) (active) - 5 milestones, 4 done

| Milestone | Status |
|-----------|--------|
| [04-web-dashboard](epics/web-ui/milestones/04-web-dashboard/) | done |
| [17-modern-web-interface](epics/web-ui/milestones/17-modern-web-interface/) | done |
| [26-infinite-event-stream](epics/web-ui/milestones/26-infinite-event-stream/) | done |
| [27-crt-visual-design](epics/web-ui/milestones/27-crt-visual-design/) | done |
| [47-image-understanding](epics/web-ui/milestones/47-image-understanding/) | in-progress |

## Done

<details>
<summary>Completed Epics & Milestones</summary>

### [agents](epics/agents/) (done)

- [38-autonomous-agents](epics/agents/milestones/38-autonomous-agents/)

### [cli](epics/cli/) - Completed Milestones

- [02-command-line-control](epics/cli/milestones/02-command-line-control/)
- [10-live-terminal-sync](epics/cli/milestones/10-live-terminal-sync/)
- [20-event-management-commands](epics/cli/milestones/20-event-management-commands/)
- [35-guided-setup](epics/cli/milestones/35-guided-setup/)

### [core](epics/core/) (done)

- [01-remote-session-access](epics/core/milestones/01-remote-session-access/)
- [08-persistent-conversations](epics/core/milestones/08-persistent-conversations/)
- [09-parallel-workspaces](epics/core/milestones/09-parallel-workspaces/)
- [11-reliable-test-suite](epics/core/milestones/11-reliable-test-suite/)
- [12-full-terminal-emulation](epics/core/milestones/12-full-terminal-emulation/)
- [13-efficient-scrollback](epics/core/milestones/13-efficient-scrollback/)
- [14-visual-project-planning](epics/core/milestones/14-visual-project-planning/)
- [16-bundled-event-store](epics/core/milestones/16-bundled-event-store/)
- [18-native-event-storage](epics/core/milestones/18-native-event-storage/)
- [19-event-driven-architecture](epics/core/milestones/19-event-driven-architecture/)
- [28-organized-project-tracking](epics/core/milestones/28-organized-project-tracking/)

### [groove](epics/groove/) (done)

- [15-ai-harness-discovery](epics/groove/milestones/15-ai-harness-discovery/)
- [21-persistent-learning-storage](epics/groove/milestones/21-persistent-learning-storage/)
- [22-secure-data-handling](epics/groove/milestones/22-secure-data-handling/)
- [24-context-injection](epics/groove/milestones/24-context-injection/)
- [25-structured-assessments](epics/groove/milestones/25-structured-assessments/)
- [29-assessment-pipeline](epics/groove/milestones/29-assessment-pipeline/)
- [30-automatic-learning-capture](epics/groove/milestones/30-automatic-learning-capture/)
- [31-learning-impact-tracking](epics/groove/milestones/31-learning-impact-tracking/)
- [32-smart-recommendations](epics/groove/milestones/32-smart-recommendations/)
- [33-learning-insights-dashboard](epics/groove/milestones/33-learning-insights-dashboard/)
- [34-handling-new-situations](epics/groove/milestones/34-handling-new-situations/)
- [36-novelty-monitoring](epics/groove/milestones/36-novelty-monitoring/)

### [mobile](epics/mobile/) - Completed Milestones

- [07-mobile-alerts](epics/mobile/milestones/07-mobile-alerts/)

### [models](epics/models/) (done)

- [37-model-management](epics/models/milestones/37-model-management/)

### [networking](epics/networking/) (done)

- [05-secure-remote-access](epics/networking/milestones/05-secure-remote-access/)
- [06-team-authentication](epics/networking/milestones/06-team-authentication/)

### [observability](epics/observability/) (done)

- [40-distributed-tracing](epics/observability/milestones/40-distributed-tracing/)

### [plugin-system](epics/plugin-system/) - Completed Milestones

- [03-extensible-plugin-system](epics/plugin-system/milestones/03-extensible-plugin-system/)
- [23-rich-plugin-apis](epics/plugin-system/milestones/23-rich-plugin-apis/)

### [tui](epics/tui/) - Completed Milestones

- [41-terminal-ui-framework](epics/tui/milestones/41-terminal-ui-framework/)
- [42-terminal-dashboard](epics/tui/milestones/42-terminal-dashboard/)
- [43-terminal-agent-control](epics/tui/milestones/43-terminal-agent-control/)

### [web-ui](epics/web-ui/) - Completed Milestones

- [04-web-dashboard](epics/web-ui/milestones/04-web-dashboard/)
- [17-modern-web-interface](epics/web-ui/milestones/17-modern-web-interface/)
- [26-infinite-event-stream](epics/web-ui/milestones/26-infinite-event-stream/)
- [27-crt-visual-design](epics/web-ui/milestones/27-crt-visual-design/)

</details>
