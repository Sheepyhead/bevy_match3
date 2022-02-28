use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use derive_deref::Deref;
use queues::{IsQueue, Queue};
use rand::Rng;

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

#[derive(Deref, Default)]
pub struct BoardCommands(Queue<BoardCommand>);

impl BoardCommands {
    pub fn push(&mut self, command: BoardCommand) -> Result<(), &str> {
        self.0.add(command).map(|_| ())
    }

    pub(crate) fn pop(&mut self) -> Result<BoardCommand, &str> {
        self.0.remove()
    }
}

#[derive(Clone, Copy)]
pub enum BoardCommand {
    Swap(UVec2, UVec2),
}

impl BoardEvents {
    pub(crate) fn push(&mut self, command: BoardEvent) -> Result<(), &str> {
        self.0.add(command).map(|_| ())
    }

    pub fn pop(&mut self) -> Result<BoardEvent, &str> {
        self.0.remove()
    }
}

#[derive(Deref, Default)]
pub struct BoardEvents(Queue<BoardEvent>);

#[derive(Clone, Copy)]
pub enum BoardEvent {
    Swapped(UVec2, UVec2),
}

#[derive(PartialEq, Debug, Clone)]
pub struct Board {
    dimensions: UVec2,
    gems: HashMap<UVec2, u32>,
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = (0..self.dimensions.y).map(|y| {
            f.write_fmt(format_args!(
                "{:?}\n",
                (0..self.dimensions.x)
                    .map(|x| self.gems[&[x, y].into()])
                    .collect::<Vec<_>>()
            ))
        });
        for res in res {
            match res {
                Ok(_) => {}
                err => return err,
            }
        }
        Ok(())
    }
}

fn read_commands(
    mut commands: ResMut<BoardCommands>,
    mut events: ResMut<BoardEvents>,
    mut board: ResMut<Board>,
) {
    if commands.is_changed() {
        while let Ok(command) = commands.pop() {
            match command {
                BoardCommand::Swap(pos1, pos2) => {
                    if board.swap(&pos1, &pos2).is_ok() {
                        events
                            .push(BoardEvent::Swapped(pos1, pos2))
                            .map_err(|err| println!("{err}"))
                            .unwrap();
                    };
                }
            }
        }
    }
}

impl From<Vec<Vec<u32>>> for Board {
    fn from(rows: Vec<Vec<u32>>) -> Self {
        let mut gems = HashMap::default();
        let mut width = 0;
        let mut height = 0;
        rows.iter().enumerate().for_each(|(y, row)| {
            height += 1;
            row.iter().enumerate().for_each(|(x, gem)| {
                gems.insert([x as u32, y as u32].into(), *gem);
                if height == 1 {
                    width += 1;
                }
            })
        });
        Board {
            gems,
            dimensions: [width, height].into(),
        }
    }
}

impl Board {
    pub fn get(&self, pos: &UVec2) -> Option<&u32> {
        self.gems.get(pos)
    }

    pub(crate) fn swap(&mut self, pos1: &UVec2, pos2: &UVec2) -> Result<(), &str> {
        let gem1 = self
            .get(pos1)
            .copied()
            .ok_or("No gems at position {pos1}")?;
        let gem2 = self
            .get(pos2)
            .copied()
            .ok_or("No gems at position {pos2}")?;
        self.gems.insert(*pos1, gem2);
        self.gems.insert(*pos2, gem1);
        // TODO: Add check for matches here and swap back if no matches
        Ok(())
    }

    pub(crate) fn get_matches(&self) -> Matches {
        let mut matches = self.straight_matches(MatchDirection::Horizontal);
        matches.append(&mut self.straight_matches(MatchDirection::Vertical));
        matches
    }

    fn straight_matches(&self, direction: MatchDirection) -> Matches {
        let mut matches = Matches::default();
        let mut current_match = vec![];
        let mut previous_type = None;
        for one in match direction {
            MatchDirection::Horizontal => 0..self.dimensions.x,
            MatchDirection::Vertical => 0..self.dimensions.y,
        } {
            for two in match direction {
                MatchDirection::Horizontal => 0..self.dimensions.y,
                MatchDirection::Vertical => 0..self.dimensions.x,
            } {
                let pos = [
                    match direction {
                        MatchDirection::Horizontal => one,
                        MatchDirection::Vertical => two,
                    },
                    match direction {
                        MatchDirection::Horizontal => two,
                        MatchDirection::Vertical => one,
                    },
                ]
                .into();

                let current_type = *self.get(&pos).unwrap();
                if current_match.is_empty() || previous_type.unwrap() == current_type {
                    previous_type = Some(current_type);
                    current_match.push(pos);
                } else if previous_type.unwrap() != current_type {
                    match current_match.len() {
                        0 | 1 | 2 => {}
                        3 => matches.add(Match::Straight(current_match.iter().cloned().collect())),
                        _ => unimplemented!("Match bigger than three found"),
                    }
                    current_match = vec![pos];
                    previous_type = Some(current_type);
                }
            }
            match current_match.len() {
                0 | 1 | 2 => {}
                3 => matches.add(Match::Straight(current_match.iter().cloned().collect())),
                _ => unimplemented!("Match bigger than three found"),
            }
            current_match = vec![];
            previous_type = None;
        }
        matches
    }
}

enum MatchDirection {
    Horizontal,
    Vertical,
}

pub enum Match {
    Straight(HashSet<UVec2>),
}

#[derive(Default)]
pub(crate) struct Matches {
    matches: Vec<Match>,
}

