use aigc_core::evidence_bundle::artifact_hashes::{render_artifact_hashes_csv, ArtifactHashRow};

#[test]
fn artifact_hashes_csv_is_sorted_by_id_then_path() {
    let csv = render_artifact_hashes_csv(vec![
        ArtifactHashRow {
            artifact_id: "o:exports/p/deliverables/b.md".to_string(),
            bundle_rel_path: "exports/p/deliverables/b.md".to_string(),
            sha256: "b".repeat(64),
            bytes: 2,
            content_type: "text/markdown".to_string(),
            logical_role: "DELIVERABLE".to_string(),
        },
        ArtifactHashRow {
            artifact_id: "a_1".to_string(),
            bundle_rel_path: "".to_string(),
            sha256: "a".repeat(64),
            bytes: 1,
            content_type: "text/plain".to_string(),
            logical_role: "INPUT".to_string(),
        },
        ArtifactHashRow {
            artifact_id: "o:exports/p/attachments/templates_used.json".to_string(),
            bundle_rel_path: "exports/p/attachments/templates_used.json".to_string(),
            sha256: "c".repeat(64),
            bytes: 3,
            content_type: "application/json".to_string(),
            logical_role: "ATTACHMENT".to_string(),
        },
    ])
    .unwrap();

    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(
        lines[0],
        "artifact_id,bundle_rel_path,sha256,bytes,content_type,logical_role"
    );
    assert!(lines[1].starts_with("a_1,"));
    assert!(lines[2].starts_with("o:exports/p/attachments/templates_used.json,"));
    assert!(lines[3].starts_with("o:exports/p/deliverables/b.md,"));
}
