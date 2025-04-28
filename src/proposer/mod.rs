use async_trait::async_trait;
use std::collections::HashMap;
use std::collections::HashSet;
use rig::agent::Agent;
use rig::agent::AgentBuilder;
use rig::completion::{CompletionModel, Prompt};
//use tokio::sync::Mutex;

use crate::domain::{
    models::{PerlModule, ResponsibilityCluster, RefactoringProposal, NewModuleProposal, RefactoringImpact},
    traits::RefactoringProposer,
};
use crate::error::Error;

/// Default implementation of the RefactoringProposer trait
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
    ) -> Result<RefactoringProposal, Error> {
        // TODO: Implement actual proposal generation logic
        Err(Error::ValidationError("Not implemented".to_string()))
    }
}

/// AI-powered implementation of the RefactoringProposer trait
pub struct AIRefactoringProposer<M: CompletionModel> {
    agent: Agent<M>,
    // Cancel channel for future async operation cancellation
    //cancel: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
}

impl<M: CompletionModel> AIRefactoringProposer<M> {
    pub fn new(agent_builder: AgentBuilder<M>) -> Self {
        Self {
            agent: agent_builder
                .preamble("You are a Perl refactoring expert. You will generate clean, well-structured Perl modules based on responsibility clusters.")
                .build(),
            //cancel: Mutex::new(None),
        }
    }

    async fn generate_module_code(&self, original_module: &PerlModule, cluster: &ResponsibilityCluster) -> Result<String, Error> {
        // Get all subroutines in this cluster
        let mut subroutines = Vec::new();
        let mut dependencies = HashSet::new();

        for sub_name in &cluster.related_subroutines {
            if let Some(sub) = original_module.subroutines.iter().find(|s| &s.name == sub_name) {
                subroutines.push(sub);
                
                // Collect dependencies from each subroutine
                for dep in &sub.dependencies {
                    dependencies.insert(dep.clone());
                }
            }
        }

        // Get module name
        let module_name = cluster.suggested_module_name.clone()
            .unwrap_or_else(|| format!("{}::{}", original_module.name, cluster.name.replace(" ", "")));

        // Format subroutines as text
        let subroutines_text = subroutines.iter()
            .map(|s| s.code.clone())
            .collect::<Vec<_>>()
            .join("\n\n");

        // Create prompt
        let prompt = format!(
            r#"You are a Perl refactoring expert. Your task is to create a new Perl module for a specific responsibility cluster.

            Original module: {}
            New module name: {}
            Responsibility: {} - {}
            Confidence: {}
                    
            Subroutines from the original module that should be included in this new module:
            ```perl
            {}
            ```
                    
            Based on this information, generate a complete, well-structured Perl module that:
            1. Has proper package declaration
            2. Includes all necessary pragmas (strict, warnings) and dependencies
            3. Contains all the subroutines provided, keeping their functionality identical
            4. Exports the necessary subroutines
            5. Has appropriate documentation and structure
            6. Makes minimal changes to the actual code of each subroutine
                    
            Only return the complete Perl module code with no additional explanation.
            "#,
            original_module.name,
            module_name,
            cluster.name,
            cluster.description,
            cluster.confidence,
            subroutines_text
        );

        // Generate code using AI - pass the prompt as a value
        let response = self.agent.prompt(prompt).await
            .map_err(|e| Error::AIError(format!("Failed to generate module code: {}", e)))?;

        Ok(response)
    }
}

#[async_trait]
impl<M: CompletionModel> RefactoringProposer for AIRefactoringProposer<M> {
    async fn generate_proposal(
        &self,
        module: &PerlModule,
    ) -> Result<RefactoringProposal, Error> {
        // Skip low confidence clusters
        let high_confidence_clusters: Vec<&ResponsibilityCluster> = module.responsibility_clusters.iter()
            .filter(|c| c.confidence >= 0.7)
            .collect();

        if high_confidence_clusters.is_empty() {
            return Err(Error::ValidationError("No high confidence responsibility clusters found".to_string()));
        }

        // Generate new modules for each responsibility cluster
        let mut suggested_modules = Vec::new();
        let mut module_sub_map = HashMap::new();

        for cluster in &high_confidence_clusters {
            // Generate code for this module
            let code = self.generate_module_code(module, cluster).await?;
            
            // Get all subroutines in this cluster
            let mut subroutines = Vec::new();
            let mut dependencies = HashSet::new();

            for sub_name in &cluster.related_subroutines {
                if let Some(sub) = module.subroutines.iter().find(|s| &s.name == sub_name) {
                    // Track which subroutines have been assigned to which modules
                    module_sub_map.entry(sub.name.clone())
                       .or_insert_with(Vec::new)
                       .push(cluster.name.clone());
                    
                    subroutines.push(sub.clone());
                    
                    // Collect dependencies from each subroutine
                    for dep in &sub.dependencies {
                        dependencies.insert(dep.clone());
                    }
                }
            }

            // Determine module name
            let name = cluster.suggested_module_name.clone()
                .unwrap_or_else(|| format!("{}::{}", module.name, cluster.name.replace(" ", "")));

            // Create new module proposal
            suggested_modules.push(NewModuleProposal {
                name,
                responsibility: cluster.description.clone(),
                subroutines,
                dependencies: dependencies.into_iter().collect(),
                suggested_code: code,
                confidence: cluster.confidence,
            });
        }

        // Analyze impact of the refactoring
        let impact = self.analyze_impact(&suggested_modules, &module_sub_map).await?;

        Ok(RefactoringProposal {
            original_module: module.clone(),
            suggested_modules,
            impact,
        })
    }
}

impl<M: CompletionModel> AIRefactoringProposer<M> {
    async fn analyze_impact(
        &self,
        suggested_modules: &[NewModuleProposal],
        module_sub_map: &HashMap<String, Vec<String>>
    ) -> Result<RefactoringImpact, Error> {
        // Count duplicated subroutines (appear in multiple modules)
        let duplicated_subs = module_sub_map.iter()
            .filter(|(_, modules)| modules.len() > 1)
            .count();

        // Estimate complexity based on number of modules and duplicated subroutines
        let complexity = suggested_modules.len() as u32 + duplicated_subs as u32;

        // Estimate effort based on complexity and total subroutines
        let effort = match complexity {
            0..=2 => "Low: Less than 1 day",
            3..=5 => "Medium: 1-3 days",
            _ => "High: More than 3 days",
        }.to_string();

        // Identify risks
        let mut risks = Vec::new();
        
        if duplicated_subs > 0 {
            risks.push(format!("{} subroutines appear in multiple modules, which may lead to code duplication", duplicated_subs));
        }
        
        if suggested_modules.len() > 5 {
            risks.push("Large number of modules may increase maintenance overhead".to_string());
        }
        
        // Identify benefits
        let benefits = vec![
            "Improved code organization with clear separation of responsibilities".to_string(),
            "Better maintainability through modular design".to_string(),
            format!("Reduced cognitive load with {} focused modules", suggested_modules.len()),
        ];
        
        Ok(RefactoringImpact {
            complexity,
            effort,
            risks,
            benefits,
        })
    }
} 