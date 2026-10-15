//! `varde chapkit`: the chapkit CLI in the chap-core image, against the
//! stand-in `docker` of the `varde chap` tests, so Unix only.
#![cfg(unix)]

use crate::chap::{chap_run, fake_docker, value_of, workdir};
use crate::common::*;
use predicates::prelude::PredicateBooleanExt;

#[test]
fn chapkit_runs_in_the_core_image_in_the_current_directory() {
    let sandbox = Sandbox::new();
    let work = workdir(&sandbox);
    let (_temp, bin, log) = fake_docker();

    let out = chap_with_docker(
        &sandbox,
        &work,
        &bin,
        &["chapkit", "mlproject", "migrate", "model", "--yes"],
    )
    .env("WRITE", "main.py")
    .assert()
    .success()
    .get_output()
    .clone();
    let stderr = String::from_utf8_lossy(&out.stderr);

    let args = chap_run(&log);
    assert!(args.contains(&"ghcr.io/dhis2-chap/chap-core:v2.3.1".to_string()));
    assert_eq!(
        args[args.len() - 5..],
        ["chapkit", "mlproject", "migrate", "model", "--yes"]
    );
    let workdir = work.display().to_string();
    assert_eq!(value_of(&args, "-w"), Some(workdir.as_str()));
    assert!(args.contains(&format!("{workdir}:{workdir}")));
    assert!(
        stderr.contains("running `chapkit mlproject` in ghcr.io/dhis2-chap/chap-core:v2.3.1"),
        "{stderr}"
    );
    assert!(
        stderr.contains("chapkit finished; it wrote main.py"),
        "{stderr}"
    );
}

#[test]
fn a_new_project_with_no_lock_gets_uv_lock_in_the_same_image() {
    let sandbox = Sandbox::new();
    let work = workdir(&sandbox);
    let model = work.join("model");
    std::fs::create_dir_all(&model).unwrap();
    std::fs::write(model.join("Dockerfile"), "COPY pyproject.toml uv.lock ./\n").unwrap();
    let (_temp, bin, log) = fake_docker();

    let out = chap_with_docker(
        &sandbox,
        &work,
        &bin,
        &["chapkit", "mlproject", "migrate", "model", "--yes"],
    )
    .env("WRITE", "model/pyproject.toml")
    .assert()
    .success()
    .get_output()
    .clone();
    let stderr = String::from_utf8_lossy(&out.stderr);

    let calls = std::fs::read_to_string(&log).unwrap();
    let lock = calls
        .lines()
        .find(|line| line.contains(" uv lock "))
        .unwrap_or_else(|| panic!("no uv lock in:\n{calls}"));
    assert!(
        lock.ends_with(&format!("uv lock --quiet --directory {}", model.display())),
        "{lock}"
    );
    // The stand-in writes no uv.lock, so varde says that the build needs it.
    assert!(
        stderr.contains("could not write model/uv.lock, and the `Dockerfile` copies it"),
        "{stderr}"
    );
}

#[test]
fn chapkit_in_a_deployment_uses_its_tag() {
    let sandbox = Sandbox::new();
    let dir = sandbox.project();
    sandbox.init(&[]).assert().success();
    let tag = state(&dir)["chap_image_tag"].as_str().unwrap().to_string();
    let (_temp, bin, log) = fake_docker();

    chap_with_docker(&sandbox, &dir, &bin, &["chapkit", "--version"])
        .assert()
        .success();
    assert!(chap_run(&log).contains(&format!("ghcr.io/dhis2-chap/chap-core:{tag}")));
}

#[test]
fn chapkit_test_is_refused_with_the_varde_command() {
    let sandbox = Sandbox::new();
    let work = workdir(&sandbox);
    let (_temp, bin, log) = fake_docker();

    chap_with_docker(&sandbox, &work, &bin, &["chapkit", "test"])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("`varde models test ID`"));
    assert!(
        !std::fs::read_to_string(&log)
            .unwrap_or_default()
            .contains("run --rm -i")
    );
}

