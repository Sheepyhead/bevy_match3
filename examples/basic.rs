use bevy::{
    input::{mouse::MouseButtonInput, ElementState},
    math::Vec3Swizzles,
    prelude::*,
    utils::HashMap,
};
use bevy_editor_pls::EditorPlugin;
use bevy_match3::{
    board::Board,
    systems::{BoardCommand, BoardCommands, BoardEvents},
    Match3Plugin,
};

const GEM_SIDE_LENGTH: f32 = 50.0;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            resizable: false,
            title: "bevy_match3 basic example".to_string(),
            ..WindowDescriptor::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin)
        .insert_resource(Selection::default())
        .add_plugin(Match3Plugin)
        .add_startup_system(setup_graphics)
        .add_system(move_to)
        .add_system(consume_events)
        .add_system(input)
        .add_system(visualize_selection)
        .add_system(control)
        .run();
}

#[derive(Component)]
struct VisibleBoard(HashMap<UVec2, Entity>);

#[derive(Component)]
struct MainCamera;

fn setup_graphics(mut commands: Commands, board: Res<Board>, ass: Res<AssetServer>) {
    let board_side_length = GEM_SIDE_LENGTH * 10.0;
    let centered_offset_x = board_side_length / 2.0 - GEM_SIDE_LENGTH / 2.0;
    let centered_offset_y = board_side_length / 2.0 - GEM_SIDE_LENGTH / 2.0;
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform = Transform::from_xyz(
        centered_offset_x,
        0.0 - centered_offset_y,
        camera.transform.translation.z,
    );
    commands.spawn_bundle(camera).insert(MainCamera);

    let mut gems = HashMap::default();

    let vis_board = commands
        .spawn_bundle((Transform::default(), GlobalTransform::default()))
        .id();
    board.iter().for_each(|(position, typ)| {
        let transform = Transform::from_xyz(
            position.x as f32 * GEM_SIDE_LENGTH,
            position.y as f32 * -GEM_SIDE_LENGTH,
            0.0,
        );
        let child = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(GEM_SIDE_LENGTH, GEM_SIDE_LENGTH)),
                    ..Sprite::default()
                },
                transform,
                texture: ass.load(&map_type_to_path(*typ)),
                ..SpriteBundle::default()
            })
            .insert(Name::new(format!("{};{}", position.x, position.y)))
            .id();
        gems.insert(*position, child);
        commands.entity(vis_board).add_child(child);
    });

    commands.entity(vis_board).insert(VisibleBoard(gems));
}

fn map_type_to_path(typ: u32) -> String {
    format!("{typ}.png")
}

#[derive(Component)]
struct MoveTo(Vec2);

fn move_to(
    mut commands: Commands,
    time: Res<Time>,
    mut moves: Query<(Entity, &mut Transform, &MoveTo)>,
) {
    for (entity, mut transform, MoveTo(move_to)) in moves.iter_mut() {
        if transform.translation == Vec3::new(move_to.x, move_to.y, transform.translation.z) {
            println!("{entity:?} reached destination!");
            commands.entity(entity).remove::<MoveTo>();
        } else {
            let mut movement = *move_to - transform.translation.xy();
            movement = // Multiplying the move by GEM_SIDE_LENGTH as well as delta seconds means the animation takes exactly 1 second
                (movement.normalize() * time.delta_seconds() * GEM_SIDE_LENGTH).clamp_length_max(movement.length());
            let movement = movement.extend(transform.translation.z);
            transform.translation += movement;
        }
    }
}

fn consume_events(
    mut commands: Commands,
    mut events: ResMut<BoardEvents>,
    mut board: Query<&mut VisibleBoard>,
) {
    while let Ok(event) = events.pop() {
        let mut board = board.single_mut();
        match event {
            bevy_match3::systems::BoardEvent::Swapped(pos1, pos2) => {
                let gem1 = board.0.get(&pos1).copied().unwrap();
                let gem2 = board.0.get(&pos2).copied().unwrap();

                commands
                    .entity(gem1)
                    .insert(MoveTo(board_pos_to_world_pos(&pos2)));

                commands
                    .entity(gem2)
                    .insert(MoveTo(board_pos_to_world_pos(&pos1)));

                board.0.insert(pos2, gem1);
                board.0.insert(pos1, gem2);
            }
            _ => {
                println!("Received unimplemented event")
            }
        }
    }
}

