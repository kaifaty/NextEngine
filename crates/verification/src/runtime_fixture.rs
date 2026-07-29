use std::error::Error;
use std::fmt::{Display, Formatter};

use next_contracts::command::IssuerPrincipal;
use next_contracts::ids::CapabilityId;

pub type NeutralRuntimeFixture = next_reference_game::ReferenceRuntimeBootstrap;

pub fn build_neutral_runtime_fixture(
    project_id: &str,
    grants: impl IntoIterator<Item = (IssuerPrincipal, Vec<CapabilityId>)>,
) -> Result<NeutralRuntimeFixture, NeutralFixtureError> {
    next_reference_game::build_reference_runtime_bootstrap(project_id, grants)
        .map_err(NeutralFixtureError::ReferenceGame)
}

#[derive(Debug)]
pub enum NeutralFixtureError {
    ReferenceGame(next_reference_game::ReferenceGameError),
    ProjectCook(next_project::ProjectCookError),
    ProjectStore(next_assets::ContentStoreError),
    ProjectActivation(next_project::ProjectActivationError),
    Cleanup(std::io::Error),
}

impl Display for NeutralFixtureError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReferenceGame(error) => write!(formatter, "{error}"),
            Self::ProjectCook(error) => write!(formatter, "reference project cook failed: {error}"),
            Self::ProjectStore(error) => {
                write!(formatter, "reference project publication failed: {error}")
            }
            Self::ProjectActivation(error) => {
                write!(formatter, "reference project activation failed: {error}")
            }
            Self::Cleanup(error) => write!(formatter, "reference fixture cleanup failed: {error}"),
        }
    }
}

impl Error for NeutralFixtureError {}

impl From<next_reference_game::ReferenceGameError> for NeutralFixtureError {
    fn from(value: next_reference_game::ReferenceGameError) -> Self {
        Self::ReferenceGame(value)
    }
}

impl From<next_project::ProjectCookError> for NeutralFixtureError {
    fn from(value: next_project::ProjectCookError) -> Self {
        Self::ProjectCook(value)
    }
}

impl From<next_assets::ContentStoreError> for NeutralFixtureError {
    fn from(value: next_assets::ContentStoreError) -> Self {
        Self::ProjectStore(value)
    }
}

impl From<next_project::ProjectActivationError> for NeutralFixtureError {
    fn from(value: next_project::ProjectActivationError) -> Self {
        Self::ProjectActivation(value)
    }
}
