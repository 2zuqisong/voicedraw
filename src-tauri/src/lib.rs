mod engine;
mod commands;
mod preprocessor;
mod llm;
mod error;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志（可通过 RUST_LOG 环境变量控制级别）
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("voice-draw 启动中...");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::process_command,
            commands::quick_action,
            commands::confirm_plan,
            commands::cancel_plan,
            commands::modify_plan,
            commands::apply_style_transfer,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
