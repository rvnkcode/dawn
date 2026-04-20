use colored::Colorize;
use std::process::ExitCode;

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error(transparent)]
    Usage(anyhow::Error),
    #[error(transparent)]
    Runtime(anyhow::Error),
    #[error("no matches")]
    NoMatch,
}

impl CliError {
    pub(crate) fn usage(e: impl Into<anyhow::Error>) -> Self {
        Self::Usage(e.into())
    }

    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::Usage(_) => ExitCode::from(2),
            Self::Runtime(_) | Self::NoMatch => ExitCode::from(1),
        }
    }

    /// Mirror Taskwarrior: usage/filter errors render white-on-red on stderr;
    /// runtime errors and "No matches." render yellow (footnote color) on stderr.
    pub fn write_stderr(&self) {
        match self {
            Self::Usage(e) => eprintln!("{}", format!("{e:#}").white().on_red()),
            Self::Runtime(e) => eprintln!("{}", format!("{e:#}").yellow()),
            Self::NoMatch => eprintln!("{}", "No matches.".yellow()),
        }
    }
}

impl From<anyhow::Error> for CliError {
    fn from(e: anyhow::Error) -> Self {
        Self::Runtime(e)
    }
}
