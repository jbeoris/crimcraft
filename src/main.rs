use bevy::prelude::*;
use rand::prelude::*;
use std::collections::HashMap;

// Constants
const PLAYER_SPEED: f32 = 5.0;
const CRIM_SPEED: f32 = 3.5;
const BLOCK_SIZE: f32 = 1.0;
const WORLD_SIZE: i32 = 20;
const WORLD_HEIGHT: i32 = 10;

// Components
#[derive(Component)]
struct Player {
    has_pickaxe: bool,
}

#[derive(Component)]
struct Crim;

#[derive(Component)]
struct Block {
    block_type: BlockType,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum BlockType {
    Dirt,
    Stone,
    Wood,
}

#[derive(Component, Clone)]
struct Position {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Resource)]
struct GameWorld {
    blocks: HashMap<(i32, i32, i32), BlockType>,
}

// Systems
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_world: ResMut<GameWorld>,
) {
    // Spawn camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Spawn light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Generate world
    generate_world(&mut commands, &mut meshes, &mut materials, &mut game_world);

    // Spawn player with pickaxe
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.0, 1.0))),
        Transform::from_xyz(0.0, WORLD_HEIGHT as f32 + 1.0, 0.0),
        Player {
            has_pickaxe: true,
        },
    ));

    // Spawn Crim (the monster)
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::default())),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
        Transform::from_xyz(10.0, WORLD_HEIGHT as f32 + 1.0, 10.0),
        Crim,
    ));
}

fn generate_world(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    game_world: &mut ResMut<GameWorld>,
) {
    let mut rng = rand::rng();
    let cube_mesh = meshes.add(Cuboid::default());
    
    let dirt_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.3, 0.1),
        ..default()
    });
    let stone_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.5),
        ..default()
    });
    let wood_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.4, 0.2),
        ..default()
    });

    // Generate terrain
    for x in -WORLD_SIZE..WORLD_SIZE {
        for z in -WORLD_SIZE..WORLD_SIZE {
            let height = (rng.random::<f32>() * 3.0).floor() as i32;
            
            // Create ground layer
            for y in 0..=height {
                let block_type = if y == height {
                    BlockType::Dirt
                } else if rng.random_bool(0.8) {
                    BlockType::Stone
                } else {
                    BlockType::Dirt
                };
                
                game_world.blocks.insert((x, y, z), block_type);
                
                let material = match block_type {
                    BlockType::Dirt => dirt_material.clone(),
                    BlockType::Stone => stone_material.clone(),
                    BlockType::Wood => wood_material.clone(),
                };
                
                commands.spawn((
                    Mesh3d(cube_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_xyz(
                        x as f32 * BLOCK_SIZE,
                        y as f32 * BLOCK_SIZE,
                        z as f32 * BLOCK_SIZE,
                    ),
                    Block { block_type },
                    Position { x, y, z },
                ));
            }
        }
    }

    // Add some trees
    for _ in 0..10 {
        let x = rng.random_range(-WORLD_SIZE+2..WORLD_SIZE-2);
        let z = rng.random_range(-WORLD_SIZE+2..WORLD_SIZE-2);
        
        if let Some(base_height) = game_world.blocks.keys()
            .filter(|(bx, _, bz)| *bx == x && *bz == z)
            .map(|(_, by, _)| *by)
            .max()
        {
            // Tree trunk
            for y in base_height + 1..base_height + 6 {
                game_world.blocks.insert((x, y, z), BlockType::Wood);
                
                commands.spawn((
                    Mesh3d(cube_mesh.clone()),
                    MeshMaterial3d(wood_material.clone()),
                    Transform::from_xyz(
                        x as f32 * BLOCK_SIZE,
                        y as f32 * BLOCK_SIZE,
                        z as f32 * BLOCK_SIZE,
                    ),
                    Block { block_type: BlockType::Wood },
                    Position { x, y, z },
                ));
            }
        }
    }
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut player_transform = player_query.single_mut();
    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction.z -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.z += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Space) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        direction.y -= 1.0;
    }

    if direction != Vec3::ZERO {
        direction = direction.normalize();
        player_transform.translation += direction * PLAYER_SPEED * time.delta_secs();
    }
}

