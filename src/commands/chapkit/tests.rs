use super::refused;

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
fn run_is_refused_in_both_spellings() {
    for args in [&["run", "."][..], &["mlproject", "run", "."][..]] {
        let why = refused(&strings(args)).expect("refused");
        assert!(why.contains("`varde run IMAGE`"), "{why}");
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
