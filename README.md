# CrimCraft

A simple Minecraft-like game built with Rust and Bevy where you mine and place blocks while avoiding a monster named Crim.

## Features

- 3D voxel-based world with randomly generated terrain
- Mine blocks with your pickaxe (left-click)
- Place blocks to build structures (right-click)
- WASD + Space/Shift movement controls
- Hide from Crim by building structures

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)

### Installation

1. Clone the repository
2. Build and run the game:

```bash
cd crimcraft
cargo run --release
```

## Controls

- W/A/S/D - Move
- Space - Move up
- Shift - Move down
- Left Mouse Button - Mine blocks
- Right Mouse Button - Place blocks

## How to Play

You start in a procedurally generated world with a pickaxe. Crim, a monster that will chase you, also spawns in the world. You need to mine blocks and use them to build structures to hide from Crim.

Crim can only chase you when you're in its line of sight, so use the blocks you mine to build walls and shelters to break line of sight.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Bevy](https://bevyengine.org/) - A refreshingly simple data-driven game engine
- Inspired by Minecraft and other voxel-based games