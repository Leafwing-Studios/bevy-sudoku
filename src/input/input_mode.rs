/// Process the action events created via player inputs
use crate::logic::board::{
    marks::{CenterMarks, CornerMarks, Marks},
    Value,
};

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
