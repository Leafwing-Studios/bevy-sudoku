/// Misc logic-divorced shared utilities
use bevy::prelude::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum SudokuStage {
    PostStartup,
}
