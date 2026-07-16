import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  ArrowLeft,
  ArrowRight,
  RotateCw,
  Search,
  Star,
  Plus,
  X,
  Compass,
  Layout,
  Settings,
  History,
  Download,
  Briefcase,
  Globe,
  Video,
  Sparkles,
  Trash2,
  ExternalLink,
  ChevronRight,
  Play,
  CheckCircle2,
  FileCode2,
  Code2
} from "lucide-react";
import "./App.css";

interface TabState {
  id: string;
  title: string;
  url: string;
  is_loading: boolean;
  can_go_back: boolean;
  can_go_forward: boolean;
}

interface HistoryItem {
  url: string;
  title: string;
  timestamp: number;
}

interface BookmarkItem {
  url: string;
  title: string;
  date_added: number;
}

interface DownloadItem {
  id: string;
  url: string;
  filename: string;
  total_bytes: number;
  downloaded_bytes: number;
  status: string;
}

interface BrowserState {
  tabs: TabState[];
  active_tab_id: string | null;
  active_workspace: string;
  active_profile: string;
  history: HistoryItem[];
  bookmarks: BookmarkItem[];
  downloads: DownloadItem[];
}

interface Message {
  sender: "user" | "ai" | "system";
  text: string;
}

function App() {
  const [browserState, setBrowserState] = useState<BrowserState>({
    tabs: [],
    active_tab_id: null,
    active_workspace: "Default",
    active_profile: "Default",
    history: [],
    bookmarks: [],
    downloads: [],
  });

  const [addressBar, setAddressBar] = useState("");
  const [showAiSidebar, setShowAiSidebar] = useState(false);
  const [aiInput, setAiInput] = useState("");
  const [aiMessages, setAiMessages] = useState<Message[]>([
    { sender: "ai", text: "Hello! I am Mag AI, your browser companion. Ask me to summarize contents, search the web, or help you debug code." }
  ]);
  
  const [time, setTime] = useState(new Date());

  const webviewContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const timer = setInterval(() => setTime(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  useEffect(() => {
    invoke<BrowserState>("get_state")
      .then((state) => {
        setBrowserState(state);
        if (state.tabs.length === 0) {
          createInitialTab();
        }
      })
      .catch(console.error);

    const unlistenPromise = listen<BrowserState>("state-changed", (event) => {
      setBrowserState(event.payload);
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const activeTab = browserState.tabs.find((t) => t.id === browserState.active_tab_id);
  useEffect(() => {
    if (activeTab) {
      setAddressBar(activeTab.url === "mag://start" ? "" : activeTab.url);
    } else {
      setAddressBar("");
    }
  }, [browserState.active_tab_id, activeTab?.url]);

  const createInitialTab = () => {
    const defaultRect = { x: 70, y: 120, width: 800, height: 480 };
    invoke<BrowserState>("new_tab", { url: "mag://start", rect: defaultRect })
      .then(setBrowserState)
      .catch(console.error);
  };

  useEffect(() => {
    if (!webviewContainerRef.current) return;
    
    const isShowingWebview = activeTab && activeTab.url !== "mag://start" && activeTab.url !== "about:blank";

    if (!isShowingWebview) {
      if (browserState.active_tab_id) {
        invoke("resize_active_tab", {
          rect: { x: 0, y: 0, width: 0, height: 0 }
        }).catch(console.error);
      }
      return;
    }

    const resizeWebview = () => {
      if (!webviewContainerRef.current) return;
      const rect = webviewContainerRef.current.getBoundingClientRect();
      invoke("resize_active_tab", {
        rect: {
          x: Math.round(rect.x),
          y: Math.round(rect.y),
          width: Math.round(rect.width),
          height: Math.round(rect.height),
        }
      }).catch(console.error);
    };

    resizeWebview();

    const observer = new ResizeObserver(() => {
      resizeWebview();
    });
    
    observer.observe(webviewContainerRef.current);
    
    return () => observer.disconnect();
  }, [browserState.active_tab_id, activeTab?.url, showAiSidebar]);

  const handleNewTab = (url: string = "mag://start") => {
    if (!webviewContainerRef.current) return;
    const rect = webviewContainerRef.current.getBoundingClientRect();
    invoke<BrowserState>("new_tab", {
      url,
      rect: {
        x: Math.round(rect.x),
        y: Math.round(rect.y),
        width: Math.round(rect.width),
        height: Math.round(rect.height),
      }
    })
      .then(setBrowserState)
      .catch(console.error);
  };

  const handleCloseTab = (e: React.MouseEvent, tabId: string) => {
    e.stopPropagation();
    invoke<BrowserState>("close_tab", { tabId })
      .then(setBrowserState)
      .catch(console.error);
  };

  const handleSwitchTab = (tabId: string) => {
    invoke<BrowserState>("switch_tab", { tabId })
      .then(setBrowserState)
      .catch(console.error);
  };

  const handleNavigate = (e?: React.FormEvent, directUrl?: string) => {
    if (e) e.preventDefault();
    
    const inputUrl = directUrl !== undefined ? directUrl : addressBar;
    if (!browserState.active_tab_id || !inputUrl) return;
    
    let targetUrl = inputUrl.trim();
    if (!/^https?:\/\//i.test(targetUrl) && !/^mag:\/\//i.test(targetUrl)) {
      if (targetUrl.includes(".") && !targetUrl.includes(" ")) {
        targetUrl = "https://" + targetUrl;
      } else {
        targetUrl = `https://www.google.com/search?q=${encodeURIComponent(targetUrl)}`;
      }
    }

    const rect = webviewContainerRef.current?.getBoundingClientRect();
    const viewportRect = rect ? {
      x: Math.round(rect.x),
      y: Math.round(rect.y),
      width: Math.round(rect.width),
      height: Math.round(rect.height),
    } : { x: 70, y: 120, width: 800, height: 480 };

    invoke("navigate_tab", { 
      tabId: browserState.active_tab_id, 
      url: targetUrl,
      rect: viewportRect
    })
      .then(() => {
        setBrowserState((prev) => ({
          ...prev,
          tabs: prev.tabs.map((t) =>
            t.id === prev.active_tab_id ? { ...t, url: targetUrl } : t
          ),
        }));
      })
      .catch(console.error);
  };

  const handleGoBack = () => {
    if (browserState.active_tab_id) {
      invoke("go_back", { tabId: browserState.active_tab_id }).catch(console.error);
    }
  };

  const handleGoForward = () => {
    if (browserState.active_tab_id) {
      invoke("go_forward", { tabId: browserState.active_tab_id }).catch(console.error);
    }
  };

  const handleReload = () => {
    if (browserState.active_tab_id) {
      invoke("reload_tab", { tabId: browserState.active_tab_id }).catch(console.error);
    }
  };

  const handleSendAiMessage = (text?: string) => {
    const msgText = text || aiInput;
    if (!msgText.trim()) return;

    const userMsg: Message = { sender: "user", text: msgText };
    setAiMessages((prev) => [...prev, userMsg]);
    if (!text) setAiInput("");

    setTimeout(() => {
      let aiText = "I can help with web search, summarization, and commands. I'm operating within the MagBrowser environment!";
      
      const lower = msgText.toLowerCase();
      if (lower.includes("summarize") || lower.includes("/summarize")) {
        aiText = `Summarizing current page: "${activeTab?.title || 'mag://start'}". This page contains information about Tauri v2 browser configurations and React frontend wiring.`;
      } else if (lower.includes("search") || lower.includes("/search")) {
        aiText = "Searching for relevant guides on Google. You can type queries in the address bar to perform active web searches.";
      } else if (lower.includes("help") || lower.includes("workspaces")) {
        aiText = "Workspaces allow you to bundle sets of tabs together. Switch workspaces on the left sidebar to isolate research contexts.";
      }

      setAiMessages((prev) => [...prev, { sender: "ai", text: aiText }]);
    }, 800);
  };

  // Bookmark toggler
  const isBookmarked = activeTab && browserState.bookmarks.some((b) => b.url === activeTab.url);
  const handleToggleBookmark = () => {
    if (!activeTab || activeTab.url === "mag://start") return;
    if (isBookmarked) {
      invoke<BrowserState>("remove_bookmark", { url: activeTab.url })
        .then(setBrowserState)
        .catch(console.error);
    } else {
      invoke<BrowserState>("add_bookmark", { url: activeTab.url, title: activeTab.title || activeTab.url })
        .then(setBrowserState)
        .catch(console.error);
    }
  };

  // Clear History
  const handleClearHistory = () => {
    invoke<BrowserState>("clear_history")
      .then(setBrowserState)
      .catch(console.error);
  };

  // Trigger Mock Download
  const handleStartDownload = () => {
    invoke<BrowserState>("trigger_mock_download", {
      url: "https://tauri.app/tauri_installer.msi",
      filename: "tauri_installer.msi"
    })
      .then(setBrowserState)
      .catch(console.error);
  };

  const formattedTime = time.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  const formattedDate = time.toLocaleDateString([], { weekday: 'long', month: 'short', day: 'numeric' });

  // Check if there are active downloads
  const hasActiveDownloads = browserState.downloads.some(d => d.status === 'downloading');

  return (
    <div className="app-container">
      {/* Workspace Left Sidebar */}
      <aside className="workspace-sidebar">
        <div className="workspace-logo">M</div>
        <button 
          className={`workspace-btn ${browserState.active_workspace === "Default" ? "active" : ""}`}
          title="Personal Workspace"
          onClick={() => setBrowserState(prev => ({ ...prev, active_workspace: "Default" }))}
        >
          <Layout className="lucide-icon" />
        </button>
        <button 
          className={`workspace-btn ${browserState.active_workspace === "Work" ? "active" : ""}`}
          title="Work Workspace"
          onClick={() => setBrowserState(prev => ({ ...prev, active_workspace: "Work" }))}
        >
          <Briefcase className="lucide-icon" />
        </button>
        <button 
          className={`workspace-btn ${browserState.active_workspace === "Research" ? "active" : ""}`}
          title="Research Workspace"
          onClick={() => setBrowserState(prev => ({ ...prev, active_workspace: "Research" }))}
        >
          <Compass className="lucide-icon" />
        </button>
        <div className="workspace-divider"></div>
        <button className="workspace-btn" title="Settings">
          <Settings className="lucide-icon" />
        </button>
      </aside>

      {/* Central Area */}
      <main className="browser-main">
        {/* Navigation & Address Bar Chrome */}
        <header className="chrome-header">
          {/* Tab Strip */}
          <div className="tab-bar">
            {browserState.tabs.map((tab) => (
              <div 
                key={tab.id}
                className={`browser-tab ${tab.id === browserState.active_tab_id ? "active" : ""}`}
                onClick={() => handleSwitchTab(tab.id)}
              >
                {tab.is_loading && <div className="tab-loading-spinner" />}
                <span className="tab-title">
                  {tab.url === "mag://start" ? "Dashboard" : (tab.title || tab.url)}
                </span>
                <span 
                  className="tab-close-btn"
                  onClick={(e) => handleCloseTab(e, tab.id)}
                >
                  <X className="lucide-icon icon-close" style={{ width: '12px', height: '12px' }} />
                </span>
              </div>
            ))}
            <button className="add-tab-btn" onClick={() => handleNewTab()}>
              <Plus className="lucide-icon icon-add" style={{ width: '15px', height: '15px' }} />
            </button>
          </div>

          {/* Control Row */}
          <div className="control-row">
            <div className="nav-controls">
              <button 
                className="nav-btn" 
                onClick={handleGoBack}
                disabled={activeTab && activeTab.url === "mag://start"}
                title="Go Back"
              >
                <ArrowLeft className="lucide-icon icon-back" />
              </button>
              <button 
                className="nav-btn" 
                onClick={handleGoForward}
                disabled={activeTab && activeTab.url === "mag://start"}
                title="Go Forward"
              >
                <ArrowRight className="lucide-icon icon-forward" />
              </button>
              <button 
                className="nav-btn" 
                onClick={handleReload}
                disabled={activeTab && activeTab.url === "mag://start"}
                title="Reload"
              >
                <RotateCw className="lucide-icon icon-reload" />
              </button>
            </div>

            {/* Address Input */}
            <form className="address-bar-container" onSubmit={handleNavigate}>
              <Search className="address-icon lucide-icon" style={{ width: '14px', height: '14px', color: 'var(--text-tertiary)' }} />
              <input 
                type="text" 
                className="address-input"
                placeholder="Search or enter web address"
                value={addressBar}
                onChange={(e) => setAddressBar(e.target.value)}
              />
              {activeTab && activeTab.url !== "mag://start" && (
                <button 
                  type="button"
                  style={{
                    position: 'absolute',
                    right: '12px',
                    background: 'transparent',
                    border: 'none',
                    display: 'flex',
                    alignItems: 'center',
                    cursor: 'pointer',
                  }}
                  onClick={handleToggleBookmark}
                  title={isBookmarked ? "Remove Bookmark" : "Add Bookmark"}
                >
                  <Star 
                    className={`lucide-icon ${isBookmarked ? "star-bookmarked" : ""}`} 
                    style={{ 
                      width: '16px', 
                      height: '16px', 
                      fill: isBookmarked ? '#f59e0b' : 'none', 
                      color: isBookmarked ? '#f59e0b' : '#6b7280' 
                    }} 
                  />
                </button>
              )}
            </form>

            {/* Actions Panel */}
            <div className="actions-row">
              <button 
                className={`action-btn ${showAiSidebar ? "active" : ""}`}
                onClick={() => setShowAiSidebar(!showAiSidebar)}
              >
                <Sparkles className="lucide-icon" style={{ width: '14px', height: '14px', marginRight: '5px' }} />
                AI Assistant
              </button>
            </div>
          </div>
        </header>

        {/* Viewport for Webview */}
        <div ref={webviewContainerRef} className="viewport-area">
          {/* If the active tab is start dashboard, we display it as a React portal */}
          {activeTab && activeTab.url === "mag://start" && (
            <div className="start-page-portal" style={{ display: 'grid', gridTemplateColumns: '1fr 300px', gap: '30px', justifyContent: 'stretch', alignItems: 'start' }}>
              
              {/* Left Column (Search, Clock, Quick Links, Downloads) */}
              <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '20px' }}>
                <div className="start-page-clock" style={{ marginTop: '20px' }}>{formattedTime}</div>
                <div className="start-page-date">{formattedDate}</div>
                <h2 className="start-page-greeting">Good morning, Explorer</h2>
                
                <div className="start-page-search-container" style={{ width: '100%' }}>
                  <Search className="start-page-search-icon lucide-icon" style={{ width: '18px', height: '18px', color: 'var(--text-tertiary)' }} />
                  <input 
                    type="text"
                    className="start-page-search-input"
                    placeholder="Search Google or enter URL"
                    value={addressBar}
                    onChange={(e) => setAddressBar(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        handleNavigate();
                      }
                    }}
                  />
                </div>

                {/* Quick Links Grid */}
                <div className="quick-links-grid" style={{ width: '100%', marginBottom: '20px' }}>
                  <div className="quick-link-card" onClick={() => handleNavigate(undefined, "github.com")}>
                    <Code2 className="lucide-icon" style={{ width: '22px', height: '22px', marginBottom: '8px' }} />
                    <span className="quick-link-title">GitHub</span>
                  </div>
                  <div className="quick-link-card" onClick={() => handleNavigate(undefined, "tauri.app")}>
                    <Globe className="lucide-icon" style={{ width: '22px', height: '22px', marginBottom: '8px' }} />
                    <span className="quick-link-title">Tauri</span>
                  </div>
                  <div className="quick-link-card" onClick={() => handleNavigate(undefined, "react.dev")}>
                    <FileCode2 className="lucide-icon" style={{ width: '22px', height: '22px', marginBottom: '8px' }} />
                    <span className="quick-link-title">React</span>
                  </div>
                  <div className="quick-link-card" onClick={() => handleNavigate(undefined, "youtube.com")}>
                    <Video className="lucide-icon" style={{ width: '22px', height: '22px', marginBottom: '8px' }} />
                    <span className="quick-link-title">YouTube</span>
                  </div>
                </div>

                {/* Bookmarks Section */}
                <div style={{ width: '100%', background: 'rgba(255,255,255,0.02)', border: '1px solid var(--border-color)', borderRadius: '16px', padding: '16px' }}>
                  <h4 style={{ fontSize: '14px', fontWeight: '600', marginBottom: '12px', display: 'flex', color: '#fff', alignItems: 'center', gap: '6px' }}>
                    <Star className="lucide-icon" style={{ width: '15px', height: '15px', color: '#f59e0b', fill: '#f59e0b' }} />
                    <span>Bookmarks</span>
                  </h4>
                  {browserState.bookmarks.length === 0 ? (
                    <p style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>No bookmarks saved yet.</p>
                  ) : (
                    <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
                      {browserState.bookmarks.map((bm) => (
                        <div 
                          key={bm.url} 
                          onClick={() => handleNavigate(undefined, bm.url)}
                          style={{
                            padding: '6px 12px',
                            background: 'rgba(255,255,255,0.04)',
                            border: '1px solid var(--glass-border)',
                            borderRadius: '8px',
                            fontSize: '12.5px',
                            color: 'var(--text-secondary)',
                            cursor: 'pointer',
                            transition: 'var(--transition-fast)',
                            display: 'flex',
                            alignItems: 'center',
                            gap: '6px'
                          }}
                          onMouseEnter={(e) => {
                            e.currentTarget.style.background = 'rgba(255,255,255,0.08)';
                            e.currentTarget.style.color = '#fff';
                          }}
                          onMouseLeave={(e) => {
                            e.currentTarget.style.background = 'rgba(255,255,255,0.04)';
                            e.currentTarget.style.color = 'var(--text-secondary)';
                          }}
                        >
                          <Globe style={{ width: '11px', height: '11px', color: 'var(--text-tertiary)' }} />
                          {bm.title}
                        </div>
                      ))}
                    </div>
                  )}
                </div>

                {/* Downloads Section */}
                <div style={{ width: '100%', background: 'rgba(255,255,255,0.02)', border: '1px solid var(--border-color)', borderRadius: '16px', padding: '16px' }}>
                  <h4 style={{ fontSize: '14px', fontWeight: '600', marginBottom: '12px', display: 'flex', justifyContent: 'space-between', color: '#fff', alignItems: 'center' }}>
                    <span style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                      <Download className={`lucide-icon ${hasActiveDownloads ? "animate-download" : ""}`} style={{ width: '15px', height: '15px' }} />
                      Downloads
                    </span>
                    <button 
                      onClick={handleStartDownload}
                      style={{
                        padding: '5px 12px',
                        background: 'var(--accent-color)',
                        border: 'none',
                        borderRadius: '8px',
                        fontSize: '11px',
                        color: '#fff',
                        cursor: 'pointer',
                        fontWeight: '600',
                        display: 'flex',
                        alignItems: 'center',
                        gap: '4px',
                        transition: 'var(--transition-fast)'
                      }}
                      onMouseEnter={(e) => e.currentTarget.style.background = '#2563eb'}
                      onMouseLeave={(e) => e.currentTarget.style.background = 'var(--accent-color)'}
                    >
                      <Play style={{ width: '10px', height: '10px', fill: 'white' }} />
                      Test Download
                    </button>
                  </h4>
                  {browserState.downloads.length === 0 ? (
                    <p style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>No active downloads.</p>
                  ) : (
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                      {browserState.downloads.map((dl) => {
                        const pct = Math.round((dl.downloaded_bytes / dl.total_bytes) * 100);
                        return (
                          <div key={dl.id} style={{ display: 'flex', flexDirection: 'column', gap: '6px', background: 'rgba(0,0,0,0.2)', padding: '10px 14px', borderRadius: '12px', border: '1px solid rgba(255,255,255,0.03)' }}>
                            <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '12px', fontWeight: '500', alignItems: 'center' }}>
                              <span style={{ color: '#fff', display: 'flex', alignItems: 'center', gap: '6px' }}>
                                {dl.status === 'completed' ? (
                                  <CheckCircle2 style={{ width: '13px', height: '13px', color: '#10b981' }} />
                                ) : (
                                  <Download style={{ width: '13px', height: '13px', color: 'var(--accent-color)' }} />
                                )}
                                {dl.filename}
                              </span>
                              <span style={{ color: dl.status === 'completed' ? '#10b981' : 'var(--text-secondary)' }}>
                                {dl.status === 'completed' ? 'Finished' : `${pct}%`}
                              </span>
                            </div>
                            <div style={{ width: '100%', height: '6px', background: 'rgba(255,255,255,0.05)', borderRadius: '3px', overflow: 'hidden' }}>
                              <div style={{ width: `${pct}%`, height: '100%', background: dl.status === 'completed' ? '#10b981' : 'var(--accent-color)', transition: 'width 0.2s' }}></div>
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  )}
                </div>
              </div>

              {/* Right Column (History sidebar) */}
              <div style={{ background: 'rgba(0,0,0,0.15)', borderLeft: '1px solid var(--border-color)', height: '100%', padding: '20px', display: 'flex', flexDirection: 'column', gap: '16px', borderRadius: '16px' }}>
                <h4 style={{ fontSize: '14px', fontWeight: '600', display: 'flex', justifyContent: 'space-between', color: '#fff', alignItems: 'center' }}>
                  <span style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <History className="lucide-icon" style={{ width: '15px', height: '15px', color: 'var(--text-secondary)' }} />
                    History
                  </span>
                  {browserState.history.length > 0 && (
                    <button 
                      onClick={handleClearHistory}
                      style={{
                        background: 'transparent',
                        border: 'none',
                        color: '#ef4444',
                        cursor: 'pointer',
                        fontSize: '11px',
                        fontWeight: '600',
                        display: 'flex',
                        alignItems: 'center',
                        gap: '4px'
                      }}
                    >
                      <Trash2 style={{ width: '12px', height: '12px' }} />
                      Clear
                    </button>
                  )}
                </h4>
                <div style={{ flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '10px', maxHeight: '420px', paddingRight: '4px' }}>
                  {browserState.history.length === 0 ? (
                    <p style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>No history logged yet.</p>
                  ) : (
                    browserState.history.slice().reverse().map((h, idx) => (
                      <div 
                        key={idx} 
                        onClick={() => handleNavigate(undefined, h.url)}
                        style={{
                          display: 'flex',
                          flexDirection: 'column',
                          gap: '2px',
                          cursor: 'pointer',
                          padding: '8px 10px',
                          borderRadius: '10px',
                          background: 'rgba(255,255,255,0.01)',
                          border: '1px solid transparent',
                          transition: 'var(--transition-fast)'
                        }}
                        onMouseEnter={(e) => {
                          e.currentTarget.style.background = 'rgba(255,255,255,0.03)';
                          e.currentTarget.style.borderColor = 'rgba(255,255,255,0.04)';
                        }}
                        onMouseLeave={(e) => {
                          e.currentTarget.style.background = 'rgba(255,255,255,0.01)';
                          e.currentTarget.style.borderColor = 'transparent';
                        }}
                      >
                        <span style={{ fontSize: '12.5px', color: '#fff', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', display: 'flex', alignItems: 'center', gap: '4px' }}>
                          <ExternalLink style={{ width: '10px', height: '10px', color: 'var(--text-tertiary)' }} />
                          {h.title}
                        </span>
                        <span style={{ fontSize: '10px', color: 'var(--text-tertiary)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', paddingLeft: '14px' }}>{h.url}</span>
                      </div>
                    ))
                  )}
                </div>
              </div>

            </div>
          )}
        </div>
      </main>

      {/* AI Sidebar Panel */}
      {showAiSidebar && (
        <aside className="ai-sidebar">
          <div className="ai-sidebar-header">
            <h3 className="ai-sidebar-title">
              <Sparkles className="ai-sidebar-icon lucide-icon" style={{ color: 'var(--accent-color)' }} /> Mag AI
            </h3>
            <button className="ai-close-btn" onClick={() => setShowAiSidebar(false)}>
              <X className="lucide-icon" style={{ width: '16px', height: '16px' }} />
            </button>
          </div>
          
          <div className="ai-messages-container">
            {aiMessages.map((msg, i) => (
              <div key={i} className={`ai-message ${msg.sender}`}>
                {msg.text}
              </div>
            ))}
          </div>

          <div className="ai-quick-actions">
            <button className="ai-quick-btn" onClick={() => handleSendAiMessage("/summarize page")}>Summarize</button>
            <button className="ai-quick-btn" onClick={() => handleSendAiMessage("What are workspaces?")}>Explain Workspaces</button>
          </div>

          <div className="ai-input-container">
            <input 
              type="text" 
              className="ai-input" 
              placeholder="Ask AI anything..."
              value={aiInput}
              onChange={(e) => setAiInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  handleSendAiMessage();
                }
              }}
            />
            <button className="ai-send-btn" onClick={() => handleSendAiMessage()}>
              <ChevronRight className="lucide-icon" style={{ width: '16px', height: '16px' }} />
            </button>
          </div>
        </aside>
      )}
    </div>
  );
}

export default App;