#[test]
fn a_usage_error_of_chapkit_names_the_help_and_exits_2() {
    let sandbox = Sandbox::new();
    let work = workdir(&sandbox);
    let (_temp, bin, _log) = fake_docker();

    chap_with_docker(&sandbox, &work, &bin, &["chapkit", "artifact", "--nope"])
        .env("CHAP_EXIT", "2")
        .assert()
        .code(2)
        .stderr(predicates::str::contains(
            "chapkit did not accept the arguments, and its usage is above; \
             `varde chapkit artifact --help` lists what the command takes",
        ))
        .stderr(predicates::str::contains("exited with status").not());
}

#[test]
fn a_group_alone_runs_its_help() {
    let sandbox = Sandbox::new();
    let work = workdir(&sandbox);
    let (_temp, bin, log) = fake_docker();

    chap_with_docker(&sandbox, &work, &bin, &["chapkit", "artifact"])
        .assert()
        .success()
        .stderr(predicates::str::contains("files:").not());
    let args = chap_run(&log);
    assert_eq!(args[args.len() - 3..], ["chapkit", "artifact", "--help"]);
}

/// A directory `model` in `dir` with an MLproject whose train command is `command`.
fn mlproject(dir: &std::path::Path, command: &str) -> std::path::PathBuf {
    let model = dir.join("model");
    std::fs::create_dir_all(&model).unwrap();
    std::fs::write(
        model.join("MLproject"),
        format!(
            "name: rain\nentry_points:\n  train:\n    command: \"{command}\"\n  predict:\n    \
             command: \"{command}\"\n"
        ),
    )
    .unwrap();
    std::fs::canonicalize(model).unwrap()
}

#[test]
fn mlproject_run_serves_an_r_model_in_the_tidyverse_image() {
    let sandbox = Sandbox::new();
    let work = workdir(&sandbox);
    let model = mlproject(&work, "Rscript train.R {train_data}");
    let (_temp, bin, log) = fake_docker();

    let out = chap_with_docker(
        &sandbox,
        &work,
        &bin,
        &[
            "chapkit",
            "mlproject",
            "run",
            "model",
            "--config-format",
            "flat",
        ],
    )
    .assert()
    .success()
    .get_output()
    .clone();
    let stderr = String::from_utf8_lossy(&out.stderr);

    let calls = std::fs::read_to_string(&log).unwrap();
    assert!(
        calls.contains("pull ghcr.io/dhis2-chap/chapkit-r-tidyverse-cli:latest"),
        "{calls}"
    );
    let args = chap_run(&log);
    let joined = args.join(" ");
    assert!(
        joined.contains(&format!("-v {}:/work -w /work", model.display())),
        "{joined}"
    );
    let published = value_of(&args, "-p").expect("a published port");
    assert!(published.ends_with(":8000"), "{published}");
    assert!(value_of(&args, "--network").is_none(), "{joined}");
    assert!(!joined.contains("--platform"), "{joined}");
    assert!(
        joined.ends_with(
            "ghcr.io/dhis2-chap/chapkit-r-tidyverse-cli:latest chapkit mlproject run . \
             --host 0.0.0.0 --port 8000 --config-format flat"
        ),
        "{joined}"
    );
    assert!(
        stderr.contains("from the train command `Rscript train.R {train_data}`"),
        "{stderr}"
    );
    assert!(
        stderr.contains("stopped the MLproject service on http://localhost:"),
        "{stderr}"
    );
}

#[test]
fn mlproject_run_takes_a_short_name_or_a_local_image() {
    let sandbox = Sandbox::new();
    let work = workdir(&sandbox);
    mlproject(&work, "Rscript train.R");
    let (_temp, bin, log) = fake_docker();

    chap_with_docker(
        &sandbox,
        &work,
        &bin,
        &["chapkit", "--image", "r-inla", "mlproject", "run", "model"],
    )
    .assert()
    .success();
    let joined = chap_run(&log).join(" ");
    assert!(joined.contains("--platform linux/amd64"), "{joined}");
    assert!(
        joined.contains("ghcr.io/dhis2-chap/chapkit-r-inla-cli:latest chapkit"),
        "{joined}"
    );

    let (_temp, bin, log) = fake_docker();
    chap_with_docker(
        &sandbox,
        &work,
        &bin,
        &[
            "chapkit",
            "--image",
            "chapkit_ghr_model:dev",
            "mlproject",
            "run",
            "model",
        ],
    )
    .env("IMAGE_HERE", "1")
    .assert()
    .success();
    let calls = std::fs::read_to_string(&log).unwrap();
    assert!(!calls.contains("pull "), "{calls}");
    assert!(
        chap_run(&log)
            .join(" ")
            .contains(" chapkit_ghr_model:dev chapkit mlproject run .")
    );
}

