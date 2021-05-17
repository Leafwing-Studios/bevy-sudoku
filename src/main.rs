use bevy::{input::system::exit_on_esc_system, prelude::*};

mod aesthetics;
mod board;
mod interaction;
mod sudoku_generation;
mod utils;

fn main() {
    App::build()
        .insert_resource(ClearColor(aesthetics::BACKGROUND_COLOR))
        .add_plugins(DefaultPlugins)
        // Must occur after DefaultPlugins, but before our stage is used
        // Implicitly inserts a startup stage after the default CoreStage::Startup
        .add_startup_stage(utils::SudokuStage::PostStartup, SystemStage::parallel())
        .add_plugin(aesthetics::AssetLoadingPlugin)
        .add_plugin(board::setup::SetupPlugin)
        .add_plugin(interaction::InteractionPlugin)
        .add_plugin(sudoku_generation::GenerationPlugin)
        .add_system(exit_on_esc_system.system())
        .run();
}
