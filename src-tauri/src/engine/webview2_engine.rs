#![cfg(target_os = "windows")]

use async_trait::async_trait;
use tauri::{AppHandle, Rect, Window};
use super::{BrowserEngine, tauri_engine::TauriEngine};

pub struct WebView2Engine {
    inner: TauriEngine,
}

impl WebView2Engine {
    pub fn new() -> Self {
        Self {
            inner: TauriEngine::new(),
        }
    }

    #[allow(dead_code)]
    pub fn get_core_webview2(&self, _tab_id: &str) -> Result<*mut std::ffi::c_void, String> {
        Err("Not implemented yet".to_string())
    }
}

#[async_trait]
impl BrowserEngine for WebView2Engine {
    async fn initialize(&self, app_handle: &tauri::AppHandle, main_window: &tauri::Window) -> Result<(), String> {
        self.inner.initialize(app_handle, main_window).await
    }

    async fn create_tab(&self, tab_id: &str, url: &str, rect: Rect) -> Result<(), String> {
        self.inner.create_tab(tab_id, url, rect).await
    }

    async fn close_tab(&self, tab_id: &str) -> Result<(), String> {
        self.inner.close_tab(tab_id).await
    }

    async fn navigate(&self, tab_id: &str, url: &str) -> Result<(), String> {
        self.inner.navigate(tab_id, url).await
    }

    async fn go_back(&self, tab_id: &str) -> Result<(), String> {
        self.inner.go_back(tab_id).await
    }

    async fn go_forward(&self, tab_id: &str) -> Result<(), String> {
        self.inner.go_forward(tab_id).await
    }

    async fn reload(&self, tab_id: &str) -> Result<(), String> {
        self.inner.reload(tab_id).await
    }

    async fn resize_tab(&self, tab_id: &str, rect: Rect) -> Result<(), String> {
        self.inner.resize_tab(tab_id, rect).await
    }

    async fn set_tab_visibility(&self, tab_id: &str, visible: bool) -> Result<(), String> {
        self.inner.set_tab_visibility(tab_id, visible).await
    }
}
