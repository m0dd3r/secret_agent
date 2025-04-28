use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerlModule {
    pub name: String,
    pub path: PathBuf,
    pub content: String,
    pub subroutines: Vec<Subroutine>,
    pub dependencies: Vec<String>,
    pub responsibility_clusters: Vec<ResponsibilityCluster>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subroutine {
    pub name: String,
    pub code: String,
    pub line_start: usize,
    pub line_end: usize,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsibilityCluster {
    pub name: String,
    pub description: String,
    pub related_subroutines: Vec<String>,
    pub suggested_module_name: Option<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringProposal {
    pub original_module: PerlModule,
    pub suggested_modules: Vec<NewModuleProposal>,
    pub impact: RefactoringImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewModuleProposal {
    pub name: String,
    pub responsibility: String,
    pub subroutines: Vec<Subroutine>,
    pub dependencies: Vec<String>,
    pub suggested_code: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringImpact {
    pub complexity: u32,
    pub effort: String,
    pub risks: Vec<String>,
    pub benefits: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
} 