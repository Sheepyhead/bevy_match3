use bevy::{
    prelude::*,
    utils::{hashbrown::hash_map, HashMap, HashSet},
};
use rand::prelude::IteratorRandom;

use crate::mat::*;

/// The main struct representing the logical match 3 board
#[derive(Eq, PartialEq, Debug, Clone, Resource)]
pub struct Board {
    pub(crate) dimensions: UVec2,
    pub(crate) gems: HashMap<UVec2, u32>,
    pub(crate) types: HashSet<u32>,
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = (0..self.dimensions.y).map(|y| {
            f.write_fmt(format_args!(
                "{:?}\n",
                (0..self.dimensions.x)
                    .map(|x| self.gems[&<[u32; 2] as Into<UVec2>>::into([x, y])])
                    .collect::<Vec<u32>>()
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

impl From<Vec<Vec<u32>>> for Board {
    fn from(rows: Vec<Vec<u32>>) -> Self {
        let mut gems = HashMap::default();
        let mut width = 0;
        let mut height = 0;
        let mut types = HashSet::default();
        rows.iter().enumerate().for_each(|(y, row)| {
            height += 1;
            row.iter().enumerate().for_each(|(x, gem)| {
                gems.insert([x as u32, y as u32].into(), *gem);
                types.insert(*gem);
                if height == 1 {
                    width += 1;
                }
            })
        });
        Board {
            gems,
            dimensions: [width, height].into(),
            types,
        }
    }
}

impl Board {
    /// Returns a reference to the gem type at the given position.
    pub fn get(&self, pos: &UVec2) -> Option<&u32> {
        self.gems.get(pos)
    }

    /// Returns an iterator over the kvps in the board
    pub fn iter(&self) -> hash_map::Iter<UVec2, u32> {
        self.gems.iter()
    }

    pub(crate) fn remove(&mut self, pos: &UVec2) {
        self.gems.remove(pos);
    }

    pub(crate) fn insert(&mut self, pos: UVec2, typ: u32) {
        self.gems.insert(pos, typ);
    }

    pub(crate) fn drop(&mut self) -> HashSet<(UVec2, UVec2)> {
        let mut moves = HashSet::default();
        for x in 0..self.dimensions.x {
            for y in (0..self.dimensions.y).rev() {
                if self.get(&[x, y].into()).is_none() {
                    let mut offset = 0;
                    for above in (0..y).rev() {
                        if let Some(typ) = self.get(&[x, above].into()).cloned() {
                            let new_pos = [x, y - offset];
                            moves.insert(([x, above].into(), new_pos.into()));
                            self.remove(&[x, above].into());
                            self.insert(new_pos.into(), typ);
                            offset += 1;
                        }
                    }
                }
            }
        }
        moves
    }

    pub(crate) fn fill(&mut self) -> HashSet<(UVec2, u32)> {
        let mut drops = HashSet::default();
        for x in 0..self.dimensions.x {
            for y in 0..self.dimensions.y {
                let pos = [x, y];
                if self.get(&pos.into()).is_none() {
                    let new_type = self
                        .types
                        .iter()
                        .choose(&mut rand::thread_rng())
                        .copied()
                        .unwrap();
                    self.insert(pos.into(), new_type);
                    drops.insert((pos.into(), new_type));
                }
            }
        }
        drops
    }

    pub(crate) fn swap(&mut self, pos1: &UVec2, pos2: &UVec2) -> Result<(), SwapError> {
        let gem1 = self.get(pos1).copied().ok_or(SwapError::NoGem(*pos1))?;
        let gem2 = self.get(pos2).copied().ok_or(SwapError::NoGem(*pos2))?;
        self.gems.insert(*pos1, gem2);
        self.gems.insert(*pos2, gem1);
        if self.get_matches().is_empty() {
            self.gems.insert(*pos1, gem1);
            self.gems.insert(*pos2, gem2);
            Err(SwapError::NoMatches)
        } else {
            Ok(())
        }
    }

    /// Like swap but doesn't permanently change the board, useful for match checking
    fn try_swap(&mut self, pos1: &UVec2, pos2: &UVec2) -> Result<(), SwapError> {
        let gem1 = self.get(pos1).copied().ok_or(SwapError::NoGem(*pos1))?;
        let gem2 = self.get(pos2).copied().ok_or(SwapError::NoGem(*pos2))?;
        self.gems.insert(*pos1, gem2);
        self.gems.insert(*pos2, gem1);
        if self.get_matches().is_empty() {
            self.gems.insert(*pos1, gem1);
            self.gems.insert(*pos2, gem2);
            Err(SwapError::NoMatches)
        } else {
            self.gems.insert(*pos1, gem1);
            self.gems.insert(*pos2, gem2);
            Ok(())
        }
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
                        _ => matches.add(Match::Straight(current_match.iter().cloned().collect())),
                    }
                    current_match = vec![pos];
                    previous_type = Some(current_type);
                }
            }
            match current_match.len() {
                0 | 1 | 2 => {}
                _ => matches.add(Match::Straight(current_match.iter().cloned().collect())),
            }
            current_match = vec![];
            previous_type = None;
        }
        matches
    }

    pub(crate) fn clear_matches(&mut self) {
        loop {
            let matches = self.get_matches();
            if matches.is_empty() {
                break;
            }
            for mat in matches.matches.iter() {
                match mat {
                    Match::Straight(gems) => {
                        for gem in gems {
                            self.remove(gem);
                        }
                    }
                }
            }
            self.drop();
            self.fill();
        }
    }

    fn adjacents(&self, pos: UVec2) -> Vec<UVec2> {
        let mut adjacents = Vec::with_capacity(4);
        if pos.x != 0 {
            adjacents.push(pos.left());
        }
        if pos.x != self.dimensions.x {
            adjacents.push(pos.right());
        }
        if pos.y != 0 {
            adjacents.push(pos.up());
        }
        if pos.y != self.dimensions.y {
            adjacents.push(pos.down());
        }
        adjacents
    }

    /// Returns any moves that would result in a match by swapping with a neighboring gem
    pub fn get_matching_moves(&self) -> HashSet<BoardMove> {
        let mut moves = HashSet::new();
        let mut temp_board = self.clone(); // NOTE: This clone is not ideal. First candidate for optimizing
        for (pos, _) in self.iter() {
            for adjacent in self.adjacents(*pos) {
                if temp_board.try_swap(pos, &adjacent).is_ok() {
                    moves.insert(BoardMove(*pos, adjacent));
                }
            }
        }
        moves
    }
}

pub(crate) enum SwapError {
    NoGem(UVec2),
    NoMatches,
}

/// Represents a swap between two gems, order of gems doesn't matter
#[derive(Eq, Debug)]
pub struct BoardMove(pub UVec2, pub UVec2);

impl PartialEq for BoardMove {
    fn eq(&self, other: &Self) -> bool {
        (self.0 == other.0 && self.1 == other.1) || (self.0 == other.1 && self.1 == other.0)
    }
}

impl core::hash::Hash for BoardMove {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let Self(a, b) = self;
        if a.x < b.x {
            a.hash(state);
            b.hash(state);
        } else if b.x < a.x {
            b.hash(state);
            a.hash(state);
        } else if a.y < b.y {
            a.hash(state);
            b.hash(state);
        } else {
            b.hash(state);
            a.hash(state);
        }
    }
}

