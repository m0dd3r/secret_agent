use std::env;
use dotenv::dotenv;
use secret_agent::{
    parser::AIModuleParser,
    domain::traits::ModuleParser,
};
use rig::providers::azure::Client;

#[tokio::test]
async fn test_parse() {
    // Load environment variables from .env file
    dotenv().ok();

    let file = "test_modules/Calculator.pm";

    // Get Azure Openai API key from environment
    let api_key = env::var("AZURE_API_KEY").unwrap();

    // Get Azure OpenAI API base URL from environment
    let base_url = env::var("AZURE_API_BASE_URL").unwrap();

    // Get Azure OpenAI API base URL from environment
    let api_version = env::var("AZURE_API_VERSION").unwrap();

    // Initialize the parser with API key and base url
    let client = Client::from_api_key(&api_key, &api_version, &base_url);
    let agent_builder = client.agent("gpt-4o-2024-08-06");
    let parser = AIModuleParser::new(agent_builder);

    // Parse and analyze the module
    let module = parser.parse_module(&file).await.unwrap();

    assert_eq!(module.name, "Calculator");
    assert_eq!(module.dependencies, vec!["Math::Complex", "List::Util"]);
    assert_eq!(module.subroutines.len(), 3);
}
