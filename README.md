# Bevy Match 3

[![crates.io](https://img.shields.io/crates/v/bevy_match3)](https://crates.io/crates/bevy_match3)
[![docs.rs](https://docs.rs/bevy_match3/badge.svg)](https://docs.rs/bevy_match3)
[![Crates.io](https://img.shields.io/crates/d/bevy_match3.svg)](https://crates.io/crates/bevy_match3)


<img src="example.gif" width="300" height="300" />

`bevy_match3` is a crate for handling the logic side of match 3 games in [Bevy](https://bevyengine.org/).

## Bevy Version Support
| `bevy` | `bevy_match3` |
|--------|---------------|
| 0.6    | 0.0.1         |
| 0.7    | 0.0.2         |
| 0.8    | 0.1.0         |
| 0.9    | 0.2.0         |
| 0.10   | N/A           |
| 0.11   | 0.3.0         |

## Features
- Configurable number of gem types and board dimensions
- Guaranteed no matches at board creation
- Cascading matches
- Check for matches when board is done moving
- Shuffle board

## Immediate todo
- [ ] Decouple board from plugin and make multiple boards example

## Possible todo based on demand
- [ ] Entities instead of u32 gem types
- [ ] More Match types
- [ ] Customizing various aspects like letting swaps succeed always and allowing matches at board creation

## Examples
To get started with this crate all you need is to set up the plugin
```rust
use bevy_match3::prelude::*;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Match3Plugin)
        .run();
}
```
React to board events 
```rust
fn consume_events(
    mut events: ResMut<BoardEvents>,
) {
    if let Ok(event) = events.pop() {
        match event {
            BoardEvent::Swapped(pos1, pos2) => todo!(),
            BoardEvent::FailedSwap(pos1, pos2) => todo!(),
            BoardEvent::Popped(pos) => todo!(),
            BoardEvent::Matched(matches) => todo!(),
            BoardEvent::Dropped(drops) => todo!(),
            BoardEvent::Spawned(spawns) => todo!(),
        }
    }
}
```
and start sending commands to the board using the `BoardCommands` resource!


For now there is one example of all features at [`basic.rs`](examples/basic.rs)

## License
All code in this repository is dual-licensed under either:

- MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

at your option. 

## Credits
- **Ilustragm** for their awesome gem icon pack used in the examples! https://ilustragm.itch.io/set-gems-icon-01