#![forbid(unsafe_code)]

use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::{PROFILE_MISMATCH, SCENARIO_INVALID, WaterError};
use crate::model::CanonicalSample;
use crate::profile::{
    CORPUS_ROOT_HEX, FLOAT_PROFILE_ROOT_HEX, IMPACT_ENERGY_CONTRACT_ROOT_HEX,
    IMPACT_ENERGY_CORPUS_ROOT_HEX, IMPACT_ENERGY_DOCUMENT_ROOT_HEX,
    IMPACT_ENERGY_EXECUTION_PROFILE_ROOT_HEX, IMPACT_ENERGY_SCENARIO_ROOTS_HEX,
    SUCCESSOR_CORPUS_ROOT_HEX, SUCCESSOR_DOCUMENT_ROOT_HEX, SUCCESSOR_EXECUTION_MANIFEST_ROOT_HEX,
    SUCCESSOR_EXECUTION_PROFILE_ROOT_HEX, SUCCESSOR_FIXTURE_ROOT_HEX,
    SUCCESSOR_FLOAT_PROFILE_ROOT_HEX, SUCCESSOR_GEOMETRY_ROOT_HEX, SUCCESSOR_SCENARIO_ROOTS_HEX,
    W0B_DOCUMENT_ROOT_HEX,
};

const W0B_PATH: &str = "docs/plans/continuum-water/00b-numeric-execution-and-corpus-closure.md";
const FLOAT_DOMAIN: &[u8] = b"nextengine.continuum-water.float-profile.v1\0";
const CORPUS_DOMAIN: &[u8] = b"nextengine.continuum-water.corpus.v1\0";
const EXECUTION_DOMAIN: &[u8] = b"nextengine.continuum-water.execution-profile.v1\0";
const SCENARIO_DOMAIN: &[u8] = b"nextengine.continuum-water.scenario.v1\0";
const FRAME_DOMAIN: &[u8] = b"nextengine.continuum-water.frame.v1\0";
const TRAJECTORY_DOMAIN: &[u8] = b"nextengine.continuum-water.trajectory.v1\0";
const SUCCESSOR_PATH: &str = "docs/plans/continuum-water/00f-geometry-capacity-and-root-closure.md";
const SUCCESSOR_FLOAT_DOMAIN: &[u8] = b"nextengine.continuum-water.successor-float-profile.v1\0";
const SUCCESSOR_EXECUTION_MANIFEST_DOMAIN: &[u8] =
    b"nextengine.continuum-water.successor-execution-manifest.v1\0";
const SUCCESSOR_CORPUS_DOMAIN: &[u8] = b"nextengine.continuum-water.successor-corpus.v1\0";
const SUCCESSOR_FIXTURE_DOMAIN: &[u8] = b"nextengine.continuum-water.successor-fixtures.v1\0";
const SUCCESSOR_EXECUTION_PROFILE_DOMAIN: &[u8] =
    b"nextengine.continuum-water.successor-execution-profile.v1\0";
const SUCCESSOR_SCENARIO_DOMAIN: &[u8] = b"nextengine.continuum-water.successor-scenario.v1\0";
const IMPACT_ENERGY_PATH: &str =
    "docs/plans/continuum-water/00g-impact-energy-contract-reclosure.md";
const IMPACT_ENERGY_CONTRACT_DOMAIN: &[u8] =
    b"nextengine.continuum-water.impact-energy-contract.v1\0";
const IMPACT_ENERGY_CORPUS_DOMAIN: &[u8] = b"nextengine.continuum-water.impact-energy-corpus.v1\0";
const IMPACT_ENERGY_SCENARIO_DOMAIN: &[u8] =
    b"nextengine.continuum-water.impact-energy-scenario.v1\0";
const IMPACT_ENERGY_EXECUTION_PROFILE_DOMAIN: &[u8] =
    b"nextengine.continuum-water.impact-energy-execution-profile.v1\0";

#[derive(Clone, Debug)]
pub(crate) struct FrozenRoots {
    pub(crate) document: [u8; 32],
    pub(crate) float_profile: [u8; 32],
    pub(crate) corpus: [u8; 32],
    pub(crate) execution_profile: [u8; 32],
    corpus_bytes: Vec<u8>,
}

