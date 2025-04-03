use std::path::Path;
use async_trait::async_trait;
use crate::domain::models::{PerlModule, ResponsibilityCluster, RefactoringProposal, ValidationResult};
use crate::error::Error;

#[async_trait]
pub trait ModuleParser: Send + Sync {
    /// Parse a Perl module from the given path
    /// 
    /// # Errors
    /// 
    /// Returns `Error::ParseError` if the module cannot be parsed
    /// Returns `Error::IOError` if the file cannot be read
    async fn parse_module(&self, path: impl AsRef<Path> + Send) -> Result<PerlModule, Error>;
}

#[async_trait]
pub trait ResponsibilityAnalyzer: Send + Sync {
    /// Analyze a Perl module to identify distinct responsibilities
    /// 
    /// # Errors
    /// 
    /// Returns `Error::AnalysisError` if the analysis fails
    /// Returns `Error::AIError` if the AI service fails
    async fn analyze_module(&self, module: &PerlModule) -> Result<Vec<ResponsibilityCluster>, Error>;
    
    /// Cancel any ongoing analysis
    async fn cancel(&self);
}

#[async_trait]
pub trait RefactoringProposer: Send + Sync {
    /// Generate a refactoring proposal based on identified responsibilities
    /// 
    /// # Errors
    /// 
    /// Returns `Error::ValidationError` if the proposal cannot be generated
    async fn generate_proposal(
        &self,
        module: &PerlModule,
        responsibilities: &[ResponsibilityCluster]
    ) -> Result<RefactoringProposal, Error>;
}

pub trait DependencyValidator: Send + Sync {
    /// Validate dependencies in a refactoring proposal
    /// 
    /// # Errors
    /// 
    /// Returns `Error::ValidationError` if dependencies are invalid
    fn validate_dependencies(&self, proposal: &RefactoringProposal) -> Result<ValidationResult, Error>;
} 