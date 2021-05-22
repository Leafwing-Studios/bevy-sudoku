/// A simple but polished Sudoku game, written in Bevy
use bevy::prelude::*;

mod graphics;
mod input;
mod logic;

fn main() {
    App::build()
        .insert_resource(ClearColor(graphics::config::BACKGROUND_COLOR))
        .add_plugins(DefaultPlugins)
        .add_plugin(graphics::GraphicsPlugin)
        .add_plugin(input::InputPlugin)
        .add_plugin(logic::board::LogicPlugin)
        .add_plugin(logic::sudoku_generation::GenerationPlugin)
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}

#[derive(SystemLabel, Clone, Hash, Copy, PartialEq, Eq, Debug)]
enum CommonLabels {
    Input,
    Action,
}
