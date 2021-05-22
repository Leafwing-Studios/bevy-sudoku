/// Build and display the Sudoku board
use crate::{
    input::Selected,
    logic::board::{Cell, Coordinates, Fixed, Value},
    CommonLabels,
};
use bevy::prelude::*;

use self::assets::*;
use self::config::*;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            // ASSETS
            .init_resource::<FixedFont>()
            .init_resource::<FillableFont>()
            .init_resource::<BackgroundColor>()
            .init_resource::<SelectionColor>()
            // SETUP
            .add_startup_system(setup::spawn_board.system())
            // ACTION HANDLING
            .add_system_set(
                SystemSet::new()
                    .after(CommonLabels::Action)
                    .with_system(actions::color_selected.system())
                    .with_system(actions::update_cell_numbers.system())
                    .with_system(actions::style_numbers.system()),
            );
    }
}

mod config {
    use super::*;

    // Colors
    pub const SELECTION_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

    pub const GRID_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
    pub const NUMBER_COLOR: Color = Color::BLACK;

    // Fonts
    pub const FIXED_NUM_FONT: &str = "fonts/Ubuntu-Bold.ttf";
    pub const FILLABLE_NUM_FONT: &str = "fonts/Ubuntu-Light.ttf";

    // Sizes
    pub const CELL_SIZE: f32 = 50.0;
    pub const FONT_SIZE: f32 = 30.0;
    pub const GRID_SIZE: f32 = 9.0 * CELL_SIZE;
    pub const MINOR_LINE_THICKNESS: f32 = 2.0;
    pub const MAJOR_LINE_THICKNESS: f32 = 4.0;

    // Positions
    pub const NUM_OFFSET_X: f32 = 0.0 * CELL_SIZE;
    pub const NUM_OFFSET_Y: f32 = 0.5 * CELL_SIZE;
}

// QUALITY: reduce asset loading code duplication dramatically
pub mod assets {
    use crate::graphics::BACKGROUND_COLOR;

    use super::*;
    // Various colors for our cells
    /// The color of the game's background, and the default color of the cells
    pub struct BackgroundColor(pub Handle<ColorMaterial>);
    /// The color of cells when selected
    pub struct SelectionColor(pub Handle<ColorMaterial>);

    impl FromWorld for BackgroundColor {
        fn from_world(world: &mut World) -> Self {
            let mut materials = world
                .get_resource_mut::<Assets<ColorMaterial>>()
                .expect("ResMut<Assets<ColorMaterial>> not found.");
            BackgroundColor(materials.add(BACKGROUND_COLOR.into()))
        }
    }

    impl FromWorld for SelectionColor {
        fn from_world(world: &mut World) -> Self {
            let mut materials = world
                .get_resource_mut::<Assets<ColorMaterial>>()
                .expect("ResMut<Assets<ColorMaterial>> not found.");
            SelectionColor(materials.add(SELECTION_COLOR.into()))
        }
    }

    // Fonts used in our game
    pub struct FixedFont(pub Handle<Font>);

    impl FromWorld for FixedFont {
        fn from_world(world: &mut World) -> Self {
            let asset_server = world
                .get_resource_mut::<AssetServer>()
                .expect("ResMut<AssetServer> not found.");
            FixedFont(asset_server.load(FIXED_NUM_FONT))
        }
    }

    pub struct FillableFont(pub Handle<Font>);

    impl FromWorld for FillableFont {
        fn from_world(world: &mut World) -> Self {
            let asset_server = world
                .get_resource_mut::<AssetServer>()
                .expect("ResMut<AssetServer> not found.");
            FillableFont(asset_server.load(FILLABLE_NUM_FONT))
        }
    }
}

// FIXME: redo logic in UI
mod setup {
    use crate::graphics::{assets::NoneColor, layout::SudokuBox};

    use super::*;

    /// Marker component for our GridLine entities
    struct GridLine;
    /// Simple rectangular lines that form the Sudoku grid
    #[derive(Bundle)]
    struct GridLineBundle {
        gridline: GridLine,
        #[bundle]
        node_bundle: NodeBundle,
    }

    enum Orientation {
        Horizontal,
        Vertical,
    }

    impl GridLineBundle {
        fn new(orientation: Orientation, i: u8, material: Handle<ColorMaterial>) -> Self {
            // The grid lines that define the boxes need to be thicker
            let thickness = if (i % 3) == 0 {
                MAJOR_LINE_THICKNESS
            } else {
                MINOR_LINE_THICKNESS
            };

            let size = match orientation {
                Orientation::Horizontal => Size::new(Val::Px(GRID_SIZE), Val::Px(thickness)),
                Orientation::Vertical => Size::new(Val::Px(thickness), Val::Px(GRID_SIZE)),
            };

            GridLineBundle {
                gridline: GridLine,
                node_bundle: NodeBundle {
                    style: Style {
                        size,
                        ..Default::default()
                    },
                    material,
                    ..Default::default()
                },
            }
        }
    }

