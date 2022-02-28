use bevy::{math::UVec2, utils::HashSet};

pub(crate) enum MatchDirection {
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

    pub(crate) fn append(&mut self, other: &mut Matches) {
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
