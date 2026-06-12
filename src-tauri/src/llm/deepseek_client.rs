use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionTool,
        CreateChatCompletionRequest,
    },
    Client,
};
/// DeepSeek API 客户端（兼容 OpenAI API 协议）
pub struct DeepSeekClient {
    client: Client<OpenAIConfig>,
    model: String,
}

impl DeepSeekClient {
    /// 从 API Key 和可选的 Base URL 创建客户端
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        let config = OpenAIConfig::new()
            .with_api_base(
                base_url.unwrap_or_else(|| "https://api.deepseek.com".into()),
            )
            .with_api_key(api_key);
        Self {
            client: Client::with_config(config),
            model: "deepseek-chat".into(),
        }
    }

    /// 发送对话请求（含工具定义），返回 LLM 响应
    pub async fn chat(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
        tools: Vec<ChatCompletionTool>,
        stream: bool,
    ) -> Result<ChatCompletionResponse, String> {
        let request = CreateChatCompletionRequest {
            model: self.model.clone(),
            messages,
            tools: if tools.is_empty() { None } else { Some(tools) },
            tool_choice: None,
            stream: Some(stream),
            max_completion_tokens: Some(4096u32),
            temperature: Some(0.3),
            response_format: None,
            ..Default::default()
        };

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| format!("DeepSeek API 调用失败: {}", e))?;

        // 解析返回内容
        let choice = response
            .choices
            .into_iter()
            .next()
            .ok_or("DeepSeek 返回空响应".to_string())?;

        let message = choice.message;

        Ok(ChatCompletionResponse {
            content: message.content,
            tool_calls: message.tool_calls.map(|tc| {
                tc.into_iter()
                    .map(|t| ToolCall {
                        id: t.id,
                        name: t.function.name,
                        arguments: t.function.arguments,
                    })
                    .collect()
            }),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ChatCompletionResponse {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String, // JSON string
}
