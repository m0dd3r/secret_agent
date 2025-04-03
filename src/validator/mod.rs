use crate::domain::{
    models::{RefactoringProposal, ValidationResult},
    traits::DependencyValidator,
};
use crate::error::Error;

pub struct DefaultDependencyValidator;

impl DefaultDependencyValidator {
    pub fn new() -> Self {
        Self
    }
}

impl DependencyValidator for DefaultDependencyValidator {
    fn validate_dependencies(&self, _proposal: &RefactoringProposal) -> Result<ValidationResult, Error> {
        // TODO: Implement actual validation logic
        Err(Error::ValidationError("Not implemented".to_string()))
    }
} 