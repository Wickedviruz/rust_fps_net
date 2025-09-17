use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiContexts, egui};
use crate::game::app_state::AppState;

use super::crosshair;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Startup, crosshair::spawn_crosshair)
            .add_systems(Update, main_menu_ui.run_if(in_state(AppState::MainMenu)))
            .add_systems(Update, pause_menu_ui.run_if(in_state(AppState::Paused)))
            .add_systems(Update, toggle_pause);
    }
}

fn main_menu_ui(
    mut egui_ctx: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let ctx = egui_ctx.ctx_mut();
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Rust FPS Net");
        if ui.button("Start Game").clicked() {
            next_state.set(AppState::InGame);
        }
        if ui.button("Quit").clicked() {
            std::process::exit(0);
        }
    });
}

fn pause_menu_ui(
    mut egui_ctx: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let ctx = egui_ctx.ctx_mut();
    egui::Window::new("Paused").show(ctx, |ui| {
        if ui.button("Resume").clicked() {
            next_state.set(AppState::InGame);
        }
        if ui.button("Main Menu").clicked() {
            next_state.set(AppState::MainMenu);
        }
    });
}

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match state.get() {
            AppState::InGame => next_state.set(AppState::Paused),
            AppState::Paused => next_state.set(AppState::InGame),
            _ => {}
        }
    }
}
