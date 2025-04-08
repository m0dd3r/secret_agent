use std::path::PathBuf;
use std::env;
use clap::Parser;
use dotenv::dotenv;
use secret_agent::{
    parser::AIModuleParser,
    domain::traits::ModuleParser,
    error::Error,
};
use rig::providers::azure::Client;

#[derive(Parser, Debug)]
#[command(
    name = "secret_agent",
    about = "AI-powered Perl module refactoring tool",
    version,
    author
)]
struct Args {
    /// Path to the Perl module to analyze
    #[arg(short = 'p', long)]
    file: PathBuf,

    /// Output format (text or json)
    #[arg(short = 'o', long, default_value = "text")]
    format: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load environment variables from .env file
    dotenv().ok();

    let args = Args::parse();

    // Get Azure Openai API key from environment
    let api_key = env::var("AZURE_API_KEY")
        .map_err(|_| Error::AIError("AZURE_API_KEY environment variable not set. Please set it in your .env file.".to_string()))?;

    // Get Azure OpenAI API base URL from environment
    let base_url = env::var("AZURE_API_BASE_URL")
        .map_err(|_| Error::AIError("AZURE_API_BASE_URL environment variable not set. Please set it in your .env file.".to_string()))?;

    // Get Azure OpenAI API base URL from environment
    let api_version = env::var("AZURE_API_VERSION")
        .map_err(|_| Error::AIError("AZURE_API_VERSION environment variable not set. Please set it in your .env file.".to_string()))?;

    // Initialize the parser with API key and base url
    let client = Client::from_api_key(&api_key, &api_version, &base_url);
    let agent_builder = client.agent("gpt-4o-2024-08-06");
    let parser = AIModuleParser::new(agent_builder);

    // Parse and analyze the module
    let module = parser.parse_module(&args.file).await?;

    // Output the results based on format
    match args.format.as_str() {
        "json" => println!("{}", serde_json::to_string_pretty(&module)?),
        _ => {
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
        }
    }

    Ok(())
}
