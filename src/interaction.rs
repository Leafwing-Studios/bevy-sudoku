use crate::board::{Cell, DisplayedBy, Fixed, Value};
/// Player input handling for actually playing Sudoku
use crate::{
    aesthetics::{BACKGROUND_COLOR, SELECTION_COLOR},
    MainCamera,
};
use bevy::prelude::*;
use bevy::utils::HashMap;

use cell_indexing::{index_cells, CellIndex};
pub struct InteractionPlugin;

/// Marker component for selected cells
#[derive(Debug)]
pub struct Selected;

/// Event to dispatch cell clicks
struct CellClick {
    /// Some(entity) if a cell was clicked, otherwise None
    selected_cell: Option<Entity>,
    /// Should we select multiple cells at once
    multi: bool,
    /// Was the mouse dragged
    drag: bool,
}

// Various colors for our cells
/// The color of the game's background, and the default color of the cells
struct BackgroundColor(Handle<ColorMaterial>);
/// The color of cells when selected
struct SelectionColor(Handle<ColorMaterial>);

// QUALITY: use typed system labels instead of strings
// QUALITY: use system sets and label them all at once
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(load_cell_colors.system())
            .init_resource::<CellIndex>()
            .add_event::<CellClick>()
            .add_event::<CellInput>()
            .init_resource::<CellInputMap>()
            .init_resource::<InputMode>()
            // Should run before input to ensure mapping from position to cell is correct
            .add_system(index_cells.system().before("input"))
            // Various input systems
            .add_system(cell_click.system().label("input"))
            .add_system(select_all.system().label("input"))
            .add_system(cell_keyboard_input.system().label("input"))
            .add_system(erase_selected_cells.system().label("input"))
            .add_system(swap_input_mode.system().label("input"))
            // Should immediately run to process input events after
            .add_system(handle_clicks.system().label("actions").after("input"))
            .add_system(set_cell_value.system().label("actions").after("input"))
            // Should run after actions to avoid delays
            .add_system(color_selected.system().after("actions"))
            .add_system(update_cell_numbers.system().after("actions"));
    }
}
/// Initializes cell color resources
fn load_cell_colors(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(BackgroundColor(materials.add(BACKGROUND_COLOR.into())));
    commands.insert_resource(SelectionColor(materials.add(SELECTION_COLOR.into())));
}

/// Turns raw clicks into `CellClick` events
fn cell_click(
    camera_query: Query<&Transform, With<MainCamera>>,
    mouse_button_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
    windows: Res<Windows>,
    cell_index: Res<CellIndex>,
    mut cell_click_events: EventWriter<CellClick>,
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        // Our game only has one window
        let window = windows.get_primary().unwrap();
        // These coordinates are in terms of the window's coordinates
        // and must be converted to the world coordinates used by our cell
        let mut cursor_position = window.cursor_position().unwrap();
        // QUALITY: use https://github.com/bevyengine/bevy/pull/1799 once merged instead
        let camera_transform = camera_query.single().unwrap();
        let window_size = Vec2::new(window.width() as f32, window.height() as f32);

        // World coordinates are measured from the center
        // while screen coordinates are measures from the bottom left.
        cursor_position -= 0.5 * window_size;

        // Apply the camera's transform to correct for scale, angle etc.
        // Returning a quaternion
        let world_quat =
            camera_transform.compute_matrix() * cursor_position.extend(0.0).extend(1.0);

        let cursor_position_world = Vec2::new(world_quat.x, world_quat.y);

        // Use the CellIndex resource to map the mouse position to a particular cell
        let selected_cell = cell_index.get(cursor_position_world);

        // Send a multi select event when Shift or Control is held
        let multi = keyboard_input.pressed(KeyCode::LShift)
            || keyboard_input.pressed(KeyCode::RShift)
            || keyboard_input.pressed(KeyCode::LControl)
            || keyboard_input.pressed(KeyCode::RControl);

        // Send a drag event when the mouse was not just pressed
        let drag = !mouse_button_input.just_pressed(MouseButton::Left);

        cell_click_events.send(CellClick {
            selected_cell,
            multi,
            drag,
        })
    }
}

