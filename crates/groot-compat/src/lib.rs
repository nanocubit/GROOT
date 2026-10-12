pub mod adapters;

pub use groot_core::{
    backend_from_env,
    default_backend_kind,
    resolve_backend,
    AdapterConfig,
    BackendKind,
    GitBackend,
    GitoxideBackend,
    LegacyBackend,
    RepositoryHandle,
    RepositoryIndex,
};
