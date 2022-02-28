use crate::board::*;
use bevy::prelude::*;

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
                BoardCommand::Pop(_) => todo!(),
            }
        }
    }
}
