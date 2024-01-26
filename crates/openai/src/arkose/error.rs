use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum ArkoseError {
    /// Anyhow error
    #[error("{0}")]
    AnyhowError(#[from] anyhow::Error),

    #[error("Submit funcaptcha answer error ({0:?})")]
    SubmitAnswerError(anyhow::Error),
    #[error("Invalid arkose platform type ({0})")]
    InvalidPlatformType(String),
    #[error("Invalid public key ({0})")]
    InvalidPublicKey(String),
    #[error("No solver available or solver is invalid")]
    NoSolverAvailable,
    #[error("Solver task error: {0}")]
    SolverTaskError(String),
    #[error("Error creating arkose session error ({0:?})")]
    CreateSessionError(anyhow::Error),
    #[error("Invalid funcaptcha error")]
    InvalidFunCaptcha,
    #[error("Hex decode error")]
    HexDecodeError,
    #[error("Unsupported hash algorithm")]
    UnsupportedHashAlgorithm,
    #[error("Unable to find har related request entry")]
    HarEntryNotFound,
    #[error("Invalid HAR file")]
    InvalidHarFile,
    #[error("{0} not a file")]
    NotAFile(String),
    #[error("Failed to get HAR entry error ({0:?})")]
    FailedToGetHarEntry(Arc<anyhow::Error>),

    /// Deserialize error
    #[error("Deserialize error {0:?}")]
    DeserializeError(reqwest::Error),

    /// Serialize error
    #[error("Serialize error {0:?}")]
    SerializeError(#[from] serde_urlencoded::ser::Error),

    /// Funcaptcha error
    #[error("Funcaptcha submit error ({0})")]
    FuncaptchaSubmitError(String),
    #[error("Funcaptcha not solved error ({0})")]
    FuncaptchaNotSolvedError(String),
    #[error("Unknown game type ({0})")]
    UnknownGameType(u32),
    #[error("Unknown challenge type key: ({0})")]
    UnknownChallengeTypeKey(String),
    #[error("Unknow challenge")]
    UnknownChallenge,
    #[error("Invalid arkose token ({0})")]
    InvalidArkoseToken(String),

    /// Header parse error
    #[error("Invalid header ({0})")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),

    /// Request error
    #[error("Arkose request error ({0})")]
    RequestError(#[from] reqwest::Error),
}
