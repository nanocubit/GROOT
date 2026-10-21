use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    #[error("git command failed: {0}")]
    GitCommand(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
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

fn git_output(dir: &Path, args: &[&str]) -> BackendResult<String> {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .map_err(BackendError::from)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(BackendError::GitCommand(stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_output_with_git_dir(git_dir: &Path, work_tree: Option<&Path>, args: &[&str]) -> BackendResult<String> {
    let mut cmd = Command::new("git");
    cmd.arg("--git-dir").arg(git_dir);
    if let Some(work_tree) = work_tree {
        cmd.arg("--work-tree").arg(work_tree);
    }
    cmd.args(args);

    let output = cmd.output().map_err(BackendError::from)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(BackendError::GitCommand(stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn repository_kind_from_git_dir(git_dir: &Path, work_tree: Option<&Path>) -> BackendResult<RepositoryKind> {
    let is_bare = git_output_with_git_dir(git_dir, work_tree, &["rev-parse", "--is-bare-repository"])?
        .trim()
        .eq("true");

    if is_bare {
        Ok(RepositoryKind::Bare)
    } else {
        Ok(RepositoryKind::Worktree)
    }
}

#[derive(Debug, Clone, Default)]
pub struct LegacyBackend;

impl GitBackend for LegacyBackend {
    fn discover(start: &Path) -> BackendResult<RepositoryHandle> {
        let candidate = start.canonicalize().map_err(|_| BackendError::NotARepository(start.display().to_string()))?;

        let git_dir = if candidate.join(".git").exists() {
            candidate.join(".git")
        } else if candidate.ends_with(".git") {
            candidate
        } else {
            return Err(BackendError::NotARepository(candidate.display().to_string()));
        };

        let git_dir = git_dir.canonicalize().map_err(|_| BackendError::NotARepository(git_dir.display().to_string()))?;
        let work_tree = if git_dir.file_name() == Some(OsStr::new(".git")) {
            Some(candidate.clone())
        } else {
            None
        };

        if !git_dir.join("HEAD").exists() && !git_dir.join("objects").exists() {
            return Err(BackendError::NotARepository(git_dir.display().to_string()));
        }

        let kind = repository_kind_from_git_dir(&git_dir, work_tree.as_deref())?;

        Ok(RepositoryHandle {
            git_dir,
            work_tree,
            kind,
            backend: BackendKind::Legacy,
        })
    }

    fn open(git_dir: &Path, work_tree: Option<&Path>) -> BackendResult<RepositoryHandle> {
        let git_dir = git_dir.canonicalize().map_err(|_| BackendError::NotARepository(git_dir.display().to_string()))?;

        if !git_dir.join("HEAD").exists() && !git_dir.join("objects").exists() {
            return Err(BackendError::NotARepository(git_dir.display().to_string()));
        }

        let work_tree = work_tree.map(PathBuf::from);
        let kind = repository_kind_from_git_dir(&git_dir, work_tree.as_deref())?;

        Ok(RepositoryHandle {
            git_dir,
            work_tree,
            kind,
            backend: BackendKind::Legacy,
        })
    }

    fn load_index(repo: &RepositoryHandle) -> BackendResult<RepositoryIndex> {
        let path = repo.git_dir.join("index");
        let file_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        Ok(RepositoryIndex {
            entries: vec![format!("git-index:{}:{}", path.display(), file_size)],
        })
    }

    fn write_index(_: &RepositoryHandle, index: &mut RepositoryIndex) -> BackendResult<()> {
        index.entries.push("write-index:placeholder".to_string());
        Ok(())
    }

    fn read_object(repo: &RepositoryHandle, object_id: &str) -> BackendResult<String> {
        let work_tree = repo.work_tree.as_deref();
        let raw = git_output_with_git_dir(&repo.git_dir, work_tree, &["cat-file", "-p", object_id])?;
        Ok(raw)
    }
}

#[derive(Debug, Clone, Default)]
pub struct GitoxideBackend;

impl GitBackend for GitoxideBackend {
    fn discover(start: &Path) -> BackendResult<RepositoryHandle> {
        let candidate = start.canonicalize().map_err(|_| BackendError::NotARepository(start.display().to_string()))?;

        let git_dir = if candidate.join(".git").exists() {
            candidate.join(".git")
        } else if candidate.ends_with(".git") {
            candidate
        } else {
            return Err(BackendError::NotARepository(candidate.display().to_string()));
        };

        let git_dir = git_dir.canonicalize().map_err(|_| BackendError::NotARepository(git_dir.display().to_string()))?;
        let work_tree = if git_dir.file_name() == Some(OsStr::new(".git")) {
            Some(candidate.clone())
        } else {
            None
        };

        if !git_dir.join("HEAD").exists() && !git_dir.join("objects").exists() {
            return Err(BackendError::NotARepository(git_dir.display().to_string()));
        }

        let kind = repository_kind_from_git_dir(&git_dir, work_tree.as_deref())?;

        Ok(RepositoryHandle {
            git_dir,
            work_tree,
            kind,
            backend: BackendKind::Gitoxide,
        })
    }

    fn open(git_dir: &Path, work_tree: Option<&Path>) -> BackendResult<RepositoryHandle> {
        let git_dir = git_dir.canonicalize().map_err(|_| BackendError::NotARepository(git_dir.display().to_string()))?;

        if !git_dir.join("HEAD").exists() && !git_dir.join("objects").exists() {
            return Err(BackendError::NotARepository(git_dir.display().to_string()));
        }

        let work_tree = work_tree.map(PathBuf::from);
        let kind = repository_kind_from_git_dir(&git_dir, work_tree.as_deref())?;

        Ok(RepositoryHandle {
            git_dir,
            work_tree,
            kind,
            backend: BackendKind::Gitoxide,
        })
    }

    fn load_index(repo: &RepositoryHandle) -> BackendResult<RepositoryIndex> {
        let path = repo.git_dir.join("index");
        let file_size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        Ok(RepositoryIndex {
            entries: vec![format!("gitoxide-index:{}:{}", path.display(), file_size)],
        })
    }

    fn write_index(_: &RepositoryHandle, index: &mut RepositoryIndex) -> BackendResult<()> {
        index.entries.push("write-index:placeholder".to_string());
        Ok(())
    }

    fn read_object(repo: &RepositoryHandle, object_id: &str) -> BackendResult<String> {
        let work_tree = repo.work_tree.as_deref();
        let raw = git_output_with_git_dir(&repo.git_dir, work_tree, &["cat-file", "-p", object_id])?;
        Ok(raw)
    }
}

#[derive(Debug, Clone, Default)]
pub struct AdapterConfig {
    pub default_backend: BackendKind,
}

pub fn default_backend_kind() -> BackendKind {
    BackendKind::Legacy
}
