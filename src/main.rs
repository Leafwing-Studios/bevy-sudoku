use bevy::{input::system::exit_on_esc_system, prelude::*};

mod graphics;
mod input;
mod logic;

fn main() {
    App::build()
        .insert_resource(ClearColor(graphics::aesthetics::BACKGROUND_COLOR))
        .add_plugins(DefaultPlugins)
        // Must occur after DefaultPlugins, but before our stage is used
        // Implicitly inserts a startup stage after the default CoreStage::Startup
        .add_plugin(graphics::aesthetics::AssetLoadingPlugin)
        .add_plugin(logic::board::setup::SetupPlugin)
        .add_plugin(input::interaction::InteractionPlugin)
        .add_plugin(logic::sudoku_generation::GenerationPlugin)
        .add_plugin(graphics::ui::BoardButtonsPlugin)
        .add_startup_system(spawn_cameras.system())
        .add_system(exit_on_esc_system.system())
        .run();
}
/// Marker component for game camera
struct MainCamera;
/// Marker component for UI camera
struct UiCamera;

/// Adds cameras to our game
fn spawn_cameras(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(UiCamera);
}
