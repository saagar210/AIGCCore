# Golden Corpus for AIGC Core Regression Tests

This directory contains test assets for validating extraction parity, determinism, and pack functionality across macOS and Windows.

## Structure

- **contracts/** — Contract PDFs for RedlineOS testing (3 types: digital, scanned, mixed)
- **expected_outputs/** — Known-good outputs for each contract (risk memos, clause maps, manifests)
- **incidents/** — Incident log files for IncidentOS testing (Phase 5)
- **finance/** — Finance documents (invoices, statements) for FinanceOS testing (Phase 6)
- **healthcare/** — Healthcare transcripts and consent for HealthcareOS testing (Phase 7)

## How to Validate

Run the full test suite:
```bash
cargo test --workspace
pnpm gate:all
```

This executes:
- `REDLINEOS.EXTRACTION_PARITY_V1` — Compares actual extraction to golden outputs
- `REDLINEOS.CLAUSE_MAPPING_DETERMINISTIC_V1` — Checks clause segmentation stability
- `REDLINEOS.ANCHOR_STABILITY_V1` — Validates deterministic anchor generation
- And others defined in `tools/gates/check-redlineos-parity.mjs`

## Contract Assets

### digital_sample.pdf
- **Type:** Native PDF (extractable text)
- **Pages:** 3
- **Expected Clauses:** 5
- **Expected Risks:** 2 HIGH, 1 MEDIUM
- **Source:** Real contract (sanitized)
- **Extraction Confidence:** >95%

### scanned_sample.pdf
- **Type:** Scanned image → OCR
- **Pages:** 2
- **Expected Clauses:** 3
- **Expected Risks:** 1 HIGH
- **Source:** Real contract scanned at 300 DPI
- **Extraction Confidence:** 80-90%

### mixed_sample.pdf
- **Type:** Mixed (first page native text, remaining pages scanned)
- **Pages:** 2
- **Expected Clauses:** 4
- **Source:** Real contract with mixed rendering

## Adding New Corpus Assets

1. Place artifact in appropriate subdirectory (`contracts/`, `incidents/`, etc.)
2. Run pack workflow against artifact
3. Save output to `expected_outputs/` with descriptive filename
4. Commit both artifact + expected output
5. Update this README with description
6. New gate should check parity automatically

## Regression Test Process

When a test fails due to corpus mismatch:

1. **Investigate:** Is the change intentional or a regression?
2. **If intentional:** Update expected outputs in `expected_outputs/`
3. **If regression:** Fix extraction/segmentation code
4. **Validate:** Confirm all tests pass locally on macOS and Windows
5. **Commit:** Include both code fix + corpus updates

## Data Privacy

All corpus assets are sanitized of PII and real company information.
They are purely for regression testing and can be shared internally.

---

**Last Updated:** 2026-02-12
**Corpus Version:** 1.0
