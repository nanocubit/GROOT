use std::fs;
use std::path::{Path, PathBuf};

pub mod backend_interface;

pub use backend_interface::{BackendV2, BackendV2Error, BackendV2Result, RepoHandleV2, RepoIndexV2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

impl LegacyBackend {
    fn detect_worktree(start: &Path) -> Option<PathBuf> {
        let mut current = start.to_path_buf();

        loop {
            let git_dir = current.join(".git");
            if git_dir.exists() || current.join("HEAD").exists() && current.join("config").exists() {
                return Some(current);
            }

            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => return None,
            }
        }
    }
}

impl GitBackend for LegacyBackend {
    fn discover(start: &Path) -> BackendResult<RepositoryHandle> {
        if !start.exists() {
            return Err(BackendError::NotARepository(start.display().to_string()));
        }

        if let Some(work_tree) = Self::detect_worktree(start) {
            let git_dir = work_tree.join(".git");
            return Ok(RepositoryHandle {
                git_dir: git_dir.clone(),
                work_tree: Some(work_tree.clone()),
                kind: RepositoryKind::Worktree,
                backend: BackendKind::Legacy,
            });
        }

        if start.join("HEAD").exists() && start.join("objects").exists() {
            return Ok(RepositoryHandle {
                git_dir: start.to_path_buf(),
                work_tree: None,
                kind: RepositoryKind::Bare,
                backend: BackendKind::Legacy,
            });
        }

        Err(BackendError::NotARepository(start.display().to_string()))
    }

    fn open(git_dir: &Path, work_tree: Option<&Path>) -> BackendResult<RepositoryHandle> {
        if !git_dir.exists() {
            return Err(BackendError::NotARepository(git_dir.display().to_string()));
        }

        let work_tree = work_tree.map(PathBuf::from);
        if work_tree.is_some() {
            return Ok(RepositoryHandle {
                git_dir: git_dir.to_path_buf(),
                work_tree,
                kind: RepositoryKind::Worktree,
                backend: BackendKind::Legacy,
            });
        }

        if git_dir.join("HEAD").exists() && git_dir.join("objects").exists() {
            return Ok(RepositoryHandle {
                git_dir: git_dir.to_path_buf(),
                work_tree: None,
                kind: RepositoryKind::Bare,
                backend: BackendKind::Legacy,
            });
        }

        Err(BackendError::NotARepository(git_dir.display().to_string()))
    }

    fn load_index(repo: &RepositoryHandle) -> BackendResult<RepositoryIndex> {
        Ok(RepositoryIndex {
            entries: vec![format!("legacy-index:{}", repo.git_dir.display())],
        })
    }

    fn write_index(repo: &RepositoryHandle, index: &mut RepositoryIndex) -> BackendResult<()> {
        index.entries.push(format!("legacy-write:{}", repo.git_dir.display()));
        Ok(())
    }

    fn read_object(repo: &RepositoryHandle, object_id: &str) -> BackendResult<String> {
        Ok(format!("legacy-object:{}:{}", repo.git_dir.display(), object_id))
    }
}

#[derive(Debug, Clone, Default)]
pub struct GitoxideBackend;

impl GitBackend for GitoxideBackend {
    fn discover(start: &Path) -> BackendResult<RepositoryHandle> {
        LegacyBackend::discover(start)
            .map(|mut handle| {
                handle.backend = BackendKind::Gitoxide;
                handle
            })
    }

    fn open(git_dir: &Path, work_tree: Option<&Path>) -> BackendResult<RepositoryHandle> {
        LegacyBackend::open(git_dir, work_tree)
            .map(|mut handle| {
                handle.backend = BackendKind::Gitoxide;
                handle
            })
    }

    fn load_index(repo: &RepositoryHandle) -> BackendResult<RepositoryIndex> {
        Ok(RepositoryIndex {
            entries: vec![format!("gix-index:{}", repo.git_dir.display())],
        })
    }

    fn write_index(repo: &RepositoryHandle, index: &mut RepositoryIndex) -> BackendResult<()> {
        index.entries.push(format!("gix-write:{}", repo.git_dir.display()));
        Ok(())
    }

    fn read_object(repo: &RepositoryHandle, object_id: &str) -> BackendResult<String> {
        Ok(format!("gix-object:{}:{}", repo.git_dir.display(), object_id))
    }
}

#[derive(Debug, Clone, Default)]
pub struct AdapterConfig {
    pub default_backend: BackendKind,
}

impl Default for AdapterConfig {
    fn default() -> Self {
        Self {
            default_backend: default_backend_kind(),
        }
    }
}

pub fn default_backend_kind() -> BackendKind {
    BackendKind::Legacy
}

// Normalized API: stable v2-compatible types and adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoHandleCompat {
    pub git_dir: PathBuf,
    pub work_tree: Option<PathBuf>,
    pub is_bare: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoIndexCompat {
    pub entries: Vec<String>,
}

