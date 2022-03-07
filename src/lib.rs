//! # ``bevy_match3``
//!
//! An implementation of the logical and algorithmic side of match 3 games
//!

#![deny(missing_docs, clippy::doc_markdown)]

use bevy::{prelude::*, utils::HashMap};
use board::*;
use rand::Rng;
use systems::*;

mod board;
mod mat;
mod systems;

/// Use `bevy_match3::*;` to import common structs and plugins
pub mod prelude {
    pub use crate::board::*;
    pub use crate::mat::*;
    pub use crate::systems::*;
    pub use crate::Match3Plugin;
}

/// The central logic plugin of the ``bevy_match3`` crate
pub struct Match3Plugin;

impl Plugin for Match3Plugin {
    fn build(&self, app: &mut App) {
        let Match3Config {
            board_dimensions,
            gem_types,
        } = app
            .world
            .get_resource::<Match3Config>()
            .copied()
            .unwrap_or_default();

        if gem_types < 3 {
            panic!("Cannot generate board with fewer than 3 different gem types");
        }

        let mut gems = HashMap::default();
        (0..board_dimensions.x).for_each(|x| {
            (0..board_dimensions.y).for_each(|y| {
                gems.insert([x, y].into(), rand::thread_rng().gen_range(0..gem_types));
            })
        });

        let mut board = Board {
            dimensions: board_dimensions,
            gems,
            types: (0..gem_types).collect(),
        };

        board.clear_matches();

        app.insert_resource(board)
            .insert_resource(BoardCommands::default())
            .insert_resource(BoardEvents::default())
            .add_system(read_commands);
    }
}

/// An optional config for the match3 board. This should be inserted as a resource before the `Match3Plugin`
/// 
/// # Examples
/// 
/// ```
/// use bevy_match3::prelude::*;
/// 
/// App::new()
///     .insert_resource(Match3Config {
///         gem_types: 5,
///         board_dimensions: [10, 10].into(),
///     })
///     .add_plugin(Match3Plugin)
///     .run();
/// ```
#[derive(Clone, Copy)]
pub struct Match3Config {
    /// The number of different gem types the board can spawn
    pub gem_types: u32,
    /// The rectangular dimensions of the board
    pub board_dimensions: UVec2,
}

