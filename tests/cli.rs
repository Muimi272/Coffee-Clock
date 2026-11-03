use std::process::{Command, Output};

fn coffee(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_coffee"))
        .args(args)
        .output()
        .expect("coffee should start")
}

#[test]
fn invalid_durations_fail_on_stderr_before_starting_the_terminal() {
    for duration in ["invalid", "0s", "25", "1s1m", "18446744073709551616s"] {
        let output = coffee(&["--time", duration]);
        assert_eq!(output.status.code(), Some(2), "{duration}");
        assert!(output.stdout.is_empty(), "{duration}");
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains("positive duration"), "{error}");
        assert!(error.contains("--time"), "{error}");
    }
}

#[test]
fn help_and_information_work_without_an_interactive_terminal() {
    let help = coffee(&["--help"]);
    assert!(help.status.success());
    assert!(help.stderr.is_empty());
    assert!(String::from_utf8(help.stdout).unwrap().contains("--pomo"));

    for option in ["--version", "-v", "--coffee"] {
        let output = coffee(&[option]);
        assert!(output.status.success(), "{option}");
        assert!(output.stderr.is_empty(), "{option}");
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains(env!("CARGO_PKG_VERSION")), "{text}");
        if option == "--coffee" {
            assert!(text.contains("Here is a cup of coffee."));
        }
    }
}

#[test]
fn incompatible_startup_options_fail_without_starting_the_terminal() {
    for args in [
        vec!["--pomo", "--time", "5m"],
        vec!["--version", "--pomo"],
        vec!["--coffee", "--time", "5m"],
        vec!["--coffee", "--version"],
    ] {
        let output = coffee(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("cannot be used")
        );
    }
}
