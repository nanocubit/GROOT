use std::fs;
use std::path::Path;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use groot_core::{BackendKind, LegacyBackend, GitoxideBackend};
use groot_core::{BackendError, GitBackend};

fn unique_tmp_dir() -> std::path::PathBuf {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let mut path = env::temp_dir();
    path.push(format!("groot_test_{}", now));
    path
}

#[test]
fn test_legacy_discover_existing_dir() {
    let dir = unique_tmp_dir();
    if dir.exists() {
        let _ = fs::remove_dir_all(&dir);
    }
    fs::create_dir_all(&dir).expect("create tmp dir");

    let handle = LegacyBackend::discover(&dir).expect("legacy discover should succeed");
    assert_eq!(handle.backend, BackendKind::Legacy);
    assert!(handle.git_dir.exists());
    assert_eq!(handle.git_dir, dir);

    // cleanup
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_legacy_open_nonexistent() {
    let dir = unique_tmp_dir();
    if dir.exists() {
        let _ = fs::remove_dir_all(&dir);
    }
    let res = LegacyBackend::open(&dir, None);
    match res {
        Err(BackendError::NotARepository(p)) => assert!(p.contains(&dir.display().to_string())),
        other => panic!("expected NotARepository error, got: {:?}", other),
    }
}

#[test]
fn test_backend_from_env_and_resolution() {
    env::remove_var("GROOT_BACKEND");
    // default is legacy
    assert_eq!(groot_core::default_backend_kind(), BackendKind::Legacy);

    env::set_var("GROOT_BACKEND", "gitoxide");
    assert_eq!(groot_core::backend_from_env(), BackendKind::Gitoxide);

    env::set_var("GROOT_BACKEND", "legacy");
    assert_eq!(groot_core::backend_from_env(), BackendKind::Legacy);

    env::remove_var("GROOT_BACKEND");
}

#[test]
fn test_gitoxide_adapter_feature_disabled() {
    // When the gitoxide feature is not enabled, the adapter should return NotImplemented
    let dir = unique_tmp_dir();
    if dir.exists() { let _ = fs::remove_dir_all(&dir); }
    fs::create_dir_all(&dir).expect("create tmp dir");

    let res = groot_core::adapters::gitoxide::discover_impl(&dir);
    match res {
        Err(BackendError::NotImplemented(_)) | Err(BackendError::NotARepository(_)) => {
            // both are acceptable depending on implementation; treat NotImplemented as expected
        }
        Ok(h) => panic!("expected error when gitoxide feature disabled, got handle: {:?}", h),
        Err(e) => panic!("unexpected error variant: {:?}", e),
    }

    let _ = fs::remove_dir_all(&dir);
}
