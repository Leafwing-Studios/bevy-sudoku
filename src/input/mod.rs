use bevy::prelude::*;

use crate::CommonLabels;

pub mod input_mode;
// These are low-level, and shouldn't need to be exposed
pub mod board;
pub mod buttons;
mod keyboard;

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // INPUT EVENTS
            .add_event::<buttons::NewPuzzle>()
            .add_event::<buttons::ResetPuzzle>()
            .add_event::<buttons::SolvePuzzle>()
            .add_event::<board::CellClick>()
            .add_event::<CellInput>()
            .init_resource::<keyboard::cell_input::CellInputMap>()
            .init_resource::<board::cell_index::CellIndex>()
            .init_resource::<input_mode::InputMode>()
            // Should run before input to ensure mapping from position to cell is correct
            .add_system(
                board::cell_index::index_cells
                    .system()
                    .before(CommonLabels::Input),
            )
            // INPUT HANDLING
            .add_system_set(
                SystemSet::new()
                    .label(CommonLabels::Input)
                    // BOARD
                    .with_system(board::cell_click.system())
                    // BUTTONS
                    .with_system(buttons::puzzle_button::<buttons::NewPuzzle>.system())
                    .with_system(buttons::puzzle_button::<buttons::ResetPuzzle>.system())
                    .with_system(buttons::puzzle_button::<buttons::SolvePuzzle>.system())
                    .with_system(buttons::puzzle_button::<CellInput>.system())
                    .with_system(buttons::input_mode_buttons.system())
                    // KEYBOARD
                    .with_system(keyboard::select_all.system())
                    .with_system(keyboard::cell_input::cell_keyboard_input.system())
                    .with_system(keyboard::erase_selected_cells.system())
                    .with_system(keyboard::swap_input_mode.system()),
            );
    }
}

/// Marker component for selected cells
#[derive(Debug)]
pub struct Selected;

/// Events that change the value stored in a cell
#[derive(Clone)]
pub struct CellInput {
    pub num: u8,
}
