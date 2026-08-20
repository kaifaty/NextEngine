use super::*;

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}

#[test]
fn parser_accepts_only_the_seven_bounded_project_operations() {
    assert!(matches!(
        parse_arguments(args(&[
            "project",
            "create",
            "--template",
            "rpg-starter",
            "--project-id",
            "org.example.game",
            "--output",
            "sample",
        ])),
        Ok(CreatorCommand::Create { template, project_id, output })
            if template == OsStr::new("rpg-starter")
                && project_id == OsStr::new("org.example.game")
                && output.as_path() == Path::new("sample")
    ));
    assert!(matches!(
        parse_arguments(args(&["project", "validate", "--project", "sample"])),
        Ok(CreatorCommand::Validate { project })
            if project.as_path() == Path::new("sample")
    ));
    assert!(matches!(
        parse_arguments(args(&["project", "run", "--project", "sample"])),
        Ok(CreatorCommand::RunProject { project })
            if project.as_path() == Path::new("sample")
    ));
    assert!(matches!(
        parse_arguments(args(&["project", "run", "--package", "bundle"])),
        Ok(CreatorCommand::RunPackage { package })
            if package.as_path() == Path::new("bundle")
    ));
    assert!(matches!(
        parse_arguments(args(&[
            "project",
            "package",
            "--project",
            "sample",
            "--output",
            "bundle",
        ])),
        Ok(CreatorCommand::Package { project, output })
            if project.as_path() == Path::new("sample")
                && output.as_path() == Path::new("bundle")
    ));
    assert!(
        parse_arguments(args(&[
            "project",
            "run",
            "--project",
            "sample",
            "--package",
            "bundle",
        ]))
        .is_err()
    );
    assert!(matches!(
        parse_arguments(args(&[
            "project", "cook", "--output", "out", "--project", "sample",
        ])),
        Ok(CreatorCommand::Cook { project, output })
            if project.as_path() == Path::new("sample")
                && output.as_path() == Path::new("out")
    ));
    assert!(matches!(
        parse_arguments(args(&["project", "inspect", "--package", "bundle"])),
        Ok(CreatorCommand::Inspect {
            input: CreatorProjectInput::Package(package),
        }) if package.as_path() == Path::new("bundle")
    ));
    assert!(matches!(
        parse_arguments(args(&[
            "project",
            "diff",
            "--base-project",
            "before",
            "--candidate-package",
            "after",
        ])),
        Ok(CreatorCommand::Diff {
            base: CreatorProjectInput::Project(base),
            candidate: CreatorProjectInput::Package(candidate),
        }) if base.as_path() == Path::new("before")
            && candidate.as_path() == Path::new("after")
    ));
    assert!(parse_arguments(args(&["project", "diff"])).is_err());
    assert!(
        parse_arguments(args(&[
            "project",
            "create",
            "--template",
            "rpg-starter",
            "--project-id",
            "org.example.game",
        ]))
        .is_err()
    );
    assert!(
        parse_arguments(args(&[
            "project",
            "inspect",
            "--project",
            "sample",
            "--package",
            "bundle",
        ]))
        .is_err()
    );
    assert!(
        parse_arguments(args(&[
            "project",
            "diff",
            "--base-project",
            "before",
            "--base-package",
            "before-package",
            "--candidate-project",
            "after",
        ]))
        .is_err()
    );
    assert!(
        parse_arguments(args(&[
            "project",
            "validate",
            "--project",
            "one",
            "--project",
            "two",
        ]))
        .is_err()
    );
}

#[test]
fn argument_failure_is_one_stable_path_free_json_object() {
    let report = execute(args(&["project", "cook", "--project", "sample"]));
    assert!(!report.is_pass());
    assert_eq!(
        report.to_json().expect("report JSON"),
        concat!(
            "{\"schema_version\":1,\"status\":\"FAIL\",",
            "\"command\":\"project.cook\",\"diagnostic\":{",
            "\"code\":\"CREATOR_CLI_ARGUMENT_INVALID\",",
            "\"subsystem\":\"creator-cli\",",
            "\"message_key\":\"creator.cli.argument-invalid\"}}"
        )
    );
}
