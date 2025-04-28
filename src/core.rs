use std::path::PathBuf;
use std::fs;
use crate::{
    config::Config,
    parser::AIModuleParser,
    proposer::AIRefactoringProposer,
    domain::{
        models::{PerlModule, RefactoringProposal},
        traits::{ModuleParser, RefactoringProposer},
    },
    error::Error,
};


pub struct App {
    config: Config,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn parse_module(&self, file: &PathBuf, format: &str, save: Option<&PathBuf>) -> Result<PerlModule, Error> {

        let parser = AIModuleParser::new(self.config.get_agent());
        let module = parser.parse_module(file).await?;

        // Save analysis to file if requested
        if let Some(save_path) = save {
            self.save_analysis_to_file(&module, save_path)?;
        }

        // Output the results based on format
        match format {
            "json" => println!("{}", serde_json::to_string_pretty(&module)?),
            _ => self.print_module_analysis(&module),
        }

        Ok(module)
    }

    pub async fn propose_refactoring(
        &self,
        module: &PerlModule,
        format: &str,
        output_dir: Option<&PathBuf>
    ) -> Result<(), Error> {
        if module.responsibility_clusters.is_empty() {
            return Err(Error::ValidationError("No responsibility clusters found to base refactoring on".to_string()));
        }


        let proposer = AIRefactoringProposer::new(self.config.get_agent());
        println!("Generating refactoring proposal...");
        let proposal = proposer.generate_proposal(module).await?;
        
        self.print_proposal(&proposal, format)?;
        self.save_modules(&proposal, output_dir)?;

        Ok(())
    }

    fn save_analysis_to_file(&self, module: &PerlModule, path: &PathBuf) -> Result<(), Error> {
        let json = serde_json::to_string_pretty(module)
            .map_err(|e| Error::SerializationError(e.to_string()))?;
        
        fs::write(path, json)
            .map_err(|e| Error::IOError(e))?;
        
        println!("Analysis saved to: {}", path.display());
        Ok(())
    }

    pub fn load_analysis_from_file(&self, path: &PathBuf) -> Result<PerlModule, Error> {
        let content = fs::read_to_string(path)
            .map_err(|e| Error::IOError(e))?;
        
        let module: PerlModule = serde_json::from_str(&content)
            .map_err(|e| Error::DeserializationError(format!("Failed to parse saved analysis: {}", e)))?;
        
        Ok(module)
    }

    fn print_module_analysis(&self, module: &PerlModule) {
        println!("Module Analysis Results:");
        println!("Name: {}", module.name);
        println!("Path: {}", module.path.display());
        println!("\nDependencies:");
        for dep in &module.dependencies {
            println!("  - {}", dep);
        }
        println!("\nSubroutines:");
        for sub in &module.subroutines {
            println!("\n  {}", sub.name);
            println!("  Lines: {}-{}", sub.line_start, sub.line_end);
            if !sub.dependencies.is_empty() {
                println!("  Dependencies:");
                for dep in &sub.dependencies {
                    println!("    - {}", dep);
                }
            }
        }
        println!("\nResponsibility Clusters:");
        for cluster in &module.responsibility_clusters {
            println!("\n  {}", cluster.name);
            println!("  Description: {}", cluster.description);
            println!("  Confidence: {:.2}", cluster.confidence);
            println!("  Related subroutines:");
            for sub in &cluster.related_subroutines {
                println!("    - {}", sub);
            }
            if let Some(name) = &cluster.suggested_module_name {
                println!("  Suggested module name: {}", name);
            }
        }
    }

    fn print_proposal(&self, proposal: &RefactoringProposal, format: &str) -> Result<(), Error> {
        match format {
            "json" => println!("{}", serde_json::to_string_pretty(&proposal)?),
            _ => {
                println!("Refactoring Proposal for {}", proposal.original_module.name);
                println!("\nSuggested modules:");
                
                for module in &proposal.suggested_modules {
                    println!("\n  {} (confidence: {:.2})", module.name, module.confidence);
                    println!("  Responsibility: {}", module.responsibility);
                    println!("  Subroutines: {}", module.subroutines.iter().map(|s| s.name.clone()).collect::<Vec<_>>().join(", "));
                    println!("  Dependencies: {}", module.dependencies.join(", "));
                }
                
                println!("\nImpact Analysis:");
                println!("  Complexity: {}", proposal.impact.complexity);
                println!("  Effort: {}", proposal.impact.effort);
                
                println!("\n  Risks:");
                for risk in &proposal.impact.risks {
                    println!("    - {}", risk);
                }
                
                println!("\n  Benefits:");
                for benefit in &proposal.impact.benefits {
                    println!("    - {}", benefit);
                }
            }
        }
        
        Ok(())
    }

    fn save_modules(&self, proposal: &RefactoringProposal, output_dir: Option<&PathBuf>) -> Result<(), Error> {
        let base_dir = match output_dir {
            Some(dir) => dir.clone(),
            None => {
                // Create a directory based on the original module name
                let dir = PathBuf::from(format!("refactored_{}", proposal.original_module.name));
                if !dir.exists() {
                    fs::create_dir_all(&dir).map_err(|e| Error::IOError(e))?;
                }
                dir
            }
        };
        
        println!("\nWriting refactored modules to: {}", base_dir.display());
        
        // Save each suggested module
        for module in &proposal.suggested_modules {
            // Convert module name to path (MyModule::Submodule -> MyModule/Submodule.pm)
            let path_parts: Vec<_> = module.name.split("::").collect();
            let mut file_path = base_dir.clone();
            
            // Create directory structure if needed
            if path_parts.len() > 1 {
                for part in &path_parts[0..path_parts.len()-1] {
                    file_path.push(part);
                    if !file_path.exists() {
                        fs::create_dir_all(&file_path).map_err(|e| Error::IOError(e))?;
                    }
                }
            }
            
            // Add filename with .pm extension
            file_path.push(format!("{}.pm", path_parts.last().unwrap_or(&"Unknown")));
            
            // Write the module code to file
            fs::write(&file_path, &module.suggested_code).map_err(|e| Error::IOError(e))?;
            
            println!("  - Written: {}", file_path.display());
        }
        
        Ok(())
    }
} 