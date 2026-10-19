use super::chapkit_counts;
use crate::cli::{Cli, Command, ModelsCmd};
use clap::Parser;

fn test_args(argv: &[&str]) -> crate::cli::ModelsTestArgs {
    let mut full = vec!["chap-devops", "models", "test"];
    full.extend_from_slice(argv);
    let cli = Cli::try_parse_from(full).expect("the arguments parse");
    let Command::Models(models) = cli.command else {
        panic!("expected models");
    };
    let ModelsCmd::Test(args) = models.command else {
        panic!("expected test");
    };
    args
}

#[test]
fn no_count_option_gives_no_chapkit_option() {
    assert!(chapkit_counts(&test_args(&["m"])).is_empty());
}

#[test]
fn each_count_option_goes_to_chapkit_test() {
    let args = test_args(&[
        "m",
        "--configs",
        "2",
        "--trainings",
        "3",
        "--predictions",
        "4",
        "--rows",
        "600",
        "--predict-rows",
        "300",
    ]);
    assert_eq!(
        chapkit_counts(&args),
        [
            "--configs",
            "2",
            "--trainings",
            "3",
            "--predictions",
            "4",
            "--rows",
            "600",
            "--predict-rows",
            "300"
        ]
    );
}

#[test]
fn a_count_option_is_refused_with_backtest() {
    let mut full = vec![
        "chap-devops",
        "models",
        "test",
        "m",
        "--backtest",
        "--trainings",
        "2",
    ];
    assert!(Cli::try_parse_from(&full).is_err());
    full.truncate(5);
    assert!(Cli::try_parse_from(&full).is_ok());
}

#[test]
fn a_count_of_zero_is_refused() {
    assert!(Cli::try_parse_from(["chap-devops", "models", "test", "m", "--rows", "0"]).is_err());
}
