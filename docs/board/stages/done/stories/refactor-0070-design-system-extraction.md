# Extract Design System Primitives

Extract reusable patterns from web-ui into the design-system package and add Ladle tooling.

## Problem

The web-ui has ~50 CSS files with ad-hoc styling patterns that duplicate the same visual patterns. For example:
- `StrategyTabs.css` and `AttributionTabs.css` are 100% identical
- The `metric` pattern (label + glowing value) appears 10+ times
- Cards with status borders repeat across 5+ components
- The design-system has primitives (Panel, StatusIndicator) that web-ui doesn't use

## Solution

1. Add `just web ladle` task for component development
2. Extract **Tabs** primitive from identical StrategyTabs/AttributionTabs
3. Extract **Metric** primitive (label + phosphor-glowing value)
4. Migrate web-ui components to use existing **Panel** and **StatusIndicator**
5. Update design-system documentation

## Acceptance Criteria

- [ ] `just web ladle` runs Ladle dev server
- [ ] `Tabs` primitive exists with stories and tests
- [ ] `Metric` primitive exists with stories and tests
- [ ] At least 2 web-ui components migrated to use `Panel`
- [ ] At least 2 web-ui components migrated to use `StatusIndicator`
- [ ] DESIGN_TOKENS.md updated with usage examples

## Technical Notes

### Tabs Primitive API
```tsx
<Tabs value={activeTab} onChange={setActiveTab}>
  <Tabs.Tab value="distribution">Distribution</Tabs.Tab>
  <Tabs.Tab value="overrides">Overrides</Tabs.Tab>
</Tabs>
```

### Metric Primitive API
```tsx
<Metric label="Success Rate" value="94.2%" />
<Metric label="Sessions" value={42} size="lg" />
```

### Migration Targets

**Panel adoption:**
- `DashboardCards.tsx` → use `<Panel title="...">`
- `SubsystemCard.tsx` → use `<Panel variant="status" status="ok|degraded|error">`

**StatusIndicator adoption:**
- `DashboardCards.css` `.status-indicator` → use design-system component
- `SubsystemCard.tsx` indicator → use design-system component

## References

- Design system: `design-system/src/primitives/`
- Existing tokens: `design-system/DESIGN_TOKENS.md`
- Ladle config: `design-system/.ladle/`
