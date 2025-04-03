# PerlModuleRefactorAgent Specification

## 1. System Overview

### Core Purpose
Build an AI-powered tool that analyzes Perl modules and identifies potential refactoring opportunities based on the Single Responsibility Principle, suggesting logical separations into smaller, more focused modules.

### High-Level Architecture
- Built in Rust for performance and safety
- Uses OpenAI API for semantic analysis
- Provides actionable refactoring suggestions with confidence scores
- Validates dependencies and module relationships

## 2. Technical Stack

### Dependencies
```toml
[package]
name = "perl-module-refactor"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "AI-powered Perl module refactoring tool"

[dependencies]
# Async Runtime
tokio = { version = "1.35", features = ["full", "time"] }
async-trait = "0.1.77"
futures = "0.3"
pin-project = "1.1"  # For proper async trait implementation

# AI Integration
async-openai = "0.18"
backoff = { version = "0.4", features = ["tokio"] }  # For API retries

# Perl parsing
tree-sitter = "0.20"
tree-sitter-perl = "0.0.1"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# CLI
clap = { version = "4.4", features = ["derive"] }
indicatif = "0.17"
console = "0.15"
tracing = "0.1"  # For structured logging
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Testing
rstest = "0.18"
mockall = "0.12"
tempfile = "3.9"
tokio-test = "0.4"  # For testing async code
test-log = "0.2"    # For capturing logs in tests

[dev-dependencies]
criterion = { version = "0.5", features = ["async_tokio"] }  # For benchmarking
```

## 3. Core Components

### Domain Models

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerlModule {
    pub name: String,
    pub path: String,
    pub content: String,
    pub subroutines: Vec<Subroutine>,
    pub dependencies: Vec<String>,
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
    pub related_subroutines: Vec<Subroutine>,
    pub suggested_module_name: String,
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
```

### Core Traits

```rust
use std::path::Path;
use async_trait::async_trait;

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
```

### Error Handling

```rust
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to parse Perl module at {path}: {message}")]
    ParseError {
        path: PathBuf,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("Failed to analyze module: {message}")]
    AnalysisError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("Invalid module structure: {message}")]
    ValidationError {
        message: String,
        details: Vec<String>,
    },
    
    #[error("AI service error: {message}")]
    AIError {
        message: String,
        retryable: bool,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    
    #[error("Operation cancelled")]
    Cancelled,
}

impl Error {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Error::AIError { retryable: true, .. })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,  // Added for non-blocking issues
}
```

## 4. Implementation Components

### Module Parser
- Uses tree-sitter for accurate Perl parsing
- Extracts subroutines and their dependencies
- Handles Perl's complex syntax features
- Provides detailed source location information

### AI Integration
- Uses OpenAI API for semantic analysis
- Custom prompt engineering for responsibility identification
- Confidence scoring for suggestions
- Fallback mechanisms for API failures

### Refactoring Engine
- Generates clean, maintainable code
- Preserves original functionality
- Handles dependencies correctly
- Provides detailed impact analysis

### CLI Interface
```rust
use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Path to the Perl module to analyze
    #[arg(short, long)]
    module: PathBuf,

    /// Minimum confidence threshold (0.0-1.0)
    #[arg(short, long, default_value = "0.8")]
    confidence: f32,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    format: String,
    
    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value = "info")]
    log_level: String,
    
    /// Timeout in seconds for the analysis
    #[arg(long, default_value = "30")]
    timeout: u64,
}
```

## 5. Usage Example

```rust
use tracing::{info, error};
use tokio::time::timeout;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(&args.log_level)
        .init();
    
    info!("Starting analysis of {}", args.module.display());
    
    let analyzer = PerlModuleAnalyzer::new(Config::default());
    
    // Add timeout for the analysis
    match timeout(
        Duration::from_secs(args.timeout),
        analyzer.analyze_module(&args.module)
    ).await {
        Ok(Ok(proposal)) => {
            match args.format.as_str() {
                "json" => println!("{}", serde_json::to_string_pretty(&proposal)?),
                _ => print_proposal(&proposal),
            }
            Ok(())
        }
        Ok(Err(e)) => {
            error!("Analysis failed: {}", e);
            Err(e)
        }
        Err(_) => {
            error!("Analysis timed out after {} seconds", args.timeout);
            Err(Error::AnalysisError {
                message: "Analysis timed out".to_string(),
                source: None,
            })
        }
    }
}
```

## 6. Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use mockall::predicate::*;
    use test_log::test;  // For log capture in tests
    
    #[rstest]
    #[case("test.pm", true)]
    #[case("test.pl", true)]
    #[case("test.txt", false)]
    #[test]
    fn test_file_validation(#[case] path: &str, #[case] expected: bool) {
        let validator = FileValidator::new();
        assert_eq!(validator.is_valid_perl_file(path), expected);
    }
    
    #[tokio::test]
    async fn test_responsibility_analysis() {
        let module = create_test_module();
        let mut analyzer = MockResponsibilityAnalyzer::new();
        analyzer
            .expect_analyze_module()
            .times(1)
            .returning(|_| Ok(vec![/* ... */]));
            
        let result = analyzer.analyze_module(&module).await.unwrap();
        
        assert!(!result.is_empty());
        assert!(result[0].confidence > 0.8);
    }
    
    // Add benchmark tests
    #[cfg(test)]
    mod benches {
        use super::*;
        use criterion::{criterion_group, criterion_main, Criterion};
        
        pub fn benchmark_parsing(c: &mut Criterion) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            c.bench_function("parse large module", |b| {
                b.to_async(&rt).iter(|| async {
                    let parser = PerlModuleParser::new();
                    parser.parse_module("test_data/large_module.pm").await
                })
            });
        }
        
        criterion_group!(benches, benchmark_parsing);
        criterion_main!(benches);
    }
}
```

### Integration Tests
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_full_analysis_workflow() {
        let test_file = create_test_perl_module();
        let analyzer = PerlModuleAnalyzer::new(Config::default());
        
        let result = analyzer.analyze_module(test_file.path()).await.unwrap();
        assert!(result.suggested_modules.len() > 0);
        
        for module in &result.suggested_modules {
            assert!(module.confidence > 0.8);
            assert!(!module.suggested_code.is_empty());
        }
    }
}
```

## 7. Performance Requirements

1. Analysis Speed
   - Module parsing: < 100ms for files up to 10,000 lines
   - Responsibility analysis: < 2s per module
   - Total processing: < 5s for typical modules

2. Resource Usage
   - Memory: < 200MB per analysis
   - CPU: Efficient use of async/await for I/O operations
   - Disk: Minimal temporary storage needed

3. Reliability
   - Graceful handling of API failures
   - Clear error messages
   - No data loss or corruption 