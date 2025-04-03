use async_trait::async_trait;
use crate::domain::{
    models::{PerlModule, ResponsibilityCluster, RefactoringProposal},
    traits::RefactoringProposer,
};
use crate::error::Error;

pub struct DefaultRefactoringProposer;

impl DefaultRefactoringProposer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RefactoringProposer for DefaultRefactoringProposer {
    async fn generate_proposal(
        &self,
        _module: &PerlModule,
        _responsibilities: &[ResponsibilityCluster]
    ) -> Result<RefactoringProposal, Error> {
        // TODO: Implement actual proposal generation logic
        Err(Error::ValidationError("Not implemented".to_string()))
    }
} 