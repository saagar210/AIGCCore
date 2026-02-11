# Phase 0 — AI Governance Bible (Blueprint)

**Status:** Approved blueprint (to be turned into the formal “Bible” document).  
**Scope:** Applies to **AIGC Core** and every Pack (EvidenceOS, RedlineOS, Security Packetizer, Finance Exceptions, Healthcare Drafting).  
**Audience:** Prospects, customers, auditors, security reviewers, internal engineering and product teams.  
**Non‑goal:** This document is not a certification claim (e.g., “FedRAMP authorized” or “GDPR compliant”). It is a **design + operating policy** with **verifiable evidence outputs** that help customers meet requirements.

---

## 0.1 Document conventions

### Policy strength words
- **SHALL** = mandatory, enforced by product behavior or process gate.
- **SHOULD** = strongly recommended, enforced where feasible; deviations documented.
- **MAY** = optional.

### Definitions (canonical vocabulary)
- **Vault:** A logically isolated workspace containing artifacts, runs, policies, and exports.
- **Artifact:** Any input or intermediate object ingested/produced (PDF, image, audio, CSV, extracted text, embeddings).
- **Run:** A single execution unit that consumes artifacts under a policy and produces outputs + evidence.
- **Evidence Bundle:** Versioned export package that proves what happened in a run.
- **Policy Pack:** A named, versioned set of enforcement rules (e.g., Strict / Balanced / Draft‑only).
- **Adapter:** A runtime connector for LLM/VLM/STT/Embeddings execution (local‑only by default).
- **Egress:** Any attempt to access network resources outside the local host.

---

## 0.2 Governance principles (public commitments)

### P0‑01: Privacy by default / local-first execution
- The system **SHALL** default to **offline mode** (no egress).
- The system **SHALL** support explicit **Online Mode** gated by:
  - user acknowledgement,
  - allowlist configuration,
  - egress logging and reporting.

### P0‑02: Least data / purpose limitation
- The system **SHALL** minimize stored data to what is necessary for the selected workflow.
- The system **SHALL** require Vault purpose metadata (e.g., “SOC2 evidence collection”) to guide retention defaults.

### P0‑03: Traceability and auditability
- The system **SHALL** produce **tamper-evident audit logs**.
- The system **SHALL** produce a per‑Run **Run Manifest** including:
  - input hashes,
  - policy snapshots,
  - model snapshots,
  - output hashes,
  - evaluation results.

### P0‑04: Citation-grounded outputs for high-stakes use
- Under Strict policy, the system **SHALL** enforce “**no citation, no claim**” for generated narrative outputs.
- Citations **SHALL** reference the source artifact with stable locators (page/line span; timestamp ranges; bounding boxes).

### P0‑05: Human oversight (no silent automation)
- The system **SHALL** require explicit user confirmation for:
  - exporting final deliverables,
  - enabling Online Mode,
  - applying irreversible deletion,
  - any tool/action that changes external systems (future integrations).

### P0‑06: Model governance, reproducibility, and updates
- The system **SHALL** record **model identity** and **configuration** used for every Run.
- The system **SHALL** support **version pinning** for models and adapters.
- The system **SHALL NOT** silently update models or adapters affecting output behavior.
- Updates **SHALL** be accompanied by:
  - evaluation suite results,
  - a change log entry,
  - a version bump.

### P0‑07: Security-by-design and secure development lifecycle
- The project **SHALL** follow a secure development lifecycle aligned to SSDF practices:
  - secure design review,
  - dependency hygiene,
  - release signing,
  - vulnerability response.

### P0‑08: Regulatory awareness (FedRAMP + GDPR included)
- The system **SHALL** provide evidence artifacts and controls aligned to common security/compliance expectations, including:
  - **NIST control family patterns** (supporting FedRAMP‑style reviews),
  - **GDPR data handling** (retention, deletion, export, minimization),
  - and other crosswalked frameworks (SOC2/ISO/NIST).

---

## 0.3 Roles and responsibilities (RACI)

### Required roles
- **Governance Owner (GO):** accountable for Bible content, approval workflow, and exception policy.
- **Security Owner (SO):** accountable for threat model, security controls, crypto/key handling decisions.
- **Release Manager (RM):** accountable for release gates, signing/notarization, SBOM, bundle schema versioning.
- **Pack Owner (PO):** accountable for pack‑specific workflow correctness and citations/exports quality.
- **Evaluation Owner (EO):** accountable for eval suite, regressions, and policy gates.

### Decision log and exceptions
- All policy exceptions **SHALL** be documented in a Decision Log (Appendix C).
- Exceptions **SHALL** include:
  - scope,
  - risk rationale,
  - compensating controls,
  - expiration date,
  - approval signatures (GO + SO).

---

## 0.4 Policy domains (the “Bible chapters”)

### Chapter 1 — Data governance
- Data classification levels (Public/Internal/Confidential/Restricted) + tags (PII/PHI/PCI).
- Retention defaults per classification.
- Secure deletion workflow and verification.
- Data export and portability (“right to access” support posture).

