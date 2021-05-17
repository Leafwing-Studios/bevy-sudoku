/// Core data structures and setup logic for the Sudoku game board
use crate::aesthetics::{
    FixedFont, CELL_SIZE, GRID_BOT_EDGE, GRID_COLOR, GRID_LEFT_EDGE, GRID_SIZE,
    MAJOR_LINE_THICKNESS, MINOR_LINE_THICKNESS, NUMBER_COLOR,
};
use bevy::prelude::*;

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

/// The number marked inside of each cell
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Value(pub Option<u8>);

// Marker relation to designate that the Value on the source entity (the Cell entity)
// is displayed by the target entity (the Text2d entity in the same location)
pub struct DisplayedBy;

/// A component that specifies whether digits were provided by the puzzle
pub struct Fixed(pub bool);

pub mod setup {
    use super::*;
    pub struct SetupPlugin;

    impl Plugin for SetupPlugin {
        fn build(&self, app: &mut AppBuilder) {
            app.add_startup_system(spawn_camera.system())
                .add_startup_system(spawn_grid.system())
                .add_startup_system(spawn_cells.system())
                // Must occur in a new stage to ensure that the cells are initialized
                // as commands are not processed until the end of the stage
                .add_startup_system_to_stage(
                    crate::utils::SudokuStage::PostStartup,
                    spawn_cell_numbers.system(),
                );
        }
    }

    fn spawn_camera(mut commands: Commands) {
        commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    }

    fn spawn_grid(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
        let grid_handle = materials.add(GRID_COLOR.into());

        for row in 0..=9 {
            commands.spawn_bundle(new_gridline(
                Orientation::Horizontal,
                row,
                grid_handle.clone(),
            ));
        }

        for column in 0..=9 {
            commands.spawn_bundle(new_gridline(
                Orientation::Vertical,
                column,
                grid_handle.clone(),
            ));
        }
    }

    enum Orientation {
        Horizontal,
        Vertical,
    }

    fn new_gridline(
        orientation: Orientation,
        i: u8,
        grid_handle: Handle<ColorMaterial>,
    ) -> SpriteBundle {
        // The grid lines that define the boxes need to be thicker
        let thickness = if (i % 3) == 0 {
            MAJOR_LINE_THICKNESS
        } else {
            MINOR_LINE_THICKNESS
        };

        let length = GRID_SIZE + thickness;

        let size = match orientation {
            Orientation::Horizontal => Vec2::new(length, thickness),
            Orientation::Vertical => Vec2::new(thickness, length),
        };

        // Each objects' position is defined by its center
        let offset = i as f32 * CELL_SIZE;

        let (x, y) = match orientation {
            Orientation::Horizontal => (GRID_LEFT_EDGE + 0.5 * GRID_SIZE, GRID_BOT_EDGE + offset),
            Orientation::Vertical => (GRID_LEFT_EDGE + offset, GRID_BOT_EDGE + 0.5 * GRID_SIZE),
        };

        SpriteBundle {
            sprite: Sprite::new(size),
            // We want these grid lines to cover any cell that it might overlap with
            transform: Transform::from_xyz(x, y, 1.0),
            material: grid_handle,
            ..Default::default()
        }
    }

    fn spawn_cells(mut commands: Commands) {
        for row in 1..=9 {
            for column in 1..=9 {
                commands.spawn_bundle(CellBundle::new(row, column));
            }
        }
    }

    #[derive(Bundle)]
    struct CellBundle {
        cell: Cell,
        coordinates: Coordinates,
        value: Value,
        fixed: Fixed,
        #[bundle]
        cell_fill: SpriteBundle,
    }

    impl CellBundle {
        fn new(row: u8, column: u8) -> Self {
            let x = GRID_LEFT_EDGE + CELL_SIZE * row as f32 - 0.5 * CELL_SIZE;
            let y = GRID_BOT_EDGE + CELL_SIZE * column as f32 - 0.5 * CELL_SIZE;

            CellBundle {
                cell: Cell,
                coordinates: Coordinates {
                    row,
                    column,
                    square: Coordinates::compute_square(row, column),
                },
                // No digits are filled in to begin with
                value: Value(None),
                fixed: Fixed(false),
                cell_fill: SpriteBundle {
                    // The material for this sprite begins with the same material as our background
                    sprite: Sprite::new(Vec2::new(CELL_SIZE, CELL_SIZE)),
                    // We want this cell to be covered by any grid lines that it might overlap with
                    transform: Transform::from_xyz(x, y, 0.0),
                    ..Default::default()
                },
            }
        }
    }

    /// Marker component for the visual representation of a cell's values
    pub struct CellNumber;

    /// Adds a text number associated with each cell to display its value
    fn spawn_cell_numbers(
        query: Query<(Entity, &Transform), With<Cell>>,
        mut commands: Commands,
        font_res: Res<FixedFont>,
    ) {
        const TEXT_ALIGNMENT: TextAlignment = TextAlignment {
            vertical: VerticalAlign::Center,
            horizontal: HorizontalAlign::Center,
        };

        for (cell_entity, cell_transform) in query.iter() {
            // These numbers must be displayed on top of the cells they are in
            let mut number_transform = cell_transform.clone();
            number_transform.translation.z += 1.0;

            let text_style = TextStyle {
                font: font_res.0.clone(),
                font_size: 0.8 * CELL_SIZE,
                color: NUMBER_COLOR,
            };

            let text_entity = commands.spawn().id();

            commands
                .entity(text_entity)
                .insert_bundle(Text2dBundle {
                    // This value begins empty, but then is later set in update_cell_numbers system
                    // to match the cell's `value` field
                    text: Text::with_section("", text_style.clone(), TEXT_ALIGNMENT),
                    transform: number_transform,
                    ..Default::default()
                })
                .insert(CellNumber);

            commands
                .entity(cell_entity)
                .insert_relation(DisplayedBy, text_entity);
        }
    }
}
