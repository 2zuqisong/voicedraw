use serde::Serialize;
use thiserror::Error;

/// 应用全局错误类型
#[derive(Error, Debug, Clone, Serialize)]
pub enum AppError {
    #[error("LLM 调用失败: {0}")]
    LLM(String),

    #[error("引擎错误: {0}")]
    Engine(String),

    #[error("预处理错误: {0}")]
    Preprocess(String),

    #[error("画布未初始化")]
    CanvasNotReady,

    #[error("配置错误: {0}")]
    Config(String),

    #[error("序列化错误: {0}")]
    Serialize(String),
}

impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}

/// 为返回 String 错误的地方提供便利构造方法
impl AppError {
    pub fn llm(msg: impl Into<String>) -> Self {
        Self::LLM(msg.into())
    }

    pub fn engine(msg: impl Into<String>) -> Self {
        Self::Engine(msg.into())
    }

    pub fn preprocess(msg: impl Into<String>) -> Self {
        Self::Preprocess(msg.into())
    }
}
