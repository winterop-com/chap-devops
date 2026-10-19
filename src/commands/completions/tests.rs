use super::*;

/// The five shells `chap-devops completions` accepts, which is every shell
/// `clap_complete::Shell` knows.
const SHELLS: &[Shell] = &[
    Shell::Bash,
    Shell::Zsh,
    Shell::Fish,
    Shell::PowerShell,
    Shell::Elvish,
];

#[test]
fn every_shell_gets_a_script_that_completes_chap_devops() {
    for shell in SHELLS {
        let script = script(*shell);
        assert!(!script.trim().is_empty(), "{shell} produced nothing");
        assert!(
            script.contains("chap-devops"),
            "{shell} does not name chap-devops"
        );
    }
}

/// The scripts are generated from the live command tree, so a command
/// added to `src/cli.rs` is completable without a second edit.
#[test]
fn the_scripts_know_the_commands_this_build_has() {
    for shell in SHELLS {
        let script = script(*shell);
        for command in ["init", "models", "status", "self", "completions"] {
            assert!(
                script.contains(command),
                "{shell} does not mention `{command}`"
            );
        }
    }
}

/// Each shell's script is in that shell's language, so a script filed
/// under the wrong name would be noticed here rather than by a user whose
/// shell refuses to load it.
#[test]
fn each_script_is_written_for_its_own_shell() {
    assert!(script(Shell::Bash).contains("complete -F _chap__devops"));
    assert!(script(Shell::Zsh).starts_with("#compdef chap-devops"));
    assert!(script(Shell::Fish).contains("complete -c chap-devops"));
    assert!(script(Shell::PowerShell).contains("Register-ArgumentCompleter"));
    assert!(script(Shell::Elvish).contains("edit:completion:arg-completer"));
}

/// `cdx` is the link `install.sh` and `make install` add, so every script
/// registers it next to `chap-devops`.
#[test]
fn every_script_also_completes_the_short_form() {
    let bash = script(Shell::Bash);
    assert!(
        bash.contains("-o nosort -o bashdefault -o default chap-devops cdx\n"),
        "{bash}"
    );
    assert!(bash.contains("complete -F _chap__devops -o bashdefault -o default chap-devops cdx\n"));
    let zsh = script(Shell::Zsh);
    assert!(zsh.starts_with("#compdef chap-devops cdx\n"));
    assert!(zsh.contains("compdef _chap-devops chap-devops cdx\n"));
    assert!(script(Shell::Fish).ends_with("complete -c cdx -w chap-devops\n"));
    assert!(script(Shell::PowerShell).contains("-CommandName 'chap-devops', 'cdx'"));
    assert!(
        script(Shell::Elvish)
            .contains("arg-completer[cdx] = $edit:completion:arg-completer[chap-devops]")
    );
}
