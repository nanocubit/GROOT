use std::path::{Path, PathBuf};

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

impl GitBackend for LegacyBackend {
    fn discover(start: &Path) -> BackendResult<RepositoryHandle> {
        let candidate = if start.exists() {
            start.to_path_buf()
        } else {
            return Err(BackendError::NotARepository(start.display().to_string()));
        };

        Ok(RepositoryHandle {
            git_dir: candidate.clone(),
            work_tree: Some(candidate.clone()),
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
        let candidate = if start.exists() {
            start.to_path_buf()
        } else {
            return Err(BackendError::NotARepository(start.display().to_string()));
        };

        Ok(RepositoryHandle {
            git_dir: candidate.clone(),
            work_tree: Some(candidate.clone()),
            kind: RepositoryKind::Worktree,
            backend: BackendKind::Gitoxide,
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
            backend: BackendKind::Gitoxide,
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

pub fn default_backend_kind() -> BackendKind {
    BackendKind::Legacy
}