impl FrozenRoots {
    pub(crate) fn verify(repository_root: &Path) -> Result<Self, WaterError> {
        verify_cargo_profile(repository_root)?;
        let path = repository_root.join(W0B_PATH);
        let document = fs::read(&path).map_err(|error| {
            WaterError::new(
                PROFILE_MISMATCH,
                format!("cannot read {}: {error}", path.display()),
            )
        })?;
        if document.contains(&b'\r') {
            return Err(WaterError::new(
                PROFILE_MISMATCH,
                "W0B document contains CR bytes",
            ));
        }
        let document_root = digest(&document);
        require_root("W0B document", document_root, W0B_DOCUMENT_ROOT_HEX)?;

        let float_bytes = extract_marked_block(
            &document,
            b"FLOAT_PROFILE_V1_BEGIN\n",
            b"FLOAT_PROFILE_V1_END\n",
        )?;
        let float_root = domain_digest(FLOAT_DOMAIN, float_bytes);
        require_root("float profile", float_root, FLOAT_PROFILE_ROOT_HEX)?;

        let corpus_bytes = extract_marked_block(
            &document,
            b"CORPUS_MANIFEST_V1_BEGIN\n",
            b"CORPUS_MANIFEST_V1_END\n",
        )?;
        let corpus_root = domain_digest(CORPUS_DOMAIN, corpus_bytes);
        require_root("corpus", corpus_root, CORPUS_ROOT_HEX)?;

        let mut hasher = Sha256::new();
        hasher.update(EXECUTION_DOMAIN);
        hasher.update(document_root);
        hasher.update(float_root);
        hasher.update(corpus_root);
        let execution_profile = finalize(hasher);
        Ok(Self {
            document: document_root,
            float_profile: float_root,
            corpus: corpus_root,
            execution_profile,
            corpus_bytes: corpus_bytes.to_vec(),
        })
    }

    pub(crate) fn scenario_root(&self, scenario_id: &str) -> Result<[u8; 32], WaterError> {
        Ok(domain_digest(
            SCENARIO_DOMAIN,
            &self.scenario_projection(scenario_id)?,
        ))
    }

    pub(crate) fn scenario_projection(&self, scenario_id: &str) -> Result<Vec<u8>, WaterError> {
        let prefix = format!("scenario.{scenario_id}.");
        let mut projection = Vec::new();
        let mut count = 0_u32;
        for line in self.corpus_bytes.split_inclusive(|byte| *byte == b'\n') {
            if line.starts_with(prefix.as_bytes()) {
                projection.extend_from_slice(line);
                count = count.checked_add(1).ok_or_else(|| {
                    WaterError::new(SCENARIO_INVALID, "scenario line count overflow")
                })?;
            }
        }
        if count == 0 {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                format!("scenario {scenario_id:?} is absent from the frozen corpus"),
            ));
        }
        Ok(projection)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SuccessorRoots {
    pub(crate) document: [u8; 32],
    pub(crate) float_profile: [u8; 32],
    pub(crate) execution_manifest: [u8; 32],
    pub(crate) corpus: [u8; 32],
    pub(crate) fixtures: [u8; 32],
    pub(crate) geometry: [u8; 32],
    pub(crate) execution_profile: [u8; 32],
    corpus_bytes: Vec<u8>,
}

