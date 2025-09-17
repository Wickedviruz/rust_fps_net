use bevy::prelude::*;

pub mod game;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<game::app_state::AppState>() 
        .add_plugins(game::game::GamePlugin)
        .run();
}