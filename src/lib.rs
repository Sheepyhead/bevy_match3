use bevy::{prelude::*, utils::HashMap};
use rand::Rng;

pub struct Match3Plugin;

impl Plugin for Match3Plugin {
    fn build(&self, app: &mut App) {
        let (dimensions, types) = app.world.get_resource::<Match3Config>().map_or(
            (UVec2::new(10, 10), 5),
            |Match3Config {
                 gem_types,
                 board_dimensions,
             }| (*board_dimensions, *gem_types),
        );

        let mut gems = HashMap::default();
        (0..dimensions.x).for_each(|x| {
            (0..dimensions.y).for_each(|y| {
                gems.insert(UVec2::new(x, y), rand::thread_rng().gen_range(0..types));
            })
        });

        app.insert_resource(Board { dimensions, gems });
    }
}

pub struct Match3Config {
    pub gem_types: u32,
    pub board_dimensions: UVec2,
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
                (0..self.dimensions.y).map(|y| self.gems[&UVec2::new(x, y)]).collect::<Vec<_>>()
            ))
        });
        for res in res {
            match res {
                Ok(_) => {},
                err => return err,
            }
        }
        Ok(())
    }
}
