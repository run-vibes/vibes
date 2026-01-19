# Mobile - Product Requirements

> Monitor and control vibes from mobile devices

## Problem Statement

Developers are not always at their desks. When agents are running long tasks, users want to monitor progress, receive alerts, and perform basic controls from their phones. A native mobile app extends vibes reach beyond the terminal and browser.

## Users

- **Primary**: Developers monitoring long-running agent tasks
- **Secondary**: Team leads tracking project activity remotely
- **Tertiary**: On-call engineers responding to agent issues

## Requirements

### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-01 | Push notifications for agent events and alerts | must |
| FR-02 | Session and agent status monitoring | must |
| FR-03 | Basic agent controls (pause, resume, cancel) | should |
| FR-04 | Event stream viewing | should |
| FR-05 | Cost tracking dashboard | could |

### Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-01 | iOS support (primary platform) | must |
| NFR-02 | Secure authentication matching web/CLI | must |
| NFR-03 | Battery-efficient background updates | should |

## Success Criteria

- [ ] Users receive timely notifications for important events
- [ ] Can monitor agent status without opening laptop
- [ ] Basic controls work reliably from mobile
- [ ] App available in App Store

## Milestones

| ID | Milestone | Status |
|----|-----------|--------|
| 07 | [Mobile Alerts](milestones/07-mobile-alerts/) | done |
| 55 | [iOS Mobile App](milestones/55-ios-mobile-app/) | planned |
