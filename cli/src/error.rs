use std::process::ExitCode;

use colored::Colorize;

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error(transparent)]
    Usage(anyhow::Error),
    #[error(transparent)]
    Runtime(anyhow::Error),
    #[error("no matches")]
    NoMatch,
    #[error("no tasks specified")]
    NoSpecified,
    #[error("partial success")]
    Partial,
}

impl CliError {
    pub(crate) fn usage(e: impl Into<anyhow::Error>) -> Self {
        Self::Usage(e.into())
    }

    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::Usage(_) => ExitCode::from(2),
            Self::Runtime(_) | Self::NoMatch | Self::NoSpecified | Self::Partial => {
                ExitCode::from(1)
            }
        }
    }

    pub fn write_stderr(&self) {
        match self {
            Self::Usage(e) => eprintln!("{}", format!("{e:#}").white().on_red()),
            Self::Runtime(e) => eprintln!("{}", format!("{e:#}").yellow()),
            Self::NoMatch => eprintln!("{}", "No matches.".yellow()),
            Self::NoSpecified => eprintln!("{}", "No tasks specified.".yellow()),
            Self::Partial => {} // Print nothing
        }
    }
}

impl From<anyhow::Error> for CliError {
    fn from(e: anyhow::Error) -> Self {
        Self::Runtime(e)
    }
}