pub trait BackendCompat: Send + Sync {
    fn discover(start: &Path) -> BackendV2Result<RepoHandleCompat>;
    fn open(git_dir: &Path, work_tree: Option<&Path>) -> BackendV2Result<RepoHandleCompat>;
    fn load_index(handle: &RepoHandleCompat) -> BackendV2Result<RepoIndexCompat>;
    fn write_index(handle: &RepoHandleCompat, index: &mut RepoIndexCompat) -> BackendV2Result<()>;
    fn read_object(handle: &RepoHandleCompat, object_id: &str) -> BackendV2Result<String>;
}

impl From<&RepositoryHandle> for RepoHandleCompat {
    fn from(value: &RepositoryHandle) -> Self {
        Self {
            git_dir: value.git_dir.clone(),
            work_tree: value.work_tree.clone(),
            is_bare: matches!(value.kind, RepositoryKind::Bare),
        }
    }
}

impl From<&RepoHandleCompat> for RepositoryHandle {
    fn from(value: &RepoHandleCompat) -> Self {
        Self {
            git_dir: value.git_dir.clone(),
            work_tree: value.work_tree.clone(),
            kind: if value.is_bare {
                RepositoryKind::Bare
            } else {
                RepositoryKind::Worktree
            },
            backend: BackendKind::Legacy,
        }
    }
}

pub struct LegacyAdapter;

impl BackendV2 for LegacyAdapter {
    fn discover(start: &Path) -> BackendV2Result<RepoHandleV2> {
        let handle = LegacyBackend::discover(start)
            .map_err(|err| BackendV2Error::NotARepository(err.to_string()))?;
        Ok(RepoHandleV2 {
            git_dir: handle.git_dir,
            work_tree: handle.work_tree,
            is_bare: matches!(handle.kind, RepositoryKind::Bare),
        })
    }

    fn open(git_dir: &Path, work_tree: Option<&Path>) -> BackendV2Result<RepoHandleV2> {
        let handle = LegacyBackend::open(git_dir, work_tree)
            .map_err(|err| BackendV2Error::NotARepository(err.to_string()))?;
        Ok(RepoHandleV2 {
            git_dir: handle.git_dir,
            work_tree: handle.work_tree,
            is_bare: matches!(handle.kind, RepositoryKind::Bare),
        })
    }

    fn load_index(handle: &RepoHandleV2) -> BackendV2Result<RepoIndexV2> {
        Ok(RepoIndexV2 {
            entries: vec![format!("legacy-index:{}", handle.git_dir.display())],
        })
    }

    fn write_index(handle: &RepoHandleV2, index: &mut RepoIndexV2) -> BackendV2Result<()> {
        index.entries.push(format!("legacy-write:{}", handle.git_dir.display()));
        Ok(())
    }

    fn read_object(handle: &RepoHandleV2, object_id: &str) -> BackendV2Result<String> {
        Ok(format!("legacy-object:{}:{}", handle.git_dir.display(), object_id))
    }
}

pub struct GitoxideAdapter;

impl BackendV2 for GitoxideAdapter {
    fn discover(start: &Path) -> BackendV2Result<RepoHandleV2> {
        LegacyAdapter::discover(start)
    }

    fn open(git_dir: &Path, work_tree: Option<&Path>) -> BackendV2Result<RepoHandleV2> {
        LegacyAdapter::open(git_dir, work_tree)
    }

    fn load_index(handle: &RepoHandleV2) -> BackendV2Result<RepoIndexV2> {
        Ok(RepoIndexV2 {
            entries: vec![format!("gix-index:{}", handle.git_dir.display())],
        })
    }

    fn write_index(handle: &RepoHandleV2, index: &mut RepoIndexV2) -> BackendV2Result<()> {
        index.entries.push(format!("gix-write:{}", handle.git_dir.display()));
        Ok(())
    }

    fn read_object(handle: &RepoHandleV2, object_id: &str) -> BackendV2Result<String> {
        Ok(format!("gix-object:{}:{}", handle.git_dir.display(), object_id))
    }
}

pub fn normalize_repository_path(start: &Path) -> BackendV2Result<RepoHandleV2> {
    LegacyAdapter::discover(start)
}

pub fn is_git_directory(path: &Path) -> bool {
    path.is_dir() && (path.join("HEAD").exists() || path.join("config").exists())
}

pub fn make_repo_handle(git_dir: &Path, work_tree: Option<&Path>, is_bare: bool) -> RepositoryHandle {
    RepositoryHandle {
        git_dir: git_dir.to_path_buf(),
        work_tree: work_tree.map(PathBuf::from),
        kind: if is_bare { RepositoryKind::Bare } else { RepositoryKind::Worktree },
        backend: BackendKind::Legacy,
    }
}
