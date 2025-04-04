use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    /// Error occurred while parsing a Perl module
    ParseError(String),
    
    /// Error occurred during responsibility analysis
    AnalysisError(String),
    
    /// Error occurred during validation
    ValidationError(String),
    
    /// Error occurred while interacting with AI service
    AIError(String),
    
    /// IO error occurred
    IOError(io::Error),

    /// JSON serialization/deserialization error
    SerdeError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Error::AnalysisError(msg) => write!(f, "Analysis error: {}", msg),
            Error::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Error::AIError(msg) => write!(f, "AI service error: {}", msg),
            Error::IOError(err) => write!(f, "IO error: {}", err),
            Error::SerdeError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IOError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IOError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeError(err.to_string())
    }
} 