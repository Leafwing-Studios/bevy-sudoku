/// Misc logic-divorced shared utilities
use bevy::prelude::*;

/// Custom stages for our game
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum SudokuStage {
    PostStartup,
}
