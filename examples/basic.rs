use bevy::{prelude::*, window::WindowResizeConstraints};
use bevy_editor_pls::EditorPlugin;
use bevy_match3::{board::Board, Match3Plugin};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            resizable: false,
            ..WindowDescriptor::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(Match3Plugin)
        .add_startup_system(setup_graphics)
        .run();
}

#[derive(Component)]
struct VisibleBoard;

fn setup_graphics(mut commands: Commands, board: Res<Board>, ass: Res<AssetServer>) {
    let gem_side_length = 50.0;
    let board_side_length = gem_side_length * 10.0;
    let centered_offset_x = board_side_length / 2.0 - gem_side_length / 2.0;
    let centered_offset_y = board_side_length / 2.0 - gem_side_length / 2.0;
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform = Transform::from_xyz(
        centered_offset_x,
        centered_offset_y,
        camera.transform.translation.z,
    );
    commands.spawn_bundle(camera);

    let vis_board = commands
        .spawn_bundle((
            VisibleBoard,
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();
    board.iter().for_each(|(position, typ)| {
        let transform = Transform::from_xyz(
            position.x as f32 * gem_side_length,
            position.y as f32 * gem_side_length,
            0.0,
        );
        let child = commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(gem_side_length, gem_side_length)),
                    ..Sprite::default()
                },
                transform,
                texture: ass.load(&map_type_to_path(*typ)),
                ..SpriteBundle::default()
            })
            .id();
        commands.entity(vis_board).add_child(child);
    });
}

fn map_type_to_path(typ: u32) -> String {
    format!("{typ}.png")
}
