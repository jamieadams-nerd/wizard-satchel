/// All errors produced by the UMRS c2pa library.
#[derive(Debug, thiserror::Error)]
pub enum InspectError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("C2PA error: {0}")]
    C2pa(#[from] c2pa::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Signing error: {0}")]
    Signing(String),

    #[error("Hash error: {0}")]
    Hash(String),

    #[error("Algorithm '{0}' is not in the FIPS-safe allowed set")]
    UnsafeAlgorithm(String),

    #[error("Refusing to overwrite previously signed file: {0}")]
    AlreadySigned(String),
}
