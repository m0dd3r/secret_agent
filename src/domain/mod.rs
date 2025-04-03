pub mod models;
pub mod traits;

#[cfg(test)]
mod tests {
    use super::*;
    use models::*;

    #[test]
    fn test_perl_module_creation() {
        let module = PerlModule {
            name: "Test::Module".to_string(),
            path: "test/module.pm".into(),
            content: "package Test::Module;\n\nuse strict;\n".to_string(),
            subroutines: vec![],
            dependencies: vec![],
        };

        assert_eq!(module.name, "Test::Module");
        assert_eq!(module.path.to_str().unwrap(), "test/module.pm");
    }

    #[test]
    fn test_subroutine_creation() {
        let sub = Subroutine {
            name: "test_sub".to_string(),
            code: "sub test_sub { return 42; }".to_string(),
            line_start: 1,
            line_end: 3,
            dependencies: vec!["Dependency::One".to_string()],
        };

        assert_eq!(sub.name, "test_sub");
        assert_eq!((sub.line_start, sub.line_end), (1, 3));
        assert_eq!(sub.dependencies.len(), 1);
    }

    #[test]
    fn test_responsibility_cluster_creation() {
        let cluster = ResponsibilityCluster {
            name: "Data validation".to_string(),
            description: "Handles input validation logic".to_string(),
            related_subroutines: vec![
                Subroutine {
                    name: "validate_input".to_string(),
                    code: "sub validate_input { ... }".to_string(),
                    line_start: 1,
                    line_end: 5,
                    dependencies: vec![],
                },
                Subroutine {
                    name: "check_format".to_string(),
                    code: "sub check_format { ... }".to_string(),
                    line_start: 7,
                    line_end: 10,
                    dependencies: vec![],
                },
            ],
            suggested_module_name: "MyApp::Validation".to_string(),
            confidence: 0.85,
        };

        assert_eq!(cluster.name, "Data validation");
        assert_eq!(cluster.related_subroutines.len(), 2);
        assert_eq!(cluster.suggested_module_name, "MyApp::Validation");
    }
} 