impl SuccessorRoots {
    pub(crate) fn load(repository_root: &Path) -> Result<Self, WaterError> {
        let path = repository_root.join(SUCCESSOR_PATH);
        let document = fs::read(&path).map_err(|error| {
            WaterError::new(
                PROFILE_MISMATCH,
                format!("cannot read {}: {error}", path.display()),
            )
        })?;
        if document.contains(&b'\r') {
            return Err(WaterError::new(
                PROFILE_MISMATCH,
                "successor profile document contains CR bytes",
            ));
        }
        let float_bytes = successor_block(
            &document,
            b"SUCCESSOR_FLOAT_PROFILE_V1_BEGIN\n",
            b"SUCCESSOR_FLOAT_PROFILE_V1_END\n",
            "float profile",
        )?;
        let execution_bytes = successor_block(
            &document,
            b"SUCCESSOR_EXECUTION_PROFILE_V1_BEGIN\n",
            b"SUCCESSOR_EXECUTION_PROFILE_V1_END\n",
            "execution profile",
        )?;
        let corpus_bytes = successor_block(
            &document,
            b"SUCCESSOR_CORPUS_MANIFEST_V1_BEGIN\n",
            b"SUCCESSOR_CORPUS_MANIFEST_V1_END\n",
            "corpus manifest",
        )?;
        let fixture_bytes = successor_block(
            &document,
            b"SUCCESSOR_FIXTURE_MANIFEST_V1_BEGIN\n",
            b"SUCCESSOR_FIXTURE_MANIFEST_V1_END\n",
            "fixture manifest",
        )?;
        let document_root = digest(&document);
        let float_profile = domain_digest(SUCCESSOR_FLOAT_DOMAIN, float_bytes);
        let execution_manifest =
            domain_digest(SUCCESSOR_EXECUTION_MANIFEST_DOMAIN, execution_bytes);
        let corpus = domain_digest(SUCCESSOR_CORPUS_DOMAIN, corpus_bytes);
        let fixtures = domain_digest(SUCCESSOR_FIXTURE_DOMAIN, fixture_bytes);
        let orifice = crate::scenario::find("CW-ORIFICE-001")?;
        let geometry =
            crate::geometry::AxisAlignedGeometryManifest::from_geometry(orifice.geometry)?.root();
        let mut hasher = Sha256::new();
        hasher.update(SUCCESSOR_EXECUTION_PROFILE_DOMAIN);
        hasher.update(document_root);
        hasher.update(float_profile);
        hasher.update(execution_manifest);
        hasher.update(corpus);
        hasher.update(fixtures);
        hasher.update(geometry);
        Ok(Self {
            document: document_root,
            float_profile,
            execution_manifest,
            corpus,
            fixtures,
            geometry,
            execution_profile: finalize(hasher),
            corpus_bytes: corpus_bytes.to_vec(),
        })
    }

    pub(crate) fn verify(repository_root: &Path) -> Result<Self, WaterError> {
        let roots = Self::load(repository_root)?;
        require_root(
            "successor document",
            roots.document,
            SUCCESSOR_DOCUMENT_ROOT_HEX,
        )?;
        require_root(
            "successor float profile",
            roots.float_profile,
            SUCCESSOR_FLOAT_PROFILE_ROOT_HEX,
        )?;
        require_root(
            "successor execution manifest",
            roots.execution_manifest,
            SUCCESSOR_EXECUTION_MANIFEST_ROOT_HEX,
        )?;
        require_root("successor corpus", roots.corpus, SUCCESSOR_CORPUS_ROOT_HEX)?;
        require_root(
            "successor fixtures",
            roots.fixtures,
            SUCCESSOR_FIXTURE_ROOT_HEX,
        )?;
        require_root(
            "successor geometry",
            roots.geometry,
            SUCCESSOR_GEOMETRY_ROOT_HEX,
        )?;
        require_root(
            "successor execution profile",
            roots.execution_profile,
            SUCCESSOR_EXECUTION_PROFILE_ROOT_HEX,
        )?;
        for (scenario_id, expected) in SUCCESSOR_SCENARIO_ROOTS_HEX {
            require_root(
                &format!("successor scenario {scenario_id}"),
                roots.scenario_root(scenario_id)?,
                expected,
            )?;
        }
        Ok(roots)
    }

    pub(crate) fn scenario_root(&self, scenario_id: &str) -> Result<[u8; 32], WaterError> {
        Ok(domain_digest(
            SUCCESSOR_SCENARIO_DOMAIN,
            &self.scenario_projection(scenario_id)?,
        ))
    }

    pub(crate) fn scenario_projection(&self, scenario_id: &str) -> Result<Vec<u8>, WaterError> {
        let prefix = format!("scenario.{scenario_id}.");
        let mut projection = Vec::new();
        for line in self.corpus_bytes.split_inclusive(|byte| *byte == b'\n') {
            if line.starts_with(prefix.as_bytes()) {
                projection.extend_from_slice(line);
            }
        }
        if projection.is_empty() {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                format!("scenario {scenario_id:?} is absent from the successor corpus"),
            ));
        }
        Ok(projection)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ImpactEnergyRoots {
    pub(crate) parent: SuccessorRoots,
    pub(crate) document: [u8; 32],
    pub(crate) contract: [u8; 32],
    pub(crate) corpus: [u8; 32],
    pub(crate) execution_profile: [u8; 32],
    contract_bytes: Vec<u8>,
}

