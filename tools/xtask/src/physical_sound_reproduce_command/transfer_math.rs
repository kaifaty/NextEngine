use super::ResidualReport;

const AUDITION_PEAK_SAMPLE: f32 = 29_490.0;

pub(super) fn calculate_residual_f32(
    reference: &[f32],
    rendered: &[f32],
) -> Result<(usize, ResidualReport), String> {
    let reference = reference.iter().map(|sample| f64::from(*sample));
    calculate_residual_iter(reference, rendered)
}

pub(super) fn calculate_residual(
    reference: &[f64],
    rendered: &[f32],
) -> Result<(usize, ResidualReport), String> {
    calculate_residual_iter(reference.iter().copied(), rendered)
}

fn calculate_residual_iter(
    reference: impl ExactSizeIterator<Item = f64>,
    rendered: &[f32],
) -> Result<(usize, ResidualReport), String> {
    if reference.len() != rendered.len() || rendered.is_empty() {
        return Err("rendered/reference sample counts differ or are empty".to_owned());
    }
    let mut exact_sample_count = 0_usize;
    let mut error_energy = 0.0_f64;
    let mut reference_energy = 0.0_f64;
    let mut rendered_energy = 0.0_f64;
    let mut cross = 0.0_f64;
    let mut maximum_absolute = 0.0_f64;
    for (reference, rendered) in reference.zip(rendered) {
        let rendered = f64::from(*rendered);
        if reference.to_bits() == rendered.to_bits() {
            exact_sample_count = exact_sample_count.saturating_add(1);
        }
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
        return Err("non-finite reproduction residual".to_owned());
    }
    Ok((
        exact_sample_count,
        ResidualReport {
            maximum_absolute,
            rms,
            signal_to_noise_db,
            correlation,
        },
    ))
}

pub(super) fn scale_sample_count(
    sample_count: usize,
    source_sample_rate_hz: u32,
    target_sample_rate_hz: u32,
) -> Result<usize, String> {
    if source_sample_rate_hz == 0 || target_sample_rate_hz == 0 {
        return Err("sample rates must be nonzero".to_owned());
    }
    let numerator = sample_count
        .checked_mul(target_sample_rate_hz as usize)
        .ok_or_else(|| "resampled sample count overflow".to_owned())?;
    let source_sample_rate_hz = source_sample_rate_hz as usize;
    if !numerator.is_multiple_of(source_sample_rate_hz) {
        return Err("sample count does not preserve an exact duration at target rate".to_owned());
    }
    Ok(numerator / source_sample_rate_hz)
}

pub(super) fn resample_zero_extended_linear(
    samples: &[f64],
    source_sample_rate_hz: u32,
    target_sample_rate_hz: u32,
) -> Result<Vec<f64>, String> {
    if samples.iter().any(|sample| !sample.is_finite()) {
        return Err("cannot resample a non-finite transient".to_owned());
    }
    let target_sample_count =
        scale_sample_count(samples.len(), source_sample_rate_hz, target_sample_rate_hz)?;
    let source_rate = source_sample_rate_hz as usize;
    let target_rate = target_sample_rate_hz as usize;
    (0..target_sample_count)
        .map(|target_index| {
            let source_position_numerator = target_index
                .checked_mul(source_rate)
                .ok_or_else(|| "resampling position overflow".to_owned())?;
            let left_index = source_position_numerator / target_rate;
            let remainder = source_position_numerator % target_rate;
            let left = samples.get(left_index).copied().unwrap_or(0.0);
            let right = samples
                .get(left_index.saturating_add(1))
                .copied()
                .unwrap_or(0.0);
            let fraction = remainder as f64 / target_rate as f64;
            Ok(left + (right - left) * fraction)
        })
        .collect()
}

pub(super) fn normalized_mono_to_stereo_s16(samples: &[f32]) -> Result<Vec<i16>, String> {
    let mut stereo = Vec::with_capacity(
        samples
            .len()
            .checked_mul(2)
            .ok_or_else(|| "audition sample count overflow".to_owned())?,
    );
    for sample in samples {
        if !sample.is_finite() || sample.abs() > 1.000_001 {
            return Err("audition sample is non-finite or outside normalized range".to_owned());
        }
        let sample = (*sample * AUDITION_PEAK_SAMPLE).round() as i16;
        stereo.extend_from_slice(&[sample, sample]);
    }
    Ok(stereo)
}

pub(super) fn concatenate_audition_pair(
    first: &[i16],
    second: &[i16],
    sample_rate_hz: u32,
    silence_milliseconds: u32,
) -> Result<Vec<i16>, String> {
    if !first.len().is_multiple_of(2) || !second.len().is_multiple_of(2) {
        return Err("audition pair must contain stereo samples".to_owned());
    }
    let silence_frames = usize::try_from(
        u64::from(sample_rate_hz)
            .checked_mul(u64::from(silence_milliseconds))
            .ok_or_else(|| "audition silence length overflow".to_owned())?
            / 1_000,
    )
    .map_err(|_| "audition silence length exceeds usize".to_owned())?;
    let silence_samples = silence_frames
        .checked_mul(2)
        .ok_or_else(|| "audition silence sample count overflow".to_owned())?;
    let capacity = first
        .len()
        .checked_add(silence_samples)
        .and_then(|length| length.checked_add(second.len()))
        .ok_or_else(|| "audition pair sample count overflow".to_owned())?;
    let mut combined = Vec::with_capacity(capacity);
    combined.extend_from_slice(first);
    combined.resize(first.len() + silence_samples, 0);
    combined.extend_from_slice(second);
    Ok(combined)
}
