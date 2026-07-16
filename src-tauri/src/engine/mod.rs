use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum EngineEvent {
    TitleChanged { tab_id: String, title: String },
    UrlChanged { tab_id: String, url: String },
    LoadingStateChanged { tab_id: String, is_loading: bool },
    ProgressChanged { tab_id: String, progress: f64 },
    NavigationStateChanged { tab_id: String, can_go_back: bool, can_go_forward: bool },
    NewWindowRequested { url: String },
}

#[async_trait]
pub trait BrowserEngine: Send + Sync + 'static {
    /// Initialize the engine within the context of a Tauri window.
    async fn initialize(&self, app_handle: &tauri::AppHandle, main_window: &tauri::Window) -> Result<(), String>;

    /// Create a new tab (webview) and assign it an ID.
    async fn create_tab(&self, tab_id: &str, url: &str, rect: tauri::Rect) -> Result<(), String>;

    /// Close an existing tab and release its webview resources.
    async fn close_tab(&self, tab_id: &str) -> Result<(), String>;

    /// Navigate a specific tab to a URL.
    async fn navigate(&self, tab_id: &str, url: &str) -> Result<(), String>;

    /// Navigate back in the tab's history.
    async fn go_back(&self, tab_id: &str) -> Result<(), String>;

    /// Navigate forward in the tab's history.
    async fn go_forward(&self, tab_id: &str) -> Result<(), String>;

    /// Reload the active tab.
    async fn reload(&self, tab_id: &str) -> Result<(), String>;

    /// Resize or position a specific tab's webview.
    async fn resize_tab(&self, tab_id: &str, rect: tauri::Rect) -> Result<(), String>;

    /// Show or hide a tab's webview (for switching tabs).
    async fn set_tab_visibility(&self, tab_id: &str, visible: bool) -> Result<(), String>;
}

// Re-export the engines
pub mod tauri_engine;

#[cfg(target_os = "windows")]
pub mod webview2_engine;
