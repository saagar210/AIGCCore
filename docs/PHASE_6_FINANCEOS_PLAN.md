# Phase 6 Implementation Plan — FinanceOS Pack

**Date:** 2026-02-12
**Scope:** Financial statement analysis, exception detection, dual-template rendering, and policy enforcement
**Status:** STARTING

---

## Executive Summary

Phase 6 FinanceOS implements financial audit and compliance analysis with dual-template rendering (auditor-facing exceptions + internal policy compliance). It reuses Phase 4-5 patterns (citation enforcement, deterministic processing, non-bypassable export pipeline) and extends them with exception detection and accounting policy validation.

**Key Features:**
- Parse financial statements (JSON format with transaction records)
- Detect exceptions using rule-based analysis (threshold violations, duplicates, anomalies)
- Render dual templates: exceptions_audit.md (what auditors see) + compliance_internal.md (full policy details)
- Enforce accounting policy via gates
- Route exports through RunManager with policy validation gates

---

## Step-by-Step Implementation

### Step 1: Financial Statement Parsing
**Goal:** Parse financial artifacts and extract transaction records

**Files to create/modify:**
- `core/src/financeos/parser.rs` (NEW) — Parse financial statements
- `core/src/financeos/model.rs` (EXTEND) — Add transaction and statement models
- `core/corpus/financials/` (NEW) — Golden corpus for financial statements

**Scope:**
- Parse JSON financial statements with transaction arrays
- Extract transactions: date, amount, account, category, description
- Validate amounts and dates
- Generate deterministic transaction IDs
- Create statement summary (total, count, date range)

**Tests:**
- Parse valid financial statement structure
- Handle missing/malformed amounts
- Determinism: same statement yields same transaction IDs
- Date range calculation

### Step 2: Exception Detection Engine
**Goal:** Identify anomalies and policy violations

**Files to create/modify:**
- `core/src/financeos/exceptions.rs` (NEW) — Exception detection rules

**Scope:**
- Rule 1: Threshold violations (e.g., transactions > $10,000)
- Rule 2: Duplicate detection (same amount+account within 24 hours)
- Rule 3: Category anomalies (unusual transaction categories)
- Rule 4: Date gaps (missing expected recurring transactions)
- Rule 5: Round numbers (suspiciously round amounts)
- Generate exception IDs and severity (HIGH/MEDIUM/LOW)
- Track evidence and remediation

**Tests:**
- Threshold detection
- Duplicate identification
- Category validation
- Round number detection

### Step 3: Dual-Template Rendering
**Goal:** Render auditor and internal compliance views

**Files to create/modify:**
- `core/src/financeos/render.rs` (NEW) — Dual template rendering

**Scope:**
- Exceptions_audit.md: What auditors see (suspicious items only)
- Compliance_internal.md: Full policy analysis (all rules + compliance status)
- Both include citation markers
- Generate exception map (JSON)
- Generate policy map (JSON)

**Tests:**
- Audit template accuracy
- Compliance template completeness
- Citation marker presence
- Maps valid JSON

### Step 4: Workflow Orchestration
**Goal:** Execute complete analysis pipeline

**Files to create/modify:**
- `core/src/financeos/workflow.rs` (EXTEND) — Complete pipeline

**Scope:**
- State machine: Ingested → Parsed → Analyzed → Reviewed → ExportReady
- Full integration: parse → analyze → render
- Return FinanceWorkflowOutput

**Tests:**
- End-to-end workflow
- Determinism verification
- Citation enforcement
- Exception detection accuracy

### Step 5: RunManager Integration and Policy Gates
**Goal:** Route exports through 16-step pipeline with policy validation

**Files to create/modify:**
- `src-tauri/src/main.rs` (ADD) — run_financeos handler
- `tools/gates/check-financeos-policy.mjs` (NEW) — Policy enforcement gate

**Scope:**
- Tauri handler orchestration
- Gate validates: exceptions documented, compliance status clear
- Export blocked if policy violated

### Step 6-8: UI, Tests, and Closure
- Integration test suite (8+ tests)
- Golden corpus and regression testing
- Final closure report

---

## Data Models

```rust
FinancialStatement {
    statement_id: String,
    period_start: String,      // YYYY-MM-DD
    period_end: String,
    transactions: Vec<Transaction>,
    summary: StatementSummary,
}

Transaction {
    transaction_id: String,    // FINANCE_*_*
    date: String,              // YYYY-MM-DD
    amount: f64,
    account: String,
    category: String,
    description: String,
    is_flagged: bool,
}

Exception {
    exception_id: String,
    transaction_id: String,
    rule_triggered: String,    // THRESHOLD, DUPLICATE, ANOMALY, etc.
    severity: String,          // HIGH/MEDIUM/LOW
    description: String,
    recommended_action: String,
}

FinanceWorkflowOutput {
    exceptions_audit: String,      // For auditors
    compliance_internal: String,   // For internal teams
    exceptions_csv: String,
    transaction_count: usize,
    exception_count: usize,
    high_severity_count: usize,
}
```

---

## Exception Detection Rules

**Rule 1: THRESHOLD_VIOLATION (HIGH)**
- Trigger: Transaction amount > threshold (configurable, default $10,000)
- Severity: HIGH
- Action: "Manual approval required for large transactions"

**Rule 2: DUPLICATE_DETECTED (MEDIUM)**
- Trigger: Same amount + account within 24 hours
- Severity: MEDIUM
- Action: "Verify transaction not duplicate entry"

**Rule 3: CATEGORY_ANOMALY (MEDIUM)**
- Trigger: Transaction category unusual for account
- Severity: MEDIUM
- Action: "Confirm correct categorization"

**Rule 4: ROUND_NUMBER (LOW)**
- Trigger: Amount is suspiciously round ($5000, $10000, $50000)
- Severity: LOW
- Action: "Review for possible rounding or estimate"

**Rule 5: POLICY_VIOLATION (HIGH)**
- Trigger: Account/category violates retention policy
- Severity: HIGH
- Action: "Document or dispose per policy"

---

## Success Criteria

✅ All Phase 4-5 patterns successfully applied to financial domain
✅ 20+ unit + integration tests passing
✅ Full end-to-end workflow deterministic
✅ Exception detection working correctly
✅ Dual-template rendering accurate
✅ Golden corpus regression testing in place
✅ Ready for Phase 7 (HealthcareOS)

---

## Timeline

- **Steps 1-2** (NOW): Parser and exception detection
- **Step 3** (NOW): Dual-template rendering
- **Step 4** (NOW): Workflow orchestration
- **Steps 5-8** (NOW): RunManager integration, tests, closure

**Target:** Complete all 8 steps in single session

---
