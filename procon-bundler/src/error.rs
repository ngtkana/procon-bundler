use std::path::PathBuf;
use thiserror::Error;

/// Custom error types for the procon-bundler
#[derive(Error, Debug)]
pub enum BundlerError {
    #[error("Cargo.toml not found at {path:?}")]
    CargoTomlNotFound { path: PathBuf },

    #[error("Failed to read Cargo.toml at {path:?}: {source}")]
    CargoTomlReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse Cargo.toml: {source}")]
    CargoTomlParseError {
        #[source]
        source: toml::de::Error,
    },

    #[error("Module file not found for path {module_path:?} at {file_path:?}: {source}")]
    ModuleFileNotFound {
        module_path: PathBuf,
        file_path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to read module file {path:?}: {source}")]
    ModuleFileReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid path conversion: {path:?}")]
    InvalidPathConversion { path: PathBuf },

    #[error("Invalid file stem: {path:?}")]
    InvalidFileStem { path: PathBuf },

    #[error("TOML file is not a table")]
    TomlNotTable,

    #[error("Dependencies section is not a table")]
    DependenciesNotTable,

    #[error("Path value is not a string: {value:?}")]
    PathNotString { value: String },
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, BundlerError>;