#[test]
fn mlproject_run_says_when_the_image_cannot_be_pulled() {
    let sandbox = Sandbox::new();
    let work = workdir(&sandbox);
    mlproject(&work, "python train.py");
    let (_temp, bin, log) = fake_docker();

    chap_with_docker(
        &sandbox,
        &work,
        &bin,
        &["chapkit", "mlproject", "run", "model"],
    )
    .env("PULL_EXIT", "1")
    .assert()
    .code(2)
    .stderr(predicates::str::contains(
        "this docker does not have ghcr.io/dhis2-chap/chapkit-py-cli:latest, and could not pull it",
    ));
    assert!(
        !std::fs::read_to_string(&log)
            .unwrap()
            .contains("run --rm -i")
    );
}

#[test]
fn mlproject_run_refuses_what_it_cannot_do() {
    let sandbox = Sandbox::new();
    let work = workdir(&sandbox);
    mlproject(&work, "julia train.jl");
    let (_temp, bin, log) = fake_docker();

    for (args, message) in [
        (
            &["chapkit", "mlproject", "run", "model"][..],
            "give `--image py`, `r`, `r-tidyverse`, `r-inla`",
        ),
        (
            &["chapkit", "mlproject", "run", "model", "--port", "9000"][..],
            "varde sets `--port`",
        ),
        (
            &["chapkit", "--tag", "v2.3.1", "mlproject", "run", "model"][..],
            "drop --tag",
        ),
        (
            &["chapkit", "--image", "py", "mlproject", "migrate", "model"][..],
            "--image picks the image of `mlproject run` only",
        ),
        (
            &["chapkit", "mlproject", "run", "nowhere"][..],
            "does not exist",
        ),
    ] {
        chap_with_docker(&sandbox, &work, &bin, args)
            .assert()
            .code(2)
            .stderr(predicates::str::contains(message));
    }
    assert!(
        !std::fs::read_to_string(&log)
            .unwrap_or_default()
            .contains("run --rm -i")
    );
}

#[test]
fn mlproject_run_names_the_cli_images_when_the_image_has_no_chapkit() {
    let sandbox = Sandbox::new();
    let work = workdir(&sandbox);
    mlproject(&work, "Rscript train.R");
    let (_temp, bin, _log) = fake_docker();

    chap_with_docker(
        &sandbox,
        &work,
        &bin,
        &[
            "chapkit",
            "--image",
            "r-base:4",
            "mlproject",
            "run",
            "model",
        ],
    )
    .env("IMAGE_HERE", "1")
    .env("CHAP_EXIT", "127")
    .assert()
    .code(127)
    .stderr(predicates::str::contains("r-base:4 may have no `chapkit`"));
}

#[test]
fn mlproject_run_in_a_deployment_registers_with_its_chap_core() {
    let sandbox = Sandbox::new();
    let dir = sandbox.project();
    sandbox.init(&["--api-token"]).assert().success();
    mlproject(&dir, "Rscript train.R");
    let name = std::fs::read_to_string(dir.join(".env")).unwrap();
    assert!(name.contains("SERVICEKIT_REGISTRATION_KEY="), "{name}");
    let (_temp, bin, log) = fake_docker();

    chap_with_docker(
        &sandbox,
        &dir,
        &bin,
        &["chapkit", "mlproject", "run", "model"],
    )
    .env("NETWORK_UP", "1")
    .env("IMAGE_HERE", "1")
    .assert()
    .success();
    let args = chap_run(&log);
    let joined = args.join(" ");
    let network = value_of(&args, "--network").expect("the deployment network");
    assert!(network.ends_with("_default"), "{network}");
    assert!(
        joined.contains("-e SERVICEKIT_ORCHESTRATOR_URL=http://chap:8000/v2/services/$register"),
        "{joined}"
    );
    assert!(
        args.windows(2)
            .any(|pair| pair == ["-e", "SERVICEKIT_REGISTRATION_KEY"])
    );
    assert!(
        !args
            .iter()
            .any(|arg| arg.starts_with("SERVICEKIT_REGISTRATION_KEY="))
    );
}
