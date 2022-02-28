use bevy::{prelude::*, utils::HashMap};
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
                gems.insert(UVec2::new(x, y), rand::thread_rng().gen_range(0..gem_types));
            })
        });

        app.insert_resource(Board {
            dimensions: board_dimensions,
            gems,
        });
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
            board_dimensions: UVec2::new(10, 10),
        }
    }
}

pub struct Board {
    dimensions: UVec2,
    gems: HashMap<UVec2, u32>,
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = (0..self.dimensions.x).map(|x| {
            f.write_fmt(format_args!(
                "{:?}\n",
                (0..self.dimensions.y)
                    .map(|y| self.gems[&UVec2::new(x, y)])
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
