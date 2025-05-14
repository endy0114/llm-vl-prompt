///
/// 对外提供模型检测restful接口
///
/// 1、接收图片文件检测
/// 2、接收图片base64编码字符串检测
///
use axum::{Json, Router, extract::State, routing::post};
use regex::Regex;
use rig::{
    agent::Agent,
    completion::Prompt,
    message::{Image, ImageMediaType},
    providers::openai::{self, CompletionModel},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::info;

use crate::llm::{AlgorithmConfig, LLMConfig};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LLMOperation {
    pub config: LLMConfig,
    pub algorithms: Vec<AlgorithmConfig>,
}

impl LLMOperation {
    pub fn new() -> Self {
        // Read config file
        let config: LLMConfig = serde_json::from_str("config/config.json").unwrap();
        // Read algorithm config file
        let algorithms: Vec<AlgorithmConfig> =
            serde_json::from_str("config/algorithm.json").unwrap();
        // 过滤掉没有开启的算法
        let algorithms = algorithms
            .into_iter()
            .filter(|a| a.activate)
            .collect::<Vec<_>>();

        Self { config, algorithms }
    }

    fn get_agent(self, algorithm_name: &str) -> Result<Agent<CompletionModel>, anyhow::Error> {
        let algorithm = self
            .algorithms
            .iter()
            .find(|a| a.name == algorithm_name)
            .ok_or(anyhow::anyhow!("Algorithm not found"))?;

        info!("算法配置: {:?}", algorithm);

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

    pub async fn infer(
        self,
        algorithm_name: String,
        image_base64: String,
    ) -> Result<Value, anyhow::Error> {
        let agent = self.clone().get_agent(algorithm_name.as_str()).unwrap();
        // Compose `Image` for prompt
        let image = Image {
            data: format!("data:image/jpeg;base64,{}", image_base64),
            media_type: Some(ImageMediaType::JPEG),
            format: Some(rig::message::ContentFormat::Base64),
            ..Default::default()
        };
        let response = agent.prompt(image).await.unwrap();
        Ok(self.clone().parse_json_from_markdown(&response).unwrap())
    }

    fn parse_json_from_markdown(self, markdown: &str) -> Result<Value, anyhow::Error> {
        info!("Parsing JSON from markdown : \n{}", markdown);
        let re = Regex::new(r#"(?s)```json\s*\n(.*?)\n\s*```"#).unwrap();
        let mut json_list = Vec::new();

        for cap in re.captures_iter(markdown) {
            if let Some(m) = cap.get(1) {
                json_list.push(m.as_str().to_string());
            }
        }
        Ok(json!(json_list.concat()))
    }
}

pub struct Server {
    addr: String,
    app: Router,
}

impl Server {
    // 创建一个新的Server实例
    pub fn new(host: &str, port: u16) -> Self {
        // 创建一个新的Router实例，并添加一个路由，该路由处理POST请求，并使用base64_handler函数处理
        let app = Router::new()
            .route("/base64", post(base64_handler))
            .with_state(LLMOperation::new());

        // 将host和port格式化为字符串，作为Server的地址
        let addr = format!("{}:{}", host, port);
        // 打印Server的地址
        info!("Server listening on {}", addr);
        // 返回一个新的Server实例
        Self { addr, app }
    }

    // 运行Server
    pub async fn run(self) {
        // 绑定Server的地址，创建一个TcpListener实例
        let listener = tokio::net::TcpListener::bind(self.addr).await.unwrap();
        // 打印TcpListener的本地地址
        info!("listening on {}", listener.local_addr().unwrap());
        // 使用axum库的serve函数，将TcpListener和Router实例传入，启动Server
        axum::serve(listener, self.app).await.unwrap();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Base64Request {
    image: String,
    algorithm: String,
}

async fn base64_handler(
    State(llm): State<LLMOperation>,
    Json(payload): Json<Base64Request>,
) -> Result<String, String> {
    let response = &llm.infer(payload.algorithm, payload.image).await.unwrap();
    info!("推理结果: {:?}", response);
    Ok(response.to_string())
}
