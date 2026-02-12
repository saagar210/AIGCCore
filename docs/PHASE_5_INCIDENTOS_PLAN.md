# Phase 5 Implementation Plan — IncidentOS Pack

**Date:** 2026-02-12
**Scope:** Build incident log analysis, timeline reconstruction, dual-template rendering, and redaction enforcement
**Status:** STARTING

---

## Executive Summary

Phase 5 IncidentOS implements forensic incident log analysis with dual-template rendering (customer-facing redacted + internal full-fidelity). It reuses Phase 4 patterns (citation enforcement, deterministic processing, non-bypassable export pipeline) and extends them with redaction policy enforcement.

**Key Features:**
- Parse incident logs from JSON/NDJSON/text artifacts
- Reconstruct chronological timeline with evidence citations
- Render dual templates: customer_packet.md (sanitized) + internal_packet.md (full)
- Enforce redaction policy via gates (customer profile → what can be shown)
- Route exports through RunManager with redaction validation gates

---

## Step-by-Step Implementation

### Step 1: Log Parsing and Timeline Builder (STARTING)
**Goal:** Parse incident artifacts and create deterministic timeline

**Files to create/modify:**
- `core/src/incidentos/parser.rs` (NEW) — Parse logs from various formats
- `core/src/incidentos/timeline.rs` (NEW) — Build timeline from events
- `core/src/incidentos/model.rs` (EXTEND) — Add event and timeline models
- `core/corpus/incidents/` (NEW) — Golden corpus for incident logs

**Scope:**
- Parse JSON/NDJSON incident logs
- Extract events with timestamp, actor, action, affected system, evidence
- Generate deterministic event anchors (similar to RedlineOS clause anchors)
- Create timeline CSV with chronological ordering
- Create sample incident corpus

**Tests:**
- Parse valid JSON log structure
- Handle missing/malformed timestamps
- Determinism: same log yields same timeline hash
- Parse NDJSON and JSON variants

### Step 2: Dual-Template Rendering
**Goal:** Render customer and internal narratives with different redaction levels

**Files to create/modify:**
- `core/src/incidentos/render.rs` (EXTEND) — Add dual template rendering
- `core/src/incidentos/redaction.rs` (NEW) — Redaction policy rules

**Scope:**
- Load customer redaction profile (BASIC/STANDARD/STRICT)
- Apply profile rules: hide IP addresses, mask user names, sanitize system paths
- Render customer_packet.md with [REDACTED: reason] placeholders
- Render internal_packet.md with full text + citation markers
- Generate redactions_map.json documenting all redactions

**Tests:**
- BASIC profile: redact PII only
- STANDARD profile: redact PII + system paths
- STRICT profile: redact PII + system paths + command outputs
- Citation markers present in both templates
- Redaction map complete and accurate

### Step 3: Untrusted Input Sanitization (Completed in Scaffolding)
**Goal:** Protect against malicious log content

**Files:**
- `core/src/incidentos/sanitize.rs` (ALREADY IMPLEMENTED)

**Scope:**
- Remove NUL bytes and control characters
- Normalize line endings
- Validate UTF-8 (log files can contain binary)
- Treat untrusted text as inert evidence (preserve but escape)

### Step 4: Workflow Orchestration
**Goal:** Execute complete timeline → dual-render → export workflow

**Files to create/modify:**
- `core/src/incidentos/workflow.rs` (EXTEND) — Add execute_incidentos_workflow()

**Scope:**
- State machine: Ingested → Analyzed → Reviewed → Renderable → ExportReady
- Parse artifacts
- Build timeline
- Apply redaction profile
- Render templates
- Return IncidentWorkflowOutput

**Tests:**
- End-to-end workflow execution
- Determinism: same artifacts yield same output hashes
- State transitions validate correctly
- All required fields present in output

### Step 5: RunManager Integration and Redaction Gates
**Goal:** Route exports through 16-step pipeline with redaction validation

**Files to create/modify:**
- `src-tauri/src/main.rs` (ADD) — run_incidentos handler
- `tools/gates/check-incidentos-redaction.mjs` (NEW) — Redaction enforcement gate

**Scope:**
- Tauri handler validates input, executes workflow, creates ExportRequest
- Gate validates: customer_packet redacted per profile, internal_packet unredacted, both have citations
- Export blocked if redaction profile violated
- RunManager enforces gate before bundle generation

**Tests:**
- Handler invocation with valid input
- Customer profile correctly applied
- Export blocked when redaction violated
- Citations enforced in both templates

### Step 6: UI Panel
**Goal:** Provide incident analysis workflow UI

**Files to create/modify:**
- `src/ui/packs/IncidentOSPanel.tsx` (NEW)

**Scope:**
- Form: artifact selection, timeline bounds, redaction profile
- Show timeline preview
- Display redaction summary
- Result: export status, timeline length, redaction count

### Step 7: Integration Tests
**Goal:** Full end-to-end test suite

**Files to create/modify:**
- `core/src/incidentos/workflow.rs` (ADD tests)

**Tests:**
- Full workflow end-to-end
- Determinism verification
- Dual-template rendering accuracy
- Redaction enforcement
- State transitions

### Step 8: Golden Corpus and Closure
**Goal:** Create regression test assets and final documentation