impl ImpactEnergyRoots {
    pub(crate) fn load(repository_root: &Path) -> Result<Self, WaterError> {
        let parent = SuccessorRoots::verify(repository_root)?;
        let path = repository_root.join(IMPACT_ENERGY_PATH);
        let document_bytes = fs::read(&path).map_err(|error| {
            WaterError::new(
                PROFILE_MISMATCH,
                format!("cannot read {}: {error}", path.display()),
            )
        })?;
        if document_bytes.contains(&b'\r') {
            return Err(WaterError::new(
                PROFILE_MISMATCH,
                "impact energy profile document contains CR bytes",
            ));
        }
        let contract_bytes = successor_block(
            &document_bytes,
            b"IMPACT_ENERGY_CONTRACT_V1_BEGIN\n",
            b"IMPACT_ENERGY_CONTRACT_V1_END\n",
            "impact energy contract",
        )?;
        let document = digest(&document_bytes);
        let contract = domain_digest(IMPACT_ENERGY_CONTRACT_DOMAIN, contract_bytes);

        let mut corpus_hasher = Sha256::new();
        corpus_hasher.update(IMPACT_ENERGY_CORPUS_DOMAIN);
        corpus_hasher.update(parent.corpus);
        corpus_hasher.update(contract);
        let corpus = finalize(corpus_hasher);

        let mut execution_hasher = Sha256::new();
        execution_hasher.update(IMPACT_ENERGY_EXECUTION_PROFILE_DOMAIN);
        execution_hasher.update(parent.execution_profile);
        execution_hasher.update(document);
        execution_hasher.update(contract);
        execution_hasher.update(corpus);
        let execution_profile = finalize(execution_hasher);
        Ok(Self {
            parent,
            document,
            contract,
            corpus,
            execution_profile,
            contract_bytes: contract_bytes.to_vec(),
        })
    }

    pub(crate) fn verify(repository_root: &Path) -> Result<Self, WaterError> {
        let roots = Self::load(repository_root)?;
        require_root(
            "impact energy document",
            roots.document,
            IMPACT_ENERGY_DOCUMENT_ROOT_HEX,
        )?;
        require_root(
            "impact energy contract",
            roots.contract,
            IMPACT_ENERGY_CONTRACT_ROOT_HEX,
        )?;
        require_root(
            "impact energy corpus",
            roots.corpus,
            IMPACT_ENERGY_CORPUS_ROOT_HEX,
        )?;
        require_root(
            "impact energy execution profile",
            roots.execution_profile,
            IMPACT_ENERGY_EXECUTION_PROFILE_ROOT_HEX,
        )?;
        for (scenario_id, expected) in IMPACT_ENERGY_SCENARIO_ROOTS_HEX {
            require_root(
                &format!("impact energy scenario {scenario_id}"),
                roots.scenario_root(scenario_id)?,
                expected,
            )?;
        }
        Ok(roots)
    }

    pub(crate) fn scenario_root(&self, scenario_id: &str) -> Result<[u8; 32], WaterError> {
        let parent_root = self.parent.scenario_root(scenario_id)?;
        let projection = self.scenario_projection(scenario_id)?;
        let mut hasher = Sha256::new();
        hasher.update(IMPACT_ENERGY_SCENARIO_DOMAIN);
        hasher.update(parent_root);
        hasher.update(projection);
        Ok(finalize(hasher))
    }

    pub(crate) fn scenario_projection(&self, scenario_id: &str) -> Result<Vec<u8>, WaterError> {
        let prefix = format!("scenario.{scenario_id}.energy-class=");
        let mut projection = Vec::new();
        let mut matches = 0_u8;
        for line in self.contract_bytes.split_inclusive(|byte| *byte == b'\n') {
            if line.starts_with(prefix.as_bytes()) {
                projection.extend_from_slice(line);
                matches = matches.checked_add(1).ok_or_else(|| {
                    WaterError::new(PROFILE_MISMATCH, "impact energy scenario count overflow")
                })?;
            }
        }
        if matches != 1 {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                format!(
                    "impact energy scenario {scenario_id:?} has {matches} contract projections"
                ),
            ));
        }
        Ok(projection)
    }
}

