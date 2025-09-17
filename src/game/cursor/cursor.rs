use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use crate::game::app_state::AppState;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), unlock_cursor)
           .add_systems(OnEnter(AppState::Paused), unlock_cursor)
           .add_systems(OnEnter(AppState::InGame), lock_cursor);
    }
}

fn lock_cursor(mut q: Query<&mut Window, With<PrimaryWindow>>) {
    info!("Locking cursor (InGame)");
    let mut window = q.single_mut();
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Locked;
}

fn unlock_cursor(mut q: Query<&mut Window, With<PrimaryWindow>>) {
    info!("Unlocking cursor (Menu/Pause)");
    let mut window = q.single_mut();
    window.cursor.visible = true;
    window.cursor.grab_mode = CursorGrabMode::None;
}
