use std::marker::PhantomData;

use bevy::{ecs::component::Component, prelude::*};

use crate::{interaction::InputMode, utils::SudokuStage};

use self::config::*;

mod config {
    // The percentage of the screen that the UI panel takes up
    pub const UI_FRACTION: f32 = 40.0;
    /// The side length of the UI buttons
    pub const BUTTON_LENGTH: f32 = 128.0;
}

pub struct BoardButtonsPlugin;

// TODO: input cell values by button presses
// QUALITY: use system sets for clarity
impl Plugin for BoardButtonsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<NoneColor>()
            .add_startup_system(spawn_layout_boxes.system())
            .add_startup_system_to_stage(SudokuStage::PostStartup, spawn_buttons.system())
            // Puzzle control buttons
            .add_system(responsive_buttons.system().label("responsive_buttons"))
            .add_event::<NewPuzzle>()
            .add_system(puzzle_button::<NewPuzzle>.system())
            .add_event::<ResetPuzzle>()
            .add_system(puzzle_button::<ResetPuzzle>.system())
            .add_event::<SolvePuzzle>()
            .add_system(puzzle_button::<SolvePuzzle>.system())
            // Input mode buttons
            .add_system(input_mode_buttons.system().label("input"))
            // Must overwrite default button responsivity for selected input mode
            .add_system(
                show_selected_input_mode
                    .system()
                    .after("input")
                    .after("responsive_buttons"),
            );
    }
}

/// The null, transparent color
struct NoneColor(Handle<ColorMaterial>);

impl FromWorld for NoneColor {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        NoneColor(materials.add(Color::NONE.into()))
    }
}

/// Resource that contains the raw materials for each button type
/// correspsonding to the Marker type marker component
pub struct ButtonMaterials<Marker: Component + Default> {
    pub normal: Handle<ColorMaterial>,
    pub hovered: Handle<ColorMaterial>,
    pub pressed: Handle<ColorMaterial>,
    pub _marker: PhantomData<Marker>,
}

/// Component for the material of a button at rest
struct NormalMaterial(Handle<ColorMaterial>);
/// Component for the material of a button when hovered
struct HoveredMaterial(Handle<ColorMaterial>);
/// Component for the material of a button when pressed
struct PressedMaterial(Handle<ColorMaterial>);

#[derive(Bundle)]
struct BoardButtonBundle<Marker: Component + Default> {
    marker: Marker,
    #[bundle]
    button_bundle: ButtonBundle,
    normal_material: NormalMaterial,
    hovered_material: HoveredMaterial,
    pressed_material: PressedMaterial,
}

impl<Marker: Component + Default> BoardButtonBundle<Marker> {
    fn new(size: Size<Val>, materials: &ButtonMaterials<Marker>) -> Self {
        let data = Marker::default();
        Self::new_with_data(size, materials, data)
    }

    fn new_with_data(size: Size<Val>, materials: &ButtonMaterials<Marker>, data: Marker) -> Self {
        let normal_material = materials.normal.clone();
        let hovered_material = materials.hovered.clone();
        let pressed_material = materials.pressed.clone();

        BoardButtonBundle {
            marker: data,
            button_bundle: ButtonBundle {
                // TODO: add padding between buttons
                style: Style {
                    size,
                    ..Default::default()
                },
                material: normal_material.clone(),
                ..Default::default()
            },
            normal_material: NormalMaterial(normal_material),
            hovered_material: HoveredMaterial(hovered_material),
            pressed_material: PressedMaterial(pressed_material),
        }
    }
}

#[derive(Default)]
pub struct NewPuzzle;
#[derive(Default)]
pub struct ResetPuzzle;
#[derive(Default)]
pub struct SolvePuzzle;

/// Marker component for layout box of Sudoku game elements
struct SudokuBox;
/// Marker component for layout box of UI elements
struct UiBox;

fn spawn_layout_boxes(mut commands: Commands, none_color: Res<NoneColor>) {
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
                        size: Size::new(Val::Percent(100.0 - UI_FRACTION), Val::Percent(100.0)),
                        ..Default::default()
                    },
                    material: none_color.0.clone(),
                    ..Default::default()
                })
                .insert(SudokuBox);

            // Interface on right
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(UI_FRACTION), Val::Percent(100.0)),
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

