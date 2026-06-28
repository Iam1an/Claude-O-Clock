mod agent_store;
mod db;
mod hook_server;
mod sound;
mod types;

use std::sync::{Arc, Mutex};

use agent_store::AgentStore;
use hook_server::SharedDb;
use tauri::Manager;

#[tauri::command]
fn get_agents(store: tauri::State<agent_store::SharedStore>) -> Vec<types::AgentInfo> {
    store.lock().unwrap_or_else(|e| e.into_inner()).list()
}

#[tauri::command]
fn set_dnd(enabled: bool, store: tauri::State<agent_store::SharedStore>) {
    store.lock().unwrap_or_else(|e| e.into_inner()).set_dnd(enabled);
}

#[tauri::command]
fn get_alerts(db: tauri::State<SharedDb>) -> Vec<types::AlertInfo> {
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    db::load_alerts(&conn).unwrap_or_default()
}

#[tauri::command]
fn clear_alerts(db: tauri::State<SharedDb>) {
    let conn = db.lock().unwrap_or_else(|e| e.into_inner());
    let _ = db::clear_alerts(&conn);
}

#[tauri::command]
fn rename_agent(
    id: String,
    name: String,
    store: tauri::State<agent_store::SharedStore>,
    db: tauri::State<SharedDb>,
    app: tauri::AppHandle,
) {
    use tauri::Emitter;
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return;
    }
    let agents = {
        let mut s = store.lock().unwrap_or_else(|e| e.into_inner());
        s.rename_agent(&id, &trimmed);
        s.list()
    };
    {
        let conn = db.lock().unwrap_or_else(|e| e.into_inner());
        let _ = db::update_agent_name(&conn, &id, &trimmed);
    }
    let _ = app.emit("agent-update", &agents);
}

pub fn run() {
    let store: agent_store::SharedStore = Arc::new(Mutex::new(AgentStore::new()));
    let hook_store = store.clone();

    tauri::Builder::default()
        .manage(store.clone())
        .invoke_handler(tauri::generate_handler![get_agents, set_dnd, get_alerts, clear_alerts, rename_agent])
        .setup(move |app| {
            // Open the database in the app data directory
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("could not resolve app data dir");
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("claudeoclock.db");

            let conn = rusqlite::Connection::open(&db_path)
                .expect("could not open database");
            db::init(&conn).expect("could not initialise database schema");

            // Restore agents from the previous session
            let persisted = db::load_agents(&conn).unwrap_or_default();
            {
                let mut s = store.lock().unwrap_or_else(|e| e.into_inner());
                for agent in persisted {
                    s.load_agent(agent);
                }
            }

            let db: SharedDb = Arc::new(Mutex::new(conn));
            app.manage(db.clone());

            let hook_db = db.clone();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                hook_server::start(hook_store, hook_db, handle).await;
            });

            // Idle timeout: if no hook fires for 30 s while Working/Running,
            // transition the agent to Inactive and persist the change so dead
            // agents don't come back as Working after a COC restart.
            let idle_store = store.clone();
            let idle_db = db.clone();
            let idle_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                use tokio::time::{interval, Duration};
                use tauri::Emitter;
                let mut ticker = interval(Duration::from_millis(500));
                loop {
                    ticker.tick().await;
                    let updated = {
                        let mut s = idle_store.lock().unwrap_or_else(|e| e.into_inner());
                        s.tick_idle(3)
                    };
                    if let Some(agents) = updated {
                        let _ = idle_handle.emit("agent-update", &agents);
                        // Persist so dead agents don't resurrect on next COC start
                        let conn = idle_db.lock().unwrap_or_else(|e| e.into_inner());
                        for agent in &agents {
                            if agent.state == crate::types::AgentState::Inactive {
                                let _ = db::upsert_agent(&conn, agent);
                            }
                        }
                    }
                }
            });

            // macOS: menu-bar tray app, no Dock icon, dropdown window on click
            #[cfg(target_os = "macos")]
            init_macos(app)?;

            // Linux/Hyprland: always-on-top floating window shown immediately
            #[cfg(not(target_os = "macos"))]
            app.get_webview_window("main")
                .expect("no main window")
                .show()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running claudeoclock")
}

/// macOS: remove Dock icon, add menu-bar tray, toggle window on click,
/// position below the tray icon, auto-hide on focus loss.
#[cfg(target_os = "macos")]
fn init_macos<R: tauri::Runtime>(app: &mut tauri::App<R>) -> tauri::Result<()> {
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    // Belt-and-suspenders: set at runtime in addition to LSUIElement in Info.plist
    let _ = app.handle()
        .set_activation_policy(tauri::ActivationPolicy::Accessory);

    let win = app.get_webview_window("main").expect("no main window");
    let win_for_tray = win.clone();
    let win_for_blur = win.clone();

    // Clone the bundled app icon into an owned Image for the tray
    let src = app.default_window_icon().expect("no default window icon");
    let icon = tauri::image::Image::new_owned(
        src.rgba().to_vec(),
        src.width(),
        src.height(),
    );

    TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(true) // renders correctly in both light and dark menu bars
        .tooltip("Claude'O'Clock")
        .on_tray_icon_event(move |tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if win_for_tray.is_visible().unwrap_or(false) {
                    let _ = win_for_tray.hide();
                } else {
                    position_below_tray(tray, &win_for_tray);
                    let _ = win_for_tray.show();
                    let _ = win_for_tray.set_focus();
                }
            }
        })
        .build(app)?;

    // Auto-dismiss when the user clicks outside the window
    win.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            let _ = win_for_blur.hide();
        }
    });

    Ok(())
}

/// Position the window directly below the menu-bar tray icon,
/// horizontally centred on it and clamped to the screen width.
#[cfg(target_os = "macos")]
fn position_below_tray<R: tauri::Runtime>(
    tray: &tauri::tray::TrayIcon<R>,
    window: &tauri::WebviewWindow<R>,
) {
    let Ok(Some(rect)) = tray.rect() else { return };
    let Ok(win_size) = window.outer_size() else { return };

    // Position/Size are enums in Tauri v2; extract physical pixels from either variant
    let (tray_x, tray_y) = match rect.position {
        tauri::Position::Physical(p) => (p.x, p.y),
        tauri::Position::Logical(p) => (p.x as i32, p.y as i32),
    };
    let (tray_w, tray_h) = match rect.size {
        tauri::Size::Physical(s) => (s.width as i32, s.height as i32),
        tauri::Size::Logical(s) => (s.width as i32, s.height as i32),
    };

    let mut x = tray_x + tray_w / 2 - win_size.width as i32 / 2;
    let y = tray_y + tray_h + 4;

    if let Ok(Some(monitor)) = window.current_monitor() {
        let screen_w = monitor.size().width as i32;
        x = x.clamp(0, (screen_w - win_size.width as i32).max(0));
    }

    let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
}
