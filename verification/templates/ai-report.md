# AI Verification Report: {{id}}

**Story:** {{title}}
**Scope:** {{scope}}
**Model:** {{model}}
**Generated:** {{timestamp}}

## Summary

| Result | Count |
|--------|-------|
| Pass | {{summary.pass}} |
| Fail | {{summary.fail}} |
| Needs Review | {{summary.needsReview}} |

**Overall: {{#if allPassed}}All criteria passed{{else}}{{summary.fail}} failed, {{summary.needsReview}} need review{{/if}}**

## Criteria

{{#each criteria}}
### {{@index}}. {{this.text}}
**Artifact:** `{{this.artifact}}`
{{#if this.missing}}
**Verdict:** Artifact not found
{{else}}
**Verdict:** {{this.verdict}} ({{this.confidenceLevel}} confidence: {{this.confidence}}%)

> {{this.evidence}}
{{#if this.suggestion}}

**Suggestion:** {{this.suggestion}}
{{/if}}
{{/if}}

{{/each}}
{{#if failedCriteria.length}}
## Failed Criteria Summary

The following criteria did not pass and may require attention:

{{#each failedCriteria}}
- **{{this.text}}** ({{this.confidence}}%): {{this.suggestion}}
{{/each}}
{{/if}}
