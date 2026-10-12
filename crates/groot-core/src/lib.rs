//! Core backend contract for GROOT.

use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BackendKind {
    #[default]
    Legacy,
    Gitoxide,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryKind {
    Worktree,
    Bare,
    LinkedWorktree,
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("path is not a repository: {0}")]
    NotARepository(String),
    #[error("backend not implemented: {0}")]
    NotImplemented(String),
}

pub type BackendResult<T> = Result<T, BackendError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryHandle {
    pub git_dir: PathBuf,
    pub work_tree: Option<PathBuf>,
    pub kind: RepositoryKind,
    pub backend: BackendKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryIndex {
    pub entries: Vec<String>,
}

pub trait GitBackend: Send + Sync {
    fn discover(start: &Path) -> BackendResult<RepositoryHandle>;
    fn open(git_dir: &Path, work_tree: Option<&Path>) -> BackendResult<RepositoryHandle>;
    fn load_index(repo: &RepositoryHandle) -> BackendResult<RepositoryIndex>;
    fn write_index(repo: &RepositoryHandle, index: &mut RepositoryIndex) -> BackendResult<()>;
    fn read_object(repo: &RepositoryHandle, object_id: &str) -> BackendResult<String>;
}

#[derive(Debug, Clone, Default)]
pub struct LegacyBackend;

impl GitBackend for LegacyBackend {
    fn discover(start: &Path) -> BackendResult<RepositoryHandle> {
        if !start.exists() {
            return Err(BackendError::NotARepository(start.display().to_string()));
        }
        Ok(RepositoryHandle {
            git_dir: start.to_path_buf(),
            work_tree: Some(start.to_path_buf()),
            kind: RepositoryKind::Worktree,
            backend: BackendKind::Legacy,
        })
    }

    fn open(git_dir: &Path, work_tree: Option<&Path>) -> BackendResult<RepositoryHandle> {
        if !git_dir.exists() {
            return Err(BackendError::NotARepository(git_dir.display().to_string()));
        }
        Ok(RepositoryHandle {
            git_dir: git_dir.to_path_buf(),
            work_tree: work_tree.map(PathBuf::from),
            kind: if work_tree.is_some() {
                RepositoryKind::Worktree
            } else {
                RepositoryKind::Bare
            },
            backend: BackendKind::Legacy,
        })
    }

    fn load_index(_repo: &RepositoryHandle) -> BackendResult<RepositoryIndex> {
        Ok(RepositoryIndex {
            entries: vec!["legacy-placeholder".to_string()],
        })
    }

    fn write_index(_repo: &RepositoryHandle, index: &mut RepositoryIndex) -> BackendResult<()> {
        index.entries.push("legacy-write".to_string());
        Ok(())
    }

    fn read_object(_repo: &RepositoryHandle, object_id: &str) -> BackendResult<String> {
        Ok(format!("legacy-object:{}", object_id))
    }
}

#[derive(Debug, Clone, Default)]
pub struct GitoxideBackend;

impl GitBackend for GitoxideBackend {
    fn discover(start: &Path) -> BackendResult<RepositoryHandle> {
        crate::adapters::gitoxide::discover_impl(start)
    }

    fn open(git_dir: &Path, work_tree: Option<&Path>) -> BackendResult<RepositoryHandle> {
        crate::adapters::gitoxide::open_impl(git_dir, work_tree)
    }

    fn load_index(repo: &RepositoryHandle) -> BackendResult<RepositoryIndex> {
        crate::adapters::gitoxide::load_index_impl(repo)
    }

    fn write_index(repo: &RepositoryHandle, index: &mut RepositoryIndex) -> BackendResult<()> {
        crate::adapters::gitoxide::write_index_impl(repo, index)
    }

    fn read_object(repo: &RepositoryHandle, object_id: &str) -> BackendResult<String> {
        crate::adapters::gitoxide::read_object_impl(repo, object_id)
    }
}

#[derive(Debug, Clone, Default)]
pub struct AdapterConfig {
    pub default_backend: BackendKind,
}

impl AdapterConfig {
    pub fn new(default_backend: BackendKind) -> Self {
        Self { default_backend }
    }
}

pub fn default_backend_kind() -> BackendKind {
    BackendKind::Legacy
}

pub fn backend_from_env() -> BackendKind {
    match env::var("GROOT_BACKEND").ok().as_deref() {
        Some("gitoxide") | Some("gix") => BackendKind::Gitoxide,
        Some("legacy") => BackendKind::Legacy,
        _ => default_backend_kind(),
    }
}

pub fn resolve_backend(cfg: Option<AdapterConfig>) -> BackendKind {
    cfg.map(|c| c.default_backend).unwrap_or_else(|| backend_from_env())
}

pub mod adapters;
pub use adapters::gitoxide::GitoxideBackend as GixBackend;
