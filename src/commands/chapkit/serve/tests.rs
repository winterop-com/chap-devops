use super::*;

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| arg.to_string()).collect()
}

fn serve() -> Serve {
    Serve {
        name: "chap-devops-mlproject-1-0".to_string(),
        image: image_for("r-tidyverse"),
        dir: PathBuf::from("/models/ghr"),
        argument: ".".to_string(),
        port: 5001,
        registration: None,
        tty: false,
        pass: Vec::new(),
        revision: None,
    }
}

#[test]
fn parse_takes_the_directory_and_passes_the_other_options() {
    let request = parse(&strings(&[
        "ghr",
        "--config-format",
        "flat",
        "--database=state.db",
        "--param",
        "dataset=data.csv",
    ]))
    .unwrap();
    assert_eq!(request.path.as_deref(), Some("ghr"));
    assert_eq!(
        request.pass,
        strings(&[
            "--config-format",
            "flat",
            "--database=state.db",
            "--param",
            "dataset=data.csv"
        ])
    );
    assert!(!request.help);
    assert_eq!(parse(&[]).unwrap(), Request::default());
    assert!(parse(&strings(&["--help"])).unwrap().help);
}

#[test]
fn parse_refuses_what_chap_devops_sets_and_a_second_directory() {
    for owned in ["--port", "--host=0.0.0.0", "--register-url"] {
        let why = parse(&strings(&[".", owned, "x"])).unwrap_err();
        assert!(why.contains("chap-devops sets"), "{why}");
    }
    let why = parse(&strings(&["one", "two"])).unwrap_err();
    assert!(why.contains("`two` is a second one"), "{why}");
    let why = parse(&strings(&["--config-format"])).unwrap_err();
    assert!(why.contains("needs a value"), "{why}");
}

#[test]
fn image_for_maps_the_short_names_and_keeps_an_image_name() {
    assert_eq!(image_for("py"), "ghcr.io/dhis2-chap/chapkit-py-cli:latest");
    assert_eq!(
        image_for("py-data"),
        "ghcr.io/dhis2-chap/chapkit-py-data-cli:latest"
    );
    assert_eq!(
        image_for("r-inla"),
        "ghcr.io/dhis2-chap/chapkit-r-inla-cli:latest"
    );
    assert_eq!(image_for("chapkit_ghr_model:dev"), "chapkit_ghr_model:dev");
    assert!(amd64_only(&image_for("r-inla")));
    assert!(!amd64_only(&image_for("r-tidyverse")));
}

#[test]
fn the_default_image_follows_the_train_command() {
    let mlproject =
        |command: &str| format!("name: m\nentry_points:\n  train:\n    command: \"{command}\"\n");
    assert_eq!(
        default_flavour(&mlproject("Rscript train.R {train_data}")).map(|(f, _)| f),
        Some("r-tidyverse")
    );
    assert_eq!(
        default_flavour(&mlproject("/usr/bin/Rscript train.R")).map(|(f, _)| f),
        Some("r-tidyverse")
    );
    assert_eq!(
        default_flavour(&mlproject("python train.py")).map(|(f, _)| f),
        Some("py-data")
    );
    assert_eq!(
        default_flavour(&mlproject("uv run train.py")).map(|(f, _)| f),
        Some("py-data")
    );
    assert_eq!(default_flavour(&mlproject("julia train.jl")), None);
    assert_eq!(default_flavour("name: m\n"), None);
    assert_eq!(
        default_flavour(&mlproject("python train.py")).map(|(_, c)| c),
        Some("python train.py".to_string())
    );
}

#[test]
fn docker_args_mount_the_project_and_publish_the_port() {
    let mut serve = serve();
    serve.pass = strings(&["--config-format", "flat"]);
    let args = docker_args(&serve);
    let joined = args.join(" ");
    assert!(joined.starts_with("run --rm -i --init --name chap-devops-mlproject-1-0 -p 5001:8000"));
    assert!(joined.contains("-v /models/ghr:/work -w /work"), "{joined}");
    assert!(!joined.contains("--platform"), "{joined}");
    assert!(!joined.contains("--network"), "{joined}");
    assert!(joined.ends_with(
        "ghcr.io/dhis2-chap/chapkit-r-tidyverse-cli:latest chapkit mlproject run . \
         --host 0.0.0.0 --port 8000 --config-format flat"
    ));
}

