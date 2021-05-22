/// Display the Sudoku game
pub mod board;
pub mod buttons;

use bevy::prelude::*;

pub const BACKGROUND_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);

/// Marker component for game camera
pub struct MainCamera;
/// Marker component for UI camera
pub struct UiCamera;

/// Adds cameras to our game
pub fn spawn_cameras(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(UiCamera);
}
