use std::sync::{Arc, Mutex};
use std::time::Duration;

use rusqlite::Connection;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::agent_store::SharedStore;
use crate::sound;
use crate::types::HookPayload;
use crate::db;

pub type SharedDb = Arc<Mutex<Connection>>;

pub const HOOK_PORT: u16 = 22362;

const HTTP_200: &[u8] =
    b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Type: text/plain\r\n\r\nOK";
const HTTP_400: &[u8] = b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";

pub async fn start(store: SharedStore, db_conn: SharedDb, app: AppHandle) {
    let listener = match TcpListener::bind(("127.0.0.1", HOOK_PORT)).await {
        Ok(l) => {
            println!("[claudeoclock] hook server on 127.0.0.1:{HOOK_PORT}");
            l
        }
        Err(e) => {
            eprintln!("[claudeoclock] hook server failed to bind: {e}");
            return;
        }
    };

    loop {
        let Ok((mut stream, _)) = listener.accept().await else {
            continue;
        };
        let store = store.clone();
        let db_conn = db_conn.clone();
        let app = app.clone();

        tokio::spawn(async move {
            let body = tokio::time::timeout(Duration::from_secs(5), read_body(&mut stream))
                .await
                .ok()
                .flatten();

            let parsed = body.and_then(|b| serde_json::from_slice::<HookPayload>(&b).ok());

            match parsed {
                Some(payload) => {
                    let session_id = payload.session_id.clone();

                    let (agents, sound_kind, alert) = {
                        let mut s = store.lock().unwrap_or_else(|e| e.into_inner());
                        let (sound, alert) = s.apply_hook(payload);
                        (s.list(), sound, alert)
                    };

                    // Push updated agent list to frontend
                    let _ = app.emit("agent-update", &agents);

                    // Play sound
                    if let Some(kind) = sound_kind {
                        sound::play(kind);
                    }

                    // Persist the updated agent and any new alert to SQLite
                    {
                        let conn = db_conn.lock().unwrap_or_else(|e| e.into_inner());
                        if let Some(agent) = agents.iter().find(|a| a.id == session_id) {
                            let _ = db::upsert_agent(&conn, agent);
                        }
                        if let Some(ref a) = alert {
                            let _ = db::save_alert(&conn, a);
                        }
                    }

                    // Emit new alert to frontend so it appears in real time
                    if let Some(alert) = alert {
                        let _ = app.emit("alert-new", &alert);
                    }

                    let _ = stream.write_all(HTTP_200).await;
                }
                None => {
                    let _ = stream.write_all(HTTP_400).await;
                }
            }
        });
    }
}

async fn read_body(stream: &mut tokio::net::TcpStream) -> Option<Vec<u8>> {
    let mut buf = Vec::with_capacity(8192);
    let mut tmp = [0u8; 4096];

    loop {
        let n = stream.read(&mut tmp).await.ok()?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);

        if let Some(header_end) = find_header_end(&buf) {
            let cl = parse_content_length(&buf[..header_end]).unwrap_or(0);
            if cl > 65_536 {
                return None; // reject oversized bodies before any arithmetic
            }
            let body_start = header_end + 4;
            if buf.len() >= body_start.saturating_add(cl) {
                return Some(buf[body_start..body_start + cl].to_vec());
            }
        }

        if buf.len() > 65_536 {
            break; // safety limit
        }
    }
    None
}

fn find_header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

fn parse_content_length(headers: &[u8]) -> Option<usize> {
    std::str::from_utf8(headers)
        .ok()?
        .lines()
        .find(|l| l.to_ascii_lowercase().starts_with("content-length:"))?
        .split_once(':')?
        .1
        .trim()
        .parse()
        .ok()
}
