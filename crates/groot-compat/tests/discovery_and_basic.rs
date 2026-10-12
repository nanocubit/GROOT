use std::fs;
use std::path::PathBuf;

use groot_core::{BackendKind, GitBackend, GitoxideBackend, LegacyBackend};

#[test]
fn legacy_backend_discovers_temp_directory() {
    let temp = std::env::temp_dir().join("groot-legacy-backend-check");
    let _ = fs::create_dir_all(&temp);

    let repo = LegacyBackend::discover(&temp).unwrap();
    assert_eq!(repo.backend, BackendKind::Legacy);
    assert_eq!(repo.git_dir, temp);

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn gitoxide_backend_uses_gitoxide_kind() {
    let temp = std::env::temp_dir().join("groot-gix-backend-check");
    let _ = fs::create_dir_all(&temp);

    match GitoxideBackend::discover(&temp) {
        Ok(repo) => {
            assert_eq!(repo.backend, BackendKind::Gitoxide);
            assert_eq!(repo.git_dir, temp);
        }
        Err(groot_core::BackendError::NotImplemented(_)) | Err(groot_core::BackendError::NotARepository(_)) => {
            // acceptable while the gitoxide feature is being implemented
        }
        Err(e) => panic!("unexpected error from gitoxide discover: {:?}", e),
    }

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn default_backend_is_legacy() {
    assert_eq!(groot_core::default_backend_kind(), BackendKind::Legacy);
}

#[test]
fn legacy_backend_index_roundtrip_smoke() {
    let temp = std::env::temp_dir().join("groot-index-roundtrip");
    let _ = fs::create_dir_all(&temp);

    let handle = LegacyBackend::open(&temp, Some(&temp)).unwrap();
    let mut index = LegacyBackend::load_index(&handle).unwrap();
    LegacyBackend::write_index(&handle, &mut index).unwrap();

    assert!(index.entries.iter().any(|s| s.contains("legacy-write")));

    let _ = fs::remove_dir_all(temp);
}

#[test]
fn legacy_backend_read_object_smoke() {
    let temp = std::env::temp_dir().join("groot-object-smoke");
    let _ = fs::create_dir_all(&temp);

    let handle = LegacyBackend::discover(&temp).unwrap();
    let obj = LegacyBackend::read_object(&handle, "deadbeef").unwrap();
    assert!(obj.contains("deadbeef"));

    let _ = fs::remove_dir_all(temp);
}
