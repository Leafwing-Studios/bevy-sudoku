use bevy::{input::system::exit_on_esc_system, prelude::*};

fn main() {
    App::build()
        .insert_resource(ClearColor(interaction::BACKGROUND_COLOR))
        .add_plugins(DefaultPlugins)
        // Must occur after DefaultPlugins, but before our stage is used
        // Implicitly inserts a startup stage after the default CoreStage::Startup
        .add_startup_stage(SudokuStage::PostStartup, SystemStage::parallel())
        .add_plugin(setup::SetupPlugin)
        .add_plugin(interaction::InteractionPlugin)
        .add_plugin(sudoku_generation::GenerationPlugin)
        .add_system(exit_on_esc_system.system())
        .run();
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
enum SudokuStage {
    PostStartup,
}

pub struct Cell;
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct Coordinates {
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
struct Value(Option<u8>);

// Marker relation to designate that the Value on the source entity (the Cell entity)
// is displayed by the target entity (the Text2d entity in the same location)
pub struct DisplayedBy;

/// A component that specifies whether digits were provided by the puzzle
struct Fixed(bool);

mod setup {
    use super::*;

    pub const CELL_SIZE: f32 = 50.0;
    pub const GRID_SIZE: f32 = 9.0 * CELL_SIZE;
    pub const MINOR_LINE_THICKNESS: f32 = 2.0;
    pub const MAJOR_LINE_THICKNESS: f32 = 4.0;
    // Defines the center lines of the grid in absolute coordinates
    // (0, 0) is in the center of the screen in Bevy
    pub const GRID_CENTER_X: f32 = 0.0;
    pub const GRID_LEFT_EDGE: f32 = GRID_CENTER_X - 0.5 * GRID_SIZE;
    pub const GRID_CENTER_Y: f32 = 0.0;
    pub const GRID_BOT_EDGE: f32 = GRID_CENTER_Y - 0.5 * GRID_SIZE;
    pub const GRID_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);

    pub const NUMBER_COLOR: Color = Color::BLACK;

    pub struct SetupPlugin;

    impl Plugin for SetupPlugin {
        fn build(&self, app: &mut AppBuilder) {
            app.add_startup_system(spawn_camera.system())
                .add_startup_system(spawn_grid.system())
                .add_startup_system(spawn_cells.system())
                // Must occur in a new stage to ensure that the cells are initialized
                // as commands are not processed until the end of the stage
                .add_startup_system_to_stage(SudokuStage::PostStartup, spawn_cell_numbers.system());
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
        font_res: Res<sudoku_generation::FixedFont>,
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

mod interaction {
    use bevy::{render::camera::Camera, utils::HashMap};

    use super::*;
    use cell_indexing::{index_cells, CellIndex};
    pub struct InteractionPlugin;

    /// Marker component for selected cells
    #[derive(Debug)]
    pub struct Selected;

    /// Event to dispatch cell clicks
    struct CellClick {
        /// Some(entity) if a cell was clicked, otherwise None
        selected_cell: Option<Entity>,
        /// Was shift held down at the time the event was sent
        shift: bool,
    }

    // Various colors for our cells
    struct BackgroundColor(Handle<ColorMaterial>);
    pub const BACKGROUND_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);
    struct SelectionColor(Handle<ColorMaterial>);
    pub const SELECTION_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

    impl Plugin for InteractionPlugin {
        fn build(&self, app: &mut AppBuilder) {
            app.add_startup_system(cell_colors.system())
                .init_resource::<CellIndex>()
                .add_event::<CellClick>()
                // Should run before input to ensure mapping from position to cell is correct
                .add_system(index_cells.system().before("input"))
                // Various input systems
                .add_system(cell_click.system().label("input"))
                .add_system(select_all.system().label("input"))
                .add_system(set_cell_value.system().label("input"))
                .add_system(clear_selected.system().label("input"))
                // Should immediately run to process input events after
                .add_system(handle_clicks.system().label("actions").after("input"))
                // Should run after actions to avoid delays
                .add_system(color_selected.system().after("actions"))
                .add_system(update_cell_numbers.system().after("actions"));
        }
    }

    fn cell_colors(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
        commands.insert_resource(BackgroundColor(materials.add(BACKGROUND_COLOR.into())));
        commands.insert_resource(SelectionColor(materials.add(SELECTION_COLOR.into())));
    }

    fn cell_click(
        camera_query: Query<&Transform, With<Camera>>,
        mouse_button_input: Res<Input<MouseButton>>,
        keyboard_input: Res<Input<KeyCode>>,
        windows: Res<Windows>,
        cell_index: Res<CellIndex>,
        mut cell_click_events: EventWriter<CellClick>,
    ) {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            // Our game only has one window
            let window = windows.get_primary().unwrap();
            // These coordinates are in terms of the window's coordinates
            // and must be converted to the world coordinates used by our cell
            let mut cursor_position = window.cursor_position().unwrap();
            // QUALITY: use https://github.com/bevyengine/bevy/pull/1799 once merged instead
            let camera_transform = camera_query.single().unwrap();
            let window_size = Vec2::new(window.width() as f32, window.height() as f32);

            // World coordinates are measured from the center
            // while screen coordinates are measures from the bottom left.
            cursor_position -= 0.5 * window_size;

            // Apply the camera's transform to correct for scale, angle etc.
            // Returning a quaternion
            let world_quat =
                camera_transform.compute_matrix() * cursor_position.extend(0.0).extend(1.0);

            let cursor_position_world = Vec2::new(world_quat.x, world_quat.y);

            // Use the CellIndex resource to map the mouse position to a particular cell
            let selected_cell = cell_index.get(cursor_position_world);

            cell_click_events.send(CellClick {
                selected_cell,
                shift: keyboard_input.pressed(KeyCode::LShift)
                    || keyboard_input.pressed(KeyCode::RShift),
            })
        }
    }

    fn handle_clicks(
        mut cell_click_events: EventReader<CellClick>,
        cell_query: Query<(Entity, Option<&Selected>, &Value), With<Cell>>,
        mut commands: Commands,
    ) {
        // Usually there's just going to be one of these per frame
        // But we may as well loop through all just in case
        for click_event in cell_click_events.iter() {
            // Select multiple tiles when shift is held
            if click_event.shift {
                if let Some(entity) = click_event.selected_cell {
                    let (_, maybe_selected, _) = cell_query.get(entity).unwrap();
                    match maybe_selected {
                        // Select cells that aren't selected
                        None => commands.entity(entity).insert(Selected),
                        // Unselect cells that were already selected
                        Some(_) => commands.entity(entity).remove::<Selected>(),
                    };
                } else {
                    for (entity, _, _) in cell_query.iter() {
                        // If the user clicks outside of the grid, unselect everything
                        commands.entity(entity).remove::<Selected>();
                    }
                }
            } else {
                // Begin by deselecting everything
                for (entity, _, _) in cell_query.iter() {
                    commands.entity(entity).remove::<Selected>();
                }

                // Only select one tile at once normally
                if let Some(entity) = click_event.selected_cell {
                    let (_, maybe_selected, current_value) = cell_query.get(entity).unwrap();
                    let n_selected = cell_query
                        .iter()
                        .filter(|(_, maybe_selected, _)| maybe_selected.is_some())
                        .count();
                    // On a double click, select all tiles with a matching number
                    if maybe_selected.is_some() && n_selected <= 1 {
                        for (entity, _, value) in cell_query.iter() {
                            if *value == *current_value {
                                commands.entity(entity).insert(Selected);
                            }
                        }
                    // Normally, select just the cell clicked on
                    } else {
                        commands.entity(entity).insert(Selected);
                    }
                }
            }
        }
    }

    /// Clears all selected cells when Backspace or Delete is pressed
    fn clear_selected(
        mut query: Query<(&mut Value, &Fixed), With<Selected>>,
        keyboard_input: Res<Input<KeyCode>>,
    ) {
        if keyboard_input.just_pressed(KeyCode::Delete)
            || keyboard_input.just_pressed(KeyCode::Back)
        {
            for (mut value, is_fixed) in query.iter_mut() {
                if !is_fixed.0 {
                    *value = Value(None);
                }
            }
        }
    }

    /// Selects all cells when Ctrl + A is pressed
    fn select_all(
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

    /// Set the background color of selected cells
    fn color_selected(
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

    /// Set the value of the selected cells when 1-9 is pressed
    fn set_cell_value(
        mut query: Query<(&mut Value, &Fixed), With<Selected>>,
        keyboard_input: Res<Input<KeyCode>>,
    ) {
        for key_code in keyboard_input.get_just_pressed() {
            let key_u8 = *key_code as u8;

            // The u8 values of our key codes correspond to their digits + 1 when < 9
            if key_u8 < 9 {
                let new_value = key_u8 + 1;

                for (mut value, is_fixed) in query.iter_mut() {
                    // Don't change the values of cells given by the puzzle
                    if is_fixed.0 {
                        break;
                    }

                    *value = Value(match value.0 {
                        // Fill blank values with the key pressed
                        None => Some(new_value),
                        Some(old_value) => {
                            // Remove existing values if they match
                            if old_value == new_value {
                                None
                            } else {
                                // Otherwise overwrite them
                                Some(new_value)
                            }
                        }
                    });
                }
            }
        }
    }

    fn update_cell_numbers(
        cell_query: Query<(&Value, &Relation<DisplayedBy>), (With<Cell>, Changed<Value>)>,
        mut num_query: Query<&mut Text>,
    ) {
        for (cell_value, displayed_by) in cell_query.iter() {
            for (num_entity, _) in displayed_by {
                let mut text = num_query.get_mut(num_entity).unwrap();

                // There is only one section in our text
                text.sections[0].value = match cell_value.0 {
                    Some(n) => n.to_string(),
                    None => "".to_string(),
                }
            }
        }
    }

    mod cell_indexing {
        use super::*;
        #[derive(Default)]
        pub struct CellIndex {
            pub cell_map: HashMap<Entity, BoundingBox>,
        }

        pub struct BoundingBox {
            pub bottom_left: Vec2,
            pub top_right: Vec2,
        }

        impl CellIndex {
            pub fn get(&self, position: Vec2) -> Option<Entity> {
                // This is a slow and naive linear-time approach to spatial indexing
                // But it works fine for 81 items!
                for (entity, bounding_box) in self.cell_map.iter() {
                    // Checks if the position is in the bounding box on both x and y
                    let in_bounds = position.cmpge(bounding_box.bottom_left)
                        & position.cmple(bounding_box.top_right);
                    // Only returns true if it's inside the box on both x and y
                    if in_bounds.all() {
                        // This early return of a single item only works correctly
                        // because we know our entitities never overlap
                        // We would need a way to break ties otherwise
                        return Some(*entity);
                    }
                }
                // Return None if no matches found
                None
            }
        }

        pub fn index_cells(
            query: Query<(Entity, &Sprite, &Transform), (With<Cell>, Changed<Transform>)>,
            mut cell_index: ResMut<CellIndex>,
        ) {
            // Our Changed<Transform> filter ensures that this system only does work
            // on entities whose Transforms were added or mutated since the last time
            // this system ran
            for (entity, sprite, transform) in query.iter() {
                let center = transform.translation.truncate();
                let bottom_left = center - sprite.size / 2.0;
                let top_right = center + sprite.size / 2.0;

                // .insert overwrites existing values
                cell_index.cell_map.insert(
                    entity,
                    BoundingBox {
                        bottom_left,
                        top_right,
                    },
                );
            }
        }
    }
}

mod sudoku_generation {
    use super::*;
    use bevy::utils::HashMap;
    use sudoku::Sudoku;

    pub const FIXED_NUM_FONT: &str = "fonts/FiraSans-Bold.ttf";
    pub const FILLABLE_NUM_FONT: &str = "fonts/FiraMono-Medium.ttf";

    pub struct GenerationPlugin;

    impl Plugin for GenerationPlugin {
        fn build(&self, app: &mut AppBuilder) {
            app.add_startup_system(load_fonts.system())
                .add_startup_system(generate_sudoku.system())
                .add_startup_system_to_stage(SudokuStage::PostStartup, fill_puzzle.system())
                .add_system(style_numbers.system())
                .add_system(cheat_at_sudoku.system());
        }
    }

    /// The clues and constraints given by the puzzle
    #[derive(Debug)]
    struct InitialPuzzle {
        numbers: HashMap<Coordinates, Value>,
    }
    /// The true solution to the puzzle
    struct CompletePuzzle {
        numbers: HashMap<Coordinates, Value>,
    }

    fn parse_sudoku(sudoku: Sudoku) -> HashMap<Coordinates, Value> {
        let (mut row, mut column) = (1, 0);
        let mut map = HashMap::default();

        // Sudoku::iter() goes from left to right, top to bottom
        for val in sudoku.iter() {
            column += 1;
            if column == 10 {
                row += 1;
                column = 1;
            }
            let square = Coordinates::compute_square(row, column);

            let coordinates = Coordinates {
                row,
                column,
                square,
            };
            let value = Value(val);
            map.insert(coordinates, value);
        }
        map
    }

    /// Creates a new sudoku using the `sudoku` crate
    fn generate_sudoku(mut commands: Commands) {
        let completed = Sudoku::generate_filled();
        // Puzzles are generated by removing clues
        let initial = Sudoku::generate_unique_from(completed);

        commands.insert_resource(InitialPuzzle {
            numbers: parse_sudoku(initial),
        });
        commands.insert_resource(CompletePuzzle {
            numbers: parse_sudoku(completed),
        });
    }

    pub struct FixedFont(pub Handle<Font>);
    pub struct FillableFont(pub Handle<Font>);
    fn load_fonts(mut commands: Commands, asset_server: ResMut<AssetServer>) {
        let fixed_handle = asset_server.load(FIXED_NUM_FONT);
        commands.insert_resource(FixedFont(fixed_handle));
        let fillable_handle = asset_server.load(FILLABLE_NUM_FONT);
        commands.insert_resource(FillableFont(fillable_handle));
    }

    fn fill_puzzle(
        initial_puzzle: Res<InitialPuzzle>,
        mut query: Query<(&Coordinates, &mut Value, &mut Fixed), With<Cell>>,
    ) {
        for (coordinates, mut value, mut is_fixed) in query.iter_mut() {
            let initial_value = initial_puzzle.numbers.get(coordinates).unwrap();

            // Fill in cells from initial puzzle and mark those cells as fixed
            if initial_value.0.is_some() {
                *value = *initial_value;
                is_fixed.0 = true;
            }
        }
    }

    fn style_numbers(
        cell_query: Query<(&Fixed, &Relation<DisplayedBy>), Changed<Fixed>>,
        mut text_query: Query<&mut Text>,
        fixed_font_res: Res<FixedFont>,
        fillable_font_res: Res<FillableFont>,
    ) {
        for (is_fixed, displayed_by) in cell_query.iter() {
            for (text_entity, _) in displayed_by {
                let mut text = text_query.get_mut(text_entity).unwrap();
                text.sections[0].style.font = match is_fixed.0 {
                    true => fixed_font_res.0.clone(),
                    false => fillable_font_res.0.clone(),
                }
            }
        }
    }

    /// Cheats and fills in the solution for you when space is pressed
    fn cheat_at_sudoku(
        complete_puzzle: Res<CompletePuzzle>,
        mut query: Query<(&Coordinates, &mut Value), With<Cell>>,
        keyboard_input: Res<Input<KeyCode>>,
    ) {
        if keyboard_input.just_pressed(KeyCode::Space) {
            for (coordinates, mut value) in query.iter_mut() {
                let correct_value = complete_puzzle.numbers.get(coordinates).unwrap();

                // Fill in cells from initial puzzle and mark those cells as fixed
                *value = *correct_value;
            }
        }
    }
}
