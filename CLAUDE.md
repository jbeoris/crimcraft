# CrimCraft - Claude's Notes

## Project Context
This is a simple Minecraft-like game built with Rust and the Bevy game engine. The player can mine and place blocks while avoiding a monster named Crim.

## Development Commands
- Build the project: `cargo build`
- Run the project: `cargo run --release`
- Run with specific graphics backend: `WGPU_BACKEND=vulkan cargo run --release` (Linux/Mac) or set environment variable on Windows

## Recent Changes
- Fixed compilation errors with latest Bevy API
- Updated from Camera3dBundle to Camera3d
- Updated from DirectionalLightBundle to DirectionalLight 
- Updated from PbrBundle to use Mesh3d and MeshMaterial3d
- Changed time.delta_seconds() to time.delta_secs()
- Fixed random number generation methods to use current API

## Code Structure
- The game uses Bevy ECS (Entity Component System)
- Main components: Player, Crim, Block, Position
- Resources: GameWorld (contains block data)
- Systems: setup, generate_world, player_movement, crim_ai, block_interaction

## Game Mechanics
- WASD + Space/Shift for movement
- Left-click to mine blocks
- Right-click to place blocks
- Crim (monster) chases the player when in line of sight
- Player can break line of sight by building structures

## Known Issues
- When running in WSL, graphics support is limited and crashes can occur
- Running on native Windows or Linux with proper GPU drivers is recommended