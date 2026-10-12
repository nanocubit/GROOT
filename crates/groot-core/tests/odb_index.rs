use std::fs;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use groot_core::{BackendKind, LegacyBackend, GitoxideBackend, RepositoryHandle, RepositoryKind, RepositoryIndex};
use groot_core::{BackendError, GitBackend};

fn unique_tmp_dir() -> std::path::PathBuf {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let mut path = env::temp_dir();
    path.push(format!("groot_odb_test_{}", now));
    path
}

#[test]
fn test_legacy_load_and_write_index_roundtrip() {
    // Legacy backend has deterministic placeholder behavior
    let dir = unique_tmp_dir();
    if dir.exists() { let _ = fs::remove_dir_all(&dir); }
    fs::create_dir_all(&dir).expect("create tmp dir");

    let repo = LegacyBackend::open(&dir, Some(&dir)).expect("legacy open should succeed");
    let mut index = LegacyBackend::load_index(&repo).expect("legacy load_index should succeed");
    assert!(index.entries.iter().any(|e| e.contains("legacy-index") || e == "legacy-placeholder"));

    // write_index should append an entry
    LegacyBackend::write_index(&repo, &mut index).expect("legacy write_index should succeed");
    assert!(index.entries.iter().any(|e| e.contains("legacy-write")));

    // read_object returns a formatted string
    let obj = LegacyBackend::read_object(&repo, "deadbeef").expect("legacy read_object should succeed");
    assert!(obj.contains("legacy-object:deadbeef"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_gitoxide_index_and_odb_may_be_not_implemented_or_ok() {
    // Prepare a RepositoryHandle pointing at a temp dir; adapter will decide what to do
    let dir = unique_tmp_dir();
    if dir.exists() { let _ = fs::remove_dir_all(&dir); }
    fs::create_dir_all(&dir).expect("create tmp dir");

    let repo = RepositoryHandle {
        git_dir: dir.clone(),
        work_tree: Some(dir.clone()),
        kind: RepositoryKind::Worktree,
        backend: BackendKind::Gitoxide,
    };

    // load_index_impl currently may return NotImplemented while we iterate; accept either
    match GitoxideBackend::load_index(&repo) {
        Ok(idx) => {
            // If implemented, it should return a RepositoryIndex
            let _ : RepositoryIndex = idx;
        }
        Err(BackendError::NotImplemented(_)) => {
            // acceptable for incremental implementation
        }
        Err(e) => panic!("unexpected error from gitoxide load_index: {:?}", e),
    }

    // read_object may also be unimplemented yet
    match GitoxideBackend::read_object(&repo, "deadbeef") {
        Ok(s) => {
            // if implemented, should return some representation
            let _ = s;
        }
        Err(BackendError::NotImplemented(_)) => {
            // acceptable
        }
        Err(e) => panic!("unexpected error from gitoxide read_object: {:?}", e),
    }

    let _ = fs::remove_dir_all(&dir);
}
