use std::marker::PhantomData;

use bevy::{ecs::component::Component, prelude::*};
pub struct BoardButtonsPlugin;

impl Plugin for BoardButtonsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(spawn_buttons.system())
            .add_system(responsive_buttons.system())
            .add_event::<NewPuzzle>()
            .add_system(puzzle_button::<NewPuzzle>.system())
            .add_event::<ResetPuzzle>()
            .add_system(puzzle_button::<ResetPuzzle>.system())
            .add_event::<SolvePuzzle>()
            .add_system(puzzle_button::<SolvePuzzle>.system());
    }
}
/// Resource that contains the raw materials for each button type
/// correspsonding to the Marker type marker component
pub struct ButtonMaterials<Marker: Component + Default> {
    pub normal: Handle<ColorMaterial>,
    pub hovered: Handle<ColorMaterial>,
    pub pressed: Handle<ColorMaterial>,
    pub _marker: PhantomData<Marker>,
}

/// Component for the material of a button at rest
struct NormalMaterial(Handle<ColorMaterial>);
/// Component for the material of a button when hovered
struct HoveredMaterial(Handle<ColorMaterial>);
/// Component for the material of a button when pressed
struct PressedMaterial(Handle<ColorMaterial>);

#[derive(Bundle)]
struct BoardButtonBundle<Marker: Component + Default> {
    marker: Marker,
    #[bundle]
    button_bundle: ButtonBundle,
    normal_material: NormalMaterial,
    hovered_material: HoveredMaterial,
    pressed_material: PressedMaterial,
}

impl<Marker: Component + Default> BoardButtonBundle<Marker> {
    fn new(size: Size<Val>, materials: &ButtonMaterials<Marker>) -> Self {
        let normal_material = materials.normal.clone();
        let hovered_material = materials.hovered.clone();
        let pressed_material = materials.pressed.clone();

        BoardButtonBundle {
            marker: Marker::default(),
            button_bundle: ButtonBundle {
                style: Style {
                    size,
                    // Center button
                    margin: Rect::all(Val::Auto),
                    ..Default::default()
                },
                material: normal_material.clone(),
                ..Default::default()
            },
            normal_material: NormalMaterial(normal_material),
            hovered_material: HoveredMaterial(hovered_material),
            pressed_material: PressedMaterial(pressed_material),
        }
    }
}

#[derive(Default)]
pub struct NewPuzzle;
#[derive(Default)]
pub struct ResetPuzzle;
#[derive(Default)]
pub struct SolvePuzzle;

fn spawn_buttons(
    mut commands: Commands,
    new_button_materials: Res<ButtonMaterials<NewPuzzle>>,
    reset_button_materials: Res<ButtonMaterials<ResetPuzzle>>,
    solve_button_materials: Res<ButtonMaterials<SolvePuzzle>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let button_size = Size::new(Val::Px(100.0), Val::Px(100.0));

    // Side panel root node
    commands
        .spawn_bundle(NodeBundle {
            // FIXME: Set on right side of screen instead
            style: Style {
                size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..Default::default()
            },
            material: materials.add(Color::BLACK.into()),
            ..Default::default()
        })
        .with_children(|parent| {
            // New puzzle
            parent.spawn_bundle(BoardButtonBundle::<NewPuzzle>::new(
                button_size,
                &*new_button_materials,
            ));

            // Reset puzzle
            parent.spawn_bundle(BoardButtonBundle::<ResetPuzzle>::new(
                button_size,
                &*reset_button_materials,
            ));

            // Solve puzzle
            parent.spawn_bundle(BoardButtonBundle::<SolvePuzzle>::new(
                button_size,
                &*solve_button_materials,
            ));
        });
}

fn responsive_buttons(
    mut button_query: Query<
        (
            &Interaction,
            &mut Handle<ColorMaterial>,
            &NormalMaterial,
            &HoveredMaterial,
            &PressedMaterial,
        ),
        (With<Button>, Changed<Interaction>),
    >,
) {
    for (interaction, mut material, normal_material, hovered_material, pressed_material) in
        button_query.iter_mut()
    {
        *material = match *interaction {
            Interaction::None => normal_material.0.clone(),
            Interaction::Hovered => hovered_material.0.clone(),
            Interaction::Clicked => pressed_material.0.clone(),
        }
    }
}

fn puzzle_button<Marker: Component + Default>(
    query: Query<&Interaction, With<Marker>>,
    mut event_writer: EventWriter<Marker>,
) {
    let interaction = query.single().unwrap();

    if *interaction == Interaction::Clicked {
        event_writer.send(Marker::default());
    }
}
