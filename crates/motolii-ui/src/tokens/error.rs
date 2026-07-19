use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ProfileError {
    #[error("fixture profile parse error")]
    Parse {
        #[source]
        source: serde_json::Error,
    },
    #[error("{0}")]
    Semantic(String),
}

#[derive(Debug, Error)]
pub(crate) enum VerifyError {
    #[error("missing checked-in output {path}: {source}")]
    Read {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("checked-in adapter is stale or hand-edited (byte mismatch)")]
    Mismatch,
    #[error("{0}")]
    Semantic(String),
}

#[derive(Debug, Error)]
pub(crate) enum TokenError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("fixture profile: {0}")]
    Profile(#[source] ProfileError),
    #[error("verify: {0}")]
    Verify(#[source] VerifyError),
}
