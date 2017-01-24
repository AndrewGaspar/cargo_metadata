#![deny(missing_docs)]
//! Structured access to the output of `cargo metadata`
//! Usually used from within a `cargo-*` executable
//!
//! ```rust
//! let manifest_path_arg = std::env::args().skip(2).find(|val| val.starts_with("--manifest-path="));
//! let mut metadata = cargo::metadata(manifest_path_arg.as_ref().map(AsRef::as_ref)).unwrap();
//! ```

extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

use std::collections::HashMap;
use std::process::Command;
use std::str::{from_utf8, Utf8Error};
use std::io;

#[derive(Deserialize, Debug)]
/// Starting point for metadata returned by `cargo metadata`
pub struct Metadata {
    /// A list of all crates referenced by this crate (and the crate itself)
    pub packages: Vec<Package>,
    resolve: Option<()>,
    version: usize,
}

#[derive(Deserialize, Debug)]
/// A crate
pub struct Package {
    /// Name as given in the `Cargo.toml`
    pub name: String,
    /// Version given in the `Cargo.toml`
    pub version: String,
    id: String,
    source: Option<String>,
    /// List of dependencies of this particular package
    pub dependencies: Vec<Dependency>,
    /// Targets provided by the crate (lib, bin, example, test, ...)
    pub targets: Vec<Target>,
    features: HashMap<String, Vec<String>>,
    /// path containing the `Cargo.toml`
    pub manifest_path: String,
}

#[derive(Deserialize, Debug)]
/// A dependency of the main crate
pub struct Dependency {
    /// Name as given in the `Cargo.toml`
    pub name: String,
    source: Option<String>,
    /// Whether this is required or optional
    pub req: String,
    kind: Option<String>,
    optional: bool,
    uses_default_features: bool,
    features: Vec<String>,
    target: Option<String>,
}

#[derive(Deserialize, Debug)]
/// A single target (lib, bin, example, ...) provided by a crate
pub struct Target {
    /// Name as given in the `Cargo.toml` or generated from the file name
    pub name: String,
    /// Kind of target ("bin", "example", "test", "bench", "lib")
    pub kind: Vec<String>,
    src_path: String,
}

#[derive(Debug)]
/// Possible errors that can occur during metadata parsing.
pub enum Error {
    /// Error during execution of `cargo metadata`
    Io(io::Error),
    /// Output of `cargo metadata` was not valid utf8
    Utf8(Utf8Error),
    /// Deserialization error (structure of json did not match expected structure)
    Json(serde_json::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Error::Utf8(err)
    }
}
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

/// The main entry point to obtaining metadata
pub fn metadata(manifest_path_arg: Option<&str>) -> Result<Metadata, Error> {
    let mut cmd = Command::new("cargo");
    cmd.arg("metadata").arg("--no-deps");
    if let Some(mani) = manifest_path_arg {
        cmd.arg(mani);
    }
    let output = cmd.output()?;
    let stdout = from_utf8(&output.stdout)?;
    let meta: Metadata = serde_json::from_str(&stdout)?;
    assert_eq!(meta.version, 1, "please update your `cargo_metadata` dependency to support the new metadata version");
    Ok(meta)
}