use serde::{Deserialize, Serialize};

const DASHSCOPE_ENDPOINT: &str =
    "https://dashscope.aliyuncs.com/api/v1/services/aigc/image2image/image-synthesis";
const DASHSCOPE_TASK_BASE: &str = "https://dashscope.aliyuncs.com/api/v1/tasks";
const MAX_POLL_RETRIES: u32 = 40; // 40 × 1.5s = 60s
const POLL_INTERVAL_MS: u64 = 1500;

// ── Request types ──────────────────────────────────────────────

#[derive(Serialize)]
struct StyleRequest {
    model: String,
    input: StyleInput,
    parameters: StyleParams,
}

#[derive(Serialize)]
struct StyleInput {
    function: String,
    prompt: String,
    /// Base64 data URL, e.g. "data:image/png;base64,iVBOR..."
    base_image: String,
}

#[derive(Serialize)]
struct StyleParams {
    n: u32,
}

// ── Response types ─────────────────────────────────────────────

#[derive(Deserialize)]
struct TaskSubmitResponse {
    output: TaskSubmitOutput,
}

#[derive(Deserialize)]
struct TaskSubmitOutput {
    task_id: String,
    #[allow(dead_code)]
    task_status: String,
}

#[derive(Deserialize)]
struct TaskResultResponse {
    output: TaskResultOutput,
}

#[derive(Deserialize)]
struct TaskResultOutput {
    task_status: String,
    results: Option<Vec<TaskImage>>,
    message: Option<String>,
}

#[derive(Deserialize)]
struct TaskImage {
    url: String,
}

// ── Public API ─────────────────────────────────────────────────

/// 调用 DashScope 通义万相 API 进行图像风格转换。
///
/// * `api_key` - DashScope API Key（北京地域）
/// * `image_base64` - 输入图像的 base64 data URL
/// * `prompt` - 风格提示词（中文），如 "梵高星空油画风格，漩涡笔触"
///
/// 返回风格化后的图像 base64 data URL。
pub async fn apply_style_transfer(
    api_key: &str,
    image_base64: &str,
    prompt: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    // 1. 提交异步任务
    let request = StyleRequest {
        model: "wanx2.1-imageedit".into(),
        input: StyleInput {
            function: "stylization_all".into(),
            prompt: prompt.into(),
            base_image: image_base64.into(),
        },
        parameters: StyleParams { n: 1 },
    };

    let submit_resp: TaskSubmitResponse = client
        .post(DASHSCOPE_ENDPOINT)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("X-DashScope-Async", "enable")
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("DashScope API 请求失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("解析提交响应失败: {}", e))?;

    let task_id = submit_resp.output.task_id;
    log::info!("风格转换任务已提交: {}", task_id);

    // 2. 轮询等待结果（最多 60 秒）
    for attempt in 0..MAX_POLL_RETRIES {
        tokio::time::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS)).await;

        let task_url = format!("{}/{}", DASHSCOPE_TASK_BASE, task_id);
        let result_resp: TaskResultResponse = client
            .get(&task_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
            .map_err(|e| format!("查询任务状态失败: {}", e))?
            .json()
            .await
            .map_err(|e| format!("解析任务结果失败: {}", e))?;

        match result_resp.output.task_status.as_str() {
            "SUCCEEDED" => {
                let images = result_resp
                    .output
                    .results
                    .ok_or_else(|| "任务完成但无结果图片".to_string())?;
                let image_url = &images
                    .first()
                    .ok_or_else(|| "结果图片列表为空".to_string())?
                    .url;

                // 3. 下载结果图片并转为 base64
                let image_bytes = client
                    .get(image_url)
                    .send()
                    .await
                    .map_err(|e| format!("下载结果图片失败: {}", e))?
                    .bytes()
                    .await
                    .map_err(|e| format!("读取图片数据失败: {}", e))?;

                use base64::Engine;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&image_bytes);
                let mime = if image_url.ends_with(".png") {
                    "image/png"
                } else {
                    "image/jpeg"
                };
                let data_url = format!("data:{};base64,{}", mime, b64);

                log::info!(
                    "风格转换完成 (尝试 {}/{})",
                    attempt + 1,
                    MAX_POLL_RETRIES
                );
                return Ok(data_url);
            }
            "FAILED" => {
                let msg = result_resp
                    .output
                    .message
                    .unwrap_or_else(|| "未知错误".into());
                return Err(format!("风格转换任务失败: {}", msg));
            }
            status => {
                log::debug!(
                    "任务状态: {} (尝试 {}/{})",
                    status,
                    attempt + 1,
                    MAX_POLL_RETRIES
                );
            }
        }
    }

    Err("风格转换超时（60秒），请重试".into())
}
