use bevy::{math::Vec3Swizzles, prelude::*, utils::HashMap};
use bevy_match3::{board::Board, systems::BoardEvents, Match3Plugin};

const GEM_SIDE_LENGTH: f32 = 50.0;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            resizable: false,
            title: "bevy_match3 basic example".to_string(),
            ..WindowDescriptor::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(Match3Plugin)
        .add_startup_system(setup_graphics)
        .add_system(move_to)
        .add_system(consume_events)
        .run();
}

#[derive(Component)]
struct VisibleBoard(HashMap<UVec2, Entity>);

fn setup_graphics(mut commands: Commands, board: Res<Board>, ass: Res<AssetServer>) {
    let board_side_length = GEM_SIDE_LENGTH * 10.0;
    let centered_offset_x = board_side_length / 2.0 - GEM_SIDE_LENGTH / 2.0;
    let centered_offset_y = board_side_length / 2.0 - GEM_SIDE_LENGTH / 2.0;
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform = Transform::from_xyz(
        centered_offset_x,
        centered_offset_y,
        camera.transform.translation.z,
    );
    commands.spawn_bundle(camera);

    let mut gems = HashMap::default();

    let vis_board = commands
        .spawn_bundle((Transform::default(), GlobalTransform::default()))
        .id();
    board.iter().for_each(|(position, typ)| {
        let transform = Transform::from_xyz(
            position.x as f32 * GEM_SIDE_LENGTH,
            position.y as f32 * GEM_SIDE_LENGTH,
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
            let mut movement = transform.translation.xy() - *move_to;
            movement =
                (movement.normalize() * time.delta_seconds()).clamp_length_max(movement.length());
            transform.translation = movement.extend(transform.translation.z);
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
    Vec2::new(
        pos.x as f32 * GEM_SIDE_LENGTH,
        pos.y as f32 * GEM_SIDE_LENGTH,
    )
}
