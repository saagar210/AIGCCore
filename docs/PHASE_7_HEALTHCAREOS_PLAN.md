# Phase 7 Implementation Plan — HealthcareOS Pack

**Date:** 2026-02-12
**Scope:** Clinical transcript analysis, consent validation, draft note generation, verification checklist
**Status:** STARTING (FINAL PHASE)

---

## Executive Summary

Phase 7 HealthcareOS implements clinical documentation generation with consent enforcement and verification workflows. It reuses Phase 4-6 patterns and adds healthcare-specific consent validation and verification checkpoint enforcement.

**Key Features:**
- Parse clinical transcripts and consent records
- Validate patient consent before processing
- Generate draft clinical note with citations
- Create verification checklist for human review
- Enforce strict export block if consent/verification missing
- Track uncertainty/redaction requirements per HIPAA
- Route exports through RunManager with consent validation gates

---

## Step-by-Step Implementation

### Step 1: Transcript & Consent Parsing
**Goal:** Parse healthcare artifacts safely

**Files to create/modify:**
- `core/src/healthcareos/parser.rs` (NEW) — Parse transcripts and consent
- `core/src/healthcareos/model.rs` (EXTEND) — Add clinical data models
- `core/corpus/clinical/` (NEW) — Golden corpus

**Scope:**
- Parse JSON clinical transcripts with speaker/timestamp/content
- Extract consent records with patient/provider/date
- Validate timestamps and consent dates
- Generate deterministic clinical note IDs
- Validate patient consent before proceeding

### Step 2: Consent Validation Engine
**Goal:** Enforce consent constraints

**Files to create/modify:**
- `core/src/healthcareos/consent.rs` (NEW) — Consent validation

**Scope:**
- VALID: Consent is present and current
- EXPIRED: Consent is older than 2 years
- MISSING: No consent record found
- REVOKED: Patient revoked consent
- Block processing if MISSING or REVOKED
- Allow with warnings if EXPIRED

### Step 3: Draft Note Generation
**Goal:** Create clinical documentation with citations and uncertainty tracking

**Files to create/modify:**
- `core/src/healthcareos/render.rs` (EXTEND) — Draft generation

**Scope:**
- `render_draft_note()` — Clinical documentation with citations
- `render_verification_checklist()` — Human review checkpoints
- `render_uncertainty_map()` — Medical uncertainty tracking
- All include citation markers for audit trail

### Step 4: Workflow Orchestration
**Goal:** Execute complete healthcare pipeline

**Files to create/modify:**
- `core/src/healthcareos/workflow.rs` (EXTEND) — Complete orchestration

**Scope:**
- State machine: Ingested → Consent → Analyzed → Renderable → ExportReady
- Full integration: parse → validate consent → generate draft → render checklist
- Strict export block if consent invalid

### Step 5-8: Integration, Tests, Corpus, Closure
- RunManager integration (Tauri handler)
- 8+ integration tests
- Golden corpus (sample transcript + consent)
- Final closure report

---

## Data Models

```rust
ClinicalTranscript {
    transcript_id: String,
    patient_id: String,
    date: String,
    provider: String,
    specialty: String,
    content: String,              // Full transcript text
    confidence: f32,              // Speech recognition confidence
}

ConsentRecord {
    consent_id: String,
    patient_id: String,
    date: String,                 // When consent was given
    expires: String,              // 2-year expiry from date
    scope: String,                // "general", "research", "limited"
    status: String,               // VALID, EXPIRED, REVOKED
}

DraftNote {
    note_id: String,
    patient_id: String,
    date: String,
    content: String,              // Draft with citations
    uncertainty_count: usize,     // Segments marked uncertain
}

VerificationCheckpoint {
    checkpoint_id: String,
    section: String,              // "demographics", "chief_complaint", "assessment"
    required_review: bool,
    uncertainty_flagged: bool,
}
```

---

## Consent Validation Rules

**VALID**
- Consent exists AND date is within 2 years
- Action: Proceed with normal processing
- Citations enforced in draft

**EXPIRED**
- Consent exists but older than 2 years
- Action: Proceed with warning, flag for renewal
- Draft marked: "Note generated from expired consent"

**MISSING**
- No consent record found
- Action: BLOCK export, return error
- Cannot proceed under any circumstances

**REVOKED**
- Consent exists but marked revoked
- Action: BLOCK export, return error
- Cannot proceed under any circumstances

---

## Success Criteria

✅ All Phase 4-6 patterns successfully applied to healthcare domain
✅ 20+ unit + integration tests passing
✅ Full end-to-end workflow deterministic
✅ Consent validation enforced non-bypassably
✅ Verification checklist generation accurate
✅ Golden corpus regression testing in place
✅ Ready for production deployment

---

## Timeline

- **Steps 1-4** (NOW): Parser, consent, render, workflow
- **Steps 5-8** (NOW): Integration, tests, corpus, closure

**Target:** Complete all 8 steps in single session, matching Phase 4-6 pattern

**Overall:** 4 complete, production-ready domain packs:
- Phase 4: Contract analysis (RedlineOS)
- Phase 5: Incident analysis (IncidentOS)
- Phase 6: Financial analysis (FinanceOS)
- Phase 7: Clinical analysis (HealthcareOS)

---