fn verify_cargo_profile(repository_root: &Path) -> Result<(), WaterError> {
    const EXPECTED: &str = r#"[profile.water-oracle]
inherits = "release"
opt-level = 3
codegen-units = 1
lto = false
incremental = false
overflow-checks = true
debug-assertions = false
panic = "abort"
"#;
    let path = repository_root.join("Cargo.toml");
    let manifest = fs::read_to_string(&path).map_err(|error| {
        WaterError::new(
            PROFILE_MISMATCH,
            format!("cannot read {}: {error}", path.display()),
        )
    })?;
    if manifest.contains(EXPECTED) {
        Ok(())
    } else {
        Err(WaterError::new(
            PROFILE_MISMATCH,
            "Cargo.toml does not contain the exact W0B water-oracle profile",
        ))
    }
}

pub(crate) fn smoke_scenario_root(lines: &[u8]) -> [u8; 32] {
    domain_digest(SCENARIO_DOMAIN, lines)
}

pub(crate) fn frame_root(
    execution_profile_root: &[u8; 32],
    scenario_root: &[u8; 32],
    step: u32,
    samples: &[CanonicalSample],
) -> Result<[u8; 32], WaterError> {
    let count = u32::try_from(samples.len())
        .map_err(|_| WaterError::new(SCENARIO_INVALID, "frame sample count does not fit u32"))?;
    for pair in samples.windows(2) {
        if pair[0].id >= pair[1].id {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                "frame samples are not in strictly ascending SampleId order",
            ));
        }
    }
    let mut hasher = Sha256::new();
    hasher.update(FRAME_DOMAIN);
    hasher.update(execution_profile_root);
    hasher.update(scenario_root);
    hasher.update(step.to_le_bytes());
    hasher.update(count.to_le_bytes());
    for sample in samples {
        hasher.update(sample.id.to_le_bytes());
        for value in [
            sample.position_um.x,
            sample.position_um.y,
            sample.position_um.z,
            sample.velocity_um_s.x,
            sample.velocity_um_s.y,
            sample.velocity_um_s.z,
        ] {
            hasher.update(value.to_le_bytes());
        }
    }
    Ok(finalize(hasher))
}

pub(crate) struct TrajectoryHasher {
    hasher: Sha256,
    expected_frames: u32,
    accepted_frames: u32,
}

impl TrajectoryHasher {
    pub(crate) fn new(expected_frames: u32) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(TRAJECTORY_DOMAIN);
        hasher.update(expected_frames.to_le_bytes());
        Self {
            hasher,
            expected_frames,
            accepted_frames: 0,
        }
    }

    pub(crate) fn push(&mut self, step: u32, frame_root: &[u8; 32]) -> Result<(), WaterError> {
        if step != self.accepted_frames {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                format!(
                    "trajectory received step {step}, expected {}",
                    self.accepted_frames
                ),
            ));
        }
        self.hasher.update(step.to_le_bytes());
        self.hasher.update(frame_root);
        self.accepted_frames = self
            .accepted_frames
            .checked_add(1)
            .ok_or_else(|| WaterError::new(SCENARIO_INVALID, "trajectory frame count overflow"))?;
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<[u8; 32], WaterError> {
        if self.accepted_frames != self.expected_frames {
            return Err(WaterError::new(
                SCENARIO_INVALID,
                format!(
                    "trajectory accepted {} frames, expected {}",
                    self.accepted_frames, self.expected_frames
                ),
            ));
        }
        Ok(finalize(self.hasher))
    }
}

