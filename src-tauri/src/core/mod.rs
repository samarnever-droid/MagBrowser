use std::sync::Mutex;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabState {
    pub id: String,
    pub title: String,
    pub url: String,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub url: String,
    pub title: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkItem {
    pub url: String,
    pub title: String,
    pub date_added: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItem {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub status: String, // "downloading", "completed", "failed"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserState {
    pub tabs: Vec<TabState>,
    pub active_tab_id: Option<String>,
    pub active_workspace: String,
    pub active_profile: String,
    pub history: Vec<HistoryItem>,
    pub bookmarks: Vec<BookmarkItem>,
    pub downloads: Vec<DownloadItem>,
}

impl BrowserState {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_id: None,
            active_workspace: "Default".to_string(),
            active_profile: "Default".to_string(),
            history: Vec::new(),
            bookmarks: vec![
                BookmarkItem {
                    url: "https://tauri.app".to_string(),
                    title: "Tauri Framework".to_string(),
                    date_added: 1782000000,
                },
                BookmarkItem {
                    url: "https://react.dev".to_string(),
                    title: "React Documentation".to_string(),
                    date_added: 1782000000,
                }
            ],
            downloads: Vec::new(),
        }
    }
}

pub struct AppState {
    pub state: Mutex<BrowserState>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(BrowserState::new()),
        }
    }
}
