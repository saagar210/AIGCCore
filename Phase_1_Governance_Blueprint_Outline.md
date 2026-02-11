# Phase 1 — Governance Blueprint (Engineering Outline)

**Status:** Approved outline (implementation-ready specification).  
**Goal:** Convert the AI Governance Bible into enforceable mechanisms, schemas, and verification gates with **no ambiguity**.

---

## 1.1 Architecture overview (locked)

### Desktop platform
- **Shell:** Tauri
- **Core:** Rust (security-critical functions, deterministic exports, policy enforcement)
- **UI:** React + TypeScript
- **Storage:** SQLite + blob artifact store (file-backed)
- **Adapters:** Local-only runtime adapters for LLM/VLM/STT/Embeddings (HTTP on 127.0.0.1)

### Dataflow (canonical)
1) Ingest → 2) Normalize → 3) Classify → 4) Index → 5) Generate (pack) → 6) Verify/Eval → 7) Export Evidence Bundle

### Trust boundaries
- Vault storage is local and encrypted.
- Network is default-deny; Online Mode is explicit and allowlisted.
- Adapters bind to 127.0.0.1 only (no external listening).

---

## 1.2 Core module boundaries (no refactor later)

### A) Vault Manager
**Responsibilities**
- Create and manage vaults (workspace isolation)
- Store vault metadata: purpose, retention defaults, policy pack assignment, allowlist config
- Enforce separation: no cross-vault reads without explicit export/import

**Interfaces**
- `create_vault(name, purpose, policy_pack)`
- `set_policy_pack(vault_id, pack_id)`
- `export_vault_metadata(vault_id)`

### B) Artifact Store + Chain-of-custody
**Responsibilities**
- Ingest artifacts from disk
- Assign stable artifact IDs
- Compute SHA-256 and store metadata
- Maintain immutable record of original bytes (unless policy prevents retention)

**Key decisions**
- Artifact IDs are stable across runs within a vault
- Extracted/derived artifacts get new IDs with parent linkage

### C) Network Boundary
**Responsibilities**
- Enforce default-deny egress
- Enforce allowlist when Online Mode enabled
- Log all attempts (blocked or allowed)
- Expose real-time network state in UI

**Non-negotiables**
- No “hidden” dependency downloads at runtime.
- Any network access is user-enabled and logged.

### D) Policy Engine
**Responsibilities**
- Load policy packs (Strict/Balanced/Draft-only)
- Evaluate gates before run/export
- Provide policy snapshots to run manifests
- Drive enforcement for citations/redaction/retention/eval requirements

**Policy pack minimums**
- **Strict (Audit):** citations required, redaction required for Restricted, eval required, offline only by default
- **Balanced:** citations recommended, redaction recommended, eval recommended
- **Draft-only:** outputs marked draft; export gated; offline default

### E) Redaction Engine
**Responsibilities**
- Text masking with reversible mapping (stored locally in vault)
- Image blur/box redactions with recorded regions
- Export-time enforcement based on policy

### F) Citation Engine
**Responsibilities**
- Maintain canonical locator scheme:
  - PDFs: page + text span anchors (and optionally rendered bbox)
  - Audio: timestamp ranges
  - Images: bbox
- Enforce “no citation, no claim” in Strict mode
- Generate `citations_map.json` for each export

### G) Run Manager (Manifests + Evidence Bundle)
**Responsibilities**
- Create stable `run_id`
- Generate `run_manifest.json`
- Collect output hashes
- Package Evidence Bundle v1

### H) Model Router + Adapter Manager
**Responsibilities**
- Discover adapter capabilities
- Resolve models based on constraints
- Execute calls and record call metadata (hashes, durations, usage)
- Support “no-AI mode” fallback

### I) Evaluation Harness
**Responsibilities**
- Run required suites by policy pack
- Produce `eval_report.json`
- Provide clear failures linked to outputs and events

---

## 1.3 Evidence artifacts (schema lock-in)

**Evidence Bundle v1** and **Adapter Interface v1** are locked and versioned.

- Evidence Bundle v1: see `Annex_A_Evidence_Bundle_v1_Spec.md`
- Adapter Interface v1: see `Annex_B_Adapter_Interface_v1_Spec.md`

**Core rule:** No phase may change these formats without:
- a version bump,
- migration plan,
- backward compatible reader.

---

## 1.4 Model posture (approved, enforceable)

### Approved baseline model lanes
- **Text LLM:** BYO via adapter; default quality lane 7B–14B quant; fallback lane 3B.
- **Embeddings:** bge-m3 baseline (multilingual support).
- **VLM:** Qwen2-VL 7B class baseline (document/screenshot reasoning).
- **STT:** Voxtral + Whisper large-v3-turbo baseline (real-time + batch options).

### Model governance requirements (SHALL)
- Record model ID + adapter version in every run manifest.
- Support pinning via model snapshot.
- Block silent updates; upgrades require eval suite pass.

---

## 1.5 Cross-framework crosswalk (capability → evidence)

> This crosswalk is capability-based. It supports SOC2/ISO/NIST/FedRAMP review expectations and GDPR operational duties.

### Capability catalog (minimum)
1) Offline-by-default + allowlisted egress
2) Vault separation + encryption at rest
3) Artifact hashing + chain-of-custody
4) Tamper-evident audit logging
5) Run manifests + configuration snapshots
6) Citations + traceability
7) Redaction + sensitive handling
8) Retention + deletion controls
9) Model governance + version pinning
10) Secure SDLC + release signing + SBOM
11) Evaluation + regression testing (LLM threat tests)

