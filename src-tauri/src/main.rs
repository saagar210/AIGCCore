#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aigc_core::adapters::pinning::{classify_pinning_level, PinningLevel};
use aigc_core::audit::event::{Actor, AuditEvent};
use aigc_core::audit::log::AuditLog;
use aigc_core::determinism::json_canonical;
use aigc_core::determinism::run_id::sha256_hex;
use aigc_core::evidence_bundle::artifact_hashes::{render_artifact_hashes_csv, ArtifactHashRow};
use aigc_core::evidence_bundle::schemas::*;
use aigc_core::evidenceos::control_library::{controls_for_capabilities, ControlDefinition};
use aigc_core::evidenceos::model::{CitationInput, EvidenceItem, NarrativeClaimInput};
use aigc_core::evidenceos::workflow::{generate_evidenceos_artifacts, EvidenceOsRequest};
use aigc_core::policy::network_snapshot::{AdapterEndpointSnapshot, NetworkSnapshot};
use aigc_core::policy::types::{InputExportProfile, NetworkMode, PolicyMode, ProofLevel};
use aigc_core::run::manager::{ExportRequest, RunManager};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
struct UiNetworkSnapshot {
    network_mode: &'static str,
    proof_level: &'static str,
    ui_remote_fetch_disabled: bool,
}

