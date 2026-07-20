use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PermissionState, PluginApi},
    AppHandle, Manager, Runtime,
};

use crate::NotificationsBuilder;

#[cfg(windows)]
#[path = "windows_toast.rs"]
mod windows_toast;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Notifications<R>> {
    Ok(Notifications(app.clone()))
}

/// Access to the notification APIs.
///
/// You can get an instance of this type via [`NotificationsExt`](crate::NotificationsExt)
pub struct Notifications<R: Runtime>(AppHandle<R>);

impl<R: Runtime> crate::NotificationsBuilder<R> {
    pub async fn show(self) -> crate::Result<()> {
        #[cfg(windows)]
        {
            return windows_toast::show(
                &self.app,
                self.app.config().identifier.clone(),
                self.data.id,
                self.data.title.or_else(|| self.app.config().product_name.clone()),
                self.data.body,
                self.data.icon,
                self.data.extra,
                self.data.silent,
            );
        }

        #[cfg(not(windows))]
        {
            let mut notification = imp::Notification::new(self.app.config().identifier.clone());

            if let Some(title) = self
                .data
                .title
                .or_else(|| self.app.config().product_name.clone())
            {
                notification = notification.title(title);
            }
            if let Some(body) = self.data.body {
                notification = notification.body(body);
            }
            if let Some(icon) = self.data.icon {
                notification = notification.icon(icon);
            }

            notification.show()?;

            Ok(())
        }
    }
}

impl<R: Runtime> Notifications<R> {
    pub fn builder(&self) -> NotificationsBuilder<R> {
        NotificationsBuilder::new(self.0.clone())
    }

    pub async fn request_permission(&self) -> crate::Result<PermissionState> {
        Ok(PermissionState::Granted)
    }

    pub async fn register_for_push_notifications(&self) -> crate::Result<String> {
        Err(crate::Error::Io(std::io::Error::other(
            "Push notifications are not supported on desktop platforms",
        )))
    }

    pub fn unregister_for_push_notifications(&self) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Push notifications are not supported on desktop platforms",
        )))
    }

    pub async fn register_for_unified_push(&self) -> crate::Result<serde_json::Value> {
        Err(crate::Error::Io(std::io::Error::other(
            "UnifiedPush is not supported on desktop platforms",
        )))
    }

    pub fn unregister_from_unified_push(&self) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "UnifiedPush is not supported on desktop platforms",
        )))
    }

    pub fn get_unified_push_distributors(&self) -> crate::Result<serde_json::Value> {
        Err(crate::Error::Io(std::io::Error::other(
            "UnifiedPush is not supported on desktop platforms",
        )))
    }

    pub fn save_unified_push_distributor(&self, _distributor: String) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "UnifiedPush is not supported on desktop platforms",
        )))
    }

    pub fn get_unified_push_distributor(&self) -> crate::Result<serde_json::Value> {
        Err(crate::Error::Io(std::io::Error::other(
            "UnifiedPush is not supported on desktop platforms",
        )))
    }

    pub async fn permission_state(&self) -> crate::Result<PermissionState> {
        Ok(PermissionState::Granted)
    }

    pub async fn pending(&self) -> crate::Result<Vec<crate::PendingNotification>> {
        Err(crate::Error::Io(std::io::Error::other(
            "Pending notifications are not supported with notify-rust",
        )))
    }

    pub async fn active(&self) -> crate::Result<Vec<crate::ActiveNotification>> {
        Err(crate::Error::Io(std::io::Error::other(
            "Active notifications are not supported with notify-rust",
        )))
    }

    pub fn set_click_listener_active(&self, active: bool) -> crate::Result<()> {
        #[cfg(windows)]
        {
            windows_toast::set_click_listener_active(active);
            return Ok(());
        }
        #[cfg(not(windows))]
        {
            let _ = active;
            Err(crate::Error::Io(std::io::Error::other(
                "Click listeners are not supported with notify-rust",
            )))
        }
    }

    pub fn remove_active(&self, _ids: Vec<i32>) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Removing active notifications is not supported with notify-rust",
        )))
    }

    pub fn cancel(&self, _notifications: Vec<i32>) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Canceling notifications is not supported with notify-rust",
        )))
    }

    pub fn cancel_all(&self) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Canceling notifications is not supported with notify-rust",
        )))
    }

    pub fn register_action_types(&self, _types: Vec<crate::ActionType>) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Action types are not supported with notify-rust",
        )))
    }

    pub fn create_channel(&self, _channel: crate::Channel) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Notification channels are not supported with notify-rust",
        )))
    }

    pub fn delete_channel(&self, _id: impl Into<String>) -> crate::Result<()> {
        Err(crate::Error::Io(std::io::Error::other(
            "Notification channels are not supported with notify-rust",
        )))
    }

    pub fn list_channels(&self) -> crate::Result<Vec<crate::Channel>> {
        Err(crate::Error::Io(std::io::Error::other(
            "Notification channels are not supported with notify-rust",
        )))
    }
}

/// Bring the main window forward after a notification click.
pub(crate) fn focus_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(not(windows))]
mod imp {
    //! Types and functions related to desktop notifications (notify-rust path).

    /// The desktop notification definition.
    ///
    /// Allows you to construct a Notification data and send it.
    #[allow(dead_code)]
    #[derive(Debug, Default)]
    pub struct Notification {
        /// The notification body.
        body: Option<String>,
        /// The notification title.
        title: Option<String>,
        /// The notification icon.
        icon: Option<String>,
        /// The notification identifier
        identifier: String,
    }

    impl Notification {
        /// Initializes a instance of a Notification.
        pub fn new(identifier: impl Into<String>) -> Self {
            Self {
                identifier: identifier.into(),
                ..Default::default()
            }
        }

        /// Sets the notification body.
        #[must_use]
        pub fn body(mut self, body: impl Into<String>) -> Self {
            self.body = Some(body.into());
            self
        }

        /// Sets the notification title.
        #[must_use]
        pub fn title(mut self, title: impl Into<String>) -> Self {
            self.title = Some(title.into());
            self
        }

        /// Sets the notification icon.
        #[must_use]
        pub fn icon(mut self, icon: impl Into<String>) -> Self {
            self.icon = Some(icon.into());
            self
        }

        /// Shows the notification.
        pub fn show(self) -> crate::Result<()> {
            let mut notification = notify_rust::Notification::new();
            if let Some(body) = self.body {
                notification.body(&body);
            }
            if let Some(title) = self.title {
                notification.summary(&title);
            }
            if let Some(icon) = self.icon {
                notification.icon(&icon);
            } else {
                notification.auto_icon();
            }
            #[cfg(target_os = "macos")]
            {
                let _ = notify_rust::set_application(if tauri::is_dev() {
                    "com.apple.Terminal"
                } else {
                    &self.identifier
                });
            }

            tauri::async_runtime::spawn(async move {
                let _ = notification.show();
            });

            Ok(())
        }
    }
}
