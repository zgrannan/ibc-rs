//! Cosmos-SDK compatibility constants and diagnostic methods.
use thiserror::Error;
use tracing::debug;
use super::version;
/// Specifies the SDK module version requirement.
///
/// # Note: Should be consistent with [features] guide page.
///
/// [features]: https://hermes.informal.systems/features.html
const SDK_MODULE_VERSION_REQ: &str = ">=0.41, <0.46";
/// Specifies the IBC-go module version requirement.
/// At the moment, we support both chains with and without
/// the standalone ibc-go module, i.e., it's not an error
/// if the chain binary does not build with this module.
///
/// # Note: Should be consistent with [features] guide page.
///
/// [features]: https://hermes.informal.systems/features.html
const IBC_GO_MODULE_VERSION_REQ: &str = ">=1.1, <=3";
#[derive(Error, Debug)]
pub enum Diagnostic {
    #[error(
        "SDK module at version '{found}' does not meet compatibility requirements {requirements}"
    )]
    MismatchingSdkModuleVersion { requirements: String, found: String },
    #[error(
        "Ibc-Go module at version '{found}' does not meet compatibility requirements {requirements}"
    )]
    MismatchingIbcGoModuleVersion { requirements: String, found: String },
}
/// Runs a diagnostic check on the provided [`VersionInfo`]
/// to ensure that the Sdk & IBC-go modules version match
/// the predefined requirements.
///
/// Returns `None` upon success, or a [`Diagnostic`] upon
/// an error.
///
/// Relies on the constant [`SDK_MODULE_NAME`] to find the
/// Sdk module by name, as well as the constants
/// [`SDK_MODULE_VERSION_REQ`] and [`IBC_GO_MODULE_VERSION_REQ`]
/// for establishing compatibility requirements.
#[prusti_contracts::trusted]
pub(crate) fn run_diagnostic(v: &version::Specs) -> Result<(), Diagnostic> {
    debug!("running diagnostic on version info {:?}", v);
    sdk_diagnostic(v.sdk_version.clone())?;
    ibc_go_diagnostic(v.ibc_go_version.clone())?;
    Ok(())
}
#[prusti_contracts::trusted]
fn sdk_diagnostic(version: semver::Version) -> Result<(), Diagnostic> {
    let sdk_reqs = semver::VersionReq::parse(SDK_MODULE_VERSION_REQ)
        .expect("parsing the SDK module requirements into semver");
    match sdk_reqs.matches(&version) {
        true => Ok(()),
        false => {
            Err(Diagnostic::MismatchingSdkModuleVersion {
                requirements: SDK_MODULE_VERSION_REQ.to_string(),
                found: version.to_string(),
            })
        }
    }
}
#[prusti_contracts::trusted]
fn ibc_go_diagnostic(version_info: Option<semver::Version>) -> Result<(), Diagnostic> {
    let ibc_reqs = semver::VersionReq::parse(IBC_GO_MODULE_VERSION_REQ)
        .expect("parsing the IBC-Go module requirements into semver");
    match version_info {
        None => Ok(()),
        Some(v) => {
            match ibc_reqs.matches(&v) {
                true => Ok(()),
                false => {
                    Err(Diagnostic::MismatchingIbcGoModuleVersion {
                        requirements: IBC_GO_MODULE_VERSION_REQ.to_string(),
                        found: v.to_string(),
                    })
                }
            }
        }
    }
}

