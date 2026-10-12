// crates/groot-core/src/adapters/gitoxide.rs
//! Gitoxide adapter for GROOT backend trait.
//! Conditional: real calls are enabled when cargo feature "gitoxide" is set.
//! Without the feature this file compiles to stubs so the workspace builds.

use std::path::Path;

use crate::{BackendError, BackendResult, GitBackend, RepositoryHandle, RepositoryIndex};

#[derive(Debug, Clone, Default)]
pub struct GitoxideBackend;

impl GitoxideBackend {
    #[cfg(feature = "gitoxide")]
    fn map_repo(repo: &gix::Repository) -> RepositoryHandle {
        let git_dir = repo.path().to_path_buf();
        let work_tree = repo.work_tree().map(|p| p.to_path_buf());
        let kind = if work_tree.is_some() {
            crate::RepositoryKind::Worktree
        } else {
            crate::RepositoryKind::Bare
        };
        RepositoryHandle {
            git_dir,
            work_tree,
            kind,
            backend: crate::BackendKind::Gitoxide,
        }
    }
}

impl GitBackend for GitoxideBackend {
    fn discover(start: &Path) -> BackendResult<RepositoryHandle> {
        #[cfg(feature = "gitoxide")]
        {
            match gix::discover(start) {
                Ok(repo) => Ok(Self::map_repo(&repo)),
                Err(e) => Err(BackendError::NotARepository(format!(
                    "gix discover error: {}",
                    e
                ))),
            }
        }
        #[cfg(not(feature = "gitoxide"))]
        {
            Err(BackendError::NotImplemented(
                "gitoxide feature disabled".into(),
            ))
        }
    }

    fn open(git_dir: &Path, work_tree: Option<&Path>) -> BackendResult<RepositoryHandle> {
        #[cfg(feature = "gitoxide")]
        {
            match gix::open(git_dir) {
                Ok(repo) => Ok(Self::map_repo(&repo)),
                Err(e) => Err(BackendError::NotARepository(format!(
                    "gix open error: {}",
                    e
                ))),
            }
        }
        #[cfg(not(feature = "gitoxide"))]
        {
            Err(BackendError::NotImplemented(
                "gitoxide feature disabled".into(),
            ))
        }
    }

    fn load_index(repo: &RepositoryHandle) -> BackendResult<RepositoryIndex> {
        #[cfg(feature = "gitoxide")]
        {
            let idx_path = repo.git_dir.join("index");
            if !idx_path.exists() {
                return Ok(RepositoryIndex { entries: vec![] });
            }
            match gix::index::File::at(idx_path, gix::index::decode::Options::default()) {
                Ok(file) => {
                    let state = file.state();
                    let entries = state
                        .entries
                        .iter()
                        .map(|e| e.path(state).to_string_lossy().to_string())
                        .collect();
                    Ok(RepositoryIndex { entries })
                }
                Err(e) => Err(BackendError::NotImplemented(format!(
                    "gix-index open error: {}",
                    e
                ))),
            }
        }
        #[cfg(not(feature = "gitoxide"))]
        {
            Err(BackendError::NotImplemented(
                "gitoxide feature disabled".into(),
            ))
        }
    }

    fn write_index(_repo: &RepositoryHandle, index: &mut RepositoryIndex) -> BackendResult<()> {
        #[cfg(feature = "gitoxide")]
        {
            index
                .entries
                .push(format!("gix-write:{}", _repo.git_dir.display()));
            Ok(())
        }
        #[cfg(not(feature = "gitoxide"))]
        {
            Err(BackendError::NotImplemented(
                "gitoxide feature disabled".into(),
            ))
        }
    }

    fn read_object(_repo: &RepositoryHandle, object_id: &str) -> BackendResult<String> {
        #[cfg(feature = "gitoxide")]
        {
            let objects_dir = _repo.git_dir.join("objects");
            let handle = gix::odb::at(objects_dir)
                .map_err(|e| BackendError::NotImplemented(format!("gix-odb open: {}", e)))?;

            let oid = gix::hash::ObjectId::from_hex(object_id.as_bytes()).map_err(|_| {
                BackendError::NotImplemented(format!("invalid object id: {}", object_id))
            })?;

            match handle.find(oid) {
                Ok(obj) => {
                    let len = obj.data.len();
                    Ok(format!("gix-object:{}:len={}", object_id, len))
                }
                Err(e) => Err(BackendError::NotImplemented(format!(
                    "gix-odb find error: {}",
                    e
                ))),
            }
        }
        #[cfg(not(feature = "gitoxide"))]
        {
            Err(BackendError::NotImplemented(
                "gitoxide feature disabled".into(),
            ))
        }
    }
}

// Exported wrapper functions expected by crate::adapters::gitoxide::discover_impl etc.
// These simply delegate to the GitoxideBackend implementation so lib.rs can call them.

pub fn discover_impl(start: &Path) -> BackendResult<RepositoryHandle> {
    GitoxideBackend::discover(start)
}

pub fn open_impl(git_dir: &Path, work_tree: Option<&Path>) -> BackendResult<RepositoryHandle> {
    GitoxideBackend::open(git_dir, work_tree)
}

pub fn load_index_impl(repo: &RepositoryHandle) -> BackendResult<RepositoryIndex> {
    GitoxideBackend::load_index(repo)
}

pub fn write_index_impl(repo: &RepositoryHandle, index: &mut RepositoryIndex) -> BackendResult<()> {
    GitoxideBackend::write_index(repo, index)
}

pub fn read_object_impl(repo: &RepositoryHandle, object_id: &str) -> BackendResult<String> {
    GitoxideBackend::read_object(repo, object_id)
}
