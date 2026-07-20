//! Windows toast notifications with click activation.
//!
//! Uses `tauri-winrt-notification` (rather than the notify-rust fire-and-forget
//! path) so toast activation can deliver `notificationClicked` to the JS side
//! with the notification id and `extra` payload — the same contract macOS uses.

use std::collections::HashMap;
use std::path::{Path, MAIN_SEPARATOR as SEP};
use std::sync::{Mutex, OnceLock};

use serde_json::{json, Value};
use tauri::{AppHandle, Runtime};
use tauri_winrt_notification::{Duration, IconCrop, Sound, Toast};

use crate::listeners;

struct ClickState {
    listener_active: bool,
    pending: Option<Value>,
    /// Extra payloads keyed by notification id for activation lookup.
    extras: HashMap<i32, HashMap<String, Value>>,
}

static CLICK_STATE: OnceLock<Mutex<ClickState>> = OnceLock::new();

fn click_state() -> &'static Mutex<ClickState> {
    CLICK_STATE.get_or_init(|| {
        Mutex::new(ClickState {
            listener_active: false,
            pending: None,
            extras: HashMap::new(),
        })
    })
}

pub fn set_click_listener_active(active: bool) {
    let mut state = click_state().lock().expect("click state lock");
    state.listener_active = active;
    if active {
        if let Some(pending) = state.pending.take() {
            drop(state);
            let _ = emit_clicked(pending);
        }
    }
}

fn emit_clicked(payload: Value) -> crate::Result<()> {
    listeners::trigger(
        "notificationClicked",
        serde_json::to_string(&payload).map_err(|e| {
            crate::Error::Io(std::io::Error::other(format!(
                "serialize notification click: {e}"
            )))
        })?,
    )
}

fn deliver_click(id: i32, data: HashMap<String, String>) {
    let payload = json!({
        "id": id,
        "data": data,
    });

    let mut state = click_state().lock().expect("click state lock");
    if state.listener_active {
        drop(state);
        let _ = emit_clicked(payload);
    } else {
        // Deliver when JS registers onNotificationClicked (cold-start parity).
        state.pending = Some(payload);
    }
}

fn string_data_from_extra(extra: &HashMap<String, Value>) -> HashMap<String, String> {
    extra
        .iter()
        .filter_map(|(key, value)| match value {
            Value::String(s) => Some((key.clone(), s.clone())),
            Value::Number(n) => Some((key.clone(), n.to_string())),
            Value::Bool(b) => Some((key.clone(), b.to_string())),
            _ => None,
        })
        .collect()
}

fn is_dev_build() -> bool {
    match tauri::utils::platform::current_exe() {
        Ok(exe) => {
            let curr_dir = exe
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            curr_dir.ends_with(format!("{SEP}target{SEP}debug").as_str())
                || curr_dir.ends_with(format!("{SEP}target{SEP}release").as_str())
        }
        Err(_) => true,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn show<R: Runtime>(
    app: &AppHandle<R>,
    identifier: String,
    id: i32,
    title: Option<String>,
    body: Option<String>,
    icon: Option<String>,
    extra: HashMap<String, Value>,
    silent: bool,
) -> crate::Result<()> {
    {
        let mut state = click_state().lock().expect("click state lock");
        if !extra.is_empty() {
            state.extras.insert(id, extra.clone());
            if state.extras.len() > 200 {
                if let Some(oldest) = state.extras.keys().next().copied() {
                    state.extras.remove(&oldest);
                }
            }
        }
    }

    let app_id = if is_dev_build() {
        Toast::POWERSHELL_APP_ID.to_string()
    } else {
        identifier
    };

    let title = title.unwrap_or_default();
    let body = body.unwrap_or_default();
    let click_data = string_data_from_extra(&extra);
    let app_handle = app.clone();

    let mut toast = Toast::new(&app_id)
        .title(&title)
        .text2(&body)
        .duration(Duration::Short)
        .on_activated(move |_action| {
            // Prefer the extras captured at show time; fall back to the clone
            // closed over here if the map entry was already pruned.
            let data = {
                let mut state = click_state().lock().expect("click state lock");
                state
                    .extras
                    .remove(&id)
                    .map(|extra| string_data_from_extra(&extra))
                    .unwrap_or_else(|| click_data.clone())
            };
            super::focus_main_window(&app_handle);
            deliver_click(id, data);
            Ok(())
        });

    if silent {
        toast = toast.sound(None::<Sound>);
    }

    if let Some(icon_path) = icon {
        let path = Path::new(&icon_path);
        if path.exists() {
            toast = toast.icon(path, IconCrop::Circular, "");
        }
    }

    toast.show().map_err(|e| {
        crate::Error::Io(std::io::Error::other(format!("show windows toast: {e:?}")))
    })?;

    Ok(())
}
