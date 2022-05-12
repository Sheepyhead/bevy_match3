use crate::{board::*, mat::Matches};
use bevy::prelude::*;
use queues::{IsQueue, Queue};
use rand::{prelude::SliceRandom, thread_rng};
use std::fmt;

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

                    check_for_matches(&board, &mut events);
                }
                BoardCommand::Shuffle => {
                    let gems = board.gems.clone();
                    let mut values = gems.iter().collect::<Vec<_>>();
                    let mut moves =
                        Vec::with_capacity((board.dimensions.x * board.dimensions.y) as usize);
                    values.shuffle(&mut thread_rng());
                    for ((old_key, value), new_key) in values.iter().copied().zip(gems.keys()) {
                        board.insert(*new_key, *value);
                        moves.push((*old_key, *new_key));
                    }
                    events
                        .push(BoardEvent::Shuffled(moves))
                        .map_err(|err| println!("{err}"))
                        .unwrap();

                    check_for_matches(&board, &mut events);
                }
            }
        }
    }
}

fn check_for_matches(board: &ResMut<Board>, events: &mut ResMut<BoardEvents>) {
    let matches = board.get_matches();
    if !matches.is_empty() {
        events
            .push(BoardEvent::Matched(matches))
            .map_err(|err| println!("{err}"))
            .unwrap();
    }
}

/// The resource used to send commands to the logic board
#[derive(Default)]
pub struct BoardCommands(pub(crate) Queue<BoardCommand>);

impl BoardCommands {
    /// Pushes a new command onto the command queue
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_match3::prelude::*;
    ///
    /// fn example_system(
    ///     mut board_commands: ResMut<BoardCommands>,
    /// ) {
    ///     board_commands.push(BoardCommand::Swap(
    ///         [0, 0].into(),
    ///         [0, 1].into(),
    ///     ))
    ///     .unwrap();
    /// }
    /// ```
    pub fn push(&mut self, command: BoardCommand) -> Result<(), &str> {
        self.0.add(command).map(|_| ())
    }

    pub(crate) fn pop(&mut self) -> Result<BoardCommand, &str> {
        self.0.remove()
    }
}

/// The commands that can be issued to the logic board
#[derive(Clone)]
pub enum BoardCommand {
    /// Attempts to swap two gems, succeeds only if the swap would cause a match
    Swap(UVec2, UVec2),
    /// Pops all gems at the given positions, causing drops, spawns, and may cause matches to occur
    Pop(Vec<UVec2>),
    /// Shuffles all gems on the board, may result in matches
    Shuffle,
}

impl BoardEvents {
    pub(crate) fn push(&mut self, command: BoardEvent) -> Result<(), &str> {
        self.0.add(command).map(|_| ())
    }

    /// Removes an event from the event queue
    pub fn pop(&mut self) -> Result<BoardEvent, &str> {
        self.0.remove()
    }
}

/// The resource used to receive information about changes in the logic board
#[derive(Default)]
pub struct BoardEvents(pub(crate) Queue<BoardEvent>);

/// The events that indicate a possible change in the logic board
#[derive(Clone)]
pub enum BoardEvent {
    /// Two gems have been successfully swapped, usually as a result of a ``BoardCommand::Swap`` command
    Swapped(UVec2, UVec2),
    /// Two gems have failed to swap, this means no changes have been made to the logic board.
    ///
    /// This is usually as a result of a ``BoardCommand::Swap`` command
    FailedSwap(UVec2, UVec2),
    /// One or more gems have dropped from a higher position to a lower position, or in other words their
    /// position has changed from a lower y-value to a higher y-value with no change in x-value.
    /// These are ordered so that for each column the drop with the highest y-value on its from coordinate
    /// comes first, since we want to avoid overwriting other gems.
    Dropped(Vec<Drop>),
    /// A gem has been popped. This is usually as a result of a ``BoardCommand::Pop`` command
    Popped(UVec2),
    /// Gems have been spawned. This usually happens after a ``BoardEvent::Popped`` event
    Spawned(Vec<(UVec2, u32)>),
    /// Matches have been detected.
    Matched(Matches),
    /// The board has been shuffled, this is is the list of moves from .0 to .1
    Shuffled(Vec<(UVec2, UVec2)>),
}

/// Represents a gem dropping from a higher to a lower position
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Drop {
    /// The position the gem used to occupy
    pub from: UVec2,
    /// The new position the gem has dropped to
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