fn board_pos_to_world_pos(pos: &UVec2) -> Vec2 {
    let new_pos = Vec2::new(
        pos.x as f32 * GEM_SIDE_LENGTH,
        -(pos.y as f32) * GEM_SIDE_LENGTH,
    );
    println!("Translated {pos} to {new_pos}");
    new_pos
}

#[derive(Default, Clone, Copy)]
struct Selection(Option<Entity>);

fn input(
    windows: Res<Windows>,
    mut selection: ResMut<Selection>,
    mut button_events: EventReader<MouseButtonInput>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    board: Query<&VisibleBoard>,
) {
    for event in button_events.iter() {
        if let MouseButtonInput {
            button: MouseButton::Left,
            state: ElementState::Pressed,
        } = event
        {
            let window = windows.get_primary().unwrap();
            if let Some(pos) = window.cursor_position() {
                // The following code is boilerplate from https://bevy-cheatbook.github.io/cookbook/cursor2world.html#2d-games
                let window_size = Vec2::new(window.width() as f32, window.height() as f32);

                // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
                let ndc = (pos / window_size) * 2.0 - Vec2::ONE;

                let (camera, camera_transform) = camera.single();
                // matrix for undoing the projection and camera transform
                let ndc_to_world =
                    camera_transform.compute_matrix() * camera.projection_matrix.inverse();

                // use it to convert ndc to world-space coordinates
                let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

                // reduce it to a 2D value
                let world_pos: Vec2 = world_pos.truncate();

                // end of borrowed boilerplate
                println!("Clicked at position {world_pos}");
                // round down to the gem coordinate
                let coordinates: IVec2 = (
                    ((world_pos.x + GEM_SIDE_LENGTH / 2.0) / GEM_SIDE_LENGTH) as i32,
                    ((GEM_SIDE_LENGTH / 2.0 - world_pos.y) / GEM_SIDE_LENGTH) as i32,
                )
                    .into();

                if coordinates.x >= 0 && coordinates.y >= 0 {
                    println!("Translated to coordinates {coordinates}");

                    selection.0 = board
                        .single()
                        .0
                        .get(&[coordinates.x as u32, coordinates.y as u32].into())
                        .copied();
                }
            }
        }
    }
}

#[derive(Component)]
struct SelectionRectangle;

fn visualize_selection(
    mut commands: Commands,
    selection: Res<Selection>,
    ass: Res<AssetServer>,
    g_transforms: Query<&GlobalTransform>,
    mut rectangle: Query<(Entity, &mut Transform), With<SelectionRectangle>>,
) {
    if selection.is_changed() {
        if let Some(selected_gem) = selection.0 {
            let transform = g_transforms.get(selected_gem).unwrap();
            if let Ok((_, mut old_transform)) = rectangle.get_single_mut() {
                *old_transform = (*transform).into();
            } else {
                commands
                    .spawn_bundle(SpriteBundle {
                        texture: ass.load("rectangle.png"),
                        sprite: Sprite {
                            custom_size: Some([50.0, 50.0].into()),
                            ..Sprite::default()
                        },
                        transform: (*transform).into(),
                        ..SpriteBundle::default()
                    })
                    .insert(SelectionRectangle);
            }
        } else if let Ok((entity, _)) = rectangle.get_single_mut() {
            commands.entity(entity).despawn();
        }
    }
}

fn control(
    mut board_commands: ResMut<BoardCommands>,
    mut selection: ResMut<Selection>,
    mut last_selection: Local<Selection>,
    transforms: Query<&Transform>,
) {
    if selection.is_changed() {
        if let Some(selected_gem) = selection.0 {
            if let Some(last_selected_gem) = last_selection.0 {
                let selected_pos = transforms.get(selected_gem).unwrap().translation.xy() / 50.0;
                let last_selected_pos =
                    transforms.get(last_selected_gem).unwrap().translation.xy() / 50.0;

                board_commands
                    .push(BoardCommand::Swap(
                        [selected_pos.x as u32, -selected_pos.y as u32].into(),
                        [last_selected_pos.x as u32, -last_selected_pos.y as u32].into(),
                    ))
                    .map_err(|err| println!("{err}"))
                    .unwrap();
                selection.0 = None;
                last_selection.0 = None;
            } else {
                *last_selection = *selection;
            }
        } else {
            last_selection.0 = None
        }
    }
}
