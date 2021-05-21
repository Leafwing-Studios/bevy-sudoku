use bevy::{input::system::exit_on_esc_system, prelude::*};

mod graphics;
mod input;
mod logic;

fn main() {
    App::build()
        .insert_resource(ClearColor(graphics::BACKGROUND_COLOR))
        .add_plugins(DefaultPlugins)
        .add_startup_system(graphics::spawn_cameras.system())
        .add_plugin(graphics::board::BoardPlugin)
        .add_plugin(graphics::buttons::BoardButtonsPlugin)
        .add_plugin(input::InteractionPlugin)
        .add_plugin(logic::sudoku_generation::GenerationPlugin)
        .add_system(exit_on_esc_system.system())
        .run();
}