impl Default for Match3Config {
    fn default() -> Self {
        Self {
            gem_types: 5,
            board_dimensions: [10, 10].into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use queues::{IsQueue, Queue};

    use crate::{board::*, systems::*};

    #[test]
    fn swap_gems() {
        // setup
        #[rustfmt::skip]
        let board: Board = vec![
            vec![ 0,  1,  2,  3,  4],
            vec![ 5,  6,  7,  8,  9],
            vec![10, 11, 12, 13, 14],
            vec![15, 16, 11, 18, 19],
            vec![20, 21, 11, 23, 24],
            vec![25, 26, 27, 28, 29],
            vec![30, 31, 32, 33, 34],
        ].into();

        let mut queue = Queue::default();
        queue
            .add(BoardCommand::Swap([1, 2].into(), [2, 2].into()))
            .unwrap();

        let mut update_stage = SystemStage::parallel();
        update_stage.add_system(read_commands);

        let mut world = World::default();
        world.insert_resource(board.clone());
        world.insert_resource(BoardCommands(queue));
        world.insert_resource(BoardEvents::default());

        // run
        update_stage.run(&mut world);

        // check
        assert_ne!(board, *world.get_resource::<Board>().unwrap());
        assert_eq!(
            world
                .get_resource::<Board>()
                .unwrap()
                .get(&[1, 2].into())
                .copied()
                .unwrap(),
            12
        );
        assert_eq!(
            world
                .get_resource::<Board>()
                .unwrap()
                .get(&[2, 2].into())
                .copied()
                .unwrap(),
            11
        );
    }

    #[test]
    fn fail_to_swap_gems() {
        // setup
        #[rustfmt::skip]
        let board: Board = vec![
            vec![ 0,  1,  2,  3,  4],
            vec![ 5,  6,  7,  8,  9],
            vec![10, 11, 12, 13, 14],
            vec![15, 16, 17, 18, 19],
            vec![20, 21, 22, 23, 24],
            vec![25, 26, 27, 28, 29],
            vec![30, 31, 32, 33, 34],
        ].into();

        let mut queue = Queue::default();
        queue
            .add(BoardCommand::Swap([1, 2].into(), [2, 2].into()))
            .unwrap();

        let mut update_stage = SystemStage::parallel();
        update_stage.add_system(read_commands);

        let mut world = World::default();
        world.insert_resource(board.clone());
        world.insert_resource(BoardCommands(queue));
        world.insert_resource(BoardEvents::default());

        // run
        update_stage.run(&mut world);

        // check
        assert_eq!(board, *world.get_resource::<Board>().unwrap());
        assert_eq!(
            world
                .get_resource::<Board>()
                .unwrap()
                .get(&[1, 2].into())
                .copied()
                .unwrap(),
            11
        );
        assert_eq!(
            world
                .get_resource::<Board>()
                .unwrap()
                .get(&[2, 2].into())
                .copied()
                .unwrap(),
            12
        );
    }

    #[test]
    fn pop_gem() {
        // setup
        #[rustfmt::skip]
        let board: Board = vec![
            vec![ 0,  1,  2,  3,  4],
            vec![ 5,  6,  7,  8,  9],
            vec![10, 11, 12, 13, 14],
            vec![15, 16, 17, 18, 19],
            vec![20, 21, 22, 23, 24],
            vec![25, 26, 27, 28, 29],
            vec![30, 31, 32, 33, 34],
        ].into();

        let mut queue = Queue::default();
        queue.add(BoardCommand::Pop(vec![[1, 4].into()])).unwrap();

        let mut update_stage = SystemStage::parallel();
        update_stage.add_system(read_commands);

        let mut world = World::default();
        world.insert_resource(board.clone());
        world.insert_resource(BoardCommands(queue));
        world.insert_resource(BoardEvents::default());

        // run
        update_stage.run(&mut world);

        // check
        let new_board = world.get_resource::<Board>().unwrap();
        assert_ne!(board, *new_board);
        assert_eq!(*new_board.get(&[1, 4].into()).unwrap(), 16);
        assert_eq!(*new_board.get(&[1, 3].into()).unwrap(), 11);
        assert_eq!(*new_board.get(&[1, 2].into()).unwrap(), 6);
        assert_eq!(*new_board.get(&[1, 1].into()).unwrap(), 1);
        assert!(new_board.get(&[1, 0].into()).is_some());
    }

    #[test]
    fn pop_gems_vertical() {
        // setup
        #[rustfmt::skip]
        let board: Board = vec![
            vec![ 0,  1,  2,  3,  4],
            vec![ 5,  6,  7,  8,  9],
            vec![10, 11, 12, 13, 14],
            vec![15, 16, 17, 18, 19],
            vec![20, 21, 22, 23, 24],
            vec![25, 26, 27, 28, 29],
            vec![30, 31, 32, 33, 34],
        ].into();

        let mut queue = Queue::default();
        queue
            .add(BoardCommand::Pop(vec![
                [3, 6].into(),
                [3, 5].into(),
                [3, 4].into(),
            ]))
            .unwrap();

        let mut update_stage = SystemStage::parallel();
        update_stage.add_system(read_commands);

        let mut world = World::default();
        world.insert_resource(board.clone());
        world.insert_resource(BoardCommands(queue));
        world.insert_resource(BoardEvents::default());

        // run
        update_stage.run(&mut world);

        // check
        let new_board = world.get_resource::<Board>().unwrap();
        assert_ne!(board, *new_board);
        assert_eq!(*new_board.get(&[3, 6].into()).unwrap(), 18);
        assert_eq!(*new_board.get(&[3, 5].into()).unwrap(), 13);
        assert_eq!(*new_board.get(&[3, 4].into()).unwrap(), 8);
        assert_eq!(*new_board.get(&[3, 3].into()).unwrap(), 3);
        assert!(new_board.get(&[3, 0].into()).is_some());
        assert!(new_board.get(&[3, 1].into()).is_some());
        assert!(new_board.get(&[3, 2].into()).is_some());
    }

    #[test]
    fn pop_gems_horizontal() {
        // setup
        #[rustfmt::skip]
        let board: Board = vec![
            vec![ 0,  1,  2,  3,  4],
            vec![ 5,  6,  7,  8,  9],
            vec![10, 11, 12, 13, 14],
            vec![15, 16, 17, 18, 19],
            vec![20, 21, 22, 23, 24],
            vec![25, 26, 27, 28, 29],
            vec![30, 31, 32, 33, 34],
        ].into();

        let mut queue = Queue::default();
        queue
            .add(BoardCommand::Pop(vec![
                [0, 5].into(),
                [1, 5].into(),
                [2, 5].into(),
            ]))
            .unwrap();

        let mut update_stage = SystemStage::parallel();
        update_stage.add_system(read_commands);

        let mut world = World::default();
        world.insert_resource(board.clone());
        world.insert_resource(BoardCommands(queue));
        world.insert_resource(BoardEvents::default());

        // run
        update_stage.run(&mut world);

        // check
        let new_board = world.get_resource::<Board>().unwrap();
        assert_ne!(board, *new_board);
        assert_eq!(*new_board.get(&[0, 5].into()).unwrap(), 20);
        assert_eq!(*new_board.get(&[1, 5].into()).unwrap(), 21);
        assert_eq!(*new_board.get(&[2, 5].into()).unwrap(), 22);
        assert!(new_board.get(&[0, 0].into()).is_some());
        assert!(new_board.get(&[1, 0].into()).is_some());
        assert!(new_board.get(&[2, 0].into()).is_some());
    }
}
