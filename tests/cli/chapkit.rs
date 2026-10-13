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
fn chapkit_test_and_run_are_refused_with_the_varde_command() {
    let sandbox = Sandbox::new();
    let work = workdir(&sandbox);
    let (_temp, bin, log) = fake_docker();

    chap_with_docker(&sandbox, &work, &bin, &["chapkit", "test"])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("`varde models test ID`"));
    chap_with_docker(&sandbox, &work, &bin, &["chapkit", "mlproject", "run", "."])
        .assert()
        .code(2)
        .stderr(predicates::str::contains("`varde run IMAGE`"));
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
