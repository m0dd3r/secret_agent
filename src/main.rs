use std::path::PathBuf;
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use secret_agent::{App, Config, Error};

#[derive(Parser, Debug)]
#[command(
    name = "secret_agent",
    about = "AI-powered Perl module refactoring tool",
    version,
    author
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Parse and analyze a Perl module
    Parse {
        /// Path to the Perl module to analyze
        #[arg(short = 'p', long)]
        file: PathBuf,

        /// Output format (text or json)
        #[arg(short = 'o', long, default_value = "text")]
        format: String,
        
        /// Save analysis to file
        #[arg(short = 's', long)]
        save: Option<PathBuf>,
    },
    
    /// Generate refactoring proposals for a Perl module
    Propose {
        /// Path to the Perl module to refactor
        #[arg(short = 'p', long)]
        file: Option<PathBuf>,

        /// Path to a saved analysis file
        #[arg(short = 'a', long)]
        analysis: Option<PathBuf>,

        /// Output directory for the generated module files
        #[arg(short = 'd', long)]
        output_dir: Option<PathBuf>,

        /// Output format (text or json)
        #[arg(short = 'o', long, default_value = "text")]
        format: String,
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load environment variables from .env file
    dotenv().ok();

    let args = Args::parse();
    let app = App::new(Config::from_env());

    match &args.command {
        Commands::Parse { file, format, save } => {
            app.parse_module(file, format, save.as_ref()).await?;
        },
        Commands::Propose { file, analysis, output_dir, format } => {
            let module = match (file, analysis) {
                (Some(file_path), None) => {
                    println!("Analyzing module: {}", file_path.display());
                    app.parse_module(file_path, format, None).await?
                },
                (None, Some(analysis_path)) => {
                    println!("Loading analysis from: {}", analysis_path.display());
                    app.load_analysis_from_file(analysis_path)?
                },
                (Some(_), Some(_)) => {
                    return Err(Error::ValidationError(
                        "Cannot provide both file and analysis. Choose one or the other.".to_string()
                    ));
                },
                (None, None) => {
                    return Err(Error::ValidationError(
                        "Must provide either file to analyze or path to saved analysis.".to_string()
                    ));
                }
            };

            println!("Analysis complete. Found {} responsibility clusters.", module.responsibility_clusters.len());
            app.propose_refactoring(&module, format, output_dir.as_ref()).await?;
        }
    }

    Ok(())
}