#[test]
fn docker_args_pin_amd64_for_inla() {
    let mut serve = serve();
    serve.image = image_for("r-inla");
    serve.argument = "MLproject.yaml".to_string();
    let joined = docker_args(&serve).join(" ");
    assert!(joined.contains("--platform linux/amd64"), "{joined}");
    assert!(
        joined.contains("chapkit mlproject run MLproject.yaml"),
        "{joined}"
    );
}

#[test]
fn docker_args_register_on_the_deployment_network_without_the_key_value() {
    let mut serve = serve();
    serve.registration = Some(Registration {
        network: "dengue_default".to_string(),
        key: true,
    });
    let args = docker_args(&serve);
    let joined = args.join(" ");
    assert!(joined.contains("--network dengue_default"), "{joined}");
    assert!(
        joined.contains("-e SERVICEKIT_ORCHESTRATOR_URL=http://chap:8000/v2/services/$register")
    );
    assert!(
        joined.contains("-e SERVICEKIT_HOST=chap-devops-mlproject-1-0"),
        "{joined}"
    );
    assert!(joined.contains("-e SERVICEKIT_PORT=8000"), "{joined}");
    assert!(
        args.windows(2)
            .any(|pair| pair == ["-e", "SERVICEKIT_REGISTRATION_KEY"])
    );
}

#[test]
fn locate_finds_the_mlproject_file() {
    let temp = tempfile::tempdir().unwrap();
    let model = temp.path().join("model");
    std::fs::create_dir_all(&model).unwrap();
    std::fs::write(model.join("MLproject"), "name: m\n").unwrap();
    let (dir, argument) = locate(temp.path(), Some("model")).unwrap();
    assert_eq!(dir, std::fs::canonicalize(&model).unwrap());
    assert_eq!(argument, ".");

    let other = temp.path().join("other");
    std::fs::create_dir_all(&other).unwrap();
    std::fs::write(other.join("MLproject.yaml"), "name: m\n").unwrap();
    assert_eq!(
        locate(temp.path(), Some("other")).unwrap().1,
        "MLproject.yaml"
    );
    assert_eq!(
        locate(&other, Some("MLproject.yaml")).unwrap().1,
        "MLproject.yaml"
    );

    let why = locate(temp.path(), None).unwrap_err();
    assert!(why.contains("has no `MLproject` file"), "{why}");
    let why = locate(temp.path(), Some("missing")).unwrap_err();
    assert!(why.contains("does not exist"), "{why}");
}

#[test]
fn docker_args_give_chapkit_the_revision() {
    let mut serve = serve();
    assert!(!docker_args(&serve).join(" ").contains("GIT_REVISION"));
    serve.revision = Some("abc123".to_string());
    let joined = docker_args(&serve).join(" ");
    assert!(
        joined.contains("-e GIT_REVISION=abc123 --label"),
        "{joined}"
    );
}

#[test]
fn the_revision_is_the_commit_and_says_when_there_are_changes() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    assert!(git_revision(dir).is_none());
    let git = |args: &[&str]| {
        let status = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .expect("git runs");
        assert!(status.status.success(), "{args:?}");
    };
    git(&["init", "-q"]);
    std::fs::write(dir.join("MLproject"), "name: m\n").unwrap();
    git(&["add", "MLproject"]);
    git(&["commit", "-q", "-m", "first"]);
    let clean = git_revision(dir).expect("a revision");
    assert_eq!(clean.commit.len(), 40);
    assert!(!clean.dirty);
    std::fs::write(dir.join("MLproject"), "name: changed\n").unwrap();
    assert!(git_revision(dir).expect("a revision").dirty);
}