### Chapter 2 — Security controls
- Offline default + Online Mode gating and allowlist.
- Encryption at rest for Vault storage and sensitive caches.
- Secrets handling and local key management.
- Logging and monitoring (local audit logs; no cloud telemetry by default).

### Chapter 3 — Model governance
- Approved adapter types and local-only binding rules (127.0.0.1).
- Model selection policy and fallback tiers:
  - Text LLM (BYO) with 7B–14B default lane and 3B fallback lane.
  - Embeddings model (approved baseline: bge‑m3).
  - VLM model (approved baseline: Qwen2‑VL 7B class).
  - STT models (approved baseline: Voxtral + Whisper large‑v3‑turbo).
- Version pinning requirements and update workflow.
- Prompt handling and storage policy (default: do not persist raw prompts; allow opt‑in).

### Chapter 4 — Responsible AI and risk controls
- Prohibited and restricted use cases (customer-specific policy profiles).
- Prompt injection and data exfiltration controls.
- Sensitive disclosure controls (PII/PHI/PCI detection + redaction).
- Excessive agency prevention (no autonomous external actions).

### Chapter 5 — Quality and evaluation
- Required eval categories and pass/fail gates.
- Regression rules (model change requires baseline re-run).
- Pack-specific goldens and acceptance thresholds.

### Chapter 6 — Transparency and evidence
- Evidence Bundle v1 required contents.
- Manifest and log semantics.
- Determinism rules (deliverables avoid volatile timestamps).

---

## 0.5 Traceability Matrix (skeleton)

> This matrix is the binding contract between the Bible and the product. Every SHALL requires:
> - an enforcement mechanism,
> - a proof artifact,
> - a verification test.

| Bible Policy ID | Requirement (SHALL) | Mechanism (Core/Pack/Process) | Evidence Artifact | Verification Test |
|---|---|---|---|---|
| P0‑01 | Offline by default | Core: Network boundary | audit_log + network_snapshot | `NETWORK_DEFAULT_DENY` |
| P0‑03 | Tamper-evident logs | Core: hash-chained NDJSON | audit_log.ndjson | `LOG_CHAIN_VERIFIES` |
| P0‑04 | No citation, no claim | Core: citation enforcement | citations_map.json | `STRICT_CITATION_100` |
| P0‑06 | No silent model updates | Process: release gate | model_snapshot + changelog | `MODEL_PIN_REQUIRED` |
| P0‑08 | GDPR retention support | Core: retention engine | policy_snapshot + deletion log | `RETENTION_ENFORCED` |

---

## 0.6 Appendix strategy (keeps the Bible stable in a volatile 2026)

### Appendix A — Regulatory + standards notes (updateable)
- GDPR notes: retention, deletion, portability, minimization.
- FedRAMP notes: NIST-control-family evidence expectations, artifact discipline, package artifacts.
- EU AI Act notes (high-level risk posture).
- Update cadence: monthly review; emergency updates on major changes.

### Appendix B — Model landscape notes (updateable)
- Approved baseline models (IDs, hashes, quantization lanes).
- Deprecation policy and migration plan.
- Performance/quality tradeoffs by lane.

### Appendix C — Decision log and exceptions
- Exception register entries (see section 0.3).

### Appendix D — Change management
- Versioning policy for the Bible and the Evidence Bundle schema.
- Required approvals (GO + SO + RM).
- Public change summary guidelines (customer-facing).

---

## 0.7 “What we do NOT do” (hard boundaries)
- No autonomous external actions without explicit user confirmation.
- No silent network activity; no hidden cloud telemetry.
- No compliance/certification claims.
- No legal/medical/financial “final decision” outputs; outputs are drafts requiring review in strict modes.

---

## 0.8 External methodology anchors (reference list)
To keep this blueprint concise, the formal Bible will cite authoritative anchors. Store URLs in code blocks:

```text
NIST AI 600-1 (GenAI Profile): https://nvlpubs.nist.gov/nistpubs/ai/NIST.AI.600-1.pdf
OWASP Top 10 for LLM Apps (v1.1): https://owasp.org/www-project-top-10-for-large-language-model-applications/
ISO/IEC 42001:2023: https://www.iso.org/standard/42001
NIST SSDF 800-218 v1.1: https://csrc.nist.gov/pubs/sp/800/218/final
NIST SSDF 800-218r1 (v1.2) ipd: https://csrc.nist.gov/pubs/sp/800/218/r1/ipd
FedRAMP AI prioritization: https://www.fedramp.gov/ai/
FedRAMP 20x / automation: https://www.fedramp.gov/
```

---

## 0.9 Acceptance criteria for Phase 0
Phase 0 is complete when:
- Bible ToC and policy statements are finalized at **SHALL/SHOULD** level.
- Traceability matrix covers all critical SHALL statements.
- Appendix structure is finalized and update cadence is defined.
- Terminology is consistent across Bible + specs.
