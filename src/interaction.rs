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
struct BackgroundColor(Handle<ColorMaterial>);
struct SelectionColor(Handle<ColorMaterial>);

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(cell_colors.system())
            .init_resource::<CellIndex>()
            .add_event::<CellClick>()
            .add_event::<CellInput>()
            // Should run before input to ensure mapping from position to cell is correct
            .add_system(index_cells.system().before("input"))
            // Various input systems
            .add_system(cell_click.system().label("input"))
            .add_system(select_all.system().label("input"))
            .add_system(cell_keyboard_input.system().label("input"))
            .add_system(erase_selected_cells.system().label("input"))
            // Should immediately run to process input events after
            .add_system(handle_clicks.system().label("actions").after("input"))
            .add_system(set_cell_value.system().label("actions").after("input"))
            // Should run after actions to avoid delays
            .add_system(color_selected.system().after("actions"))
            .add_system(update_cell_numbers.system().after("actions"));
    }
}

fn cell_colors(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(BackgroundColor(materials.add(BACKGROUND_COLOR.into())));
    commands.insert_resource(SelectionColor(materials.add(SELECTION_COLOR.into())));
}

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
                *value = Value(None);
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

struct CellInput {
    value: u8,
}

fn cell_keyboard_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut event_writer: EventWriter<CellInput>,
) {
    for key_code in keyboard_input.get_just_pressed() {
        let key_u8 = *key_code as u8;

        // The u8 values of our key codes correspond to their digits + 1 when < 9
        if key_u8 < 9 {
            let value = key_u8 + 1;
            event_writer.send(CellInput { value });
        }
    }
}

/// Set the value of the selected cells when 1-9 is pressed
fn set_cell_value(
    mut query: Query<(&mut Value, &Fixed), With<Selected>>,
    mut event_reader: EventReader<CellInput>,
) {
    for event in event_reader.iter() {
        for (mut value, is_fixed) in query.iter_mut() {
            // Don't change the values of cells given by the puzzle
            if is_fixed.0 {
                break;
            }

            // Grab the value from the event that was sent
            let new_value = event.value;

            *value = Value(match value.0 {
                // Fill blank values with the key pressed
                None => Some(new_value),
                Some(old_value) => {
                    // Remove existing values if they match
                    if old_value == new_value {
                        None
                    } else {
                        // Otherwise overwrite them
                        Some(new_value)
                    }
                }
            });
        }
    }
}

fn update_cell_numbers(
    cell_query: Query<(&Value, &Relation<DisplayedBy>), (With<Cell>, Changed<Value>)>,
    mut num_query: Query<&mut Text>,
) {
    for (cell_value, displayed_by) in cell_query.iter() {
        for (num_entity, _) in displayed_by {
            let mut text = num_query.get_mut(num_entity).unwrap();

            // There is only one section in our text
            text.sections[0].value = match cell_value.0 {
                Some(n) => n.to_string(),
                None => "".to_string(),
            }
        }
    }
}

mod cell_indexing {
    use super::*;
    #[derive(Default)]
    pub struct CellIndex {
        pub cell_map: HashMap<Entity, BoundingBox>,
    }

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
