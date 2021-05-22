/// Process the action events created via player inputs
use crate::logic::board::{
    marks::{CenterMarks, CornerMarks, Marks},
    Cell, Fixed, Value,
};
use bevy::prelude::*;

use super::{board::CellClick, CellInput, Selected};

/// Set the value of the selected cells from cell input events
pub fn set_cell_value(
    mut query: Query<(&mut Value, &Fixed), With<Selected>>,
    input_mode: Res<InputMode>,
    mut event_reader: EventReader<CellInput>,
) {
    use InputMode::*;
    // FIXME: match on event's input type to control behavior
    // Existing logic is for Fill only
    for event in event_reader.iter() {
        for (mut old_value, is_fixed) in query.iter_mut() {
            // Don't change the values of cells given by the puzzle
            if is_fixed.0 {
                break;
            }

            // The behavior of setting the cell's value varies based on which input mode we're in
            *old_value = match *input_mode {
                // Set the cell's value based on the event's contents
                Fill => update_value_fill(&*old_value, event.num),
                CenterMark => update_value_center(&*old_value, event.num),
                CornerMark => update_value_corner(&*old_value, event.num),
            }
        }
    }
}

/// Selects cells based on the clicks received
pub fn handle_clicks(
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
            let entity = click_event
                .selected_cell
                .expect("Click event has no associated entity!");
            // A drag click was used
            if click_event.drag {
                // Select cells clicked
                commands.entity(entity).insert(Selected);
            // A non-drag click was used
            } else {
                let (_, maybe_selected, current_value) = cell_query.get(entity).expect(
                    "cell_query contains no entity matching the entity in this click_event",
                );

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

// QUALITY: refactor these to properly use a trait
pub fn update_value_fill(old_value: &Value, new_num: u8) -> Value {
    match old_value.clone() {
        // Fill blank values with the key pressed
        Value::Empty => Value::Filled(new_num),
        // Overwrite markings with new value
        Value::Marked(_, _) => Value::Filled(new_num),
        Value::Filled(old_value) => {
            // Remove existing values if they match
            if old_value == new_num {
                Value::Empty
            } else {
                // Otherwise overwrite them
                Value::Filled(new_num)
            }
        }
    }
}

pub fn update_value_center(old_value: &Value, num: u8) -> Value {
    match old_value.clone() {
        // Fill blank values with a center mark
        Value::Empty => Value::Marked(CenterMarks::new(num), CornerMarks::default()),
        // Update center marks with new value, adding it if it doesn't exist and removing it if it does
        Value::Marked(center, corner) => Value::Marked(center.update(num), corner),
        // Overwrite blank values with a center mark
        Value::Filled(_) => Value::Marked(CenterMarks::new(num), CornerMarks::default()),
    }
}

pub fn update_value_corner(old_value: &Value, num: u8) -> Value {
    match old_value.clone() {
        // Fill blank values with a corner mark
        Value::Empty => Value::Marked(CenterMarks::default(), CornerMarks::new(num)),
        // Update corner marks with new value, adding it if it doesn't exist and removing it if it does
        Value::Marked(center, corner) => Value::Marked(center, corner.update(num)),
        // Overwrite blank values with a center mark
        Value::Filled(_) => Value::Marked(CenterMarks::default(), CornerMarks::new(num)),
    }
}
