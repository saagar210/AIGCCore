# Release Ceremony Signoff: v0.1.0

Date: 2026-03-01
Status: In Progress

## Scope

Formal release ceremony for `0.1.0` using the release workflow with verification, signing checks, and artifact checksum capture.

## Trigger

- Workflow: `release-desktop`
- Attempt 1 (master): `https://github.com/saagar210/AIGCCore/actions/runs/22545721368` (`failure`)
- Attempt 2 (fix branch): `https://github.com/saagar210/AIGCCore/actions/runs/22545911071` (`failure`)
- Trigger mode: `workflow_dispatch`
- Input version: `0.1.0`

## Attempt Notes

- Attempt 1 root cause: Windows wix packaging could not locate configured `.ico` icon.
  - remediation: add `icons/icon.ico` and sized icons in `src-tauri/tauri.conf.json`.
- Attempt 2 root cause: Windows checksum step attempted to hash output `SHA256SUMS.txt` while writing the same file.
  - remediation: update `.github/workflows/release-desktop.yml` to precompute file list and exclude `SHA256SUMS.txt`.
- Next action: rerun `release-desktop` after checksum-fix commit is pushed.

## Checklist

- [x] Metadata version alignment validated in workflow
- [x] Signing prerequisites enforced in workflow
- [ ] Multi-platform signed artifact jobs completed
- [ ] Artifact checksums captured in this signoff
- [ ] Distribution publication evidence captured
- [ ] Final signoff status set to `Complete`

## Distribution Evidence

- GitHub Release object: `Pending`
- External store upload evidence: `Unknown` (not yet recorded)
- Installer validation evidence: `Pending`

## Go/No-Go

Current decision: `Hold` until workflow completion artifacts are finalized.