/// Selects cells based on the clicks received
fn handle_clicks(
    mut cell_click_events: EventReader<CellClick>,
    cell_query: Query<(Entity, Option<&Selected>, &Value), With<Cell>>,
    mut commands: Commands,
) {
    // Usually there's just going to be one of these per frame
    // But we may as well loop through all just in case
    for click_event in cell_click_events.iter() {
        // If the user clicks outside of the grid, unselect everything
        if click_event.selected_cell.is_none() {
            for (entity, _, _) in cell_query.iter() {
                commands.entity(entity).remove::<Selected>();
            }
        // A grid cell was clicked
        } else {
            let entity = click_event.selected_cell.unwrap();
            // A drag click was used
            if click_event.drag {
                // Select cells clicked
                commands.entity(entity).insert(Selected);
            // A non-drag click was used
            } else {
                let (_, maybe_selected, current_value) = cell_query.get(entity).unwrap();

                // Shift or control was held
                if click_event.multi {
                    match maybe_selected {
                        // Select cells that aren't selected
                        None => commands.entity(entity).insert(Selected),
                        // Unselect cells that were already selected
                        Some(_) => commands.entity(entity).remove::<Selected>(),
                    };
                // A single, instant click was used
                } else {
                    // Count the number of currently selected cells
                    let n_selected = cell_query
                        .iter()
                        .filter(|(_, maybe_selected, _)| maybe_selected.is_some())
                        .count();

                    // Clear all selections other than those made due to this click
                    for (entity, _, _) in cell_query.iter() {
                        commands.entity(entity).remove::<Selected>();
                    }

                    // On a double click, select all tiles with a matching number
                    if maybe_selected.is_some() && n_selected <= 1 {
                        for (entity, _, value) in cell_query.iter() {
                            if *value == *current_value {
                                commands.entity(entity).insert(Selected);
                            }
                        }
                    // Normally, select just the cell clicked on
                    } else {
                        commands.entity(entity).insert(Selected);
                    }
                }
            }
        }
    }
}

/// Clears all selected cells when Backspace or Delete is pressed
fn erase_selected_cells(
    mut query: Query<(&mut Value, &Fixed), With<Selected>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Delete) || keyboard_input.just_pressed(KeyCode::Back) {
        for (mut value, is_fixed) in query.iter_mut() {
            if !is_fixed.0 {
                *value = Value::Empty;
            }
        }
    }
}

/// Selects all cells when Ctrl + A is pressed
fn select_all(
    query: Query<Entity, With<Cell>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
) {
    let ctrl =
        keyboard_input.pressed(KeyCode::LControl) || keyboard_input.pressed(KeyCode::RControl);

    if ctrl && keyboard_input.just_pressed(KeyCode::A) {
        for entity in query.iter() {
            commands.entity(entity).insert(Selected);
        }
    }
}

/// Set the background color of selected cells
fn color_selected(
    mut query: Query<(Option<&Selected>, &mut Handle<ColorMaterial>), With<Cell>>,
    background_color: Res<BackgroundColor>,
    selection_color: Res<SelectionColor>,
) {
    // QUALITY: use Added and Removed queries to avoid excessive spinning
    // once https://github.com/bevyengine/bevy/issues/2148 is fixed
    for (maybe_selected, mut material_handle) in query.iter_mut() {
        match maybe_selected {
            Some(_) => *material_handle = selection_color.0.clone(),
            None => *material_handle = background_color.0.clone(),
        }
    }
}

/// Events that change the value stored in a cell
#[derive(Clone)]
pub struct CellInput {
    pub value: u8,
}

/// Contains keybindings for converting key presses into numbers
struct CellInputMap {
    map: HashMap<KeyCode, u8>,
}

impl CellInputMap {
    fn insert(&mut self, k: KeyCode, v: u8) {
        self.map.insert(k, v);
    }

    fn get(&self, k: &KeyCode) -> Option<&u8> {
        self.map.get(k)
    }
}

impl Default for CellInputMap {
    fn default() -> Self {
        use KeyCode::*;

        let mut input_map = CellInputMap {
            map: HashMap::default(),
        };

        // Numbers above the letters
        input_map.insert(Key1, 1);
        input_map.insert(Key2, 2);
        input_map.insert(Key3, 3);
        input_map.insert(Key4, 4);
        input_map.insert(Key5, 5);
        input_map.insert(Key6, 6);
        input_map.insert(Key7, 7);
        input_map.insert(Key8, 8);
        input_map.insert(Key9, 9);

        // Numpad
        input_map.insert(Numpad1, 1);
        input_map.insert(Numpad2, 2);
        input_map.insert(Numpad3, 3);
        input_map.insert(Numpad4, 4);
        input_map.insert(Numpad5, 5);
        input_map.insert(Numpad6, 6);
        input_map.insert(Numpad7, 7);
        input_map.insert(Numpad8, 8);
        input_map.insert(Numpad9, 9);

        input_map
    }
}

