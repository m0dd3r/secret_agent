# Perl Module Refactoring Assistant Specification

## 1. System Overview

### Purpose
An AI-powered system that analyzes Perl modules to identify refactoring opportunities based on responsibility clustering and code analysis. The system uses advanced language models to understand code structure, identify cohesive responsibilities, and propose well-structured refactoring solutions.

### Core Features
- Automated Perl module parsing and analysis
- Responsibility cluster identification
- AI-driven refactoring proposal generation
- Dependency validation and impact analysis
- Clean code generation with proper documentation

## 2. Architecture Components

### Module Parser (`AIModuleParser`)
- Parses Perl module files into structured representations
- Extracts subroutines, dependencies, and code structure
- Uses AI to identify responsibility clusters
- Implements the `ModuleParser` trait

### Refactoring Proposer (`AIRefactoringProposer`)
- Generates refactoring proposals based on responsibility analysis
- Creates new module suggestions with proper structure
- Validates dependencies and analyzes impact
- Implements the `RefactoringProposer` trait

### Core Models
```rust
PerlModule {
    name: String,
    path: PathBuf,
    content: String,
    subroutines: Vec<Subroutine>,
    dependencies: Vec<String>,
    responsibility_clusters: Vec<ResponsibilityCluster>
}

Subroutine {
    name: String,
    code: String,
    line_start: usize,
    line_end: usize,
    dependencies: Vec<String>
}

ResponsibilityCluster {
    name: String,
    description: String,
    related_subroutines: Vec<String>,
    suggested_module_name: Option<String>,
    confidence: f32
}

RefactoringProposal {
    original_module: PerlModule,
    suggested_modules: Vec<NewModuleProposal>,
    impact: RefactoringImpact
}
```

## 3. AI Integration

### Analysis Prompt Template
```
Analyze this Perl module and extract its structure and responsibilities. Return ONLY a raw JSON object containing:
- package_name: The name of the Perl package/module
- subroutines: Array of objects with:
    - name: Subroutine name
    - code: Complete subroutine code
    - line_start: Starting line number
    - line_end: Ending line number
    - dependencies: Array of module dependencies
- dependencies: Array of ALL module dependencies
- responsibility_clusters: Array of objects with:
    - name: Descriptive cluster name
    - description: Clear explanation of functionality
    - related_subroutines: Array of related subroutine names
    - suggested_module_name: Suggested new module name
    - confidence: Float between 0.0 and 1.0
```

### Refactoring Prompt Template
```
You are a Perl refactoring expert. Create a new Perl module for a specific responsibility cluster.

Original module: {original_module_name}
New module name: {new_module_name}
Responsibility: {cluster_name} - {cluster_description}
Confidence: {confidence}

Subroutines to include:
{subroutines_code}

Generate a complete, well-structured Perl module with:
1. Proper package declaration
2. All necessary pragmas and dependencies
3. Identical subroutine functionality
4. Appropriate exports
5. Complete documentation
6. Minimal code changes
```

### AI Agent Preambles
1. Module Parser:
   ```
   You are a Perl code analyzer. You will analyze Perl code and extract its structure and identify cohesive responsibilities.
   ```

2. Refactoring Proposer:
   ```
   You are a Perl refactoring expert. You will generate clean, well-structured Perl modules based on responsibility clusters.
   ```

## 4. Implementation Guidelines

### Responsibility Analysis
- Group subroutines by shared purpose and dependencies
- Consider semantic relationships in naming and functionality
- Allow subroutines to belong to multiple clusters
- Assign high confidence (>0.8) only for clear relationships
- Include all dependencies, including pragmas

### Code Generation
- Maintain original functionality
- Include proper documentation
- Handle dependencies correctly
- Follow Perl best practices
- Preserve existing code structure where possible

### Error Handling
- Proper error propagation
- Clear error messages
- Validation at each step
- Graceful failure handling

## 5. Testing Strategy

### Unit Tests
- Mock AI responses for deterministic testing
- Validate parsing accuracy
- Test error handling
- Verify responsibility clustering

### Integration Tests
- End-to-end refactoring workflows
- AI integration testing
- File system operations
- Error scenarios

## 6. Future Enhancements

### Potential Features
- Interactive refactoring suggestions
- Batch processing capabilities
- Configuration options for analysis sensitivity
- Custom responsibility clustering rules
- Integration with development environments

### Performance Optimizations
- Caching of analysis results
- Parallel processing for large codebases
- Optimized AI prompt engineering
- Resource usage improvements 