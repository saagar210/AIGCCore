use aigc_core::audit::log::AuditLog;
use aigc_core::run::manager::{RunManager, RunState};

#[test]
fn run_manager_starts_ready() {
    let dir = tempfile::tempdir().unwrap();
    let audit = AuditLog::open_or_create(dir.path().join("audit.ndjson")).unwrap();
    let mgr = RunManager::new(audit);
    assert_eq!(mgr.state, RunState::READY);
}
