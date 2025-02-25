use std::{
    ffi::OsStr,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use itertools::Itertools;
use miette::Diagnostic;
use thiserror::Error;

use super::{cargo, git, go, package_json, pyproject, semver::Version};
use crate::{dry_run::DryRun, workflow::Verbose};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VersionedFile {
    /// The type of file format that `content` is.
    pub(crate) format: PackageFormat,
    /// The path to the file that was parsed.
    pub(crate) path: PathBuf,
    /// The raw content of the package manager file so it doesn't have to be read again.
    content: String,
}

impl TryFrom<PathBuf> for VersionedFile {
    type Error = Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        let format = PackageFormat::try_from(&path)?;
        let content = read_to_string(&path).map_err(|e| ErrorKind::Io(path.clone(), e))?;
        Ok(Self {
            format,
            path,
            content,
        })
    }
}

#[derive(Debug, Diagnostic, Error)]
#[error(transparent)]
#[diagnostic(transparent)]
pub(crate) struct Error(Box<ErrorKind>);

impl<T: Into<ErrorKind>> From<T> for Error {
    fn from(kind: T) -> Self {
        Self(Box::new(kind.into()))
    }
}

#[derive(Debug, Diagnostic, Error)]
enum ErrorKind {
    #[error("Error reading file {0}: {1}")]
    #[diagnostic(
        code(versioned_file::io),
        help("Please check that the file exists and is readable.")
    )]
    Io(PathBuf, #[source] std::io::Error),
    #[error("Path is not a file: {0}")]
    #[diagnostic(
        code(versioned_file::not_a_file),
        help("A versioned file must be a valid relative path to a file.")
    )]
    NotAFile(PathBuf),
    #[error("The versioned file {0} is not a supported format")]
    #[diagnostic(
        code(step::versioned_file_format),
        help("All filed included in [[packages]] versioned_files must be a supported format"),
        url("https://knope-dev.github.io/knope/config/packages.html#supported-formats-for-versioning")
    )]
    VersionedFileFormat(PathBuf),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Git(#[from] git::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Go(#[from] go::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Cargo(#[from] cargo::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    PyProject(#[from] pyproject::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    PackageJson(#[from] package_json::Error),
}

type Result<T> = std::result::Result<T, Error>;

impl VersionedFile {
    pub(crate) fn get_version(&self, verbose: Verbose) -> Result<VersionFromSource> {
        self.format.get_version(&self.content, &self.path, verbose)
    }

    pub(crate) fn set_version(&mut self, dry_run: DryRun, version_str: &Version) -> Result<()> {
        self.content =
            self.format
                .set_version(dry_run, self.content.clone(), version_str, &self.path)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PackageFormat {
    Cargo,
    Go,
    JavaScript,
    Poetry,
}

impl TryFrom<&PathBuf> for PackageFormat {
    type Error = Error;

    fn try_from(path: &PathBuf) -> Result<Self> {
        let file_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| ErrorKind::NotAFile(path.clone()))?;
        PACKAGE_FORMAT_FILE_NAMES
            .iter()
            .find_position(|&name| *name == file_name)
            .and_then(|(pos, _)| ALL_PACKAGE_FORMATS.get(pos).copied())
            .ok_or_else(|| Error::from(ErrorKind::VersionedFileFormat(path.clone())))
    }
}

impl PackageFormat {
    /// Get the version from `content` for package named `name` (if any name).
    /// `path` is used for error reporting.
    pub(crate) fn get_version(
        self,
        content: &str,
        path: &Path,
        verbose: Verbose,
    ) -> Result<VersionFromSource> {
        match self {
            PackageFormat::Cargo => cargo::get_version(content, path)
                .map_err(ErrorKind::Cargo)
                .map(|version| VersionFromSource {
                    version,
                    source: path.display().to_string(),
                }),
            PackageFormat::Poetry => pyproject::get_version(content, path)
                .map_err(ErrorKind::PyProject)
                .map(|version| VersionFromSource {
                    version,
                    source: path.display().to_string(),
                }),
            PackageFormat::JavaScript => package_json::get_version(content, path)
                .map_err(ErrorKind::PackageJson)
                .map(|version| VersionFromSource {
                    version,
                    source: path.display().to_string(),
                }),
            PackageFormat::Go => go::get_version(content, path, verbose).map_err(ErrorKind::Go),
        }
        .map_err(Error::from)
    }

    /// Consume the `content` and return a version of it which contains `new_version`.
    ///
    /// `path` is only used for error reporting.
    pub(crate) fn set_version(
        self,
        dry_run: DryRun,
        content: String,
        new_version: &Version,
        path: &Path,
    ) -> Result<String> {
        match self {
            PackageFormat::Cargo => {
                cargo::set_version(dry_run, content, &new_version.to_string(), path)
                    .map_err(Error::from)
            }
            PackageFormat::Poetry => {
                pyproject::set_version(dry_run, content, &new_version.to_string(), path)
                    .map_err(Error::from)
            }
            PackageFormat::JavaScript => {
                package_json::set_version(dry_run, &content, &new_version.to_string(), path)
                    .map_err(Error::from)
            }
            PackageFormat::Go => {
                go::set_version_in_file(dry_run, &content, new_version, path).map_err(Error::from)
            }
        }
    }
}

/// A version and where it came from.
pub(crate) struct VersionFromSource {
    pub(crate) version: Version,
    pub(crate) source: String,
}

const ALL_PACKAGE_FORMATS: [PackageFormat; 4] = [
    PackageFormat::Cargo,
    PackageFormat::Go,
    PackageFormat::JavaScript,
    PackageFormat::Poetry,
];
pub(crate) const PACKAGE_FORMAT_FILE_NAMES: [&str; ALL_PACKAGE_FORMATS.len()] =
    ["Cargo.toml", "go.mod", "package.json", "pyproject.toml"];
