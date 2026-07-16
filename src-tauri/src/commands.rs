use tauri::{State, AppHandle, Emitter, Manager};
use crate::core::{AppState, TabState, BrowserState, HistoryItem, BookmarkItem, DownloadItem};
use crate::engine::BrowserEngine;
use std::sync::Arc;

pub struct EngineState {
    pub engine: Arc<dyn BrowserEngine>,
}

#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct ViewportRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl ViewportRect {
    pub fn to_tauri_rect(self) -> tauri::Rect {
        tauri::Rect {
            position: tauri::Position::Logical(tauri::LogicalPosition::new(self.x as f64, self.y as f64)),
            size: tauri::Size::Logical(tauri::LogicalSize::new(self.width as f64, self.height as f64)),
        }
    }
}

#[tauri::command]
pub async fn get_state(app_state: State<'_, AppState>) -> Result<BrowserState, String> {
    let state = app_state.state.lock().map_err(|e| e.to_string())?;
    Ok(state.clone())
}

#[tauri::command]
pub async fn new_tab(
    url: String,
    rect: ViewportRect,
    app_state: State<'_, AppState>,
    engine_state: State<'_, EngineState>,
    app_handle: AppHandle,
) -> Result<BrowserState, String> {
    let tab_id = format!(
        "tab_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    // Defer native webview creation for mag://start or about:blank
    let is_web_page = url != "mag://start" && url != "about:blank";
    if is_web_page {
        engine_state.engine.create_tab(&tab_id, &url, rect.to_tauri_rect()).await?;
    }

    let old_active_id = {
        let state = app_state.state.lock().map_err(|e| e.to_string())?;
        state.active_tab_id.clone()
    };

    // Hide old active webview if it existed
    if let Some(ref active_id) = old_active_id {
        let old_had_webview = {
            let state = app_state.state.lock().map_err(|e| e.to_string())?;
            state.tabs.iter()
                .find(|t| t.id == *active_id)
                .map(|t| t.url != "mag://start" && t.url != "about:blank")
                .unwrap_or(false)
        };
        if old_had_webview {
            engine_state.engine.set_tab_visibility(active_id, false).await?;
        }
    }

    let new_state = {
        let mut state = app_state.state.lock().map_err(|e| e.to_string())?;
        let tab = TabState {
            id: tab_id.clone(),
            title: if url == "mag://start" { "Dashboard".to_string() } else { "New Tab".to_string() },
            url: url.clone(),
            is_loading: is_web_page,
            can_go_back: false,
            can_go_forward: false,
        };
        state.tabs.push(tab);
        state.active_tab_id = Some(tab_id);
        
        if is_web_page {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            state.history.push(HistoryItem {
                url: url.clone(),
                title: "New Tab".to_string(),
                timestamp,
            });
        }
        
        state.clone()
    };

    app_handle.emit("state-changed", new_state.clone()).map_err(|e| e.to_string())?;

    Ok(new_state)
}

#[tauri::command]
pub async fn close_tab(
    tab_id: String,
    app_state: State<'_, AppState>,
    engine_state: State<'_, EngineState>,
    app_handle: AppHandle,
) -> Result<BrowserState, String> {
    let (had_webview, was_active, next_active_id) = {
        let state = app_state.state.lock().map_err(|e| e.to_string())?;
        let tab = state.tabs.iter().find(|t| t.id == tab_id);
        let had_webview = tab.map(|t| t.url != "mag://start" && t.url != "about:blank").unwrap_or(false);
        let was_active = state.active_tab_id.as_ref() == Some(&tab_id);
        let next_active_id = if was_active {
            state.tabs.iter()
                .filter(|t| t.id != tab_id)
                .last()
                .map(|t| t.id.clone())
        } else {
            None
        };
        (had_webview, was_active, next_active_id)
    };

    // Close the native webview if it existed
    if had_webview {
        engine_state.engine.close_tab(&tab_id).await?;
    }

    if was_active {
        if let Some(ref next_id) = next_active_id {
            let next_has_webview = {
                let state = app_state.state.lock().map_err(|e| e.to_string())?;
                state.tabs.iter()
                    .find(|t| t.id == *next_id)
                    .map(|t| t.url != "mag://start" && t.url != "about:blank")
                    .unwrap_or(false)
            };
            if next_has_webview {
                engine_state.engine.set_tab_visibility(next_id, true).await?;
            }
        }
    }

    let new_state = {
        let mut state = app_state.state.lock().map_err(|e| e.to_string())?;
        let index = state.tabs.iter().position(|t| t.id == tab_id);
        if let Some(idx) = index {
            state.tabs.remove(idx);
        }
        if was_active {
            state.active_tab_id = next_active_id;
        }
        state.clone()
    };

    app_handle.emit("state-changed", new_state.clone()).map_err(|e| e.to_string())?;
    
    Ok(new_state)
}

#[tauri::command]
pub async fn switch_tab(
    tab_id: String,
    app_state: State<'_, AppState>,
    engine_state: State<'_, EngineState>,
    app_handle: AppHandle,
) -> Result<BrowserState, String> {
    let (should_switch, old_active_id, old_had_webview, new_has_webview) = {
        let state = app_state.state.lock().map_err(|e| e.to_string())?;
        if state.active_tab_id.as_ref() == Some(&tab_id) {
            (false, None, false, false)
        } else {
            let old_id = state.active_tab_id.clone();
            let old_had = old_id.as_ref().map(|id| {
                state.tabs.iter()
                    .find(|t| t.id == *id)
                    .map(|t| t.url != "mag://start" && t.url != "about:blank")
                    .unwrap_or(false)
            }).unwrap_or(false);
            
            let new_has = state.tabs.iter()
                .find(|t| t.id == tab_id)
                .map(|t| t.url != "mag://start" && t.url != "about:blank")
                .unwrap_or(false);

            (true, old_id, old_had, new_has)
        }
    };

    if !should_switch {
        let state = app_state.state.lock().map_err(|e| e.to_string())?;
        return Ok(state.clone());
    }

    // Toggle native webview visibilities based on existence
    if old_had_webview {
        if let Some(ref active_id) = old_active_id {
            engine_state.engine.set_tab_visibility(active_id, false).await?;
        }
    }
    
    if new_has_webview {
        engine_state.engine.set_tab_visibility(&tab_id, true).await?;
    }

    let new_state = {
        let mut state = app_state.state.lock().map_err(|e| e.to_string())?;
        state.active_tab_id = Some(tab_id);
        state.clone()
    };

    app_handle.emit("state-changed", new_state.clone()).map_err(|e| e.to_string())?;
    
    Ok(new_state)
}

#[tauri::command]
pub async fn navigate_tab(
    tab_id: String,
    url: String,
    rect: ViewportRect,
    app_state: State<'_, AppState>,
    engine_state: State<'_, EngineState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let had_webview = {
        let state = app_state.state.lock().map_err(|e| e.to_string())?;
        state.tabs.iter()
            .find(|t| t.id == tab_id)
            .map(|t| t.url != "mag://start" && t.url != "about:blank")
            .unwrap_or(false)
    };

    let is_web_page = url != "mag://start" && url != "about:blank";

    if is_web_page {
        if had_webview {
            engine_state.engine.navigate(&tab_id, &url).await?;
        } else {
            // Lazy create the webview when transitioning to a real page
            engine_state.engine.create_tab(&tab_id, &url, rect.to_tauri_rect()).await?;
            engine_state.engine.set_tab_visibility(&tab_id, true).await?;
        }
    } else if had_webview {
        // De-instantiate webview if returning to the local start dashboard
        engine_state.engine.close_tab(&tab_id).await?;
    }

    let new_state = {
        let mut state = app_state.state.lock().map_err(|e| e.to_string())?;
        
        if let Some(tab) = state.tabs.iter_mut().find(|t| t.id == tab_id) {
            tab.url = url.clone();
            tab.title = if url == "mag://start" { "Dashboard".to_string() } else { url.clone() };
            tab.is_loading = is_web_page;
        }

        if is_web_page {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            state.history.push(HistoryItem {
                url: url.clone(),
                title: url.clone(),
                timestamp,
            });
        }

        state.clone()
    };

    app_handle.emit("state-changed", new_state).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn go_back(tab_id: String, engine_state: State<'_, EngineState>) -> Result<(), String> {
    engine_state.engine.go_back(&tab_id).await
}

#[tauri::command]
pub async fn go_forward(tab_id: String, engine_state: State<'_, EngineState>) -> Result<(), String> {
    engine_state.engine.go_forward(&tab_id).await
}

#[tauri::command]
pub async fn reload_tab(tab_id: String, engine_state: State<'_, EngineState>) -> Result<(), String> {
    engine_state.engine.reload(&tab_id).await
}

#[tauri::command]
pub async fn resize_active_tab(
    rect: ViewportRect,
    app_state: State<'_, AppState>,
    engine_state: State<'_, EngineState>,
) -> Result<(), String> {
    let (active_id, has_webview) = {
        let state = app_state.state.lock().map_err(|e| e.to_string())?;
        let active_id = state.active_tab_id.clone();
        let has_webview = active_id.as_ref().map(|id| {
            state.tabs.iter()
                .find(|t| t.id == *id)
                .map(|t| t.url != "mag://start" && t.url != "about:blank")
                .unwrap_or(false)
        }).unwrap_or(false);
        (active_id, has_webview)
    };

    if has_webview {
        if let Some(ref active_id) = active_id {
            engine_state.engine.resize_tab(active_id, rect.to_tauri_rect()).await?;
        }
    }
    Ok(())
}

// History & Bookmarks Management Commands
#[tauri::command]
pub async fn add_bookmark(
    url: String,
    title: String,
    app_state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<BrowserState, String> {
    let mut state = app_state.state.lock().map_err(|e| e.to_string())?;
    
    if !state.bookmarks.iter().any(|b| b.url == url) {
        state.bookmarks.push(BookmarkItem {
            url,
            title,
            date_added: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });
    }

    app_handle.emit("state-changed", state.clone()).map_err(|e| e.to_string())?;
    Ok(state.clone())
}

#[tauri::command]
pub async fn remove_bookmark(
    url: String,
    app_state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<BrowserState, String> {
    let mut state = app_state.state.lock().map_err(|e| e.to_string())?;
    state.bookmarks.retain(|b| b.url != url);
    app_handle.emit("state-changed", state.clone()).map_err(|e| e.to_string())?;
    Ok(state.clone())
}

#[tauri::command]
pub async fn clear_history(
    app_state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<BrowserState, String> {
    let mut state = app_state.state.lock().map_err(|e| e.to_string())?;
    state.history.clear();
    app_handle.emit("state-changed", state.clone()).map_err(|e| e.to_string())?;
    Ok(state.clone())
}

#[tauri::command]
pub async fn trigger_mock_download(
    url: String,
    filename: String,
    app_state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<BrowserState, String> {
    let download_id = format!("dl_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());

    let mut state = app_state.state.lock().map_err(|e| e.to_string())?;
    
    state.downloads.push(DownloadItem {
        id: download_id.clone(),
        url: url.clone(),
        filename: filename.clone(),
        total_bytes: 1024 * 1024 * 50,
        downloaded_bytes: 0,
        status: "downloading".to_string(),
    });

    app_handle.emit("state-changed", state.clone()).map_err(|e| e.to_string())?;

    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        for i in 1..=5 {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            
            let app_state = app_handle_clone.state::<AppState>();
            let mut state = app_state.state.lock().unwrap();
            if let Some(dl) = state.downloads.iter_mut().find(|d| d.id == download_id) {
                dl.downloaded_bytes = (dl.total_bytes / 5) * i;
                if i == 5 {
                    dl.status = "completed".to_string();
                }
            }
            let _ = app_handle_clone.emit("state-changed", state.clone());
        }
    });

    Ok(state.clone())
}
