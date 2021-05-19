use std::marker::PhantomData;

use bevy::{ecs::component::Component, prelude::*};

use crate::{
    aesthetics::{FixedFont, NUMBER_COLOR},
    interaction::CellInput,
    interaction::InputMode,
    utils::SudokuStage,
};

use self::config::*;

mod config {
    // The percentage of the screen that the UI panel takes up
    pub const UI_FRACTION: f32 = 40.0;
    /// The side length of the UI buttons
    pub const BUTTON_LENGTH: f32 = 64.0;
    /// The side length of the numpad-like input buttons
    pub const NUM_BUTTON_LENGTH: f32 = 64.0;
}

pub struct BoardButtonsPlugin;

// TODO: input cell values by button presses
// QUALITY: use system sets for clarity
impl Plugin for BoardButtonsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<NoneColor>()
            .add_startup_system(spawn_layout_boxes.system())
            .add_startup_system_to_stage(SudokuStage::PostStartup, spawn_buttons.system())
            // Number input buttons
            .add_system(puzzle_button::<CellInput>.system().label("input"))
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
/// corresponding to the Marker type marker component
pub struct ButtonMaterials<Marker: Component> {
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
struct BoardButtonBundle<Marker: Component> {
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
}

impl<Marker: Component> BoardButtonBundle<Marker> {
    fn new_with_data(size: Size<Val>, materials: &ButtonMaterials<Marker>, data: Marker) -> Self {
        let normal_material = materials.normal.clone();
        let hovered_material = materials.hovered.clone();
        let pressed_material = materials.pressed.clone();

        BoardButtonBundle {
            marker: data,
            button_bundle: ButtonBundle {
                style: Style {
                    size,
                    // Padding between buttons
                    margin: Rect::all(Val::Px(5.0)),
                    // Horizontally center child text
                    justify_content: JustifyContent::Center,
                    // Vertically center child text
                    align_items: AlignItems::Center,
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

#[derive(Default, Clone)]
pub struct NewPuzzle;
#[derive(Default, Clone)]
pub struct ResetPuzzle;
#[derive(Default, Clone)]
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
    number_materials: Res<ButtonMaterials<CellInput>>,
    // TODO: split into three? Or maybe group into two resources total?
    input_mode_button_materials: Res<ButtonMaterials<InputMode>>,
    font: Res<FixedFont>,
) {
    let button_size = Size::new(Val::Px(BUTTON_LENGTH), Val::Px(BUTTON_LENGTH));
    let num_button_size = Size::new(Val::Px(NUM_BUTTON_LENGTH), Val::Px(NUM_BUTTON_LENGTH));

    // Layout nodes
    const N_ROWS: usize = 5;
    let mut layout_nodes = [Entity::new(0); N_ROWS];
    for i in 0..N_ROWS {
        layout_nodes[i] = commands
            .spawn_bundle(NodeBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    align_content: AlignContent::Center,
                    ..Default::default()
                },
                ..Default::default()
            })
            .id();
    }

    // Number input buttons
    let mut number_buttons = [Entity::new(0); 9];
    for i in 0..9 {
        let value = i + 1;

        const TEXT_ALIGNMENT: TextAlignment = TextAlignment {
            vertical: VerticalAlign::Center,
            horizontal: HorizontalAlign::Center,
        };

        let text_style = TextStyle {
            font: font.0.clone(),
            font_size: 0.8 * NUM_BUTTON_LENGTH,
            color: NUMBER_COLOR,
        };

        number_buttons[i] = commands
            .spawn_bundle(BoardButtonBundle::<CellInput>::new_with_data(
                num_button_size,
                &*number_materials,
                CellInput { value: value as u8 },
            ))
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    text: Text::with_section(value.to_string(), text_style.clone(), TEXT_ALIGNMENT),
                    ..Default::default()
                });
            })
            .id();
    }

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

    // Building our hierarchy, from bottom to top
    let ui_root_entity = ui_root_query.single().unwrap();
    commands.entity(ui_root_entity).push_children(&layout_nodes);

    // Number buttons
    commands
        .entity(layout_nodes[0])
        .push_children(&number_buttons[0..3]);

    commands
        .entity(layout_nodes[1])
        .push_children(&number_buttons[3..6]);

    commands
        .entity(layout_nodes[2])
        .push_children(&number_buttons[6..9]);

    // Row 1 buttons
    commands.entity(layout_nodes[3]).push_children(&[
        fill_button,
        center_mark_button,
        corner_mark_button,
    ]);

    // Row 2 buttons
    commands.entity(layout_nodes[4]).push_children(&[
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
/// using the data stored on the component of that type
fn puzzle_button<Marker: Component + Clone>(
    query: Query<(&Interaction, &Marker)>,
    mut event_writer: EventWriter<Marker>,
) {
    for (interaction, marker) in query.iter() {
        if *interaction == Interaction::Clicked {
            event_writer.send(marker.clone());
        }
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
