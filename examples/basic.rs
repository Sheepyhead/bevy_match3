use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseButtonInput, ButtonState},
    math::Vec3Swizzles,
    prelude::*,
    utils::HashMap, render::texture::ImageSettings,
};
use bevy_match3::prelude::*;

const GEM_SIDE_LENGTH: f32 = 50.0;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            resizable: false,
            title: "bevy_match3 basic example".to_string(),
            ..WindowDescriptor::default()
        })
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .insert_resource(Selection::default())
        .add_plugin(Match3Plugin)
        .add_startup_system(setup_graphics)
        .add_system(move_to)
        .add_system(consume_events)
        .add_system(input)
        .add_system(visualize_selection)
        .add_system(control)
        .add_system(animate_once)
        .add_system(shuffle)
        .run();
}

#[derive(Component, Clone)]
struct VisibleBoard(HashMap<UVec2, Entity>);

#[derive(Component)]
struct MainCamera;

fn setup_graphics(mut commands: Commands, board: Res<Board>, ass: Res<AssetServer>) {
    let board_side_length = GEM_SIDE_LENGTH * 10.0;
    let centered_offset_x = board_side_length / 2.0 - GEM_SIDE_LENGTH / 2.0;
    let centered_offset_y = board_side_length / 2.0 - GEM_SIDE_LENGTH / 2.0;
    let mut camera = Camera2dBundle::default();
    camera.transform = Transform::from_xyz(
        centered_offset_x,
        0.0 - centered_offset_y,
        camera.transform.translation.z,
    );
    commands.spawn_bundle(camera).insert(MainCamera);

    let mut gems = HashMap::default();

    let vis_board = commands.spawn_bundle(SpatialBundle::default()).id();

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

    let board = VisibleBoard(gems);

    commands.entity(vis_board).insert(board);
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
            commands.entity(entity).remove::<MoveTo>();
        } else {
            let mut movement = *move_to - transform.translation.xy();
            movement = // Multiplying the move by GEM_SIDE_LENGTH as well as delta seconds means the animation takes exactly 1 second
                (movement.normalize() * time.delta_seconds() * GEM_SIDE_LENGTH * 5.0).clamp_length_max(movement.length());
            let movement = movement.extend(transform.translation.z);
            transform.translation += movement;
        }
    }
}

fn consume_events(
    mut commands: Commands,
    mut events: ResMut<BoardEvents>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut board_commands: ResMut<BoardCommands>,
    ass: Res<AssetServer>,
    mut board: Query<(Entity, &mut VisibleBoard)>,
    animations: Query<(), With<MoveTo>>,
) {
    if animations.iter().count() == 0 {
        if let Ok(event) = events.pop() {
            let (board_entity, mut board) = board.single_mut();
            match event {
                BoardEvent::Swapped(pos1, pos2) => {
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
                BoardEvent::Popped(pos) => {
                    let gem = board.0.get(&pos).copied().unwrap();
                    board.0.remove(&pos);
                    commands.entity(gem).despawn_recursive();
                    spawn_explosion(
                        &ass,
                        &mut texture_atlases,
                        &mut commands,
                        &board_pos_to_world_pos(&pos),
                    );
                }
                BoardEvent::Matched(matches) => {
                    board_commands
                        .push(BoardCommand::Pop(
                            matches.without_duplicates().iter().copied().collect(),
                        ))
                        .unwrap();
                }
                BoardEvent::Dropped(drops) => {
                    // Need to keep a buffered board clone because we read and write at the same time
                    let mut new_board = board.clone();
                    for bevy_match3::prelude::Drop { from, to } in drops {
                        let gem = board.0.get(&from).copied().unwrap();
                        new_board.0.insert(to, gem);
                        new_board.0.remove(&from);
                        commands
                            .entity(gem)
                            .insert(MoveTo(board_pos_to_world_pos(&to)));
                    }
                    // And copy the buffer to the resource
                    *board = new_board;
                }
                BoardEvent::Spawned(spawns) => {
                    let mut new_board = board.clone();

                    for (pos, typ) in spawns {
                        let world_pos = board_pos_to_world_pos(&pos);
                        let gem = commands
                            .spawn_bundle(SpriteBundle {
                                texture: ass.load(&map_type_to_path(typ)),
                                transform: Transform::from_xyz(world_pos.x, 200.0, 0.0),
                                sprite: Sprite {
                                    custom_size: Some([50.0, 50.0].into()),
                                    ..Sprite::default()
                                },
                                ..SpriteBundle::default()
                            })
                            .insert(MoveTo(world_pos))
                            .id();
                        new_board.0.insert(pos, gem);
                        commands.entity(board_entity).add_child(gem);
                    }
                    *board = new_board;
                }
                BoardEvent::Shuffled(moves) => {
                    let mut temp_board = board.clone();
                    for (from, to) in moves {
                        let gem = board.0.get(&from).copied().unwrap();

                        commands
                            .entity(gem)
                            .insert(MoveTo(board_pos_to_world_pos(&to)));

                        temp_board.0.insert(to, gem);
                    }
                    *board = temp_board;
                }
                _ => {
                    println!("Received unimplemented event");
                }
            }
        }
    }
}

fn board_pos_to_world_pos(pos: &UVec2) -> Vec2 {
    Vec2::new(
        pos.x as f32 * GEM_SIDE_LENGTH,
        -(pos.y as f32) * GEM_SIDE_LENGTH,
    )
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
            state: ButtonState::Pressed,
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
                    camera_transform.compute_matrix() * camera.projection_matrix().inverse();

                // use it to convert ndc to world-space coordinates
                let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

                // reduce it to a 2D value
                let world_pos: Vec2 = world_pos.truncate();

                // end of borrowed boilerplate
                // round down to the gem coordinate
                let coordinates: IVec2 = (
                    ((world_pos.x + GEM_SIDE_LENGTH / 2.0) / GEM_SIDE_LENGTH) as i32,
                    ((GEM_SIDE_LENGTH / 2.0 - world_pos.y) / GEM_SIDE_LENGTH) as i32,
                )
                    .into();

                if coordinates.x >= 0 && coordinates.y >= 0 {
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

#[derive(Component)]
struct AnimationTimer(Timer);

fn animate_once(
    mut commands: Commands,
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut timers: Query<(
        Entity,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (entity, mut timer, mut sprite, texture_atlas_handle) in timers.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            if sprite.index == 3 {
                commands.entity(entity).despawn_recursive();
            } else {
                sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
            }
        }
    }
}

fn spawn_explosion(
    ass: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlas>,
    commands: &mut Commands,
    pos: &Vec2,
) {
    let texture_handle = ass.load("explosion.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(49.0, 50.0), 4, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_translation(pos.extend(0.0)),
            ..SpriteSheetBundle::default()
        })
        .insert(AnimationTimer(Timer::from_seconds(0.1, true)));
}

fn shuffle(
    mut board_commands: ResMut<BoardCommands>,
    mut key_event: EventReader<KeyboardInput>,
    animations: Query<(), With<MoveTo>>,
) {
    if animations.iter().count() == 0 {
        for event in key_event.iter() {
            if let KeyboardInput {
                key_code: Some(KeyCode::S),
                state: ButtonState::Pressed,
                ..
            } = event
            {
                board_commands.push(BoardCommand::Shuffle).unwrap();
            }
        }
    }
}
