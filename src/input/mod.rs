use bevy::prelude::*;

pub mod actions;
pub mod input_mode;
// These are low-level, and shouldn't need to be exposed
mod keyboard;
mod mouse;

pub struct InteractionPlugin;

// QUALITY: use typed system labels instead of strings
// QUALITY: use system sets and label them all at once
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<mouse::CellClick>()
            .add_event::<CellInput>()
            .init_resource::<keyboard::CellInputMap>()
            .init_resource::<mouse::CellIndex>()
            .init_resource::<input_mode::InputMode>()
            // Should run before input to ensure mapping from position to cell is correct
            .add_system(mouse::index_cells.system().before("input"))
            // Various input systems
            .add_system(mouse::cell_click.system().label("input"))
            .add_system(keyboard::select_all.system().label("input"))
            .add_system(keyboard::cell_keyboard_input.system().label("input"))
            .add_system(keyboard::erase_selected_cells.system().label("input"))
            .add_system(keyboard::swap_input_mode.system().label("input"))
            // Should immediately run to process input events after
            .add_system(
                actions::handle_clicks
                    .system()
                    .label("actions")
                    .after("input"),
            )
            .add_system(
                actions::set_cell_value
                    .system()
                    .label("actions")
                    .after("input"),
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
