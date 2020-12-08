use color_eyre::eyre::WrapErr;
use color_eyre::eyre::{eyre, Result};
use semver::Identifier;
use serde::export::Formatter;
use serde::Deserialize;
use std::fmt::Display;

/// The various rules that can be used when bumping the current version of a project via
/// [`crate::step::Step::BumpVersion`].
///
/// A Rule should be selected by using `rule = "{rule_variant}"` in the declaring step. If the
/// variant requires an additional parameter (e.g. `Pre`), provide that with `value = "{param}"`.
///
/// ## Example
/// ```toml
/// [[workflows.steps]]
/// type = "BumpVersion"
/// rule = "Pre"
/// value = "rc"
/// ```
#[derive(Debug, Deserialize)]
#[serde(tag = "rule", content = "value")]
pub enum Rule {
    /// Increment the Major component of the semantic version and reset all other components.
    ///
    /// ## Example
    /// 1.2.3-rc.4 -> 2.0.0
    Major,
    /// Increment the Minor component of the semantic version and reset all lesser components.
    ///
    /// ## Example
    /// 1.2.3-rc.4 -> 1.3.0
    Minor,
    /// Increment the Patch component of the semantic version and reset all lesser components.
    ///
    /// ## Example
    /// 1.2.3-rc.4 -> 1.2.4
    Patch,
    /// Increment the pre-release component of the semantic version or add it if missing.
    ///
    /// ## Example
    /// 1.2.3-rc.4 -> 1.2.3-rc.5
    /// 1.2.3 -> 1.2.3-rc.0
    Pre(String),
    /// Remove the pre-release component of the semantic version.
    ///
    /// ## Example
    /// 1.2.3-rc.4 -> 1.2.3
    Release,
}

pub(crate) enum Version {
    Cargo(semver::Version),
}

impl Version {
    fn run_on_inner<F: FnOnce(semver::Version) -> Result<semver::Version>>(
        self,
        func: F,
    ) -> Result<Self> {
        Ok(match self {
            Version::Cargo(version) => Version::Cargo(func(version)?),
        })
    }

    fn reset_pre(self) -> Self {
        match self {
            Version::Cargo(mut version) => Version::Cargo({
                version.pre = Vec::new();
                version
            }),
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::Cargo(v) => write!(f, "{}", v.to_string()),
        }
    }
}

pub(crate) fn bump_version(state: crate::State, rule: Rule) -> Result<crate::State> {
    if let Ok(version) = get_version() {
        let version = bump(version, &rule).wrap_err("While bumping version")?;
        set_version(version)?;
    }
    Ok(state)
}

pub(crate) fn get_version() -> Result<Version> {
    if let Some(cargo_version) = crate::cargo::get_version() {
        let version = semver::Version::parse(&cargo_version).wrap_err_with(|| {
            format!(
                "Found {} in Cargo.toml which is not a valid version",
                cargo_version
            )
        })?;
        Ok(Version::Cargo(version))
    } else {
        Err(eyre!("No supported metadata found to parse version from"))
    }
}

fn set_version(version: Version) -> Result<()> {
    match version {
        Version::Cargo(version) => {
            crate::cargo::set_version(&version.to_string()).wrap_err("While bumping Cargo.toml")
        }
    }
}

fn bump(version: Version, rule: &Rule) -> Result<Version> {
    match rule {
        Rule::Major => version.run_on_inner(|mut v| {
            v.increment_major();
            Ok(v)
        }),
        Rule::Minor => version.run_on_inner(|mut v| {
            v.increment_minor();
            Ok(v)
        }),
        Rule::Patch => version.run_on_inner(|mut v| {
            v.increment_patch();
            Ok(v)
        }),
        Rule::Release => Ok(version.reset_pre()),
        Rule::Pre(prefix) => version.run_on_inner(|v| bump_pre(v, prefix)),
    }
}

fn bump_pre(mut version: semver::Version, prefix: &str) -> Result<semver::Version> {
    if version.pre.is_empty() {
        version.pre = vec![
            Identifier::AlphaNumeric(prefix.to_owned()),
            Identifier::Numeric(0),
        ];
        return Ok(version);
    } else if version.pre.len() != 2 {
        return Err(eyre!(
            "A prerelease version already exists but could not be incremented"
        ));
    }
    if let Some(Identifier::AlphaNumeric(existing_prefix)) = version.pre.get(0) {
        if existing_prefix != prefix {
            return Err(eyre!(
                "Found prefix {} which does not match provided prefix {}",
                existing_prefix,
                prefix
            ));
        }
    } else {
        return Err(eyre!(
            "A prerelease version already exists but could not be incremented"
        ));
    }
    match version.pre.remove(1) {
        Identifier::Numeric(pre_version) => {
            version.pre.insert(1, Identifier::Numeric(pre_version + 1));
            Ok(version)
        }
        _ => Err(eyre!("No numeric pre component to bump")),
    }
}
