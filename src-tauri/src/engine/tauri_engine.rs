use async_trait::async_trait;
use tauri::{AppHandle, Manager, WebviewBuilder, WebviewUrl, LogicalPosition, Position};
use std::sync::Mutex;

use super::BrowserEngine;

pub struct TauriEngine {
    app_handle: Mutex<Option<AppHandle>>,
    main_window_label: Mutex<Option<String>>,
}

impl TauriEngine {
    pub fn new() -> Self {
        Self {
            app_handle: Mutex::new(None),
            main_window_label: Mutex::new(None),
        }
    }

    fn get_window(&self) -> Result<tauri::Window, String> {
        let app_guard = self.app_handle.lock().map_err(|e| e.to_string())?;
        let app = app_guard.as_ref().ok_or("Engine not initialized")?;
        let label_guard = self.main_window_label.lock().map_err(|e| e.to_string())?;
        let label = label_guard.as_ref().ok_or("Engine not initialized")?;
        app.get_window(label).ok_or_else(|| format!("Window '{}' not found", label))
    }
}

#[async_trait]
impl BrowserEngine for TauriEngine {
    async fn initialize(&self, app_handle: &tauri::AppHandle, main_window: &tauri::Window) -> Result<(), String> {
        let mut app_guard = self.app_handle.lock().map_err(|e| e.to_string())?;
        *app_guard = Some(app_handle.clone());
        let mut label_guard = self.main_window_label.lock().map_err(|e| e.to_string())?;
        *label_guard = Some(main_window.label().to_string());
        Ok(())
    }

    async fn create_tab(&self, tab_id: &str, url: &str, rect: tauri::Rect) -> Result<(), String> {
        let window = self.get_window()?;
        
        let webview_url = if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("file://") || url.starts_with("about:") {
            WebviewUrl::External(url.parse().map_err(|e| format!("Invalid URL: {}", e))?)
        } else if url.is_empty() {
            WebviewUrl::External("about:blank".parse().unwrap())
        } else {
            WebviewUrl::App(url.into())
        };

        let builder = WebviewBuilder::new(tab_id, webview_url);
        
        window.add_child(builder, rect.position, rect.size)
            .map_err(|e| format!("Failed to create tab: {}", e))?;
            
        Ok(())
    }

    async fn close_tab(&self, tab_id: &str) -> Result<(), String> {
        let app_guard = self.app_handle.lock().map_err(|e| e.to_string())?;
        let app = app_guard.as_ref().ok_or("Engine not initialized")?;
        
        if let Some(webview) = app.get_webview(tab_id) {
            webview.close().map_err(|e| format!("Failed to close tab: {}", e))?;
        }
        
        Ok(())
    }

    async fn navigate(&self, tab_id: &str, url: &str) -> Result<(), String> {
        let app_guard = self.app_handle.lock().map_err(|e| e.to_string())?;
        let app = app_guard.as_ref().ok_or("Engine not initialized")?;
        
        let webview = app.get_webview(tab_id).ok_or_else(|| format!("Tab '{}' not found", tab_id))?;
        let parsed_url = url.parse().map_err(|e| format!("Invalid URL: {}", e))?;
        webview.navigate(parsed_url).map_err(|e| format!("Failed to navigate: {}", e))?;
        
        Ok(())
    }

    async fn go_back(&self, tab_id: &str) -> Result<(), String> {
        let app_guard = self.app_handle.lock().map_err(|e| e.to_string())?;
        let app = app_guard.as_ref().ok_or("Engine not initialized")?;
        
        let webview = app.get_webview(tab_id).ok_or_else(|| format!("Tab '{}' not found", tab_id))?;
        webview.eval("window.history.back()").map_err(|e| format!("Failed to execute back: {}", e))?;
        
        Ok(())
    }

    async fn go_forward(&self, tab_id: &str) -> Result<(), String> {
        let app_guard = self.app_handle.lock().map_err(|e| e.to_string())?;
        let app = app_guard.as_ref().ok_or("Engine not initialized")?;
        
        let webview = app.get_webview(tab_id).ok_or_else(|| format!("Tab '{}' not found", tab_id))?;
        webview.eval("window.history.forward()").map_err(|e| format!("Failed to execute forward: {}", e))?;
        
        Ok(())
    }

    async fn reload(&self, tab_id: &str) -> Result<(), String> {
        let app_guard = self.app_handle.lock().map_err(|e| e.to_string())?;
        let app = app_guard.as_ref().ok_or("Engine not initialized")?;
        
        let webview = app.get_webview(tab_id).ok_or_else(|| format!("Tab '{}' not found", tab_id))?;
        webview.eval("window.location.reload()").map_err(|e| format!("Failed to execute reload: {}", e))?;
        
        Ok(())
    }

    async fn resize_tab(&self, tab_id: &str, rect: tauri::Rect) -> Result<(), String> {
        let app_guard = self.app_handle.lock().map_err(|e| e.to_string())?;
        let app = app_guard.as_ref().ok_or("Engine not initialized")?;
        
        let webview = app.get_webview(tab_id).ok_or_else(|| format!("Tab '{}' not found", tab_id))?;
        
        webview.set_position(rect.position).map_err(|e| format!("Failed to set tab position: {}", e))?;
        webview.set_size(rect.size).map_err(|e| format!("Failed to set tab size: {}", e))?;
        
        Ok(())
    }

    async fn set_tab_visibility(&self, tab_id: &str, visible: bool) -> Result<(), String> {
        let app_guard = self.app_handle.lock().map_err(|e| e.to_string())?;
        let app = app_guard.as_ref().ok_or("Engine not initialized")?;
        
        let webview = app.get_webview(tab_id).ok_or_else(|| format!("Tab '{}' not found", tab_id))?;
        
        if visible {
            webview.set_focus().map_err(|e| e.to_string())?;
        } else {
            // Move off-screen to hide
            let hidden_position = Position::Logical(LogicalPosition::new(-10000.0, -10000.0));
            webview.set_position(hidden_position).map_err(|e| format!("Failed to hide webview: {}", e))?;
        }
        
        Ok(())
    }
}