**Files to create/modify:**
- `core/corpus/incidents/expected_outputs/` (NEW) — Golden outputs
- `docs/PHASE_5_FINAL_CLOSURE.md` (NEW) — Closure report

---

## Data Models to Implement

```rust
// Timeline event from incident log
pub struct IncidentEvent {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub source_system: String,
    pub actor: String,
    pub action: String,
    pub affected_resource: String,
    pub evidence_text: String,
    pub severity: String, // HIGH/MEDIUM/LOW
}

// Timeline with metadata
pub struct IncidentTimeline {
    pub timeline_id: String,
    pub incident_id: String,
    pub events: Vec<IncidentEvent>,
    pub total_duration: Duration,
    pub high_severity_count: usize,
}

// Redaction profile rules
pub enum RedactionProfile {
    Basic,     // Redact PII only
    Standard,  // Redact PII + system paths
    Strict,    // Redact PII + paths + command outputs
}

// What was redacted
pub struct RedactionRecord {
    pub span_start: usize,
    pub span_end: usize,
    pub original_text: String,
    pub reason: String,
    pub profile_rule: String,
}

// Workflow output
pub struct IncidentWorkflowOutput {
    pub stage: IncidentWorkflowStage,
    pub customer_packet: String,
    pub internal_packet: String,
    pub timeline_csv: String,
    pub event_count: usize,
    pub high_severity_count: usize,
    pub redaction_count: usize,
}
```

---

## Redaction Policy Rules

**BASIC Profile (PII Only):**
- Email addresses → [REDACTED: email]
- Phone numbers → [REDACTED: phone]
- Social security numbers → [REDACTED: ssn]
- Credit card numbers → [REDACTED: payment method]

**STANDARD Profile (PII + System Details):**
- All BASIC rules
- IP addresses → [REDACTED: network address]
- Usernames → [REDACTED: user account]
- File paths → [REDACTED: file path]
- Hostnames → [REDACTED: system]

**STRICT Profile (PII + System Details + Operational):**
- All STANDARD rules
- Command outputs → [REDACTED: command output]
- SQL queries → [REDACTED: query]
- API responses (JSON) → [REDACTED: api response]
- Sensitive config values → [REDACTED: configuration]

---

## Export Artifacts

**customer_packet.md** (Redacted for customer)
```
# Incident Timeline - Customer Summary

## Overview
- Total Events: N
- Timeline Duration: HH:MM:SS
- Severity Summary: X HIGH, Y MEDIUM, Z LOW

## Redacted Timeline
[Timestamp] [System] [Event Description with redactions applied]
<!-- CLAIM:C[event_id] ANCHOR:[anchor] -->
```

**internal_packet.md** (Full details for internal teams)
```
# Incident Timeline - Internal Analysis

## Overview
- Same as customer but with full text

## Timeline with Citations
[Timestamp] [System] [Full event text]
<!-- CLAIM:C[event_id] ANCHOR:[anchor] -->

## Redaction History
- At [timestamp] by [user]: [redaction reason]
```

**timeline.csv** (Deterministic event listing)
```
event_id,timestamp,system,actor,action,severity,anchor_id
INCIDENT_[id]_[hash]_0,2026-02-12T10:15:30Z,web-server,user@domain.com,login_attempt,LOW,INCIDENT_[id]_[hash]_0
...
```

---

## Integration with Phase 4 Patterns

✅ **Citation Enforcement** (from Phase 4)
- Every event has `<!-- CLAIM:C[event_id] -->` marker
- Validator checks presence in both templates
- RunManager enforces via CITATIONS.STRICT_ENFORCED_V1 gate

✅ **Deterministic Processing** (from Phase 4)
- Events sorted by timestamp (deterministic order)
- Timeline hash computed from sorted events
- Same log always produces same timeline hash

✅ **Non-Bypassable Export** (from Phase 4)
- All exports route through RunManager 16-step pipeline
- Redaction validation gate blocks export if profile violated
- Cannot generate bundles outside orchestrated flow

✅ **Audit Trail** (from Phase 2)
- All redactions logged as events
- Hash-chained audit trail proves no tampering
- Evidence that customer packet was properly redacted

---

## Testing Strategy

### Unit Tests (per module)
- parser: valid JSON, NDJSON, malformed input, edge cases
- timeline: event ordering, hash determinism, missing timestamps
- redaction: profile application, rule matching, map generation
- render: template structure, placeholders, citations

### Integration Tests
- Full workflow: log → timeline → dual-render → export
- Determinism: identical input → identical output hashes
- Redaction: customer profile applied correctly
- Citations: present in both templates

### Regression Tests
- Golden corpus: incident logs with expected timeline/templates
- Parity gate: timeline hash matches across platforms

---

## Success Criteria

✅ All Phase 4 patterns successfully applied to incident domain
✅ 20+ unit + integration tests passing
✅ Full end-to-end workflow deterministic
✅ Customer redaction profile enforced via gates
✅ Golden corpus regression testing in place
✅ Ready for Phase 6 (FinanceOS)

---

## Timeline

- **Step 1** (TODAY): Create parser, timeline, golden corpus
- **Step 2** (TODAY): Implement dual-template rendering + redaction
- **Step 3-4** (TODAY): Workflow orchestration
- **Step 5** (TODAY): RunManager integration and gates
- **Step 6-8** (TODAY): UI, tests, closure

**Target:** Complete all 8 steps in single session, matching Phase 4 pattern

---
