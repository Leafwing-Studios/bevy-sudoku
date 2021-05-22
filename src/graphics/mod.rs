/// Display the Sudoku game
pub mod board;
pub mod buttons;

use bevy::prelude::*;

use self::assets::*;
use self::config::*;

/// Adds cameras to our game
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
}

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<NoneColor>()
            .add_startup_system(spawn_camera.system())
            // Must be in an earlier stage to ensure layout nodes are ready for use
            .add_startup_system_to_stage(
                StartupStage::PreStartup,
                layout::spawn_layout_boxes.system(),
            )
            .add_plugin(board::BoardPlugin)
            .add_plugin(buttons::ButtonsPlugin);
    }
}

pub mod config {
    use super::*;
    // The horizontal percentage of the screen that the buttons take up
    pub const BUTTON_PERCENT: f32 = 50.0;

    pub const BACKGROUND_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);
}

mod assets {
    use super::*;
    /// The null, transparent color
    pub struct NoneColor(pub Handle<ColorMaterial>);

    impl FromWorld for NoneColor {
        fn from_world(world: &mut World) -> Self {
            let mut materials = world
                .get_resource_mut::<Assets<ColorMaterial>>()
                .expect("ResMut<Assets<ColorMaterial>> not found.");
            NoneColor(materials.add(Color::NONE.into()))
        }
    }
}

mod layout {
    use super::*;
    /// Marker component for layout box of Sudoku game elements
    pub struct SudokuBox;
    /// Marker component for layout box of UI elements
    pub struct UiBox;

    /// Spawns layout-only nodes for storing the game's user interface
    pub fn spawn_layout_boxes(mut commands: Commands, none_color: Res<NoneColor>) {
        // Global root node
        commands
            .spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    ..Default::default()
                },
                material: none_color.0.clone(),
                ..Default::default()
            })
            .with_children(|parent| {
                // Sudoku on left
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(
                                Val::Percent(100.0 - BUTTON_PERCENT),
                                Val::Percent(100.0),
                            ),
                            // FIXME: fix comments
                            // UI elements are arranged in stacked rows, growing from the bottom
                            flex_direction: FlexDirection::ColumnReverse,
                            // Don't wrap these elements
                            flex_wrap: FlexWrap::NoWrap,
                            // These buttons should be grouped tightly together within each row
                            align_items: AlignItems::Center,
                            // Center the UI vertically
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        //material: materials.add(Color::PURPLE.into()),
                        ..Default::default()
                    })
                    .insert(SudokuBox);

                // Interface on right
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Percent(BUTTON_PERCENT), Val::Percent(100.0)),
                            // UI elements are arranged in stacked rows, growing from the bottom
                            flex_direction: FlexDirection::ColumnReverse,
                            // Don't wrap these elements
                            flex_wrap: FlexWrap::NoWrap,
                            // These buttons should be grouped tightly together within each row
                            align_items: AlignItems::Center,
                            // Center the UI vertically
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        material: none_color.0.clone(),
                        ..Default::default()
                    })
                    .insert(UiBox);
            });
    }
}