impl Matches {
    pub(crate) fn add(&mut self, mat: Match) {
        self.matches.push(mat)
    }

    fn append(&mut self, other: &mut Matches) {
        self.matches.append(&mut other.matches);
    }

    pub(crate) fn without_duplicates(&self) -> HashSet<UVec2> {
        self.matches
            .iter()
            .flat_map(|mat| match mat {
                Match::Straight(mat) => mat,
            })
            .cloned()
            .collect()
    }

    pub(crate) fn len(&self) -> usize {
        self.matches.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use queues::{IsQueue, Queue};

    use crate::{read_commands, Board, BoardCommand, BoardCommands, BoardEvents};

    #[test]
    fn board_creation() {
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

        assert_eq!(board.dimensions, [5, 7].into());
        assert_eq!(*board.get(&[0, 0].into()).unwrap(), 0);
        assert_eq!(*board.get(&[1, 1].into()).unwrap(), 6);
        assert_eq!(*board.get(&[4, 2].into()).unwrap(), 14);
        assert_eq!(*board.get(&[2, 3].into()).unwrap(), 17);
        assert_eq!(*board.get(&[0, 4].into()).unwrap(), 20);
        assert_eq!(*board.get(&[4, 6].into()).unwrap(), 34);
    }

    #[test]
    fn swap_gems() {
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
    fn check_horizontal_matches() {
        #[rustfmt::skip]
        let board: Board = vec![
            vec![ 0,  1,  2,  3,  4],
            vec![ 5,  6,  7,  3,  9],
            vec![10, 11, 12,  3, 14],
            vec![15, 16, 12, 18, 19],
            vec![20, 26, 12, 23, 24],
            vec![25, 26, 27, 28, 24],
            vec![30, 26, 32, 33, 24],
        ].into();

        let matches = board.get_matches();

        assert_eq!(matches.len(), 4);

        let without_duplicates = matches.without_duplicates();

        assert!(without_duplicates.contains(&[2, 2].into()));
        assert!(without_duplicates.contains(&[2, 3].into()));
        assert!(without_duplicates.contains(&[2, 4].into()));
        assert!(without_duplicates.contains(&[3, 0].into()));
        assert!(without_duplicates.contains(&[3, 1].into()));
        assert!(without_duplicates.contains(&[3, 2].into()));
        assert!(without_duplicates.contains(&[1, 4].into()));
        assert!(without_duplicates.contains(&[1, 5].into()));
        assert!(without_duplicates.contains(&[1, 6].into()));
        assert!(without_duplicates.contains(&[4, 4].into()));
        assert!(without_duplicates.contains(&[4, 5].into()));
        assert!(without_duplicates.contains(&[4, 6].into()));
    }

    #[test]
    fn check_vertical_matches() {
        #[rustfmt::skip]
        let board: Board = vec![
            vec![ 0,  3,  3,  3,  4],
            vec![ 5,  6,  7,  8,  9],
            vec![11, 11, 11, 13, 14],
            vec![15, 16, 17, 18, 19],
            vec![20, 21, 22, 23, 24],
            vec![25, 26, 29, 29, 29],
            vec![30, 30, 30, 33, 34],
        ].into();


        let matches = board.get_matches();

        assert_eq!(matches.len(), 4);

        let without_duplicates = matches.without_duplicates();

        assert!(without_duplicates.contains(&[1, 0].into()));
        assert!(without_duplicates.contains(&[2, 0].into()));
        assert!(without_duplicates.contains(&[3, 0].into()));
        assert!(without_duplicates.contains(&[0, 2].into()));
        assert!(without_duplicates.contains(&[1, 2].into()));
        assert!(without_duplicates.contains(&[2, 2].into()));
        assert!(without_duplicates.contains(&[2, 5].into()));
        assert!(without_duplicates.contains(&[3, 5].into()));
        assert!(without_duplicates.contains(&[4, 5].into()));
        assert!(without_duplicates.contains(&[0, 6].into()));
        assert!(without_duplicates.contains(&[1, 6].into()));
        assert!(without_duplicates.contains(&[2, 6].into()));
    }

    #[test]
    fn check_both_directions_matches() {
        #[rustfmt::skip]
        let board: Board = vec![
            vec![ 0,  1,  2,  3,  4],
            vec![ 5,  6,  7,  8,  9],
            vec![11, 11, 11,  8, 14],
            vec![15, 16, 17,  8, 19],
            vec![20, 21, 22, 23, 24],
            vec![20, 26, 29, 29, 29],
            vec![20, 31, 32, 33, 34],
        ].into();


        let matches = board.get_matches();

        assert_eq!(matches.len(), 4);

        let without_duplicates = matches.without_duplicates();

        assert!(without_duplicates.contains(&[3, 1].into()));
        assert!(without_duplicates.contains(&[3, 2].into()));
        assert!(without_duplicates.contains(&[3, 3].into()));
        assert!(without_duplicates.contains(&[0, 2].into()));
        assert!(without_duplicates.contains(&[1, 2].into()));
        assert!(without_duplicates.contains(&[2, 2].into()));
        assert!(without_duplicates.contains(&[0, 4].into()));
        assert!(without_duplicates.contains(&[0, 5].into()));
        assert!(without_duplicates.contains(&[0, 6].into()));
        assert!(without_duplicates.contains(&[2, 5].into()));
        assert!(without_duplicates.contains(&[3, 5].into()));
        assert!(without_duplicates.contains(&[4, 5].into()));
    }
}
