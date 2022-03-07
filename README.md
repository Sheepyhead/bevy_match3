# Bevy Match 3

[![crates.io](https://img.shields.io/crates/v/bevy_match3)](https://crates.io/crates/bevy_match3)
[![docs.rs](https://docs.rs/bevy_match3/badge.svg)](https://docs.rs/bevy_match3)

`bevy_match3` is a crate for handling the logic side of match 3 games in [Bevy](https://bevyengine.org/).

## Bevy Version Support
| `bevy` | `bevy_match3` |
| ------ | ------------- |
| 0.6    | 0.0.1           |

## Features
- Configurable number of gem types and board dimensions
- Guaranteed no matches at board creation
- Cascading matches

## Immediate todo
- [ ] Event for no matches
- [ ] Board shuffling
- [ ] Decouple board from plugin and make multiple boards example

## Possible todo based on demand
- [ ] Entities instead of u32 gem types
- [ ] More Match types
- [ ] Customizing various aspects like letting swaps succeed always and allowing matches at board creation

## Examples
For now there is one example of all features at [`scene.rs`](examples/scene.rs)

## License
Note that this project is licensed under the [`Anti-Capitalist Software License`](https://anticapitalist.software/). If this proves a major obstacle for adoption I may consider a more conventional license, I would just like to avoid this library being flipped by the likes of King and similar.

## Credits
- **Ilustragm** for their awesome gem icon pack used in the examples! https://ilustragm.itch.io/set-gems-icon-01