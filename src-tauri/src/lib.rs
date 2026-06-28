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

pub fn run() {
    let store: agent_store::SharedStore = Arc::new(Mutex::new(AgentStore::new()));
    let hook_store = store.clone();

    tauri::Builder::default()
        .manage(store.clone())
        .invoke_handler(tauri::generate_handler![get_agents, set_dnd, get_alerts, clear_alerts])
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

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                hook_server::start(hook_store, db, handle).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running claudeoclock")
}
