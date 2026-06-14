use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionTool,
        CreateChatCompletionRequest,
    },
    Client,
};
/// OpenAI 兼容 API 客户端（支持 DeepSeek / OpenAI / Qwen / Zhipu / Moonshot / xAI 等）
pub struct DeepSeekClient {
    client: Client<OpenAIConfig>,
    model: String,
}

impl DeepSeekClient {
    /// 从 API Key、Base URL、Model 创建客户端
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        let config = OpenAIConfig::new()
            .with_api_base(base_url)
            .with_api_key(api_key);
        Self {
            client: Client::with_config(config),
            model,
        }
    }

    /// 发送对话请求（含工具定义），带自动重试
    pub async fn chat(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
        tools: Vec<ChatCompletionTool>,
        stream: bool,
    ) -> Result<ChatCompletionResponse, String> {
        self.chat_with_retry(messages, tools, stream, 3).await
    }

    async fn chat_with_retry(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
        tools: Vec<ChatCompletionTool>,
        stream: bool,
        retries: u32,
    ) -> Result<ChatCompletionResponse, String> {
        let msg_count = messages.len();
        let tool_count = tools.len();
        let mut last_err = String::new();

        for attempt in 0..=retries {
            if attempt > 0 {
                let delay_ms = 500 * (1 << (attempt - 1)); // 500ms, 1s, 2s
                log::warn!(
                    "LLM API 重试 {}/{} ({}ms 后)...",
                    attempt,
                    retries,
                    delay_ms
                );
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            }

            log::info!(
                "LLM API 请求: {} 条消息, {} 个工具, model={}",
                msg_count,
                tool_count,
                self.model
            );

            let request = CreateChatCompletionRequest {
                model: self.model.clone(),
                messages: messages.clone(),
                tools: if tools.is_empty() { None } else { Some(tools.clone()) },
                tool_choice: None,
                stream: Some(stream),
                max_completion_tokens: Some(4096u32),
                temperature: Some(0.3),
                response_format: None,
                ..Default::default()
            };

            match self.client.chat().create(request).await {
                Ok(response) => {
                    let choice = response
                        .choices
                        .into_iter()
                        .next()
                        .ok_or_else(|| {
                            log::error!("LLM 返回空响应");
                            "LLM 返回空响应".to_string()
                        })?;

                    let message = choice.message;
                    let has_tool_calls = message.tool_calls.is_some();
                    let has_content = message.content.is_some();

                    log::info!(
                        "LLM 响应: content={}, tool_calls={}",
                        has_content,
                        has_tool_calls
                    );

                    return Ok(ChatCompletionResponse {
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
                    });
                }
                Err(e) => {
                    let err_msg = format!("LLM API 调用失败: {}", e);
                    log::error!("{} (attempt {}/{})", err_msg, attempt + 1, retries + 1);
                    last_err = err_msg;
                }
            }
        }

        Err(last_err)
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