### Mapping approach
- **SOC2 / ISO / NIST / FedRAMP:** map to security, logging, access control, configuration management, change management, incident response evidence.
- **GDPR:** map to minimization, retention/deletion, access/export support posture, processing transparency, security of processing.

**Deliverable in Phase 1:** a table mapping each capability to:
- relevant frameworks and themes
- product mechanisms
- emitted evidence artifacts
- verification tests (gate IDs)

---

## 1.6 Evaluation suite (aligned to OWASP LLM Top 10 + governance policies)

### Required categories (minimum)
- **CITATIONS:** strict citation enforcement; random sampling validator.
- **INJECTION:** prompt injection tests (logs/PDFs contain “ignore rules” strings).
- **SENSITIVE_DISCLOSURE:** PII/PHI/PCI leak tests; redaction enforcement.
- **SUPPLY_CHAIN:** model snapshot/pinning checks; adapter version pinning.
- **AGENCY:** ensure no external actions occur without confirmation; tool calls blocked in strict mode.
- **NETWORK:** default-deny egress verified; allowlist verified.
- **CONSUMPTION:** unbounded consumption safeguards (timeouts, file size caps, batch limits).

### Gate definitions (examples)
- `GATE_NETWORK_DEFAULT_DENY_PASS`
- `GATE_STRICT_CITATIONS_100_PASS`
- `GATE_REDACTION_REQUIRED_FOR_RESTRICTED_PASS`
- `GATE_MODEL_PIN_REQUIRED_PASS`
- `GATE_ADAPTER_CAPABILITIES_LOGGED_PASS`
- `GATE_BUNDLE_VALIDATION_PASS`

---

## 1.7 Cross-platform strategy (prevent Phase 4 surprises)

### Decision
- AIGC Core is built cross-platform capable from the start (Tauri helps).
- **RedlineOS Pack SHALL ship on macOS + Windows**.

### Requirements to lock now
- Evidence Bundle and citation locators must be OS-agnostic.
- PDF extraction/rendering must produce stable locators across OS.
- Golden corpus must include OS parity tests for PDFs.

---

## 1.8 Visual/UI specification (governance surfaces)

### Global UI surfaces (must exist in Phase 2)
1) **Network State Badge**
- OFF / ON (Allowlisted)
- click reveals: allowlist, last 20 egress attempts, blocked reasons

2) **Policy Center**
- policy pack selection and details
- toggles (citations required, redaction required, eval required)
- shows current policy version

3) **Runs**
- list of runs with PASS/WARN/FAIL gate status
- each run shows: run_id, policy pack, model snapshot, evidence bundle download

4) **Artifact Vault**
- artifact list, hashes, tags, preview, redaction status
- “export artifact list” button

5) **Eval Center**
- run suites, view failures, link to offending outputs

### Pack UI surfaces (Phase 3+)
- EvidenceOS: Control mapping, evidence index builder, export preview
- RedlineOS: Clause map, playbook editor, memo export preview
- Security: Timeline builder, two-template exports
- Finance: Exceptions dashboard, review queue, export wizard

---

## 1.9 Verification plan (phase gates)

### Phase 2 gate: Self-Audit Evidence Bundle
- Demonstrate offline default deny
- Demonstrate allowlist-only egress
- Demonstrate citations enforced in Strict mode
- Demonstrate redaction enforcement
- Demonstrate valid evidence bundle structure + hash verification
- Demonstrate eval suite run and report

### Phase 3 gate: EvidenceOS “10-minute demo”
- Import messy folder
- Produce evidence index + 10 narratives with citations
- Produce missing evidence checklist
- Export evidence bundle (valid and verifiable)

---

## 1.10 Risk register (built-in to prevent rewinds)

### Top risks and mitigations
- **PDF locator drift across OS** → golden corpus + stable render rules + parity tests.
- **Model behavior drift** → version pinning + regression suite + change gates.
- **Scope creep into full GRC** → pack boundaries; avoid program management features.
- **Performance on large vaults** → job queue + chunking + limits + consumption tests.
- **Security reviewer skepticism** → evidence bundle + audit log chain + clear offline UX.

---

## 1.11 External methodology anchors (reference list)
URLs stored in code blocks for easy reuse in later documents:

```text
NIST AI 600-1 (GenAI Profile): https://nvlpubs.nist.gov/nistpubs/ai/NIST.AI.600-1.pdf
OWASP LLM Top 10 (v1.1): https://owasp.org/www-project-top-10-for-large-language-model-applications/
ISO/IEC 42001:2023: https://www.iso.org/standard/42001
NIST SSDF 800-218 v1.1: https://csrc.nist.gov/pubs/sp/800/218/final
NIST SSDF 800-218r1 (v1.2) ipd: https://csrc.nist.gov/pubs/sp/800/218/r1/ipd
FedRAMP AI prioritization: https://www.fedramp.gov/ai/
FedRAMP program overview (20x): https://www.fedramp.gov/
```

---

## 1.12 Acceptance criteria for Phase 1
Phase 1 is complete when:
- All core modules have explicit responsibilities and interfaces.
- Evidence Bundle v1 and Adapter Interface v1 are referenced as locked.
- Evaluation suite categories and gates are specified.
- Cross-platform parity requirements for Phase 4 are defined.
- Verification gates for Phase 2 and Phase 3 are explicit and testable.
