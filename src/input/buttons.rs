use bevy::{ecs::component::Component, prelude::*};

use super::input_mode::InputMode;

/// Marker component for NewPuzzle button
#[derive(Default, Clone)]
pub struct NewPuzzle;
/// Marker component for ResetPuzzle button
#[derive(Default, Clone)]
pub struct ResetPuzzle;
/// Marker component for SolvePuzzle button
#[derive(Default, Clone)]
pub struct SolvePuzzle;

/// Sends the event type associated with the button when pressed
/// using the data stored on the component of that type
pub fn puzzle_button<Marker: Component + Clone>(
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
pub fn input_mode_buttons(
    button_query: Query<(&Interaction, &InputMode), Changed<Interaction>>,
    mut input_mode: ResMut<InputMode>,
) {
    for (interaction, button_input_mode) in button_query.iter() {
        if *interaction == Interaction::Clicked {
            *input_mode = *button_input_mode;
        }
    }
}
