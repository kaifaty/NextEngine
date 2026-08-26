use super::*;

pub(super) fn evaluate_force_control(
    profile: &ExternalProfile,
    rendered: &BTreeMap<String, RenderedCondition>,
) -> Result<ForceControlReport, String> {
    let mut maximum_modal_error = 0.0_f64;
    let mut maximum_waveform_error = 0.0_f64;
    for strike in &profile.strikes {
        let conditions = profile
            .impulses
            .iter()
            .map(|impulse| {
                let id = format!("{}--{}", strike.id, impulse.id);
                let condition = find_condition(profile, &id)?;
                let rendered = rendered
                    .get(&id)
                    .ok_or_else(|| format!("missing rendered condition {id}"))?;
                Ok((impulse, condition, rendered))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let baseline = conditions[0].1;
        let baseline_rms = signal_rms(&conditions[0].2.reference);
        for (impulse, condition, render) in conditions.iter().skip(1) {
            let expected_ratio = impulse.newton_seconds / conditions[0].0.newton_seconds;
            for (baseline_amplitude, amplitude) in
                baseline.amplitudes.iter().zip(&condition.amplitudes)
            {
                let expected = baseline_amplitude * expected_ratio;
                let scale = expected.abs().max(amplitude.abs()).max(1.0e-15);
                maximum_modal_error = maximum_modal_error.max((amplitude - expected).abs() / scale);
            }
            let actual_ratio = signal_rms(&render.reference) / baseline_rms;
            maximum_waveform_error =
                maximum_waveform_error.max((actual_ratio - expected_ratio).abs() / expected_ratio);
        }
    }
    let status = if maximum_modal_error <= FORCE_RELATION_TOLERANCE
        && maximum_waveform_error <= FORCE_RELATION_TOLERANCE
    {
        "PASS"
    } else {
        "FORCE_ORDER_OR_SCALE_FAILED"
    };
    Ok(ForceControlReport {
        status,
        relation: "same strike: modal amplitudes and raw waveform RMS scale linearly with impulse; frequencies and damping remain shared",
        maximum_modal_scale_relative_error: maximum_modal_error,
        maximum_waveform_rms_ratio_relative_error: maximum_waveform_error,
    })
}

pub(super) fn evaluate_position_control(
    profile: &ExternalProfile,
) -> Result<PositionControlReport, String> {
    let low_force = &profile.impulses[0].id;
    let vectors = profile
        .strikes
        .iter()
        .map(|strike| {
            let condition = find_condition(profile, &format!("{}--{low_force}", strike.id))?;
            Ok((strike.id.as_str(), condition.amplitudes.as_slice()))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut minimum = f64::INFINITY;
    let mut maximum = 0.0_f64;
    let mut pair_count = 0_usize;
    for left in 0..vectors.len() {
        for right in left + 1..vectors.len() {
            let distance = cosine_distance(vectors[left].1, vectors[right].1)?;
            minimum = minimum.min(distance);
            maximum = maximum.max(distance);
            pair_count += 1;
        }
    }
    let status = if minimum >= POSITION_COSINE_DISTANCE_MINIMUM {
        "PASS"
    } else {
        "POSITION_RESPONSE_MISSING"
    };
    Ok(PositionControlReport {
        status,
        relation: "strike position changes normalized modal participation while shared object frequencies/damping stay fixed",
        pair_count,
        minimum_cosine_distance: minimum,
        maximum_cosine_distance: maximum,
        required_minimum_cosine_distance: POSITION_COSINE_DISTANCE_MINIMUM,
    })
}

pub(super) fn render_heldout_interpolation(
    profile: &ExternalProfile,
    rendered: &BTreeMap<String, RenderedCondition>,
    output: &Path,
    manifest_entries: &mut Vec<QualityManifestEntry>,
) -> Result<(HeldoutInterpolationReport, BTreeMap<String, Vec<f64>>), String> {
    let heldout_strike = profile
        .strikes
        .iter()
        .find(|strike| strike.id == profile.heldout.position_holdout_strike_id)
        .ok_or_else(|| "declared heldout strike is missing".to_owned())?;
    let train_strikes = profile
        .strikes
        .iter()
        .filter(|strike| strike.role == "train")
        .collect::<Vec<_>>();
    let mut inverse_squared = train_strikes
        .iter()
        .map(|strike| {
            let distance = point_distance(&heldout_strike.point_m, &strike.point_m);
            if distance <= f64::EPSILON {
                return Err("heldout strike collides with a train strike".to_owned());
            }
            Ok((strike.id.as_str(), 1.0 / distance.powi(2)))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let weight_sum = inverse_squared
        .iter()
        .map(|(_, weight)| *weight)
        .sum::<f64>();
    for (_, weight) in &mut inverse_squared {
        *weight /= weight_sum;
    }
    let weights = inverse_squared
        .iter()
        .map(|(id, weight)| ((*id).to_owned(), *weight))
        .collect::<BTreeMap<_, _>>();

    let mut condition_reports = Vec::new();
    let mut audition_signals = BTreeMap::new();
    for impulse in &profile.impulses {
        let heldout_id = format!("{}--{}", heldout_strike.id, impulse.id);
        let exact_condition = find_condition(profile, &heldout_id)?;
        let reference = &rendered
            .get(&heldout_id)
            .ok_or_else(|| format!("missing heldout reference {heldout_id}"))?
            .reference;
        let mut predicted_amplitudes = vec![0.0_f64; profile.modes.len()];
        for (train_id, weight) in &inverse_squared {
            let train_condition = find_condition(profile, &format!("{train_id}--{}", impulse.id))?;
            for (predicted, amplitude) in predicted_amplitudes
                .iter_mut()
                .zip(&train_condition.amplitudes)
            {
                *predicted += weight * amplitude;
            }
        }
        let predicted_condition = ExternalCondition {
            id: format!("idw--{heldout_id}"),
            split: exact_condition.split.clone(),
            strike_id: heldout_strike.id.clone(),
            force_id: impulse.id.clone(),
            impulse_newton_seconds: impulse.newton_seconds,
            amplitudes: predicted_amplitudes,
        };
        let modes = condition_modes(profile, &predicted_condition, 1.0)?;
        let predicted = render_offline_modal_recurrence_unscaled(
            profile.sample_rate_hz,
            profile.frame_count,
            &modes,
            &[],
        )
        .map_err(|error| error.to_string())?;
        validate_pcm_range(&predicted, &predicted_condition.id)?;
        let residual = calculate_residual(reference, &predicted)?;
        let file = format!("candidate-idw-{heldout_id}.wav");
        let wav = encode_float32_mono_wav(profile.sample_rate_hz, &predicted)?;
        write_output(output, &file, &wav)?;
        let hash = sha256_hex(&wav);
        let rendered_reference = rendered
            .get(&heldout_id)
            .ok_or_else(|| format!("missing heldout render {heldout_id}"))?;
        manifest_entries.push(QualityManifestEntry {
            id: format!("idw--{heldout_id}"),
            object_id: profile.object_id.clone(),
            material: "glass",
            impact_position: heldout_strike.id.clone(),
            force_band: impulse.id.clone(),
            candidate: QualityAudioRef {
                path: output.join(&file).display().to_string(),
                sha256: hash,
            },
            reference: Some(QualityAudioRef {
                path: output
                    .join(&rendered_reference.reference_file)
                    .display()
                    .to_string(),
                sha256: rendered_reference.reference_sha256.clone(),
            }),
        });
        audition_signals.insert(impulse.id.clone(), predicted);
        condition_reports.push(HeldoutConditionReport {
            condition_id: heldout_id,
            force_id: impulse.id.clone(),
            residual,
        });
    }
    Ok((
        HeldoutInterpolationReport {
            status: "MEASURED_NOT_GATING",
            claim: "IDW is a deliberately simple heldout baseline; its residual measures spatial-transfer difficulty and is not a quality pass",
            strike_id: heldout_strike.id.clone(),
            weights,
            conditions: condition_reports,
        },
        audition_signals,
    ))
}

pub(super) fn write_auditions(
    profile: &ExternalProfile,
    rendered: &BTreeMap<String, RenderedCondition>,
    heldout_predictions: &BTreeMap<String, Vec<f64>>,
    output: &Path,
) -> Result<AuditionReport, String> {
    let heldout = &profile.heldout.position_holdout_strike_id;
    let medium_id = format!("{heldout}--{}", profile.heldout.force_holdout_id);
    let exact_medium = &rendered
        .get(&medium_id)
        .ok_or_else(|| "heldout medium reference is missing".to_owned())?
        .reference;
    let predicted_medium = heldout_predictions
        .get(&profile.heldout.force_holdout_id)
        .ok_or_else(|| "heldout medium prediction is missing".to_owned())?;
    let exact_then_idw = concatenate_mono(
        &[exact_medium.as_slice(), predicted_medium.as_slice()],
        profile.sample_rate_hz,
        AUDITION_SILENCE_MILLISECONDS,
    )?;
    let pair_file = "audition-heldout-medium-exact-then-idw.wav";
    let pair_wav = encode_float32_mono_wav(profile.sample_rate_hz, &exact_then_idw)?;
    write_output(output, pair_file, &pair_wav)?;

    let force_signals = profile
        .impulses
        .iter()
        .map(|impulse| {
            rendered
                .get(&format!("{heldout}--{}", impulse.id))
                .map(|condition| condition.reference.as_slice())
                .ok_or_else(|| format!("heldout force reference is missing: {}", impulse.id))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let force_ladder = concatenate_mono(
        &force_signals,
        profile.sample_rate_hz,
        AUDITION_SILENCE_MILLISECONDS,
    )?;
    let force_file = "audition-heldout-force-low-medium-high.wav";
    let force_wav = encode_float32_mono_wav(profile.sample_rate_hz, &force_ladder)?;
    write_output(output, force_file, &force_wav)?;
    Ok(AuditionReport {
        heldout_exact_then_idw_file: pair_file,
        heldout_exact_then_idw_sha256: sha256_hex(&pair_wav),
        heldout_force_ladder_file: force_file,
        heldout_force_ladder_sha256: sha256_hex(&force_wav),
        order: "exact heldout then IDW prediction; separate force ladder is low, medium, high",
    })
}

pub(super) fn find_condition<'a>(
    profile: &'a ExternalProfile,
    id: &str,
) -> Result<&'a ExternalCondition, String> {
    profile
        .conditions
        .binary_search_by(|condition| condition.id.as_str().cmp(id))
        .map(|index| &profile.conditions[index])
        .map_err(|_| format!("missing controlled glass condition: {id}"))
}

pub(super) fn expected_split(strike_role: &str, force_id: &str) -> &'static str {
    match (strike_role, force_id) {
        ("train", "low" | "high") => "train",
        ("train", "medium") => "force-holdout",
        ("heldout", "low" | "high") => "position-holdout",
        ("heldout", "medium") => "joint-holdout",
        _ => "invalid",
    }
}

pub(super) fn calculate_residual(
    reference: &[f64],
    rendered: &[f64],
) -> Result<ResidualReport, String> {
    if reference.len() != rendered.len() || rendered.is_empty() {
        return Err("rendered/reference sample counts differ or are empty".to_owned());
    }
    let mut error_energy = 0.0_f64;
    let mut reference_energy = 0.0_f64;
    let mut rendered_energy = 0.0_f64;
    let mut cross = 0.0_f64;
    let mut maximum_absolute = 0.0_f64;
    for (reference, rendered) in reference.iter().zip(rendered) {
        let error = rendered - reference;
        maximum_absolute = maximum_absolute.max(error.abs());
        error_energy += error * error;
        reference_energy += reference * reference;
        rendered_energy += rendered * rendered;
        cross += reference * rendered;
    }
    let rms = (error_energy / rendered.len() as f64).sqrt();
    let signal_to_noise_db = if error_energy > f64::EPSILON {
        Some(10.0 * (reference_energy / error_energy).log10())
    } else {
        None
    };
    let correlation = cross / (reference_energy * rendered_energy).sqrt();
    if !maximum_absolute.is_finite() || !rms.is_finite() || !correlation.is_finite() {
        return Err("non-finite controlled-corpus residual".to_owned());
    }
    Ok(ResidualReport {
        maximum_absolute,
        rms,
        signal_to_noise_db,
        correlation,
    })
}

pub(super) fn signal_rms(samples: &[f64]) -> f64 {
    (samples.iter().map(|sample| sample * sample).sum::<f64>() / samples.len() as f64).sqrt()
}

pub(super) fn cosine_distance(left: &[f64], right: &[f64]) -> Result<f64, String> {
    if left.len() != right.len() || left.is_empty() {
        return Err("modal participation vectors differ in size or are empty".to_owned());
    }
    let dot = left
        .iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum::<f64>();
    let left_norm = left.iter().map(|value| value * value).sum::<f64>().sqrt();
    let right_norm = right.iter().map(|value| value * value).sum::<f64>().sqrt();
    let cosine = dot / (left_norm * right_norm);
    if !cosine.is_finite() {
        return Err("modal participation cosine is non-finite".to_owned());
    }
    Ok(1.0 - cosine.clamp(-1.0, 1.0))
}

pub(super) fn point_distance(left: &[f64; 3], right: &[f64; 3]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| (left - right).powi(2))
        .sum::<f64>()
        .sqrt()
}

pub(super) fn vector_norm(vector: &[f64; 3]) -> f64 {
    vector.iter().map(|value| value * value).sum::<f64>().sqrt()
}

pub(super) fn finite_positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

pub(super) fn validate_pcm_range(samples: &[f64], id: &str) -> Result<(), String> {
    if samples
        .iter()
        .any(|sample| !sample.is_finite() || sample.abs() > 0.950_001)
    {
        return Err(format!("controlled glass PCM is invalid or clips: {id}"));
    }
    Ok(())
}

pub(super) fn concatenate_mono(
    clips: &[&[f64]],
    sample_rate_hz: u32,
    silence_milliseconds: u32,
) -> Result<Vec<f64>, String> {
    if clips.is_empty() || clips.iter().any(|clip| clip.is_empty()) {
        return Err("audition requires non-empty mono clips".to_owned());
    }
    let silence =
        usize::try_from(u64::from(sample_rate_hz) * u64::from(silence_milliseconds) / 1_000)
            .map_err(|_| "audition silence exceeds usize".to_owned())?;
    let capacity = clips.iter().try_fold(0_usize, |total, clip| {
        total
            .checked_add(clip.len())
            .and_then(|value| value.checked_add(silence))
            .ok_or_else(|| "audition length overflow".to_owned())
    })?;
    let mut output = Vec::with_capacity(capacity);
    for (index, clip) in clips.iter().enumerate() {
        if index > 0 {
            output.resize(output.len() + silence, 0.0);
        }
        output.extend_from_slice(clip);
    }
    Ok(output)
}

pub(super) fn encode_float32_mono_wav(
    sample_rate_hz: u32,
    samples: &[f64],
) -> Result<Vec<u8>, String> {
    let data_bytes = u32::try_from(
        samples
            .len()
            .checked_mul(4)
            .ok_or_else(|| "float WAV byte length overflow".to_owned())?,
    )
    .map_err(|_| "float WAV exceeds RIFF size limit".to_owned())?;
    let mut bytes = Vec::with_capacity(44 + data_bytes as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&data_bytes.wrapping_add(36).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&3_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&sample_rate_hz.to_le_bytes());
    bytes.extend_from_slice(&(sample_rate_hz * 4).to_le_bytes());
    bytes.extend_from_slice(&4_u16.to_le_bytes());
    bytes.extend_from_slice(&32_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_bytes.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&(*sample as f32).to_le_bytes());
    }
    Ok(bytes)
}

pub(super) fn canonical_external_input(
    root: &Path,
    path: &Path,
    role: &str,
) -> Result<PathBuf, String> {
    let unresolved = if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    };
    let resolved = fs::canonicalize(&unresolved)
        .map_err(|error| format!("canonicalize {role} {}: {error}", unresolved.display()))?;
    if resolved.starts_with(root) || !resolved.is_file() {
        return Err(format!(
            "physical-sound-corpus {role} must be a file outside the repository: {}",
            resolved.display()
        ));
    }
    Ok(resolved)
}

pub(super) fn canonical_sibling(parent: &Path, file: &str) -> Result<PathBuf, String> {
    let resolved = fs::canonicalize(parent.join(file))
        .map_err(|error| format!("canonicalize corpus artifact {file}: {error}"))?;
    if resolved.parent() != Some(parent) || !resolved.is_file() {
        return Err(format!(
            "corpus artifact is not a regular sibling file: {file}"
        ));
    }
    Ok(resolved)
}

pub(super) fn validate_sibling_file_name(file: &str) -> Result<(), String> {
    let path = Path::new(file);
    if path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
    {
        return Err(format!("corpus artifact must be a sibling file: {file}"));
    }
    Ok(())
}

pub(super) fn bounded_read(path: &Path, maximum: u64, role: &str) -> Result<Vec<u8>, String> {
    let metadata =
        fs::metadata(path).map_err(|error| format!("stat {}: {error}", path.display()))?;
    if metadata.len() == 0 || metadata.len() > maximum {
        return Err(format!(
            "{role} must be 1..={maximum} bytes: {}",
            path.display()
        ));
    }
    fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))
}

