// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use thiserror::Error as DeriveError;

/// Errors returned by the bundler.
#[derive(Debug, DeriveError)]
#[non_exhaustive]
pub enum Error {

    #[error("SignTool not found")]
    SignToolNotFound,
    /// Failed to open Windows registry.
    #[error("failed to open registry {0}")]
    OpenRegistry(String),
    /// Failed to get registry value.
    #[error("failed to get {0} value on registry")]
    GetRegistryValue(String),
    /// Unsupported OS bitness.
    #[error("unsupported OS bitness")]
    UnsupportedBitness
}

/// Convenient type alias of Result type.
pub type Result<T> = std::result::Result<T, Error>;