use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// Error occurred while parsing a Perl module
    #[error("Parse error: {0}")]
    ParseError(String),
    
    /// Error occurred during responsibility analysis
    #[error("Analysis error: {0}")]
    AnalysisError(String),
    
    /// Error occurred during validation
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Error occurred while interacting with AI service
    #[error("AI service error: {0}")]
    AIError(String),
    
    /// IO error occurred
    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// JSON deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// No AI provider available
    #[error("No AI provider available")]
    NoAIProvider,

    /// Missing environment variable
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerializationError(err.to_string())
    }
} 