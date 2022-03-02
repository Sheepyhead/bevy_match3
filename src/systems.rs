use crate::board::*;
use bevy::prelude::*;
use derive_deref::Deref;
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
                    } else {
                        events
                            .push(BoardEvent::FailedSwap(pos1, pos2))
                            .map_err(|err| println!("{err}"))
                            .unwrap();
                    };
                }
                BoardCommand::Pop(gems) => {
                    gems.iter().for_each(|gem| board.remove(gem));
                    board.drop().iter().for_each(|(from, to)| {
                        events
                            .push(BoardEvent::Dropped(*from, *to))
                            .map_err(|err| println!("{err}"))
                            .unwrap()
                    });
                    board.fill().iter().for_each(|(position, typ)| {
                        events
                            .push(BoardEvent::Spawned(*position, *typ))
                            .map_err(|err| println!("{err}"))
                            .unwrap()
                    })
                }
            }
        }
    }
}

#[derive(Deref, Default)]
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

#[derive(Deref, Default)]
pub struct BoardEvents(pub(crate) Queue<BoardEvent>);

#[derive(Clone, Copy)]
pub enum BoardEvent {
    Swapped(UVec2, UVec2),
    FailedSwap(UVec2, UVec2),
    Dropped(UVec2, UVec2),
    Spawned(UVec2, u32),
}