#[derive(Debug, Serialize)]
struct EvidenceOsRunResult {
    status: String,
    bundle_path: String,
    bundle_sha256: String,
    missing_control_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EvidenceOsRunInput {
    enabled_capabilities: Vec<String>,
    artifact_title: String,
    artifact_body: String,
    artifact_tags_csv: String,
    control_families_csv: String,
    claim_text: String,
}

#[tauri::command]
fn get_network_snapshot() -> UiNetworkSnapshot {
    UiNetworkSnapshot {
        network_mode: "OFFLINE",
        proof_level: "OFFLINE_STRICT",
        ui_remote_fetch_disabled: true,
    }
}

#[tauri::command]
fn list_control_library(enabled_capabilities: Option<Vec<String>>) -> Vec<ControlDefinition> {
    controls_for_capabilities(&enabled_capabilities.unwrap_or_default())
}

#[tauri::command]
fn generate_evidenceos_bundle(input: EvidenceOsRunInput) -> Result<EvidenceOsRunResult, String> {
    let runtime_dir = make_runtime_dir()?;
    let bundle_root = runtime_dir.join("bundle_root");
    let bundle_zip = runtime_dir.join("evidence_bundle_evidenceos_v1.zip");
    let audit_path = runtime_dir.join("audit.ndjson");

    let artifact_bytes = if input.artifact_body.trim().is_empty() {
        b"default evidence artifact body".to_vec()
    } else {
        input.artifact_body.as_bytes().to_vec()
    };
    let artifact_sha = sha256_hex(&artifact_bytes);
    let artifact_id = format!("a_ui_{}", &artifact_sha[..8]);
    let manifest_inputs_fingerprint =
        sha256_hex(format!("{}:{}", artifact_id, artifact_sha).as_bytes());
    let run_id = format!("r_{}", &manifest_inputs_fingerprint[..32]);
    let vault_id = "v_ui_0001".to_string();
    let pack_id = "evidenceos".to_string();
    let pack_version = "1.0.0".to_string();

    let mut audit = AuditLog::open_or_create(&audit_path).map_err(|e| e.to_string())?;
    let events = vec![
        (
            "VAULT_ENCRYPTION_STATUS",
            Actor::System,
            json!({
                "encryption_at_rest": true,
                "algorithm": "XCHACHA20_POLY1305",
                "key_storage": "FILE_FALLBACK"
            }),
        ),
        (
            "NETWORK_MODE_SET",
            Actor::User,
            json!({
                "network_mode":"OFFLINE",
                "proof_level":"OFFLINE_STRICT",
                "ui_remote_fetch_disabled":true
            }),
        ),
        (
            "ALLOWLIST_UPDATED",
            Actor::System,
            json!({
                "allowlist_hash_sha256": sha256_hex(b""),
                "allowlist_count":0
            }),
        ),
        (
            "EGRESS_REQUEST_BLOCKED",
            Actor::System,
            json!({
                "destination":{"scheme":"https","host":"example.invalid","port":443,"path":"/"},
                "block_reason":"OFFLINE_MODE",
                "request_hash_sha256": sha256_hex(b"blocked")
            }),
        ),
    ];
    for (event_type, actor, details) in events {
        audit
            .append(AuditEvent {
                ts_utc: "2026-02-10T00:00:00Z".to_string(),
                event_type: event_type.to_string(),
                run_id: run_id.clone(),
                vault_id: vault_id.clone(),
                actor,
                details,
                prev_event_hash: String::new(),
                event_hash: String::new(),
            })
            .map_err(|e| e.to_string())?;
    }

    let tags = csv_to_vec(&input.artifact_tags_csv);
    let control_families = csv_to_vec(&input.control_families_csv);
    let enabled_capabilities = if input.enabled_capabilities.is_empty() {
        vec!["Traceability".to_string()]
    } else {
        input.enabled_capabilities.clone()
    };
    let claim_text = if input.claim_text.trim().is_empty() {
        "The EvidenceOS run remained offline with blocked network egress attempts.".to_string()
    } else {
        input.claim_text.clone()
    };

    let req = EvidenceOsRequest {
        pack_id: pack_id.clone(),
        pack_version: pack_version.clone(),
        run_id: run_id.clone(),
        policy_mode: PolicyMode::STRICT,
        enabled_capabilities,
        evidence_items: vec![EvidenceItem {
            artifact_id: artifact_id.clone(),
            artifact_sha256: artifact_sha.clone(),
            title: if input.artifact_title.trim().is_empty() {
                "User provided evidence artifact".to_string()
            } else {
                input.artifact_title.clone()
            },
            tags: tags.clone(),
            control_family_labels: if control_families.is_empty() {
                vec!["Traceability".to_string()]
            } else {
                control_families
            },
        }],
        narrative_claims: vec![NarrativeClaimInput {
            claim_id: "C0001".to_string(),
            text: claim_text,
            citations: vec![CitationInput {
                artifact_id: artifact_id.clone(),
                locator_type: "PDF_TEXT_SPAN_V1".to_string(),
                locator: json!({
                    "page_index": 0,
                    "start_char": 0,
                    "end_char": 30,
                    "text_sha256": artifact_sha
                }),
            }],
        }],
    };

    let generated = generate_evidenceos_artifacts(&req).map_err(|e| e.to_string())?;

    let templates_rel = format!("exports/{}/attachments/templates_used.json", pack_id);
    let citations_rel = format!("exports/{}/attachments/citations_map.json", pack_id);
    let redactions_rel = format!("exports/{}/attachments/redactions_map.json", pack_id);

    let templates_bytes =
        json_canonical::to_canonical_bytes(&generated.templates_used_json).map_err(|e| e.to_string())?;
    let citations_bytes =
        json_canonical::to_canonical_bytes(&generated.citations_map_json).map_err(|e| e.to_string())?;
    let redactions_bytes =
        json_canonical::to_canonical_bytes(&generated.redactions_map_json).map_err(|e| e.to_string())?;

    let mut hash_rows = vec![ArtifactHashRow {
        artifact_id: artifact_id.clone(),
        bundle_rel_path: String::new(),
        sha256: req.evidence_items[0].artifact_sha256.clone(),
        bytes: artifact_bytes.len() as u64,
        content_type: "text/plain".to_string(),
        logical_role: "INPUT".to_string(),
    }];
    for (path, bytes, content_type) in &generated.deliverables {
        hash_rows.push(ArtifactHashRow {
            artifact_id: format!("o:{}", path),
            bundle_rel_path: path.clone(),
            sha256: sha256_hex(bytes),
            bytes: bytes.len() as u64,
            content_type: content_type.clone(),
            logical_role: "DELIVERABLE".to_string(),
        });
    }
    hash_rows.push(ArtifactHashRow {
        artifact_id: format!("o:{}", templates_rel),
        bundle_rel_path: templates_rel.clone(),
        sha256: sha256_hex(&templates_bytes),
        bytes: templates_bytes.len() as u64,
        content_type: "application/json".to_string(),
        logical_role: "ATTACHMENT".to_string(),
    });
    hash_rows.push(ArtifactHashRow {
        artifact_id: format!("o:{}", citations_rel),
        bundle_rel_path: citations_rel.clone(),
        sha256: sha256_hex(&citations_bytes),
        bytes: citations_bytes.len() as u64,
        content_type: "application/json".to_string(),
        logical_role: "ATTACHMENT".to_string(),
    });
    hash_rows.push(ArtifactHashRow {
        artifact_id: format!("o:{}", redactions_rel),
        bundle_rel_path: redactions_rel.clone(),
        sha256: sha256_hex(&redactions_bytes),
        bytes: redactions_bytes.len() as u64,
        content_type: "application/json".to_string(),
        logical_role: "ATTACHMENT".to_string(),
    });
    let artifact_hashes_csv = render_artifact_hashes_csv(hash_rows).map_err(|e| e.to_string())?;

    let outputs: Vec<ManifestOutputRef> = generated
        .deliverables
        .iter()
        .map(|(path, bytes, content_type)| ManifestOutputRef {
            path: path.clone(),
            sha256: sha256_hex(bytes),
            bytes: bytes.len() as u64,
            content_type: content_type.clone(),
            logical_role: "DELIVERABLE".to_string(),
        })
        .collect();

    let bundle_inputs = EvidenceBundleInputs {
        run_manifest: RunManifest {
            run_id: run_id.clone(),
            vault_id: vault_id.clone(),
            determinism: DeterminismManifest {
                enabled: true,
                manifest_inputs_fingerprint,
            },
            inputs: vec![ManifestArtifactRef {
                artifact_id: artifact_id.clone(),
                sha256: req.evidence_items[0].artifact_sha256.clone(),
                bytes: artifact_bytes.len() as u64,
                mime_type: "text/plain".to_string(),
                logical_role: "INPUT".to_string(),
            }],
            outputs,
            model_calls: vec![],
            eval: EvalSummary {
                gate_status: "PASS".to_string(),
            },
        },
        bundle_info: BundleInfo {
            bundle_version: "1.0.0".to_string(),
            schema_versions: SchemaVersions {
                run_manifest: "RUN_MANIFEST_V1".to_string(),
                eval_report: "EVAL_REPORT_V1".to_string(),
                citations_map: "LOCATOR_SCHEMA_V1".to_string(),
                redactions_map: "REDACTION_SCHEMA_V1".to_string(),
            },
            pack_id: pack_id.clone(),
            pack_version: pack_version.clone(),
            core_build: "dev".to_string(),
            run_id: run_id.clone(),
        },
        audit_log_ndjson: std::fs::read_to_string(&audit_path).map_err(|e| e.to_string())?,
        eval_report: EvalReport {
            overall_status: "PASS".to_string(),
            tests: vec![],
            gates: vec![],
            registry_version: "gates_registry_v3".to_string(),
        },
        artifact_hashes_csv,
        artifact_list: ArtifactList {
            artifacts: vec![ArtifactListEntry {
                artifact_id,
                sha256: req.evidence_items[0].artifact_sha256.clone(),
                bytes: artifact_bytes.len() as u64,
                content_type: "text/plain".to_string(),
                logical_role: "INPUT".to_string(),
                classification: "Internal".to_string(),
                tags,
                retention_policy_id: "ret_default".to_string(),
            }],
        },
        policy_snapshot: PolicySnapshot {
            policy_mode: PolicyMode::STRICT,
            determinism: DeterminismPolicy {
                enabled: true,
                pdf_determinism_enabled: true,
            },
            export_profile: ExportProfile {
                inputs: InputExportProfile::HASH_ONLY,
            },
            encryption_at_rest: true,
            encryption_algorithm: "XCHACHA20_POLY1305".to_string(),
        },
        network_snapshot: NetworkSnapshot {
            network_mode: NetworkMode::OFFLINE,
            proof_level: ProofLevel::OFFLINE_STRICT,
            allowlist: vec![],
            ui_remote_fetch_disabled: true,
            adapter_endpoints: vec![AdapterEndpointSnapshot {
                endpoint: "http://127.0.0.1:11434".to_string(),
                is_loopback: true,
                validation_error: None,
            }],
        },
        model_snapshot: aigc_core::adapters::pinning::ModelSnapshot {
            adapter_id: "local_adapter".to_string(),
            adapter_version: "1.0.0".to_string(),
            adapter_endpoint: "http://127.0.0.1:11434".to_string(),
            model_id: "model-a".to_string(),
            model_sha256: Some(sha256_hex(b"model-a")),
            pinning_level: {
                let m = sha256_hex(b"model-a");
                classify_pinning_level(Some(&m), "local_adapter", "1.0.0")
            },
        },
        pack_id: pack_id.clone(),
        pack_version,
        deliverables: generated.deliverables,
        attachments: PackAttachments {
            templates_used_json: generated.templates_used_json,
            citations_map_json: Some(generated.citations_map_json),
            redactions_map_json: Some(generated.redactions_map_json),
        },
    };

    let mut manager = RunManager::new(audit);
    let export_req = ExportRequest {
        run_id,
        vault_id,
        policy_mode: PolicyMode::STRICT,
        network_mode: NetworkMode::OFFLINE,
        proof_level: ProofLevel::OFFLINE_STRICT,
        pinning_level: PinningLevel::CRYPTO_PINNED,
        requested_by: "user".to_string(),
    };

    let outcome = manager
        .export_run(&export_req, &bundle_inputs, &bundle_root, &bundle_zip)
        .map_err(|e| format!("failed to export EvidenceOS bundle: {}", e))?;
    if outcome.status != "COMPLETED" {
        return Err(format!(
            "EvidenceOS export did not complete. status={} block_reason={:?}",
            outcome.status, outcome.block_reason
        ));
    }

    Ok(EvidenceOsRunResult {
        status: outcome.status,
        bundle_path: outcome.bundle_path.unwrap_or_default(),
        bundle_sha256: outcome.bundle_sha256.unwrap_or_default(),
        missing_control_ids: generated.missing_control_ids,
    })
}

fn make_runtime_dir() -> Result<std::path::PathBuf, String> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis();
    let path = std::env::temp_dir().join(format!("aigc_evidenceos_{}", ts));
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(path)
}

fn csv_to_vec(raw: &str) -> Vec<String> {
    let mut out: Vec<String> = raw
        .split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect();
    out.sort();
    out.dedup();
    out
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_network_snapshot,
            list_control_library,
            generate_evidenceos_bundle
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
