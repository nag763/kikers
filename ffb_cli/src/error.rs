use std::process::{ExitCode, Termination};

/// The common CLI Errors
///
/// These errors can be thrown while using the CLI and fetching remote data.
#[derive(Debug, Display)]
pub enum CliError {
    /// When a VAR isn't set on the .env file.
    #[display(fmt = "{}", _0)]
    VarError(String),
    /// When an error is thrown during the call to crate [reqwest]
    #[display(fmt = "{}", _0)]
    RequestError(String),
    /// An error that is due to the crate [ffb_structs].
    #[display(fmt = "{}", _0)]
    StructError(String),
    /// An error linked to the writing of a local file.
    #[display(fmt = "{}", _0)]
    InputOutput(String),
    /// An URL error.
    #[display(fmt = "{}", _0)]
    UrlError(String),
    /// A [serde_json] error.
    #[display(fmt = "{}", _0)]
    SerdeErr(String),
    /// When no main bookmaker is set
    #[display(fmt = "No main bookmaker has been set, set one before fetching the odds")]
    NoMainBookmaker,
}

impl Termination for CliError {
    fn report(self) -> ExitCode {
        match self {
            CliError::VarError(_) => ExitCode::from(10),
            CliError::RequestError(_) => ExitCode::from(11),
            CliError::StructError(_) => ExitCode::from(12),
            CliError::InputOutput(_) => ExitCode::from(13),
            CliError::UrlError(_) => ExitCode::from(14),
            CliError::SerdeErr(_) => ExitCode::from(15),
            CliError::NoMainBookmaker => ExitCode::from(16),
        }
    }
}

impl std::error::Error for CliError {}

impl From<std::env::VarError> for CliError {
    fn from(err: std::env::VarError) -> Self {
        Self::VarError(err.to_string())
    }
}

impl From<reqwest::Error> for CliError {
    fn from(err: reqwest::Error) -> Self {
        Self::RequestError(err.to_string())
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        Self::InputOutput(err.to_string())
    }
}

impl From<ffb_structs::error::ApplicationError> for CliError {
    fn from(struct_err: ffb_structs::error::ApplicationError) -> Self {
        Self::StructError(struct_err.to_string())
    }
}

impl From<url::ParseError> for CliError {
    fn from(url_err: url::ParseError) -> Self {
        Self::UrlError(url_err.to_string())
    }
}

impl From<std::num::ParseIntError> for CliError {
    fn from(int_err: std::num::ParseIntError) -> Self {
        Self::VarError(int_err.to_string())
    }
}

impl From<serde_json::Error> for CliError {
    fn from(serde_err: serde_json::Error) -> Self {
        Self::SerdeErr(serde_err.to_string())
    }
}
