use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use groot_core::{LegacyBackend, GitBackend};

fn temp_repo_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("groot-core-test-{}", nanos));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn discovers_real_git_repo() {
    let repo = temp_repo_path();
    let _ = Command::new("git")
        .arg("init")
        .arg(&repo)
        .status()
        .expect("git init should work");

    let file = repo.join("README.md");
    fs::write(&file, "hello from groot\n").unwrap();

    let _ = Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("add")
        .arg("README.md")
        .status()
        .expect("git add should work");

    let _ = Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("-c")
        .arg("user.name=Test User")
        .arg("-c")
        .arg("user.email=test@example.com")
        .arg("commit")
        .arg("-m")
        .arg("baseline")
        .status()
        .expect("git commit should work");

    let handle = LegacyBackend::discover(&repo).expect("repo should be discovered");
    assert_eq!(handle.kind, groot_core::RepositoryKind::Worktree);
    assert_eq!(handle.work_tree, Some(repo.clone()));
    assert!(handle.git_dir.ends_with(".git"));
}

#[test]
fn reads_object_from_real_git_repo() {
    let repo = temp_repo_path();
    let _ = Command::new("git")
        .arg("init")
        .arg(&repo)
        .status()
        .expect("git init should work");

    let file = repo.join("README.md");
    fs::write(&file, "hello from groot\n").unwrap();

    let _ = Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("add")
        .arg("README.md")
        .status()
        .expect("git add should work");

    let _ = Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("-c")
        .arg("user.name=Test User")
        .arg("-c")
        .arg("user.email=test@example.com")
        .arg("commit")
        .arg("-m")
        .arg("baseline")
        .status()
        .expect("git commit should work");

    let head = Command::new("git")
        .arg("-C")
        .arg(&repo)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .unwrap();
    let oid = String::from_utf8(head.stdout).unwrap().trim().to_string();

    let handle = LegacyBackend::discover(&repo).unwrap();
    let object = LegacyBackend::read_object(&handle, &oid).expect("read_object should work");
    assert!(!object.is_empty());
    assert!(object.contains("hello from groot"));
}
