# Secret Agent

A Rust-based tool for refactoring Perl modules using AI. This tool analyzes Perl modules, identifies distinct responsibilities, and proposes refactoring suggestions to improve code organization and maintainability.

## Features

- Parse and analyze Perl modules
- Identify distinct responsibilities within modules
- Generate refactoring proposals
- Validate dependencies and potential impacts
- AI-powered analysis for intelligent suggestions

## Project Structure

```
src/
├── domain/
│   ├── models.rs    # Core domain models
│   └── traits.rs    # Core traits and interfaces
├── parser/          # Perl module parsing
├── analyzer/        # Responsibility analysis
├── proposer/        # Refactoring proposal generation
├── validator/       # Dependency validation
├── error.rs        # Error types
└── lib.rs          # Library root
```

## Getting Started

1. Install Rust (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/secret_agent.git
   cd secret_agent
   ```

3. Build the project:
   ```bash
   cargo build
   ```

4. Run tests:
   ```bash
   cargo test
   ```

## Usage

```rust
use secret_agent::{
    parser::DefaultModuleParser,
    analyzer::AIResponsibilityAnalyzer,
    proposer::DefaultRefactoringProposer,
    validator::DefaultDependencyValidator,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = DefaultModuleParser::new();
    let analyzer = AIResponsibilityAnalyzer::new();
    let proposer = DefaultRefactoringProposer::new();
    let validator = DefaultDependencyValidator::new();

    // Parse a Perl module
    let module = parser.parse_module("path/to/module.pm").await?;

    // Analyze responsibilities
    let responsibilities = analyzer.analyze_module(&module).await?;

    // Generate refactoring proposal
    let proposal = proposer.generate_proposal(&module, &responsibilities).await?;

    // Validate dependencies
    let validation = validator.validate_dependencies(&proposal)?;

    println!("Validation result: {:?}", validation);
    Ok(())
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details. 