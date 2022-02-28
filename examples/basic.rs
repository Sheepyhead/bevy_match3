use bevy::prelude::*;
use bevy_match3::{Match3Plugin, board::Board};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Match3Plugin)
        .add_startup_system(print)
        .run();
}

fn print(board: Res<Board>) {
  println!("{}", *board);
}