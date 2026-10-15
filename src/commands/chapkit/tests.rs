use super::{command_name, refused, serve_arguments};

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| arg.to_string()).collect()
}

#[test]
fn migrate_and_init_are_not_refused() {
    assert_eq!(refused(&strings(&["mlproject", "migrate", "model"])), None);
    assert_eq!(refused(&strings(&["init", "my-model"])), None);
    assert_eq!(refused(&strings(&["--help"])), None);
    assert_eq!(refused(&[]), None);
}

#[test]
fn test_is_refused_with_the_varde_command() {
    let why = refused(&strings(&["test", "--url", "http://localhost:8000"])).expect("refused");
    assert!(why.contains("`varde models test ID`"), "{why}");
}

#[test]
fn run_serves_in_both_spellings() {
    let args = strings(&["mlproject", "run", "model", "--config-format", "flat"]);
    assert_eq!(
        serve_arguments(&args),
        Some(&strings(&["model", "--config-format", "flat"])[..])
    );
    let args = strings(&["run", "."]);
    assert_eq!(serve_arguments(&args), Some(&strings(&["."])[..]));
    let args = strings(&["mlproject", "run"]);
    assert_eq!(serve_arguments(&args), Some(&[][..]));
    for args in [
        &["mlproject", "migrate", "model"][..],
        &["init", "run"][..],
        &["--help"][..],
    ] {
        assert_eq!(serve_arguments(&strings(args)), None, "{args:?}");
        assert_eq!(refused(&strings(args)), None, "{args:?}");
    }
}

#[test]
fn a_copy_of_uv_lock_needs_a_lock() {
    assert!(super::copies_lock(
        "FROM x\n# uv.lock comes from `uv lock`\nCOPY pyproject.toml uv.lock ./\n"
    ));
    assert!(!super::copies_lock(
        "FROM x\n# Run uv sync to make uv.lock\nCOPY . /work\n"
    ));
}

#[test]
fn needs_lock_names_a_new_project_with_no_lock() {
    let temp = tempfile::tempdir().expect("a directory");
    let new = temp.path().join("new");
    let locked = temp.path().join("locked");
    for dir in [&new, &locked] {
        std::fs::create_dir_all(dir).expect("a project");
        std::fs::write(dir.join("pyproject.toml"), "").expect("pyproject.toml");
        std::fs::write(dir.join("Dockerfile"), "COPY pyproject.toml uv.lock ./\n")
            .expect("Dockerfile");
    }
    std::fs::write(locked.join("uv.lock"), "").expect("uv.lock");
    let written = vec![new.join("pyproject.toml"), locked.join("pyproject.toml")];
    assert_eq!(super::needs_lock(&written), vec![new]);
}

#[test]
fn a_usage_error_names_the_help_of_the_command() {
    let hint = super::usage_hint(&strings(&["artifact"]));
    assert!(hint.contains("`varde chapkit artifact --help`"), "{hint}");
    let hint = super::usage_hint(&strings(&["mlproject", "migrate", "--nope"]));
    assert!(
        hint.contains("`varde chapkit mlproject migrate --help`"),
        "{hint}"
    );
    let hint = super::usage_hint(&strings(&["--nope"]));
    assert!(hint.contains("`varde chapkit --help`"), "{hint}");
}

#[test]
fn a_group_alone_gets_help() {
    assert_eq!(
        super::with_group_help(&strings(&["artifact"])),
        ["artifact", "--help"]
    );
    assert_eq!(
        super::with_group_help(&strings(&["mlproject"])),
        ["mlproject", "--help"]
    );
    assert_eq!(
        super::with_group_help(&strings(&["artifact", "list"])),
        ["artifact", "list"]
    );
    assert_eq!(super::with_group_help(&strings(&["init"])), ["init"]);
    assert!(super::with_group_help(&[]).is_empty());
}

#[test]
fn the_command_name_has_the_command_of_a_group() {
    assert_eq!(
        command_name(&strings(&["mlproject", "export", "main.py"])),
        "mlproject export"
    );
    assert_eq!(
        command_name(&strings(&["mlproject", "--help"])),
        "mlproject"
    );
    assert_eq!(command_name(&strings(&["init", "my-model"])), "init");
    assert_eq!(command_name(&[]), "--help");
}
