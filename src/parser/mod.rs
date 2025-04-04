use std::path::Path;
use async_trait::async_trait;
use rig::{
    completion::Prompt,
    providers::groq::Client as GroqClient,
};
use backoff::ExponentialBackoff;
use serde::{Deserialize, Serialize};
use tokio::fs;
use crate::{
    domain::{
        models::{PerlModule, Subroutine},
        traits::ModuleParser,
    },
    error::Error as AIError,
};

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
    client: GroqClient,
}

impl AIModuleParser {
    pub fn new() -> Self {
        let client = GroqClient::new("");
        Self { client }
    }

    pub fn new_with_key(api_key: &str) -> Self {
        let client = GroqClient::new(api_key);
        Self { client }
    }

    pub fn with_client(client: GroqClient) -> Self {
        Self { client }
    }

    async fn analyze_code(&self, content: &str) -> Result<ParseResponse, AIError> {
        let prompt = format!(
            r#"Analyze this Perl module and extract its structure. Return ONLY a raw JSON object (no markdown formatting, no code blocks) containing:
            - package_name: The name of the Perl package/module
            - subroutines: Array of objects, each containing:
                - name: Subroutine name
                - code: The complete subroutine code including its definition
                - line_start: Starting line number
                - line_end: Ending line number
                - dependencies: Array of module/package names this subroutine depends on
            - dependencies: Array of all module/package dependencies

            Module content:
            {}
            "#,
            content
        );

        let backoff = ExponentialBackoff::default();
        let response = backoff::future::retry(backoff, || async {
            let agent = self.client
                .agent("llama-3.3-70b-versatile")
                .preamble("You are a Perl code analyzer. You will analyze Perl code and extract its structure in JSON format.")
                .build();

            let response = agent
                .prompt(prompt.as_str())
                .await
                .map_err(|e| backoff::Error::Permanent(AIError::AIError(e.to_string())))?;

            serde_json::from_str::<ParseResponse>(&response)
                .map_err(|e| {
                    eprintln!("Failed to parse response content: {}", response);
                    backoff::Error::Permanent(AIError::ParseError(format!("Failed to parse AI response: {}", e)))
                })
        }).await?;

        Ok(response)
    }
}

#[async_trait]
impl ModuleParser for AIModuleParser {
    async fn parse_module(&self, path: impl AsRef<Path> + Send) -> Result<PerlModule, AIError> {
        let content = fs::read_to_string(path.as_ref())
            .await
            .map_err(|e| AIError::IOError(e))?;

        let response = self.analyze_code(&content).await?;

        let subroutines = response.subroutines
            .into_iter()
            .map(|s| Subroutine {
                name: s.name,
                code: s.code,
                line_start: s.line_start,
                line_end: s.line_end,
                dependencies: s.dependencies,
            })
            .collect();

        Ok(PerlModule {
            name: response.package_name.unwrap_or_else(|| {
                path.as_ref()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            }),
            path: path.as_ref().to_path_buf(),
            content,
            subroutines,
            dependencies: response.dependencies,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{body_json, header, method, path};
    use serde_json::json;
    use std::io::Write;

    #[tokio::test]
    async fn test_with_mock_ai() -> Result<(), Box<dyn std::error::Error>> {
        // Start a mock server
        let mock_server = MockServer::start().await;

        // Create a mock response
        let mock_response = ParseResponse {
            subroutines: vec![
                ParsedSubroutine {
                    name: "test_sub".to_string(),
                    code: "sub test_sub { }".to_string(),
                    line_start: 1,
                    line_end: 3,
                    dependencies: vec!["Test::More".to_string()],
                },
            ],
            dependencies: vec!["Test::More".to_string()],
            package_name: Some("TestModule".to_string()),
        };

        // Mock the chat completion endpoint
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("Authorization", "Bearer test-key"))
            .and(body_json(json!({
                "model": "llama-3.3-70b-versatile",
                "messages": [
                    {
                        "content": "You are a Perl code analyzer. You will analyze Perl code and extract its structure in JSON format.",
                        "role": "system"
                    },
                    {
                        "content": format!(
                            "Analyze this Perl module and extract its structure. Return ONLY a raw JSON object (no markdown formatting, no code blocks) containing:\n            - package_name: The name of the Perl package/module\n            - subroutines: Array of objects, each containing:\n                - name: Subroutine name\n                - code: The complete subroutine code including its definition\n                - line_start: Starting line number\n                - line_end: Ending line number\n                - dependencies: Array of module/package names this subroutine depends on\n            - dependencies: Array of all module/package dependencies\n\n            Module content:\n            {}\n            ",
                            "package TestModule;\nuse Test::More;\nsub test_sub { }\n1;"
                        ),
                        "role": "user"
                    }
                ],
                "temperature": null
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "test-id",
                "object": "chat.completion",
                "created": 1234567890,
                "model": "llama-3.3-70b-versatile",
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": serde_json::to_string(&mock_response)?
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": 100,
                    "completion_tokens": 50,
                    "total_tokens": 150
                }
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Create a temporary file with test Perl code
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "package TestModule;\nuse Test::More;\nsub test_sub {{ }}\n1;")?;

        // Initialize the parser with the mock server URL
        let client = GroqClient::from_url("test-key", mock_server.uri().as_str());
        let parser = AIModuleParser::with_client(client);

        // Parse the module
        let module = parser.parse_module(temp_file.path()).await?;

        // Verify the results
        assert_eq!(module.name, "TestModule");
        assert_eq!(module.subroutines.len(), 1);
        assert_eq!(module.dependencies, vec!["Test::More"]);

        Ok(())
    }
} 