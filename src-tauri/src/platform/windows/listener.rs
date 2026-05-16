use crate::types::SystemNotification;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use tauri::{AppHandle, Emitter, Manager};
use windows::Foundation::TypedEventHandler;
use windows::Storage::Streams::DataReader;
use windows::UI::Notifications::Management::{
    UserNotificationListener, UserNotificationListenerAccessStatus,
};
use windows::UI::Notifications::{UserNotificationChangedEventArgs, UserNotificationChangedKind};

#[allow(dead_code)]
pub struct ListenerState {
    pub listener: UserNotificationListener,
}

pub async fn start_notification_listener(app: AppHandle) -> Result<(), String> {
    let listener = UserNotificationListener::Current()
        .map_err(|e| format!("Failed to get listener: {}", e))?;

    let access_status = listener
        .RequestAccessAsync()
        .map_err(|e| format!("Request access error: {}", e))?
        .get()
        .map_err(|e| format!("Wait access error: {}", e))?;

    if access_status != UserNotificationListenerAccessStatus::Allowed {
        return Err(format!(
            "Notification access not allowed. Status: {:?}",
            access_status
        ));
    }

    let app_clone = app.clone();
    let listener_clone = listener.clone();

    listener
        .NotificationChanged(&TypedEventHandler::<
            UserNotificationListener,
            UserNotificationChangedEventArgs,
        >::new(move |_, args| {
            // Ignore errors inside the event handler to not crash the listener
            let args = args.as_ref().unwrap();
            if let Ok(kind) = args.ChangeKind() {
                if kind == UserNotificationChangedKind::Added {
                    if let Ok(notification_id) = args.UserNotificationId() {
                        if let Ok(notification) = listener_clone.GetNotification(notification_id) {
                            let mut sys_notif = SystemNotification {
                                id: notification.Id().unwrap_or(0),
                                app_name: String::new(),
                                app_id: String::new(),
                                title: String::new(),
                                body: String::new(),
                                icon_base64: None,
                            };

                            if let Ok(app_info) = notification.AppInfo() {
                                if let Ok(display_info) = app_info.DisplayInfo() {
                                    sys_notif.app_name =
                                        display_info.DisplayName().unwrap_or_default().to_string();

                                    // Try to get icon
                                    if let Ok(logo_ref) =
                                        display_info.GetLogo(windows::Foundation::Size {
                                            Width: 0.0,
                                            Height: 0.0,
                                        })
                                    {
                                        if let Ok(stream_with_content_type) =
                                            logo_ref.OpenReadAsync().and_then(|op| op.get())
                                        {
                                            if let Ok(size) = stream_with_content_type.Size() {
                                                if let Ok(reader) = DataReader::CreateDataReader(
                                                    &stream_with_content_type,
                                                ) {
                                                    if reader
                                                        .LoadAsync(size as u32)
                                                        .and_then(|op| op.get())
                                                        .is_ok()
                                                    {
                                                        let mut buffer = vec![0u8; size as usize];
                                                        if reader.ReadBytes(&mut buffer).is_ok() {
                                                            sys_notif.icon_base64 = Some(
                                                                BASE64_STANDARD.encode(&buffer),
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                sys_notif.app_id =
                                    app_info.AppUserModelId().unwrap_or_default().to_string();
                            }

                            if let Ok(notif_binding) = notification.Notification() {
                                if let Ok(visual) = notif_binding.Visual() {
                                    if let Ok(bindings) = visual.Bindings() {
                                        if let Ok(binding) = bindings.GetAt(0) {
                                            if let Ok(text_elements) = binding.GetTextElements() {
                                                let text_count = text_elements.Size().unwrap_or(0);
                                                if text_count > 0 {
                                                    sys_notif.title = text_elements
                                                        .GetAt(0)
                                                        .and_then(|t| t.Text())
                                                        .unwrap_or_default()
                                                        .to_string();
                                                }
                                                if text_count > 1 {
                                                    sys_notif.body = text_elements
                                                        .GetAt(1)
                                                        .and_then(|t| t.Text())
                                                        .unwrap_or_default()
                                                        .to_string();
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            let _ = app_clone.emit("new-system-notification", sys_notif);
                        }
                    }
                }
            }
            Ok(())
        }))
        .map_err(|e| format!("Failed to register event handler: {}", e))?;

    app.manage(ListenerState { listener });
    println!("✅ Notification listener started successfully");

    Ok(())
}
