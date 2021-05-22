use bevy::prelude::*;

use crate::CommonLabels;

pub mod actions;
// These are low-level, and shouldn't need to be exposed
mod keyboard;
mod mouse;

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<mouse::CellClick>()
            .add_event::<CellInput>()
            .init_resource::<keyboard::cell_input::CellInputMap>()
            .init_resource::<mouse::cell_index::CellIndex>()
            .init_resource::<actions::InputMode>()
            // Should run before input to ensure mapping from position to cell is correct
            .add_system(
                mouse::cell_index::index_cells
                    .system()
                    .before(CommonLabels::Input),
            )
            // INPUT HANDLING
            .add_system_set(
                SystemSet::new()
                    .label(CommonLabels::Input)
                    .with_system(mouse::cell_click.system())
                    .with_system(keyboard::select_all.system())
                    .with_system(keyboard::cell_input::cell_keyboard_input.system())
                    .with_system(keyboard::erase_selected_cells.system())
                    .with_system(keyboard::swap_input_mode.system()),
            )
            // ACTION HANDLING
            .add_system_set(
                SystemSet::new()
                    .label(CommonLabels::Action)
                    .after(CommonLabels::Input)
                    .with_system(actions::handle_clicks.system())
                    .with_system(actions::set_cell_value.system()),
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