/// Send `CellInput` events based on keyboard input
fn cell_keyboard_input(
    keyboard_input: Res<Input<KeyCode>>,
    input_map: Res<CellInputMap>,
    mut event_writer: EventWriter<CellInput>,
) {
    for key_code in keyboard_input.get_just_pressed() {
        let maybe_value = input_map.get(key_code);

        if let Some(value) = maybe_value {
            event_writer.send(CellInput { value: *value });
        }
    }
}

/// Set the value of the selected cells when 1-9 is pressed
fn set_cell_value(
    mut query: Query<(&mut Value, &Fixed), With<Selected>>,
    mut event_reader: EventReader<CellInput>,
) {
    use Value::*;
    for event in event_reader.iter() {
        for (mut value, is_fixed) in query.iter_mut() {
            // Don't change the values of cells given by the puzzle
            if is_fixed.0 {
                break;
            }

            // Grab the value from the event that was sent
            let new_value = event.value;

            *value = match *value {
                // Fill blank values with the key pressed
                Empty => Filled(new_value),
                // Overwrite markings with the key pressed
                Marked(_, _) => Filled(new_value),
                Filled(old_value) => {
                    // Remove existing values if they match
                    if old_value == new_value {
                        Empty
                    } else {
                        // Otherwise overwrite them
                        Filled(new_value)
                    }
                }
            };
        }
    }
}

fn update_cell_numbers(
    cell_query: Query<(&Value, &Relation<DisplayedBy>), (With<Cell>, Changed<Value>)>,
    mut num_query: Query<&mut Text>,
) {
    use Value::*;
    for (cell_value, displayed_by) in cell_query.iter() {
        for (num_entity, _) in displayed_by {
            let mut text = num_query.get_mut(num_entity).unwrap();

            // There is only one section in our text
            text.sections[0].value = match *cell_value {
                Filled(n) => n.to_string(),
                // TODO: properly display markings
                Marked(_, _) => "".to_string(),
                Empty => "".to_string(),
            }
        }
    }
}

mod cell_indexing {
    use super::*;
    /// An index that allows us to look up the entity at the correct position
    #[derive(Default)]
    pub struct CellIndex {
        pub cell_map: HashMap<Entity, BoundingBox>,
    }

    /// The axis-aligned rectangle that contains our cells
    pub struct BoundingBox {
        pub bottom_left: Vec2,
        pub top_right: Vec2,
    }

    impl CellIndex {
        pub fn get(&self, position: Vec2) -> Option<Entity> {
            // This is a slow and naive linear-time approach to spatial indexing
            // But it works fine for 81 items!
            for (entity, bounding_box) in self.cell_map.iter() {
                // Checks if the position is in the bounding box on both x and y
                let in_bounds = position.cmpge(bounding_box.bottom_left)
                    & position.cmple(bounding_box.top_right);
                // Only returns true if it's inside the box on both x and y
                if in_bounds.all() {
                    // This early return of a single item only works correctly
                    // because we know our entitities never overlap
                    // We would need a way to break ties otherwise
                    return Some(*entity);
                }
            }
            // Return None if no matches found
            None
        }
    }

    /// Builds a `CellIndex` for cells whose `Transform` has been changed
    pub fn index_cells(
        query: Query<(Entity, &Sprite, &Transform), (With<Cell>, Changed<Transform>)>,
        mut cell_index: ResMut<CellIndex>,
    ) {
        // Our Changed<Transform> filter ensures that this system only does work
        // on entities whose Transforms were added or mutated since the last time
        // this system ran
        for (entity, sprite, transform) in query.iter() {
            let center = transform.translation.truncate();
            let bottom_left = center - sprite.size / 2.0;
            let top_right = center + sprite.size / 2.0;

            // .insert overwrites existing values
            cell_index.cell_map.insert(
                entity,
                BoundingBox {
                    bottom_left,
                    top_right,
                },
            );
        }
    }
}

/// Different ways to enter a number into a cell
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum InputMode {
    /// The value of the cell
    Fill,
    /// One possible value of the cell
    CenterMark,
    /// This value must be within one of these cells in the box
    CornerMark,
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Fill
    }
}

/// Swaps the input mode based on keyboard input
fn swap_input_mode(keyboard_input: Res<Input<KeyCode>>, mut input_mode: ResMut<InputMode>) {
    if keyboard_input.just_pressed(KeyCode::Q) {
        *input_mode = InputMode::Fill;
    } else if keyboard_input.just_pressed(KeyCode::W) {
        *input_mode = InputMode::CenterMark;
    } else if keyboard_input.just_pressed(KeyCode::E) {
        *input_mode = InputMode::CornerMark;
    }
}
