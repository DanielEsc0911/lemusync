//! `doctor` failures carry a stable code and the redacted provider text only in `detail`;
//! a failed storage check skips the repo check instead of repeating it (Phase 2 Task 14).

mod common;

use common::Pc;
use lemusync_store::{ErrorClass, Fault, Faults};

#[tokio::test(start_paused = true)]
async fn a_failing_store_gives_one_coded_check_per_cause() {
    for (class, code) in [
        (ErrorClass::Auth, "backend_auth"),
        (ErrorClass::Transient, "backend_offline"),
        (ErrorClass::Quota, "backend_quota"),
        (ErrorClass::Permanent, "backend"),
    ] {
        let root = tempfile::tempdir().unwrap();
        let mut pc = Pc::new(root.path(), false, b"save");
        pc.engine
            .backend_add_fs(&root.path().join("remote"))
            .unwrap();
        pc.engine.init("pass phrase").await.unwrap();
        let mut pc = pc.reopen();
        let faults = Faults::default();
        faults.push(Fault::Fail(class), 1000);
        pc.engine.set_store_faults(Some(faults));
        let report = pc.engine.doctor().await.unwrap();
        let backend = report.checks.iter().find(|c| c.name == "backend").unwrap();
        assert!(!backend.ok, "{class:?}");
        assert_eq!(backend.code.as_deref(), Some(code), "{class:?}");
        assert!(backend.detail.contains("Unexpected"), "{}", backend.detail);
        assert!(!backend.detail.contains("FAKESIG"), "{}", backend.detail);
        let repo = report.checks.iter().find(|c| c.name == "repo").unwrap();
        assert_eq!(repo.code.as_deref(), Some("skipped"), "{class:?}");
        assert!(repo.detail.is_empty(), "no second dump: {}", repo.detail);
        let failed = report
            .checks
            .iter()
            .filter(|c| !c.ok && c.code.as_deref() != Some("skipped"))
            .count();
        assert_eq!(failed, 1, "one cause, one failure: {:?}", report.checks);
    }
}

#[tokio::test]
async fn a_healthy_store_has_no_codes() {
    let root = tempfile::tempdir().unwrap();
    let mut pc = Pc::new(root.path(), false, b"save");
    pc.engine
        .backend_add_fs(&root.path().join("remote"))
        .unwrap();
    pc.engine.init("pass phrase").await.unwrap();
    let report = pc.engine.doctor().await.unwrap();
    assert!(
        report.checks.iter().all(|c| c.code.is_none()),
        "{:?}",
        report.checks
    );
}
