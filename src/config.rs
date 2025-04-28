use std::env;
use std::mem::ManuallyDrop;
use rig::completion::CompletionModel;
use rig::providers::{/*azure,*/ azure, groq};
use rig::agent::{AgentBuilder};


pub struct Config {
    pub provider_client: groq::Client,
}

pub trait AgentProvider<M: CompletionModel> {
    fn get_agent() -> AgentBuilder<M>;
}

struct GroqConfig {}
struct AzureConfig {}

impl AgentProvider<groq::CompletionModel> for GroqConfig {
    fn get_agent() -> AgentBuilder<groq::CompletionModel> {
        groq::Client::from_env().agent(groq::LLAMA_3_2_90B_VISION_PREVIEW)
    }
}
    
impl AgentProvider<azure::CompletionModel> for AzureConfig {
    fn get_agent() -> AgentBuilder<azure::CompletionModel> {
        azure::Client::from_env().agent(azure::GPT_4O)
    }
}

impl Config {
    pub fn from_env() -> Self {
        match env::var("GROQ_API_KEY") {
            Ok(_) =>  Self {
                provider_client: groq::Client::from_env(),
            },
            //Err(_) => match env::var("AZURE_API_KEY") {
            //    Ok(_) => Self {
            //        ai_provider: AIProvider::Azure(azure::Client::from_env().agent(azure::GPT_4O)),
            //    },
            Err(_) => panic!("No AI provider found"),
        }
    }
    pub fn get_agent(&self) -> AgentBuilder<groq::CompletionModel> {
        self.provider_client.agent(groq::LLAMA_3_2_90B_VISION_PREVIEW)
    }
} 