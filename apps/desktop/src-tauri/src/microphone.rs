//! Linux microphone consent. WebKitGTK denies media access without a handler.

use crate::error::AppResult;

#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "linux")]
use tauri::Manager;

/// Consent lasts for this app session, and covers only microphone recording.
#[cfg(target_os = "linux")]
#[derive(Default)]
pub(crate) struct MicrophoneConsent(AtomicBool);

/// Ask before the recorder requests the device; other platforms prompt natively.
#[tauri::command]
pub(crate) async fn microphone_request_access(window: tauri::WebviewWindow) -> AppResult<bool> {
    #[cfg(target_os = "linux")]
    {
        use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

        if window
            .state::<MicrophoneConsent>()
            .0
            .load(Ordering::Acquire)
        {
            return Ok(true);
        }
        let (sender, receiver) = tokio::sync::oneshot::channel();
        window
            .dialog()
            .message("Allow Reflect to use your microphone for audio memos during this session?")
            .title("Microphone access")
            .buttons(MessageDialogButtons::OkCancelCustom(
                "Allow".into(),
                "Cancel".into(),
            ))
            .show(move |allowed| {
                let _ = sender.send(allowed);
            });
        let allowed = receiver.await.unwrap_or(false);
        if allowed {
            window
                .state::<MicrophoneConsent>()
                .0
                .store(true, Ordering::Release);
        }
        Ok(allowed)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = window;
        Ok(true)
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn permission_response(
    webview: tauri::Webview,
    kind: tauri::webview::PermissionKind,
) -> tauri::webview::PermissionResponse {
    use tauri::webview::{PermissionKind, PermissionResponse};

    let local_page = webview.url().ok().is_some_and(|url| {
        is_app_origin(&url, webview.app_handle().config().build.dev_url.as_ref())
    });
    match kind {
        PermissionKind::Microphone
            if local_page
                && webview
                    .state::<MicrophoneConsent>()
                    .0
                    .load(Ordering::Acquire) =>
        {
            PermissionResponse::Allow
        }
        PermissionKind::Microphone | PermissionKind::Camera | PermissionKind::DisplayCapture => {
            PermissionResponse::Deny
        }
        _ => PermissionResponse::Default,
    }
}

#[cfg(target_os = "linux")]
fn is_app_origin(url: &tauri::Url, dev_url: Option<&tauri::Url>) -> bool {
    (url.scheme() == "tauri" && url.host_str() == Some("localhost"))
        || (cfg!(debug_assertions) && dev_url.is_some_and(|dev| url.origin() == dev.origin()))
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn microphone_consent_does_not_extend_to_remote_pages() {
        assert!(is_app_origin(&"tauri://localhost/".parse().unwrap(), None));
        assert!(!is_app_origin(
            &"https://example.com/".parse().unwrap(),
            None
        ));
        assert!(!is_app_origin(
            &"tauri://example.com/".parse().unwrap(),
            None
        ));
    }
}