trait BoardPosition {
    fn left(&self) -> Self;
    fn right(&self) -> Self;
    fn up(&self) -> Self;
    fn down(&self) -> Self;
    fn cardinally_adjacent(&self, other: &Self) -> bool;
}

impl BoardPosition for UVec2 {
    fn left(&self) -> Self {
        Self::new(self.x.saturating_sub(1), self.y)
    }

    fn right(&self) -> Self {
        Self::new(self.x.saturating_add(1), self.y)
    }

    fn up(&self) -> Self {
        Self::new(self.x, self.y.saturating_sub(1))
    }

    fn down(&self) -> Self {
        Self::new(self.x, self.y.saturating_add(1))
    }

    fn cardinally_adjacent(&self, other: &Self) -> bool {
        self == &other.left()
            || self == &other.right()
            || self == &other.up()
            || self == &other.down()
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::{UVec2, Vec2};
    use bevy::utils::HashSet;

    use crate::{mat::Matches, prelude::BoardMove, Board};

    impl Matches {
        fn len(&self) -> usize {
            self.matches.len()
        }
    }

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

        assert!(without_duplicates.contains::<UVec2>(&[2, 2].into()));
        assert!(without_duplicates.contains::<UVec2>(&[2, 3].into()));
        assert!(without_duplicates.contains::<UVec2>(&[2, 4].into()));
        assert!(without_duplicates.contains::<UVec2>(&[3, 0].into()));
        assert!(without_duplicates.contains::<UVec2>(&[3, 1].into()));
        assert!(without_duplicates.contains::<UVec2>(&[3, 2].into()));
        assert!(without_duplicates.contains::<UVec2>(&[1, 4].into()));
        assert!(without_duplicates.contains::<UVec2>(&[1, 5].into()));
        assert!(without_duplicates.contains::<UVec2>(&[1, 6].into()));
        assert!(without_duplicates.contains::<UVec2>(&[4, 4].into()));
        assert!(without_duplicates.contains::<UVec2>(&[4, 5].into()));
        assert!(without_duplicates.contains::<UVec2>(&[4, 6].into()));
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

        assert!(without_duplicates.contains::<UVec2>(&[1, 0].into()));
        assert!(without_duplicates.contains::<UVec2>(&[2, 0].into()));
        assert!(without_duplicates.contains::<UVec2>(&[3, 0].into()));
        assert!(without_duplicates.contains::<UVec2>(&[0, 2].into()));
        assert!(without_duplicates.contains::<UVec2>(&[1, 2].into()));
        assert!(without_duplicates.contains::<UVec2>(&[2, 2].into()));
        assert!(without_duplicates.contains::<UVec2>(&[2, 5].into()));
        assert!(without_duplicates.contains::<UVec2>(&[3, 5].into()));
        assert!(without_duplicates.contains::<UVec2>(&[4, 5].into()));
        assert!(without_duplicates.contains::<UVec2>(&[0, 6].into()));
        assert!(without_duplicates.contains::<UVec2>(&[1, 6].into()));
        assert!(without_duplicates.contains::<UVec2>(&[2, 6].into()));
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

        assert!(without_duplicates.contains::<UVec2>(&[3, 1].into()));
        assert!(without_duplicates.contains::<UVec2>(&[3, 2].into()));
        assert!(without_duplicates.contains::<UVec2>(&[3, 3].into()));
        assert!(without_duplicates.contains::<UVec2>(&[0, 2].into()));
        assert!(without_duplicates.contains::<UVec2>(&[1, 2].into()));
        assert!(without_duplicates.contains::<UVec2>(&[2, 2].into()));
        assert!(without_duplicates.contains::<UVec2>(&[0, 4].into()));
        assert!(without_duplicates.contains::<UVec2>(&[0, 5].into()));
        assert!(without_duplicates.contains::<UVec2>(&[0, 6].into()));
        assert!(without_duplicates.contains::<UVec2>(&[2, 5].into()));
        assert!(without_duplicates.contains::<UVec2>(&[3, 5].into()));
        assert!(without_duplicates.contains::<UVec2>(&[4, 5].into()));
    }

