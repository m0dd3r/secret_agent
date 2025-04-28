use more_asserts::assert_ge;
use test_context::{AsyncTestContext, test_context};
use std::path::PathBuf;
use secret_agent::{App, Config};
use dotenv::dotenv;

struct TestContext {
    app: App,
    calculator_file: PathBuf,
    order_manager_file: PathBuf,
    calculator_analysis: PathBuf,
}

impl AsyncTestContext for TestContext {
    async fn setup() -> Self {
        // Load environment variables from .env file
        dotenv().ok();

        // Create test files
        let calculator_file = PathBuf::from("tests/data/Calculator.pm");
        let order_manager_file = PathBuf::from("tests/data/OrderManager.pm");
        let calculator_analysis = PathBuf::from("tests/data/calculator_analysis.json");

        Self {
            app: App::new(Config::from_env()),
            calculator_file,
            order_manager_file,
            calculator_analysis,
        }
    }

    async fn teardown(self) {
        // Files are automatically cleaned up when dropped
    }
}

#[test_context(TestContext)]
#[tokio::test]
async fn test_parse_calculator(ctx: &TestContext) {
    let module = ctx.app.parse_module(&ctx.calculator_file, "json", None).await.unwrap();

    assert_eq!(module.name, "Calculator");
    assert_eq!(module.dependencies, vec!["strict", "warnings", "Math::Complex", "List::Util"]);
    assert_eq!(module.subroutines.len(), 3);
    assert_eq!(module.responsibility_clusters.len(), 2);

    // Verify specific subroutine details
    let avg_sub = module.subroutines.iter().find(|s| s.name == "calculate_average").unwrap();
    assert_eq!(avg_sub.dependencies, vec!["List::Util"]);
}

#[test_context(TestContext)]
#[tokio::test]
async fn test_parse_order_manager(ctx: &TestContext) {
    let module = ctx.app.parse_module(&ctx.order_manager_file, "json", None).await.unwrap();

    assert_eq!(module.name, "OrderManager");
    assert!(module.dependencies.contains(&"strict".to_string()));
    assert!(module.dependencies.contains(&"warnings".to_string()));
    assert!(module.dependencies.contains(&"DBI".to_string()));
    assert_ge!(module.responsibility_clusters.len(), 2);

    // Verify specific cluster details
    //let order_mgmt = module.responsibility_clusters.iter()
    //    .find(|c| c.name == "Order Management")
    //    .unwrap();
    //assert_eq!(order_mgmt.confidence, 0.9);
    //assert!(order_mgmt.related_subroutines.contains(&"save_order".to_string()));
}

#[test_context(TestContext)]
#[tokio::test]
async fn test_save_and_load_analysis(ctx: &TestContext) -> Result<(), Box<dyn std::error::Error>> {
    let loaded = ctx.app.load_analysis_from_file(&ctx.calculator_analysis)?;
    
    // Verify the loaded data matches the original
    assert_eq!(loaded.name, "Calculator");
    assert_eq!(loaded.dependencies, vec!["strict", "warnings", "Math::Complex", "List::Util"]);
    assert_eq!(loaded.subroutines.len(), 3);
    assert_eq!(loaded.responsibility_clusters.len(), 2);
    
    Ok(())
} 