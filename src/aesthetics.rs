/// Stores aesthetic configuration and handles asset loading
use crate::ui::{ButtonMaterials, NewPuzzle, ResetPuzzle, SolvePuzzle};
use bevy::prelude::*;
use std::marker::PhantomData;

// Colors
pub const BACKGROUND_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);
pub const SELECTION_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

pub const GRID_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const NUMBER_COLOR: Color = Color::BLACK;

// Fonts
pub const FIXED_NUM_FONT: &str = "fonts/Ubuntu-Bold.ttf";
pub const FILLABLE_NUM_FONT: &str = "fonts/Ubuntu-Light.ttf";

// Sizes
pub const CELL_SIZE: f32 = 50.0;
pub const GRID_SIZE: f32 = 9.0 * CELL_SIZE;
pub const MINOR_LINE_THICKNESS: f32 = 2.0;
pub const MAJOR_LINE_THICKNESS: f32 = 4.0;

// Defines the center lines of the grid in absolute coordinates
// (0, 0) is in the center of the screen in Bevy
pub const GRID_CENTER_X: f32 = 0.0;
pub const GRID_LEFT_EDGE: f32 = GRID_CENTER_X - 0.5 * GRID_SIZE;
pub const GRID_CENTER_Y: f32 = 0.0;
pub const GRID_BOT_EDGE: f32 = GRID_CENTER_Y - 0.5 * GRID_SIZE;

pub struct AssetLoadingPlugin;

impl Plugin for AssetLoadingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(load_fonts.system())
            .init_resource::<ButtonMaterials<NewPuzzle>>()
            .init_resource::<ButtonMaterials<ResetPuzzle>>()
            .init_resource::<ButtonMaterials<SolvePuzzle>>();
    }
}

pub struct FixedFont(pub Handle<Font>);
pub struct FillableFont(pub Handle<Font>);
fn load_fonts(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    let fixed_handle = asset_server.load(FIXED_NUM_FONT);
    commands.insert_resource(FixedFont(fixed_handle));
    let fillable_handle = asset_server.load(FILLABLE_NUM_FONT);
    commands.insert_resource(FillableFont(fillable_handle));
}

impl FromWorld for ButtonMaterials<NewPuzzle> {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(1.0, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
            _marker: PhantomData,
        }
    }
}

impl FromWorld for ButtonMaterials<ResetPuzzle> {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 1.0, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
            _marker: PhantomData,
        }
    }
}

impl FromWorld for ButtonMaterials<SolvePuzzle> {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 1.0).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
            _marker: PhantomData,
        }
    }
}
