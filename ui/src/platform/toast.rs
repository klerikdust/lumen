use anyhow::Result;
use windows::{
    Data::Xml::Dom::XmlDocument,
    Foundation::TypedEventHandler,
    UI::Notifications::{ToastActivatedEventArgs, ToastNotification, ToastNotificationManager},
};
use windows_core::{HSTRING, Interface};

use crate::AUMID;

pub fn show_update_toast(version: &str, on_update: impl Fn() + Send + 'static) {
    let xml = format!(
        r#"
        <toast launch="action=update" activationType="foreground">
            <visual>
                <binding template="ToastGeneric">
                    <text>Lumen {version} is available</text>
                    <text>A new version of Lumen is ready to install.</text>
                </binding>
            </visual>
            <actions>
                <action
                    content="Update Now"
                    arguments="action=update"
                    activationType="foreground"/>
                <action
                    content="Later"
                    arguments="action=dismiss"
                    activationType="foreground"/>
            </actions>
        </toast>
    "#
    );

    if let Err(e) = send_toast(&xml, on_update) {
        eprintln!("[Updater] Toast failed: {e}");
    }
}

fn send_toast(xml: &str, on_update: impl Fn() + Send + 'static) -> Result<()> {
    let doc = XmlDocument::new()?;
    doc.LoadXml(&HSTRING::from(xml))?;

    let toast = ToastNotification::CreateToastNotification(&doc)?;

    toast.Activated(&TypedEventHandler::new(
        move |_, args: windows_core::Ref<'_, windows_core::IInspectable>| {
            if let Some(toast_args) =
                args.as_ref().and_then(|a| a.cast::<ToastActivatedEventArgs>().ok())
            {
                if let Ok(arguments) = toast_args.Arguments() {
                    if arguments == "action=update" {
                        on_update();
                    }
                }
            }

            Ok(())
        },
    ))?;

    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(AUMID))?;

    notifier.Show(&toast)?;

    Ok(())
}