fn crim_ai(
    player_query: Query<&Transform, (With<Player>, Without<Crim>)>,
    mut crim_query: Query<&mut Transform, With<Crim>>,
    time: Res<Time>,
    game_world: Res<GameWorld>,
) {
    let player_transform = player_query.single();
    let mut crim_transform = crim_query.single_mut();
    
    let direction = (player_transform.translation - crim_transform.translation).normalize();
    
    // Check if there's a block between Crim and player
    let _player_pos = player_transform.translation;
    let crim_pos = crim_transform.translation;
    
    // Simple line of sight check
    let block_pos_x = (crim_pos.x / BLOCK_SIZE).round() as i32;
    let block_pos_y = (crim_pos.y / BLOCK_SIZE).round() as i32;
    let block_pos_z = (crim_pos.z / BLOCK_SIZE).round() as i32;
    
    let can_see_player = !game_world.blocks.contains_key(&(block_pos_x, block_pos_y, block_pos_z));
    
    if can_see_player {
        crim_transform.translation += direction * CRIM_SPEED * time.delta_secs();
    }
}

fn block_interaction(
    mouse_button: Res<ButtonInput<MouseButton>>,
    player_query: Query<(&Transform, &Player)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_world: ResMut<GameWorld>,
    blocks_query: Query<(Entity, &Position, &Block)>,
) {
    let (player_transform, player) = player_query.single();
    
    if !player.has_pickaxe {
        return;
    }
    
    // Mining blocks
    if mouse_button.just_pressed(MouseButton::Left) {
        // Find the closest block in front of the player
        let player_pos = player_transform.translation;
        let player_forward = player_transform.forward();
        
        let max_reach = 5.0;
        let mut closest_block = None;
        let mut closest_distance = max_reach;
        
        for (entity, position, _) in blocks_query.iter() {
            let block_pos = Vec3::new(
                position.x as f32 * BLOCK_SIZE,
                position.y as f32 * BLOCK_SIZE,
                position.z as f32 * BLOCK_SIZE,
            );
            
            let distance = player_pos.distance(block_pos);
            
            if distance < closest_distance {
                let to_block = (block_pos - player_pos).normalize();
                
                if player_forward.dot(to_block) > 0.7 {
                    closest_block = Some((entity, position.clone()));
                    closest_distance = distance;
                }
            }
        }
        
        if let Some((entity, position)) = closest_block {
            // Remove the block from the world
            game_world.blocks.remove(&(position.x, position.y, position.z));
            commands.entity(entity).despawn();
        }
    }
    
    // Placing blocks
    if mouse_button.just_pressed(MouseButton::Right) {
        // Find position to place a block
        let player_pos = player_transform.translation;
        let player_forward = player_transform.forward();
        
        let position = Vec3::new(
            (player_pos.x + player_forward.x * 2.0) / BLOCK_SIZE,
            (player_pos.y + player_forward.y * 2.0) / BLOCK_SIZE, 
            (player_pos.z + player_forward.z * 2.0) / BLOCK_SIZE,
        )
        .round();
        
        let block_pos = (position.x as i32, position.y as i32, position.z as i32);
        
        // Check if there's already a block at this position
        if !game_world.blocks.contains_key(&block_pos) {
            // Add a new block
            game_world.blocks.insert(block_pos, BlockType::Dirt);
            
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::default())),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.6, 0.3, 0.1),
                    ..default()
                })),
                Transform::from_xyz(
                    block_pos.0 as f32 * BLOCK_SIZE,
                    block_pos.1 as f32 * BLOCK_SIZE,
                    block_pos.2 as f32 * BLOCK_SIZE,
                ),
                Block { block_type: BlockType::Dirt },
                Position { x: block_pos.0, y: block_pos.1, z: block_pos.2 },
            ));
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(GameWorld {
            blocks: HashMap::new(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, (player_movement, crim_ai, block_interaction))
        .run();
}