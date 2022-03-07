use std::fmt;
use crate::{board::*, mat::Matches};
use bevy::prelude::*;
use queues::{IsQueue, Queue};

pub(crate) fn read_commands(
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
                        let matches = board.get_matches();
                        events
                            .push(BoardEvent::Matched(matches))
                            .map_err(|err| println!("{err}"))
                            .unwrap();
                    } else {
                        events
                            .push(BoardEvent::FailedSwap(pos1, pos2))
                            .map_err(|err| println!("{err}"))
                            .unwrap();
                    };
                }
                BoardCommand::Pop(gems) => {
                    gems.iter().for_each(|gem| {
                        board.remove(gem);
                        events
                            .push(BoardEvent::Popped(*gem))
                            .map_err(|err| println!("{err}"))
                            .unwrap()
                    });
                    let mut drops: Vec<Drop> =
                        board.drop().iter().copied().map(|e| e.into()).collect();
                    drops.sort();
                    events
                        .push(BoardEvent::Dropped(drops))
                        .map_err(|err| println!("{err}"))
                        .unwrap();

                    events
                        .push(BoardEvent::Spawned(board.fill().iter().copied().collect()))
                        .map_err(|err| println!("{err}"))
                        .unwrap();
                }
            }
        }
    }
}

#[derive(Default)]
pub struct BoardCommands(pub(crate) Queue<BoardCommand>);

impl BoardCommands {
    pub fn push(&mut self, command: BoardCommand) -> Result<(), &str> {
        self.0.add(command).map(|_| ())
    }

    pub(crate) fn pop(&mut self) -> Result<BoardCommand, &str> {
        self.0.remove()
    }
}

#[derive(Clone)]
pub enum BoardCommand {
    Swap(UVec2, UVec2),
    Pop(Vec<UVec2>),
}

impl BoardEvents {
    pub(crate) fn push(&mut self, command: BoardEvent) -> Result<(), &str> {
        self.0.add(command).map(|_| ())
    }

    pub fn pop(&mut self) -> Result<BoardEvent, &str> {
        self.0.remove()
    }
}

#[derive( Default)]
pub struct BoardEvents(pub(crate) Queue<BoardEvent>);

#[derive(Clone)]
pub enum BoardEvent {
    Swapped(UVec2, UVec2),
    FailedSwap(UVec2, UVec2),
    Dropped(Vec<Drop>),
    Popped(UVec2),
    Spawned(Vec<(UVec2, u32)>),
    Matched(Matches),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Drop {
    pub from: UVec2,
    pub to: UVec2,
}

impl From<(UVec2, UVec2)> for Drop {
    fn from((from, to): (UVec2, UVec2)) -> Self {
        Self { from, to }
    }
}

impl PartialOrd for Drop {
    fn partial_cmp(&self, other: &Self) -> std::option::Option<std::cmp::Ordering> {
        Some(self.from.y.cmp(&other.from.y).reverse())
    }
}

impl Ord for Drop {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.from.y.cmp(&other.from.y).reverse()
    }
}

impl fmt::Debug for Drop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{} -> {}", self.from, self.to))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sort_drops() {
        let mut drops: Vec<crate::Drop> = vec![
            ((0, 1).into(), (0, 2).into()).into(),
            ((1, 0).into(), (1, 2).into()).into(),
            ((4, 1).into(), (4, 2).into()).into(),
            ((1, 1).into(), (1, 3).into()).into(),
            ((4, 0).into(), (4, 1).into()).into(),
            ((0, 2).into(), (0, 3).into()).into(),
            ((3, 0).into(), (3, 1).into()).into(),
            ((2, 2).into(), (2, 4).into()).into(),
            ((0, 3).into(), (0, 4).into()).into(),
            ((2, 0).into(), (2, 2).into()).into(),
            ((2, 1).into(), (2, 3).into()).into(),
            ((1, 2).into(), (1, 4).into()).into(),
            ((3, 2).into(), (3, 3).into()).into(),
            ((4, 2).into(), (4, 3).into()).into(),
            ((0, 0).into(), (0, 1).into()).into(),
            ((3, 1).into(), (3, 2).into()).into(),
        ];

        let sorted_correctly_drops: Vec<crate::Drop> = vec![
            ((0, 3).into(), (0, 4).into()).into(),
            ((0, 2).into(), (0, 3).into()).into(),
            ((2, 2).into(), (2, 4).into()).into(),
            ((1, 2).into(), (1, 4).into()).into(),
            ((3, 2).into(), (3, 3).into()).into(),
            ((4, 2).into(), (4, 3).into()).into(),
            ((0, 1).into(), (0, 2).into()).into(),
            ((4, 1).into(), (4, 2).into()).into(),
            ((1, 1).into(), (1, 3).into()).into(),
            ((2, 1).into(), (2, 3).into()).into(),
            ((3, 1).into(), (3, 2).into()).into(),
            ((1, 0).into(), (1, 2).into()).into(),
            ((4, 0).into(), (4, 1).into()).into(),
            ((3, 0).into(), (3, 1).into()).into(),
            ((2, 0).into(), (2, 2).into()).into(),
            ((0, 0).into(), (0, 1).into()).into(),
        ];
        drops.sort();
        assert_eq!(drops, sorted_correctly_drops);
    }
}
