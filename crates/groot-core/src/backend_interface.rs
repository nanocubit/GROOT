// Trait-based backend interface (v2) to avoid name collisions with the quick prototype in lib.rs.
// This is the intended, stable shape for adapters to implement.

use std::path::{Path, PathBuf};

/// Repository handle exposed to adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoHandleV2 {
    pub git_dir: PathBuf,
    pub work_tree: Option<PathBuf>,
    pub is_bare: bool,
}

/// Index representation used in tests and adapters (very small adapter-friendly view).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoIndexV2 {
    pub entries: Vec<String>,
}

/// Generic error type for backend adapters.
#[derive(thiserror::Error, Debug)]
pub enum BackendV2Error {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("not a repository: {0}")]
    NotARepository(String),
    #[error("not implemented: {0}")]
    NotImplemented(String),
}

pub type BackendV2Result<T> = Result<T, BackendV2Error>;

/// The trait adapters should implement. Methods are kept small and concrete so tests
/// can exercise discovery, index and object access without requiring a full grit
/// or gitoxide link.
pub trait BackendV2: Send + Sync {
    fn discover(start: &Path) -> BackendV2Result<RepoHandleV2>;
    fn open(git_dir: &Path, work_tree: Option<&Path>) -> BackendV2Result<RepoHandleV2>;
    fn load_index(handle: &RepoHandleV2) -> BackendV2Result<RepoIndexV2>;
    fn write_index(handle: &RepoHandleV2, index: &mut RepoIndexV2) -> BackendV2Result<()>;
    fn read_object(handle: &RepoHandleV2, object_id: &str) -> BackendV2Result<String>;
}