pub(super) fn resolve_external_output(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let unresolved = if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    };
    let resolved = if unresolved.exists() {
        fs::canonicalize(&unresolved)
            .map_err(|error| format!("canonicalize {}: {error}", unresolved.display()))?
    } else {
        let parent = unresolved
            .parent()
            .ok_or_else(|| "output path has no parent directory".to_owned())?;
        let parent = fs::canonicalize(parent)
            .map_err(|error| format!("canonicalize output parent {}: {error}", parent.display()))?;
        parent.join(
            unresolved
                .file_name()
                .ok_or_else(|| "output path has no directory name".to_owned())?,
        )
    };
    if resolved.starts_with(root) {
        return Err(format!(
            "physical-sound-corpus output must stay outside the repository: {}",
            resolved.display()
        ));
    }
    Ok(resolved)
}

pub(super) fn require_empty_output(output: &Path) -> Result<(), String> {
    if !output.exists() {
        return Ok(());
    }
    if !output.is_dir()
        || fs::read_dir(output)
            .map_err(|error| format!("read {}: {error}", output.display()))?
            .next()
            .is_some()
    {
        return Err(format!(
            "physical-sound-corpus output must be an empty directory: {}",
            output.display()
        ));
    }
    Ok(())
}

pub(super) fn write_output(output: &Path, file: &str, bytes: &[u8]) -> Result<(), String> {
    fs::write(output.join(file), bytes).map_err(|error| format!("write {file}: {error}"))
}

pub(super) fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    ContentHash::from_bytes(sha256(bytes)).to_hex()
}
