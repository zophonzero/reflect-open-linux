//! Reconcile external edits after logind reports resume from sleep.

use tauri::{Emitter, Manager};

fn observe(app: &tauri::AppHandle) -> zbus::Result<()> {
    let connection = zbus::blocking::Connection::system()?;
    let manager = zbus::blocking::Proxy::new(
        &connection,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )?;
    for message in manager.receive_signal("PrepareForSleep")? {
        let (sleeping,): (bool,) = message.body().deserialize()?;
        if !sleeping {
            let graph = app.state::<crate::fs::GraphState>();
            if let Ok(root) = crate::fs::current_root(&graph) {
                crate::fs::invalidate_file_catalog(&graph, &root);
            }
            let _ = app.emit(crate::watcher::RECONCILE_EVENT, ());
        }
    }
    Ok(())
}

/// Observe system sleep without blocking startup; logind is optional.
pub(crate) fn install(app: &tauri::AppHandle) {
    let app = app.clone();
    if let Err(error) = std::thread::Builder::new()
        .name("reflect-wake".into())
        .spawn(move || {
            if let Err(error) = observe(&app) {
                tracing::warn!(%error, "sleep observer unavailable; file watching remains active");
            }
        })
    {
        tracing::warn!(%error, "could not start sleep observer");
    }
}