    #[test]
    fn check_bigger_matches() {
        #[rustfmt::skip]
        let board: Board = vec![
            vec![ 0,  1,  2,  3,  4],
            vec![ 5,  6,  7,  8,  9],
            vec![10, 11, 12, 13, 14],
            vec![18, 18, 18, 18, 18],
            vec![20, 21, 22, 23, 24],
            vec![25, 26, 27, 28, 29],
            vec![30, 31, 32, 33, 34],
        ].into();

        let matches = board.get_matches();

        assert_eq!(matches.len(), 1);

        let without_duplicates = matches.without_duplicates();

        assert!(without_duplicates.contains::<UVec2>(&[0, 3].into()));
        assert!(without_duplicates.contains::<UVec2>(&[1, 3].into()));
        assert!(without_duplicates.contains::<UVec2>(&[2, 3].into()));
        assert!(without_duplicates.contains::<UVec2>(&[3, 3].into()));
        assert!(without_duplicates.contains::<UVec2>(&[4, 3].into()));
    }

    #[test]
    fn pop_gem() {
        #[rustfmt::skip]
        let mut board: Board = vec![
            vec![ 0,  1,  2,  3,  4],
            vec![ 5,  6,  7,  8,  9],
            vec![11, 11, 11,  8, 14],
            vec![15, 16, 17,  8, 19],
            vec![20, 21, 22, 23, 24],
            vec![20, 26, 29, 29, 29],
            vec![20, 31, 32, 33, 34],
        ].into();

        board.remove(&[1, 4].into());

        assert!(board.get(&[1, 4].into()).is_none());
    }

    #[test]
    fn pop_gem_and_drop() {
        #[rustfmt::skip]
        let mut board: Board = vec![
            vec![ 0,  1,  2,  3,  4],
            vec![ 5,  6,  7,  8,  9],
            vec![11, 11, 11,  8, 14],
            vec![15, 16, 17,  8, 19],
            vec![20, 21, 22, 23, 24],
            vec![20, 26, 29, 29, 29],
            vec![20, 31, 32, 33, 34],
        ].into();

        board.remove(&[1, 4].into());
        assert!(board.get(&[1, 4].into()).is_none());

        let moves = board.drop();
        assert_eq!(moves.len(), 4);
        assert!(moves.contains(&([1, 3].into(), [1, 4].into())));
        assert!(moves.contains(&([1, 2].into(), [1, 3].into())));
        assert!(moves.contains(&([1, 1].into(), [1, 2].into())));
        assert!(moves.contains(&([1, 0].into(), [1, 1].into())));
    }

