use bevy::{prelude::*, utils::HashMap};
use board::*;
use rand::Rng;
use systems::*;

pub mod board;
pub mod mat;
mod systems;

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

        let mut gems = HashMap::default();
        (0..board_dimensions.x).for_each(|x| {
            (0..board_dimensions.y).for_each(|y| {
                gems.insert([x, y].into(), rand::thread_rng().gen_range(0..gem_types));
            })
        });

        app.insert_resource(Board {
            dimensions: board_dimensions,
            gems,
        })
        .insert_resource(BoardCommands)
        .insert_resource(BoardEvents)
        .add_system(read_commands);
    }
}

#[derive(Clone, Copy)]
pub struct Match3Config {
    pub gem_types: u32,
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
}
