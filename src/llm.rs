use base64::{Engine, prelude::BASE64_STANDARD};
use rig::{
    agent::Agent,
    completion::{Prompt, message::Image},
    message::{ContentFormat, ImageMediaType},
    providers::openai::{self, CompletionModel},
};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::file_op::FileOp;

pub struct LLM {
    // pub client: AgentBuilder<CompletionModel>,
    pub config: LLMConfig,
    pub algorithms: Vec<AlgorithmConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LLMConfig {
    api_key: String,
    base_url: String,
    model: String,
    temperature: f64,
    system_prompt: String,
    result_prompt:String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlgorithmConfig {
    id: String,
    name: String,
    prompt: String,
    activate: bool,
}

impl LLM {
    pub fn new(fop: &FileOp) -> Self {
        // Read config file
        let config: LLMConfig = serde_json::from_str(fop.read_config().unwrap().as_str()).unwrap();
        // Read algorithm config file
        let algorithms: Vec<AlgorithmConfig> =
            serde_json::from_str(fop.read_algorithm_config().unwrap().as_str()).unwrap();

        Self { config, algorithms }
    }

    // 检查算法是否定义
    pub fn check_algorithm(&self, algorithm_name: &str) -> bool {
        self.algorithms
            .iter()
            .any(|a| a.name == algorithm_name && a.activate)
    }

    // 实例化agent
    pub fn build_agent(&self, algorithm_name: &str) -> Result<Agent<CompletionModel>, anyhow::Error> {
        let algorithm = self
            .algorithms
            .iter()
            .find(|a| a.name == algorithm_name)
            .ok_or(anyhow::anyhow!("Algorithm not found"))?;

        let client = openai::Client::from_url(&self.config.api_key, &self.config.base_url)
            // let client = ollama::Client::from_url(&self.config.base_url)
            .agent(&self.config.model)
            .temperature(self.config.temperature)
            .preamble(&self.config.system_prompt)
            .context(&algorithm.prompt)
            .context(&self.config.result_prompt)
            .build();
        return Ok(client);
    }

    pub async fn chat(
        &self,
        image_path: &str,
        agent: &Agent<CompletionModel>,
    ) -> Result<String, anyhow::Error> {
        // Read image and convert to base64
        let image_bytes = fs::read(image_path).await?;
        let image_base64 = BASE64_STANDARD.encode(image_bytes);

        // Compose `Image` for prompt
        let image = Image {
            data: format!("data:image/jpeg;base64,{}", image_base64),
            media_type: Some(ImageMediaType::JPEG),
            format: Some(ContentFormat::Base64),
            ..Default::default()
        };
        // Prompt the agent and print the response
        let response = agent.prompt(image).await?;
        Ok(response)
    }
}

// 测试模块
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm() {
        let fop = FileOp::new(
            "config/config.json".to_string(),
            "config/algorithm.json".to_string(),
            "images".to_string(),
        );
        let llm = LLM::new(&fop);
        println!("{:#?}", llm.algorithms);
    }

    #[tokio::test]
    async fn test_openai_image(){
        let client = openai::Client::from_url("", "http://10.40.83.188:8000/v1")
            // let client = ollama::Client::from_url(&self.config.base_url)
            .agent("InternVL3-9B")
            .temperature(0.5)
            .preamble("你是一个图像处理助手，你的任务是根据用户提供的图片进行推理和处理。")
            .context("请描述图片内容")
            .build();

        // Read image and convert to base64
        let image_bytes = fs::read("src/images/000000.jpg").await.unwrap();
        let image_base64 = BASE64_STANDARD.encode(image_bytes);

        let image_str = format!("data:image/jpeg;base64,{}", image_base64);

        let image = Image {
            data: image_str,
            media_type: Some(ImageMediaType::JPEG),
            format: Some(ContentFormat::Base64),
            ..Default::default()
        };


        let resp = client.prompt(image).await.unwrap();
        println!("Response: {:#?}", resp);
    }
}