/// Creates the side panel buttons
fn spawn_buttons(
    mut commands: Commands,
    ui_root_query: Query<Entity, With<UiBox>>,
    new_button_materials: Res<ButtonMaterials<NewPuzzle>>,
    reset_button_materials: Res<ButtonMaterials<ResetPuzzle>>,
    solve_button_materials: Res<ButtonMaterials<SolvePuzzle>>,
    // TODO: split into three? Or maybe group into two resources total?
    input_mode_button_materials: Res<ButtonMaterials<InputMode>>,
) {
    let button_size = Size::new(Val::Px(BUTTON_LENGTH), Val::Px(BUTTON_LENGTH));

    // Layout entities
    let ui_root_entity = ui_root_query.single().unwrap();
    let top_row_entity = commands
        .spawn_bundle(NodeBundle {
            ..Default::default()
        })
        .id();
    let bottom_row_entity = commands
        .spawn_bundle(NodeBundle {
            ..Default::default()
        })
        .id();

    // Input mode buttons
    let fill_button = commands
        .spawn_bundle(BoardButtonBundle::<InputMode>::new_with_data(
            button_size,
            &*input_mode_button_materials,
            InputMode::Fill,
        ))
        .id();

    let center_mark_button = commands
        .spawn_bundle(BoardButtonBundle::<InputMode>::new_with_data(
            button_size,
            &*input_mode_button_materials,
            InputMode::CenterMark,
        ))
        .id();

    let corner_mark_button = commands
        .spawn_bundle(BoardButtonBundle::<InputMode>::new_with_data(
            button_size,
            &*input_mode_button_materials,
            InputMode::CornerMark,
        ))
        .id();

    // Game control buttons
    let new_game_button = commands
        .spawn_bundle(BoardButtonBundle::<NewPuzzle>::new(
            button_size,
            &*new_button_materials,
        ))
        .id();

    let reset_game_button = commands
        .spawn_bundle(BoardButtonBundle::<ResetPuzzle>::new(
            button_size,
            &*reset_button_materials,
        ))
        .id();

    let solve_game_button = commands
        .spawn_bundle(BoardButtonBundle::<SolvePuzzle>::new(
            button_size,
            &*solve_button_materials,
        ))
        .id();

    // Building our hierarchy
    commands
        .entity(ui_root_entity)
        .push_children(&[top_row_entity, bottom_row_entity]);

    commands.entity(top_row_entity).push_children(&[
        fill_button,
        center_mark_button,
        corner_mark_button,
    ]);

    commands.entity(bottom_row_entity).push_children(&[
        new_game_button,
        reset_game_button,
        solve_game_button,
    ]);
}
/// Marker component for entities whose materials should not respond
struct FixedMaterial;

fn responsive_buttons(
    mut button_query: Query<
        (
            &Interaction,
            &mut Handle<ColorMaterial>,
            &NormalMaterial,
            &HoveredMaterial,
            &PressedMaterial,
        ),
        (Without<FixedMaterial>, Changed<Interaction>),
    >,
) {
    for (interaction, mut material, normal_material, hovered_material, pressed_material) in
        button_query.iter_mut()
    {
        *material = match *interaction {
            Interaction::None => normal_material.0.clone(),
            Interaction::Hovered => hovered_material.0.clone(),
            Interaction::Clicked => pressed_material.0.clone(),
        }
    }
}

/// Sends the event type associated with the button when pressed
fn puzzle_button<Marker: Component + Default>(
    query: Query<&Interaction, With<Marker>>,
    mut event_writer: EventWriter<Marker>,
) {
    let interaction = query.single().unwrap();

    if *interaction == Interaction::Clicked {
        event_writer.send(Marker::default());
    }
}

/// Changes the input mode of the puzzle when these buttons are pressed
fn input_mode_buttons(
    button_query: Query<(&Interaction, &InputMode), Changed<Interaction>>,
    mut input_mode: ResMut<InputMode>,
) {
    for (interaction, button_input_mode) in button_query.iter() {
        if *interaction == Interaction::Clicked {
            *input_mode = *button_input_mode;
        }
    }
}

/// Permanently displays selected input mode as pressed
fn show_selected_input_mode(
    mut button_query: Query<(
        Entity,
        &InputMode,
        &mut Handle<ColorMaterial>,
        &PressedMaterial,
        &NormalMaterial,
    )>,
    input_mode: Res<InputMode>,
    mut commands: Commands,
) {
    if input_mode.is_changed() {
        for (entity, button_input_mode, mut material, pressed_material, normal_material) in
            button_query.iter_mut()
        {
            if *button_input_mode == *input_mode {
                *material = pressed_material.0.clone();
                commands.entity(entity).insert(FixedMaterial);
            } else {
                *material = normal_material.0.clone();
                commands.entity(entity).remove::<FixedMaterial>();
            }
        }
    }
}
