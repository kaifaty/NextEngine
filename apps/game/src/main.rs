#![forbid(unsafe_code)]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = GameOptions::parse(std::env::args().skip(1))?;
    let (prepared, project_lock) = match options.project {
        Some(project_path) => {
            let store = next_assets::ContentStore::new(project_path);
            let activated = next_project::activate_project(&store)?;
            let actual_lock = activated.composition_lock.composition_lock_sha256.to_hex();
            if let Some(expected_lock) = options.expected_lock
                && expected_lock != actual_lock
            {
                return Err(format!(
                    "project lock mismatch: expected {expected_lock}, activated {actual_lock}"
                )
                .into());
            }
            (
                next_verification::prepare_game_frame_with_activated_project(activated)?,
                Some(actual_lock),
            )
        }
        None => (next_verification::prepare_game_frame()?, None),
    };
    if options.interactive {
        run_interactive(&prepared.snapshot, options.maximum_frames)?;
    }
    let report = prepared.check;
    let pose = report.play.final_pose.translation_micrometres;
    let project_lock_json =
        project_lock.map_or_else(|| "null".to_owned(), |lock| format!("\"{lock}\""));
    println!(
        "{{\"status\":\"PASS\",\"adapter\":\"reference-b0\",\"project_lock\":{},\"ticks\":{},\"final_pose_um\":[{},{},{}],\"rendered_objects\":{},\"presentation_snapshot_hash\":\"{}\",\"frame_plan_hash\":\"{}\",\"ledger_hash\":\"{}\",\"state_root\":\"{}\"}}",
        project_lock_json,
        report.play.ticks,
        pose[0],
        pose[1],
        pose[2],
        report.rendered_object_count,
        report.presentation_snapshot_hash.to_hex(),
        report.frame_plan_hash.to_hex(),
        report.play.final_command_ledger_hash.to_hex(),
        report.play.final_state_root.to_hex(),
    );
    Ok(())
}

#[derive(Debug, Default)]
struct GameOptions {
    interactive: bool,
    maximum_frames: Option<u64>,
    project: Option<std::path::PathBuf>,
    expected_lock: Option<String>,
}

impl GameOptions {
    fn parse(
        mut arguments: impl Iterator<Item = String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut options = Self::default();
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--interactive" => {
                    if std::mem::replace(&mut options.interactive, true) {
                        return Err("--interactive may only be specified once".into());
                    }
                }
                "--maximum-frames" => {
                    let value = arguments
                        .next()
                        .ok_or("--maximum-frames requires a positive integer")?;
                    let maximum = value
                        .parse::<u64>()
                        .map_err(|_| "--maximum-frames requires a positive integer")?;
                    if maximum == 0 {
                        return Err("--maximum-frames requires a positive integer".into());
                    }
                    if options.maximum_frames.replace(maximum).is_some() {
                        return Err("--maximum-frames may only be specified once".into());
                    }
                }
                "--project" => {
                    let value = arguments
                        .next()
                        .ok_or("--project requires a cooked content-store path")?;
                    if options.project.replace(value.into()).is_some() {
                        return Err("--project may only be specified once".into());
                    }
                }
                "--lock" => {
                    let value = arguments
                        .next()
                        .ok_or("--lock requires a 64-character lowercase hex digest")?;
                    validate_lock(&value)?;
                    if options.expected_lock.replace(value).is_some() {
                        return Err("--lock may only be specified once".into());
                    }
                }
                "--help" | "-h" => {
                    println!(
                        "usage: next_game [--interactive [--maximum-frames <positive-integer>]] [--project <cooked-store>] [--lock <sha256>]"
                    );
                    std::process::exit(0);
                }
                _ => return Err(format!("unsupported argument: {argument}").into()),
            }
        }
        if options.project.is_none() && options.expected_lock.is_some() {
            return Err("--lock requires --project".into());
        }
        if options.maximum_frames.is_some() && !options.interactive {
            return Err("--maximum-frames requires --interactive".into());
        }
        Ok(options)
    }
}

fn validate_lock(value: &str) -> Result<(), Box<dyn std::error::Error>> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err("--lock must be a 64-character lowercase hex digest".into())
    }
}

#[cfg(feature = "desktop-sdl-ash")]
fn run_interactive(
    snapshot: &next_contracts::PresentationSnapshotV2,
    maximum_frames: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let report = next_desktop_sdl_ash::run_interactive(
        snapshot,
        &next_desktop_sdl_ash::DesktopRunOptions {
            maximum_frames,
            ..next_desktop_sdl_ash::DesktopRunOptions::default()
        },
    )?;
    eprintln!(
        "desktop session closed: frames={}, resizes={}, focus_events={}, b0={}",
        report.rendered_frames,
        report.resize_events,
        report.focus_events,
        report.b0_capabilities_verified,
    );
    Ok(())
}

#[cfg(not(feature = "desktop-sdl-ash"))]
fn run_interactive(
    _snapshot: &next_contracts::PresentationSnapshotV2,
    _maximum_frames: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    Err(
        "interactive desktop adapter is not enabled; rebuild with --features desktop-sdl-ash"
            .into(),
    )
}

#[cfg(test)]
mod tests {
    use super::GameOptions;

    #[test]
    fn bounded_interactive_mode_requires_a_positive_frame_limit() {
        let options = GameOptions::parse(
            ["--interactive", "--maximum-frames", "1"]
                .into_iter()
                .map(str::to_owned),
        )
        .expect("bounded interactive options");
        assert!(options.interactive);
        assert_eq!(options.maximum_frames, Some(1));

        assert!(
            GameOptions::parse(["--maximum-frames", "1"].into_iter().map(str::to_owned)).is_err()
        );
        assert!(
            GameOptions::parse(
                ["--interactive", "--maximum-frames", "0"]
                    .into_iter()
                    .map(str::to_owned)
            )
            .is_err()
        );
    }
}