    #[test]
    fn pop_multiple_gems_and_drop() {
        #[rustfmt::skip]
        let mut board: Board = vec![
            vec![ 0,  1,  2,  3,  4],
            vec![ 5,  6,  7,  8,  9],
            vec![11, 11, 11,  8, 14],
            vec![15, 16, 17,  8, 19],
            vec![20, 21, 22, 23, 24],
            vec![20, 26, 29, 29, 29],
            vec![20, 31, 32, 33, 34],
        ].into();

        board.remove(&[0, 2].into());
        assert!(board.get(&[0, 2].into()).is_none());

        board.remove(&[0, 4].into());
        assert!(board.get(&[0, 4].into()).is_none());

        board.remove(&[0, 6].into());
        assert!(board.get(&[0, 6].into()).is_none());

        board.remove(&[4, 1].into());
        assert!(board.get(&[4, 1].into()).is_none());

        let moves = board.drop();
        assert_eq!(moves.len(), 5);
        assert!(moves.contains(&([0, 5].into(), [0, 6].into())));
        assert!(moves.contains(&([0, 3].into(), [0, 5].into())));
        assert!(moves.contains(&([0, 1].into(), [0, 4].into())));
        assert!(moves.contains(&([0, 0].into(), [0, 3].into())));
        assert!(moves.contains(&([4, 0].into(), [4, 1].into())));
    }

    #[test]
    fn check_simple_move() {
        #[rustfmt::skip]
        let board: Board = vec![
            vec![ 0,  1,  2,  3,  4],
            vec![ 5,  6,  7,  8,  9],
            vec![10, 11,  8, 13,  8],
            vec![15, 16, 17, 18, 19],
            vec![20, 21, 22, 23, 24],
            vec![25, 26, 27, 28, 29],
            vec![30, 31, 32, 33, 34],
        ].into();

        let matching_moves = board.get_matching_moves();

        assert_eq!(matching_moves.len(), 1);
        assert!(matching_moves.contains(&BoardMove(UVec2::new(3, 1), UVec2::new(3, 2))))
    }

    #[test]
    fn check_multiple_simple_moves() {
        #[rustfmt::skip]
        let board: Board = vec![
            vec![ 0,  1,  2,  3,  4],
            vec![ 5,  0, 13,  8, 13],
            vec![ 0, 11, 12, 13, 14],
            vec![15, 16, 17, 18, 19],
            vec![20, 21, 22, 23, 24],
            vec![25, 26, 30, 28, 24],
            vec![30, 30, 32, 24, 34],
        ].into();

        let matching_moves = board.get_matching_moves();

        assert_eq!(matching_moves.len(), 4);
        assert!(matching_moves.contains(&BoardMove(UVec2::new(0, 1), UVec2::new(1, 1))));
        assert!(matching_moves.contains(&BoardMove(UVec2::new(3, 1), UVec2::new(3, 2))));
        assert!(matching_moves.contains(&BoardMove(UVec2::new(2, 5), UVec2::new(2, 6))));
        assert!(matching_moves.contains(&BoardMove(UVec2::new(3, 6), UVec2::new(4, 6))));
    }

    #[test]
    fn check_advanced_moves() {
        #[rustfmt::skip]
        let board: Board = vec![
            vec![ 0,  6,  2,  3,  4],
            vec![ 5,  6,  7,  8,  9],
            vec![ 6, 11,  6, 13, 14],
            vec![15,  6, 17, 18, 19],
            vec![20, 21, 25, 23, 24],
            vec![25, 25, 27, 25, 25],
            vec![30, 31, 25, 33, 34],
        ].into();

        let matching_moves = board.get_matching_moves();

        assert_eq!(matching_moves.len(), 8);
        assert!(matching_moves.contains(&BoardMove(UVec2::new(0, 2), UVec2::new(1, 2))));
        assert!(matching_moves.contains(&BoardMove(UVec2::new(1, 1), UVec2::new(1, 2))));
        assert!(matching_moves.contains(&BoardMove(UVec2::new(1, 3), UVec2::new(1, 2))));
        assert!(matching_moves.contains(&BoardMove(UVec2::new(2, 2), UVec2::new(1, 2))));
        assert!(matching_moves.contains(&BoardMove(UVec2::new(2, 4), UVec2::new(2, 5))));
        assert!(matching_moves.contains(&BoardMove(UVec2::new(2, 6), UVec2::new(2, 5))));
        assert!(matching_moves.contains(&BoardMove(UVec2::new(1, 5), UVec2::new(2, 5))));
        assert!(matching_moves.contains(&BoardMove(UVec2::new(3, 5), UVec2::new(2, 5))));
    }
}