pub(crate) fn hex(root: &[u8; 32]) -> String {
    let mut output = String::with_capacity(64);
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    for byte in root {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn extract_marked_block<'a>(
    document: &'a [u8],
    begin: &[u8],
    end: &[u8],
) -> Result<&'a [u8], WaterError> {
    let start = find_subslice(document, begin).ok_or_else(|| {
        WaterError::new(PROFILE_MISMATCH, "W0B projection start marker is missing")
    })?;
    let after_start = &document[start..];
    let relative_end = find_subslice(after_start, end)
        .ok_or_else(|| WaterError::new(PROFILE_MISMATCH, "W0B projection end marker is missing"))?;
    let end_index = start + relative_end + end.len();
    Ok(&document[start..end_index])
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn successor_block<'a>(
    document: &'a [u8],
    begin: &[u8],
    end: &[u8],
    label: &str,
) -> Result<&'a [u8], WaterError> {
    let start = find_subslice(document, begin).ok_or_else(|| {
        WaterError::new(
            PROFILE_MISMATCH,
            format!("successor {label} start marker is missing"),
        )
    })?;
    let tail = &document[start..];
    let relative_end = find_subslice(tail, end).ok_or_else(|| {
        WaterError::new(
            PROFILE_MISMATCH,
            format!("successor {label} end marker is missing"),
        )
    })?;
    Ok(&document[start..start + relative_end + end.len()])
}

fn require_root(label: &str, actual: [u8; 32], expected_hex: &str) -> Result<(), WaterError> {
    let actual_hex = hex(&actual);
    if actual_hex == expected_hex {
        Ok(())
    } else {
        Err(WaterError::new(
            PROFILE_MISMATCH,
            format!("{label} root {actual_hex}, expected {expected_hex}"),
        ))
    }
}

fn domain_digest(domain: &[u8], bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(bytes);
    finalize(hasher)
}

fn digest(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    finalize(hasher)
}

fn finalize(hasher: Sha256) -> [u8; 32] {
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn hex_is_lowercase_and_fixed_width() {
        let mut root = [0_u8; 32];
        root[0] = 0xab;
        root[31] = 0x05;
        assert_eq!(
            hex(&root),
            "ab00000000000000000000000000000000000000000000000000000000000005"
        );
    }

    #[test]
    fn repository_preflight_binds_all_frozen_roots() {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let roots = FrozenRoots::verify(repository_root).unwrap();
        assert_eq!(hex(&roots.document), W0B_DOCUMENT_ROOT_HEX);
        assert_eq!(hex(&roots.float_profile), FLOAT_PROFILE_ROOT_HEX);
        assert_eq!(hex(&roots.corpus), CORPUS_ROOT_HEX);
        let freefall = roots.scenario_root("CW-FREEFALL-001").unwrap();
        let hydro = roots.scenario_root("CW-HYDRO-001").unwrap();
        assert_ne!(freefall, hydro);
    }

    #[test]
    fn root_mismatch_is_a_blocking_profile_failure() {
        assert_eq!(
            require_root("altered W0B", [0_u8; 32], W0B_DOCUMENT_ROOT_HEX)
                .unwrap_err()
                .code(),
            PROFILE_MISMATCH
        );
    }

    #[test]
    fn successor_profile_has_all_domain_separated_scenario_roots() {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let roots = SuccessorRoots::verify(repository_root).unwrap();
        let hydro = roots.scenario_root("CW-HYDRO-001").unwrap();
        let freefall = roots.scenario_root("CW-FREEFALL-001").unwrap();
        let orifice = roots.scenario_root("CW-ORIFICE-001").unwrap();
        assert_ne!(hydro, freefall);
        assert_ne!(hydro, orifice);
        assert_ne!(roots.execution_profile, roots.corpus);
    }

    #[test]
    fn impact_energy_profile_has_all_domain_separated_scenario_roots() {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let roots = ImpactEnergyRoots::verify(repository_root).unwrap();
        assert_eq!(hex(&roots.document), IMPACT_ENERGY_DOCUMENT_ROOT_HEX);
        assert_eq!(hex(&roots.contract), IMPACT_ENERGY_CONTRACT_ROOT_HEX);
        assert_eq!(hex(&roots.corpus), IMPACT_ENERGY_CORPUS_ROOT_HEX);
        assert_eq!(
            hex(&roots.execution_profile),
            IMPACT_ENERGY_EXECUTION_PROFILE_ROOT_HEX
        );
        let hydro = roots.scenario_root("CW-HYDRO-001").unwrap();
        let dam_break = roots.scenario_root("CW-DAMBREAK-001").unwrap();
        assert_ne!(hydro, dam_break);
        assert_ne!(roots.execution_profile, roots.parent.execution_profile);
    }
}
