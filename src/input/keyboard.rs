/// Handle player input from the keyboard, converting it into actions
use super::{input_mode::InputMode, CellInput, Selected};
use crate::logic::board::{Cell, Fixed, Value};
use bevy::prelude::*;

pub mod cell_input {
    use super::CellInput;
    use bevy::prelude::*;
    use bevy::utils::HashMap;
    /// Contains keybindings for converting key presses into numbers
    pub struct CellInputMap {
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
    pub fn cell_keyboard_input(
        keyboard_input: Res<Input<KeyCode>>,
        input_map: Res<CellInputMap>,
        mut event_writer: EventWriter<CellInput>,
    ) {
        for key_code in keyboard_input.get_just_pressed() {
            let maybe_value = input_map.get(key_code);

            if let Some(value) = maybe_value {
                event_writer.send(CellInput { num: *value });
            }
        }
    }
}

/// Clears all selected cells when Backspace or Delete is pressed
pub fn erase_selected_cells(
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
pub fn select_all(
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

/// Swaps the input mode based on keyboard input
pub fn swap_input_mode(keyboard_input: Res<Input<KeyCode>>, mut input_mode: ResMut<InputMode>) {
    if keyboard_input.just_pressed(KeyCode::Q) {
        *input_mode = InputMode::Fill;
    } else if keyboard_input.just_pressed(KeyCode::W) {
        *input_mode = InputMode::CenterMark;
    } else if keyboard_input.just_pressed(KeyCode::E) {
        *input_mode = InputMode::CornerMark;
    }
}