    /// Marker component for our Board entity that all the game board entities are a child of
    struct Board;

    /// Spawns our board
    // FIXME: add cells and cell numbers
    // FIXME: cell lines overextend when compressed
    pub fn spawn_board(
        mut commands: Commands,
        mut materials: ResMut<Assets<ColorMaterial>>,
        none_color: Res<NoneColor>,
        root_node_query: Query<Entity, With<SudokuBox>>,
        fixed_font: Res<FixedFont>,
    ) {
        let grid_material = materials.add(GRID_COLOR.into());
        let grid_size = Size::new(Val::Px(GRID_SIZE), Val::Px(GRID_SIZE));

        // Node that owns the left half of the screen
        let root_node = root_node_query.single().expect("Root node not found.");
        // Parent of all of our game board entities
        let grid_node = commands
            .spawn()
            .insert_bundle(NodeBundle {
                style: Style {
                    size: grid_size,
                    min_size: grid_size,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Board)
            .id();

        // The game board is a child of our SudokuBox node
        commands.entity(root_node).push_children(&[grid_node]);

        // Horizontal lines
        let horizontal_grid_node = commands
            .spawn()
            .insert_bundle(NodeBundle {
                style: Style {
                    size: grid_size,
                    // Lays out the grid lines on top of each other
                    flex_direction: FlexDirection::Column,
                    // Do not lay this out relative to siblings
                    position_type: PositionType::Absolute,
                    // Evenly space lines
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                material: none_color.0.clone(),
                ..Default::default()
            })
            .id();

        let mut horizontal_grid_lines = [Entity::new(0); 10];
        for row in 0..=9 {
            horizontal_grid_lines[row] = commands
                .spawn_bundle(GridLineBundle::new(
                    Orientation::Horizontal,
                    row as u8,
                    grid_material.clone(),
                ))
                .id();
        }

        // Vertical lines
        let vertical_grid_node = commands
            .spawn()
            .insert_bundle(NodeBundle {
                style: Style {
                    size: grid_size,
                    // Lays out the grid lines beside each other
                    flex_direction: FlexDirection::Row,
                    // Do not lay this out relative to siblings
                    position_type: PositionType::Absolute,
                    // Evenly space lines
                    justify_content: JustifyContent::SpaceBetween,
                    ..Default::default()
                },
                material: none_color.0.clone(),
                ..Default::default()
            })
            .id();

        let mut vertical_grid_lines = [Entity::new(0); 10];
        for column in 0..=9 {
            vertical_grid_lines[column] = commands
                .spawn_bundle(GridLineBundle::new(
                    Orientation::Vertical,
                    column as u8,
                    grid_material.clone(),
                ))
                .id();
        }

        // Cells
        let cells_node = commands
            .spawn()
            .insert_bundle(NodeBundle {
                style: Style {
                    size: grid_size,
                    // Wrap cells in a 9 x 9 pattern
                    // as the cell size * 9 == grid size
                    flex_wrap: FlexWrap::Wrap,
                    // Do not lay this out relative to siblings
                    position_type: PositionType::Absolute,
                    // Evenly space cells along the main (left-to-right) axis, with no gaps at the end
                    justify_content: JustifyContent::SpaceBetween,
                    // Evenly space cells along the cross (top-to-bottom) axis, with no gaps at the end
                    align_content: AlignContent::SpaceBetween,
                    ..Default::default()
                },
                material: none_color.0.clone(),
                ..Default::default()
            })
            .id();

        // Cell text
        let text_style = TextStyle {
            font: fixed_font.0.clone(),
            font_size: FONT_SIZE,
            color: NUMBER_COLOR,
        };

        const TEXT_ALIGNMENT: TextAlignment = TextAlignment {
            vertical: VerticalAlign::Center,
            horizontal: HorizontalAlign::Center,
        };

        // Spawning cells and cell texts
        let mut cells = [Entity::new(0); 9 * 9];
        let mut i = 0;
        for row in 1..=9 {
            for column in 1..=9 {
                // FIXME: color is not showing up
                let cell_material = materials.add(Color::RED.into());
                //let cell_material = none_color.0.clone();

                cells[i] = commands
                    .spawn_bundle(CellBundle::new(row, column, cell_material))
                    .with_children(|parent| {
                        parent
                            .spawn_bundle(TextBundle {
                                // This value begins empty, but then is later set in update_cell_numbers system
                                // to match the cell's `value` field
                                text: Text::with_section("3", text_style.clone(), TEXT_ALIGNMENT),
                                // FIXME: does nothing
                                // Tweaking for aesthetic perfection
                                transform: Transform::from_xyz(NUM_OFFSET_X, NUM_OFFSET_Y, 0.0),
                                style: Style {
                                    // Don't lay this out; instead just use the standard center-based inheritance
                                    position_type: PositionType::Absolute,
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(CellNumber);
                    })
                    .id();
                i += 1;
            }
        }

        // Building our hierarchy
        commands
            .entity(grid_node)
            // Must occur before grid nodes to ensure proper layering
            .push_children(&[cells_node])
            // We need several seperate nodes due to differing layout strategies
            .push_children(&[horizontal_grid_node])
            .push_children(&[vertical_grid_node]);

        commands
            .entity(horizontal_grid_node)
            .push_children(&horizontal_grid_lines);

        commands
            .entity(vertical_grid_node)
            .push_children(&vertical_grid_lines);

        commands.entity(cells_node).push_children(&cells);
    }

    #[derive(Bundle)]
    struct CellBundle {
        cell: Cell,
        coordinates: Coordinates,
        value: Value,
        fixed: Fixed,
        #[bundle]
        button: ButtonBundle,
    }

    impl CellBundle {
        fn new(row: u8, column: u8, material: Handle<ColorMaterial>) -> Self {
            CellBundle {
                cell: Cell,
                coordinates: Coordinates {
                    row,
                    column,
                    square: Coordinates::compute_square(row, column),
                },
                // No digits are filled in to begin with
                value: Value::Empty,
                fixed: Fixed(false),
                button: ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(CELL_SIZE), Val::Px(CELL_SIZE)),
                        // Controls left-right alignment of numbers
                        justify_content: JustifyContent::Center,
                        // FIXME: doesn't do anything
                        // Controls top-bottom alignment of numbers
                        align_content: AlignContent::Center,
                        ..Default::default()
                    },
                    material: material.clone(),
                    ..Default::default()
                },
            }
        }
    }

    /// Marker component for the visual representation of a cell's values
    pub struct CellNumber;

    // Marker relation to designate that the Value on the source entity (the Cell entity)
    // is displayed by the target entity (the Text2d entity in the same location)
    pub struct DisplayedBy;
}

mod actions {
    use super::setup::DisplayedBy;
    use super::*;

    /// Changes the cell displays to match their values
    pub fn update_cell_numbers(
        cell_query: Query<(&Value, &Relation<DisplayedBy>), (With<Cell>, Changed<Value>)>,
        mut num_query: Query<&mut Text>,
    ) {
        use Value::*;
        // FIXME: remove use of relations
        for (cell_value, displayed_by) in cell_query.iter() {
            for (num_entity, _) in displayed_by {
                let mut text = num_query
                    .get_mut(num_entity)
                    .expect("No corresponding entity found!");

                // There is only one section in our text
                text.sections[0].value = match cell_value.clone() {
                    Filled(n) => n.to_string(),
                    // TODO: properly display markings
                    Marked(center, corner) => {
                        format!("Center: {}", center.to_string())
                            + "|"
                            + &format!("Corner: {}", corner.to_string())
                    }
                    Empty => "".to_string(),
                }
            }
        }
    }

    /// Set the background color of selected cells
    pub fn color_selected(
        mut query: Query<(Option<&Selected>, &mut Handle<ColorMaterial>), With<Cell>>,
        background_color: Res<BackgroundColor>,
        selection_color: Res<SelectionColor>,
    ) {
        // QUALITY: use Added and Removed queries to avoid excessive spinning
        // once https://github.com/bevyengine/bevy/issues/2148 is fixed
        for (maybe_selected, mut material_handle) in query.iter_mut() {
            match maybe_selected {
                Some(_) => *material_handle = selection_color.0.clone(),
                None => *material_handle = background_color.0.clone(),
            }
        }
    }
    /// Sets the style of the numbers based on whether or not they're fixed
    pub fn style_numbers(
        cell_query: Query<(&Fixed, &Relation<DisplayedBy>), Changed<Fixed>>,
        mut text_query: Query<&mut Text>,
        fixed_font_res: Res<FixedFont>,
        fillable_font_res: Res<FillableFont>,
    ) {
        // FIXME: remove use of relations
        for (is_fixed, displayed_by) in cell_query.iter() {
            for (text_entity, _) in displayed_by {
                let mut text = text_query
                    .get_mut(text_entity)
                    .expect("Corresponding text entity not found.");
                text.sections[0].style.font = match is_fixed.0 {
                    true => fixed_font_res.0.clone(),
                    false => fillable_font_res.0.clone(),
                }
            }
        }
    }
}
