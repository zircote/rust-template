//! Binary entry point for `rust_template`.

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::io::IsTerminal;
use std::process::ExitCode;

use rust_template::{Config, Error, OutputFormat, add, divide};

/// Runs the application logic.
fn run() -> Result<(), Error> {
    let config = Config::new().with_verbose(true);

    if config.verbose() {
        eprintln!("Running rust_template with verbose mode enabled");
    }

    let sum = add(2, 3);
    println!("2 + 3 = {sum}");

    let quotient = divide(10, 2)?;
    println!("10 / 2 = {quotient}");

    Ok(())
}

/// Extracts the value of an explicit `--format <value>` / `--format=<value>`
/// argument, if present. The last occurrence wins.
fn explicit_format<I: IntoIterator<Item = String>>(args: I) -> Option<String> {
    let mut iter = args.into_iter();
    let mut value = None;
    while let Some(arg) = iter.next() {
        if let Some(rest) = arg.strip_prefix("--format=") {
            value = Some(rest.to_owned());
        } else if arg == "--format" {
            // A bare, valueless trailing `--format` must not erase a format
            // chosen earlier on the line; only update when a value follows.
            if let Some(next) = iter.next() {
                value = Some(next);
            }
        }
    }
    value
}

/// Renders a failed [`Error`] in the format selected by the `--format` flag and
/// stderr TTY state. JSON when `--format=json` or stderr is not a terminal;
/// pretty otherwise.
fn render_failure(error: &Error, args: Vec<String>, stderr_is_terminal: bool) -> String {
    let format = OutputFormat::select(explicit_format(args).as_deref(), stderr_is_terminal);
    error.render(format)
}

/// Main entry point.
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let rendered = render_failure(
                &e,
                std::env::args().collect(),
                std::io::stderr().is_terminal(),
            );
            eprintln!("{rendered}");
            ExitCode::FAILURE
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_succeeds() {
        let result = run();
        assert!(
            result.is_ok(),
            "run() should succeed with the default implementation"
        );
    }

    #[test]
    fn test_main_returns_success() {
        let exit_code = main();
        assert_eq!(exit_code, ExitCode::SUCCESS);
    }

    #[test]
    fn test_explicit_format_parses_both_spellings() {
        let split = vec![
            "bin".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ];
        assert_eq!(explicit_format(split).as_deref(), Some("json"));

        let joined = vec!["bin".to_string(), "--format=pretty".to_string()];
        assert_eq!(explicit_format(joined).as_deref(), Some("pretty"));

        let none = vec!["bin".to_string()];
        assert_eq!(explicit_format(none), None);

        // Last occurrence wins.
        let repeated = vec![
            "bin".to_string(),
            "--format=json".to_string(),
            "--format=pretty".to_string(),
        ];
        assert_eq!(explicit_format(repeated).as_deref(), Some("pretty"));

        // A bare, valueless trailing `--format` must not clobber a prior value.
        let trailing_bare = vec![
            "bin".to_string(),
            "--format=json".to_string(),
            "--format".to_string(),
        ];
        assert_eq!(explicit_format(trailing_bare).as_deref(), Some("json"));
    }

    #[test]
    fn test_render_failure_selects_format() {
        let err = divide(10, 0).unwrap_err();

        // Explicit JSON flag wins over a TTY.
        let json = render_failure(
            &err,
            vec!["bin".to_string(), "--format=json".to_string()],
            true,
        );
        assert!(json.starts_with('{'));
        assert!(json.contains("\"type\""));

        // Explicit pretty flag is byte-identical to the historical line.
        let pretty = render_failure(
            &err,
            vec!["bin".to_string(), "--format=pretty".to_string()],
            false,
        );
        assert_eq!(pretty, format!("Error: {err}"));

        // No flag, non-TTY defaults to JSON.
        let piped = render_failure(&err, vec!["bin".to_string()], false);
        assert!(piped.starts_with('{'));

        // No flag, TTY defaults to pretty.
        let tty = render_failure(&err, vec!["bin".to_string()], true);
        assert_eq!(tty, format!("Error: {err}"));
    }
}
