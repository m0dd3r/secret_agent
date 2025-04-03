use std::path::Path;
use async_trait::async_trait;
use tokio::fs;
use async_openai::{
    Client,
    types::{CreateChatCompletionRequestArgs, ChatCompletionRequestMessage, Role},
};
use backoff::ExponentialBackoff;
use serde::{Deserialize, Serialize};
use crate::domain::{
    models::{PerlModule, Subroutine},
    traits::ModuleParser,
};
use crate::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct ParsedSubroutine {
    name: String,
    code: String,
    line_start: usize,
    line_end: usize,
    dependencies: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ParseResponse {
    subroutines: Vec<ParsedSubroutine>,
    dependencies: Vec<String>,
    package_name: Option<String>,
}

pub struct AIModuleParser {
    client: Client,
}

impl AIModuleParser {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub fn with_client(client: Client) -> Self {
        Self { client }
    }

    async fn analyze_code(&self, content: &str) -> Result<ParseResponse, Error> {
        let prompt = format!(
            r#"Analyze this Perl module and extract its structure. Respond with a JSON object containing:
            1. All subroutines (name, code, line numbers, and dependencies used within each sub)
            2. Module-level dependencies (use/require statements)
            3. Package name if present

            Perl module:
            ```perl
            {}
            ```

            Format the response as a JSON object with this structure:
            {{
                "subroutines": [
                    {{
                        "name": "sub_name",
                        "code": "complete sub code",
                        "line_start": line_number,
                        "line_end": line_number,
                        "dependencies": ["modules", "used", "in", "sub"]
                    }}
                ],
                "dependencies": ["all", "module", "level", "dependencies"],
                "package_name": "Module::Name"
            }}

            Be precise with line numbers and include the complete subroutine code."#,
            content
        );

        let backoff = ExponentialBackoff::default();
        let response = backoff::future::retry(backoff, || async {
            let response = self.client
                .chat()
                .create(CreateChatCompletionRequestArgs::default()
                    .model("gpt-4")
                    .messages([ChatCompletionRequestMessage {
                        role: Role::User,
                        content: prompt.clone(),
                        name: None,
                        function_call: None,
                    }])
                    .response_format(async_openai::types::ChatCompletionResponseFormat::json_object())
                    .build()?
                )
                .await
                .map_err(|e| backoff::Error::Permanent(Error::AIError(e.to_string())))?;

            let content = response.choices
                .first()
                .ok_or_else(|| backoff::Error::Permanent(Error::AIError("No response from AI".to_string())))?
                .message.content
                .as_ref()
                .ok_or_else(|| backoff::Error::Permanent(Error::AIError("Empty response from AI".to_string())))?;

            serde_json::from_str::<ParseResponse>(content)
                .map_err(|e| backoff::Error::Permanent(Error::ParseError(format!("Failed to parse AI response: {}", e))))
        }).await?;

        Ok(response)
    }
}

#[async_trait]
impl ModuleParser for AIModuleParser {
    async fn parse_module(&self, path: impl AsRef<Path> + Send) -> Result<PerlModule, Error> {
        let content = fs::read_to_string(path.as_ref()).await
            .map_err(|err| Error::IOError(err))?;

        let analysis = self.analyze_code(&content).await?;

        let subroutines = analysis.subroutines.into_iter()
            .map(|sub| Subroutine {
                name: sub.name,
                code: sub.code,
                line_start: sub.line_start,
                line_end: sub.line_end,
                dependencies: sub.dependencies,
            })
            .collect();

        Ok(PerlModule {
            name: analysis.package_name.unwrap_or_else(|| {
                path.as_ref()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            }),
            path: path.as_ref().to_path_buf(),
            content,
            subroutines,
            dependencies: analysis.dependencies,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::NamedTempFile;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_parse_simple_module() -> Result<(), Box<dyn std::error::Error>> {
        // Skip if no OPENAI_API_KEY
        if env::var("OPENAI_API_KEY").is_err() {
            return Ok(());
        }

        let content = r#"package Test::Module;
use strict;
use warnings;

sub hello {
    my ($name) = @_;
    return "Hello, $name!";
}

sub goodbye {
    my ($name) = @_;
    return "Goodbye, $name!";
}
1;
"#;
        let temp_file = NamedTempFile::new()?;
        fs::write(&temp_file, content).await?;

        let parser = AIModuleParser::new();
        let module = parser.parse_module(temp_file.path()).await?;

        assert_eq!(module.name, "Test::Module");
        assert!(!module.subroutines.is_empty());
        assert!(module.dependencies.contains(&"strict".to_string()));
        assert!(module.dependencies.contains(&"warnings".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_with_mock_ai() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;
        
        let mock_response = ParseResponse {
            subroutines: vec![
                ParsedSubroutine {
                    name: "test_sub".to_string(),
                    code: "sub test_sub { return 42; }".to_string(),
                    line_start: 1,
                    line_end: 3,
                    dependencies: vec![],
                },
            ],
            dependencies: vec!["strict".to_string()],
            package_name: Some("Test::Mock".to_string()),
        };

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{
                    "message": {
                        "content": serde_json::to_string(&mock_response)?
                    }
                }]
            })))
            .mount(&mock_server)
            .await;

        let client = Client::with_config(
            async_openai::config::Config::new()
                .with_api_base(&mock_server.uri())
                .with_api_key("test-key"),
        );

        let parser = AIModuleParser::with_client(client);
        let content = "package Test::Mock;\nuse strict;\nsub test_sub { return 42; }\n";
        let temp_file = NamedTempFile::new()?;
        fs::write(&temp_file, content).await?;

        let module = parser.parse_module(temp_file.path()).await?;
        
        assert_eq!(module.name, "Test::Mock");
        assert_eq!(module.subroutines.len(), 1);
        assert_eq!(module.subroutines[0].name, "test_sub");
        assert!(module.dependencies.contains(&"strict".to_string()));

        Ok(())
    }
} 