use crate::{
    input::{
        board::CellClick,
        input_mode::{update_value_center, update_value_corner, update_value_fill, InputMode},
        CellInput, Selected,
    },
    CommonLabels,
};

/// Core data structures and logic for the Sudoku game board
use self::marks::{CenterMarks, CornerMarks};
use bevy::prelude::*;

pub struct LogicPlugin;

impl Plugin for LogicPlugin {
    fn build(&self, app: &mut AppBuilder) {
        // ACTION HANDLING
        app.add_system_set(
            SystemSet::new()
                .label(CommonLabels::Action)
                .after(CommonLabels::Input)
                .with_system(handle_clicks.system())
                .with_system(set_cell_value.system()),
        );
    }
}

pub struct Cell;
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Coordinates {
    /// Between 1 and 9, counted from top to bottom
    pub row: u8,
    /// Between 1 and 9, counted from left to right
    pub column: u8,
    /// Squares are counted from 1 to 9 starting at the top left,
    /// in standard left-to-right reading order
    ///
    /// The standard term for the 3x3 box a cell is in is `box`,
    /// but that's a reserved word in Rust
    pub square: u8,
}

impl Coordinates {
    /// Computes which 3x3 square a cell is in based on its row and column
    pub fn compute_square(row: u8, column: u8) -> u8 {
        const WIDTH: u8 = 3;
        let major_row = (row - 1) / WIDTH;
        let major_col = (column - 1) / WIDTH;

        major_col + major_row * WIDTH + 1
    }
}

/// The number(s) marked inside of each cell
#[derive(PartialEq, Eq, Clone)]
pub enum Value {
    /// No value is filled in this cell
    Empty,
    /// A single value is known to be in this cell
    Filled(u8),
    /// We have partial information about the state of this cell
    Marked(CenterMarks, CornerMarks),
}

impl Value {
    /// Converts empty marks into an empty cell state
    fn cleanup(&mut self) -> Value {
        let empty_marks = Value::Marked(CenterMarks::default(), CornerMarks::default());

        if *self == empty_marks {
            return Value::Empty;
        } else {
            return self.clone();
        }
    }
}

/// A component that specifies whether digits were provided by the puzzle
pub struct Fixed(pub bool);

pub mod marks {
    use bevy::utils::HashSet;
    /// Marks are notes about the possible value of a cell
    pub trait Marks: PartialEq + Eq + Clone {
        /// Creates a new object with only the value entered as its contents
        fn new(num: u8) -> Self;

        /// Updates the value of the marks given a new input
        fn update(&self, num: u8) -> Self;
    }
    /// The value of this cell could be any of the possibilities written in the center of the cell
    #[derive(PartialEq, Eq, Clone, Default)]
    pub struct CenterMarks(HashSet<u8>);

    impl Marks for CenterMarks {
        fn new(num: u8) -> CenterMarks {
            let mut marks = CenterMarks::default();
            marks.0.insert(num);
            marks
        }

        fn update(&self, num: u8) -> CenterMarks {
            let mut out = self.clone();
            if self.0.contains(&num) {
                out.0.remove(&num);
            } else {
                out.0.insert(num);
            }
            out
        }
    }

    impl ToString for CenterMarks {
        fn to_string(&self) -> String {
            let mut vec: Vec<_> = self.0.iter().collect();
            // We want to return the numbers in order, but our storage type is unordered
            vec.sort();
            let maybe_string = vec.iter().map(|m| m.to_string()).reduce(|a, b| a + &b);
            match maybe_string {
                Some(string) => string,
                None => "".to_string(),
            }
        }
    }

    /// The values marked in the corner of this cell must occur in these cells within the square
    #[derive(PartialEq, Eq, Clone, Default)]
    pub struct CornerMarks(HashSet<u8>);

    impl Marks for CornerMarks {
        fn new(num: u8) -> CornerMarks {
            let mut marks = CornerMarks::default();
            marks.0.insert(num);
            marks
        }

        fn update(&self, num: u8) -> CornerMarks {
            let mut out = self.clone();
            if self.0.contains(&num) {
                out.0.remove(&num);
            } else {
                out.0.insert(num);
            }
            out
        }
    }

    impl ToString for CornerMarks {
        fn to_string(&self) -> String {
            let mut vec: Vec<_> = self.0.iter().collect();
            // We want to return the numbers in order, but our storage type is unordered
            vec.sort();
            let maybe_string = vec.iter().map(|m| m.to_string()).reduce(|a, b| a + &b);
            match maybe_string {
                Some(string) => string,
                None => "".to_string(),
            }
        }
    }
}

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
                CenterMark => update_value_center(&*old_value, event.num).cleanup(),
                CornerMark => update_value_corner(&*old_value, event.num).cleanup(),
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
