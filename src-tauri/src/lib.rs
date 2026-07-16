use std::sync::Arc;
use tauri::Manager;

pub mod engine;
pub mod core;
pub mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Instantiate the appropriate browser engine
            #[cfg(target_os = "windows")]
            let engine = Arc::new(engine::webview2_engine::WebView2Engine::new())
                as Arc<dyn engine::BrowserEngine>;

            #[cfg(not(target_os = "windows"))]
            let engine = Arc::new(engine::tauri_engine::TauriEngine::new())
                as Arc<dyn engine::BrowserEngine>;

            let main_window = app.get_window("main").ok_or_else(|| {
                tauri::Error::AssetNotFound("Main window not found".to_string())
            })?;

            // Initialize the engine synchronously during setup
            let app_handle = app.handle().clone();
            let engine_clone = engine.clone();
            tauri::async_runtime::block_on(async move {
                engine_clone.initialize(&app_handle, &main_window).await
                    .expect("Failed to initialize browser engine");
            });

            // Register states with Tauri manager
            app.manage(core::AppState::new());
            app.manage(commands::EngineState { engine });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::new_tab,
            commands::close_tab,
            commands::switch_tab,
            commands::navigate_tab,
            commands::go_back,
            commands::go_forward,
            commands::reload_tab,
            commands::resize_active_tab,
            commands::add_bookmark,
            commands::remove_bookmark,
            commands::clear_history,
            commands::trigger_mock_download
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
