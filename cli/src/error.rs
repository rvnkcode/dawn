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

    /// Mirror Taskwarrior: usage errors render white-on-red on stderr;
    /// runtime/no-match stay silent (NoMatch already printed its own line to stdout).
    pub fn write_stderr(&self) {
        if let Self::Usage(e) = self {
            eprintln!("{}", format!("{e:#}").white().on_red());
        }
    }
}

impl From<anyhow::Error> for CliError {
    fn from(e: anyhow::Error) -> Self {
        Self::Runtime(e)
    }
}
