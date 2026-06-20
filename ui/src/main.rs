#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{
    Arc,
    atomic::AtomicBool,
};

use anyhow::{Result, anyhow};
use single_instance::SingleInstance;

use crate::{
    app::Lumen,
    geometry::{SHELL_HEIGHT, SHELL_WIDTH},
    platform::{initialize_tray, initialize_window},
    settings::UserSettings,
};

mod app;
mod geometry;
mod platform;
mod settings;
mod state;
mod sync;

slint::include_modules!();

pub const AUMID: &str = "io.risuleia.lumen";

fn main() -> Result<()> {
    let instance = SingleInstance::new(AUMID).unwrap();
    if !instance.is_single() {
        return Err(anyhow!("One instance of Lumen is already running."));
    }

    slint::platform::set_platform(Box::new(i_slint_backend_winit::Backend::new().unwrap()))
        .unwrap();

    let mut app = Lumen::new();

    let state = app.state().clone();
    let shell = Shell::new().unwrap();
    let collapsed_weak = shell.as_weak();
    let outside_click_weak = shell.as_weak();
    let settings = UserSettings::load();
    let always_on_top = Arc::new(AtomicBool::new(settings.always_on_top));

    let (_tray, _tray_timer) = initialize_tray(always_on_top.clone());

    initialize_window(
        &shell,
        SHELL_WIDTH,
        SHELL_HEIGHT,
        state.clone(),
        always_on_top.clone(),
        move || {
            collapsed_weak
                .upgrade()
                .map(|s| s.global::<IslandData>().get_collapsed())
                .unwrap_or(false)
        },
        move || {
            if let Some(shell) = outside_click_weak.upgrade() {
                shell.global::<IslandData>().invoke_action("outside-click".into(), "".into());
            }
        },
    );

    app.start(&shell)?;

    Ok(())
}
