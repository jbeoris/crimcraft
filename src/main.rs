use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use rand::prelude::*;
use std::collections::HashMap;

// Constants
const PLAYER_SPEED: f32 = 5.0;
const CRIM_SPEED: f32 = 3.5;
const BLOCK_SIZE: f32 = 1.0;
const WORLD_SIZE: i32 = 20;
const WORLD_HEIGHT: i32 = 10;
const MOUSE_SENSITIVITY: f32 = 0.002;

// Components
#[derive(Component)]
struct Player {
    has_pickaxe: bool,
    velocity: Vec3,
    is_grounded: bool,
    selected_block_type: BlockType,
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Crim {
    chase_timer: f32,
    spotted_player: bool,
}

#[derive(Component)]
struct ParticleEffect {
    lifetime: f32,
    velocity: Vec3,
    created: f32,
}

#[derive(Component)]
struct Block {
    block_type: BlockType,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum BlockType {
    Dirt,
    Stone,
    Wood,
    Grass,
    Sand,
    Water,
    Ore,
    Glass,
    Obsidian,
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

// Resource for tracking player stats
#[derive(Resource)]
struct PlayerStats {
    health: f32,
    max_health: f32,
    inventory: HashMap<BlockType, u32>,
}

// Resource for game UI
#[derive(Resource)]
struct GameUI {
    show_debug: bool,
    show_crosshair: bool,
}

// Resource for game settings
#[derive(Resource)]
struct GameSettings {
    render_distance: i32,
    gravity_enabled: bool,
}

// Systems
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_world: ResMut<GameWorld>,
) {
    // First-person camera will be attached to the player in the camera_follow system

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
            velocity: Vec3::ZERO,
            is_grounded: false,
            selected_block_type: BlockType::Dirt,
        },
    ))
    .with_children(|parent| {
        // Attach first-person camera to player
        parent.spawn((
            Camera3d::default(),
            MainCamera,
            Transform::from_xyz(0.0, 0.7, 0.0), // Position camera slightly above player center for eyes
        ));
        
        // Add player arms/tool model visible in first person
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.5))),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.6, 0.4))),
            Transform::from_xyz(0.3, -0.2, -0.4)
                .with_rotation(Quat::from_rotation_x(-0.3)),
        ));
    });

    // Spawn Crim (the monster)
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.0, 0.0),
            emissive: Color::srgb(0.8, 0.0, 0.0).into(),
            ..default()
        })),
        Transform::from_xyz(10.0, WORLD_HEIGHT as f32 + 1.0, 10.0),
        Crim {
            chase_timer: 0.0,
            spotted_player: false,
        },
    ))
    .with_children(|parent| {
        // Add glowing eyes
        parent.spawn((
            Mesh3d(meshes.add(Sphere::new(0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 0.3),
                emissive: Color::srgb(1.0, 1.0, 0.3).into(),
                ..default()
            })),
            Transform::from_xyz(0.2, 0.3, -0.3),
        ));
        
        parent.spawn((
            Mesh3d(meshes.add(Sphere::new(0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 0.3),
                emissive: Color::srgb(1.0, 1.0, 0.3).into(),
                ..default()
            })),
            Transform::from_xyz(-0.2, 0.3, -0.3),
        ));
    });
}

fn generate_world(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    game_world: &mut ResMut<GameWorld>,
) {
    let mut rng = thread_rng();
    let cube_mesh = meshes.add(Cuboid::default());
    
    // Create materials for all block types
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
    let grass_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.7, 0.2),
        ..default()
    });
    let sand_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.8, 0.5),
        ..default()
    });
    let water_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.4, 0.8),
        alpha_mode: AlphaMode::Blend,
        metallic: 0.0,
        perceptual_roughness: 0.1,
        reflectance: 0.5,
        ..default()
    });
    let ore_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.2, 0.6),
        metallic: 0.7,
        perceptual_roughness: 0.1,
        ..default()
    });
    let glass_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.9, 1.0),
        alpha_mode: AlphaMode::Blend,
        metallic: 0.0,
        perceptual_roughness: 0.0,
        reflectance: 0.5,
        ..default()
    });
    let obsidian_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.2),
        metallic: 0.2,
        perceptual_roughness: 0.1,
        ..default()
    });

    // Generate terrain
    for x in -WORLD_SIZE..WORLD_SIZE {
        for z in -WORLD_SIZE..WORLD_SIZE {
            let height = (rng.gen::<f32>() * 3.0).floor() as i32;
            
            // Create ground layer with more varied terrain
            for y in 0..=height {
                // More varied terrain generation
                let block_type = if y == height && height > 0 {
                    if rng.gen_bool(0.6) { BlockType::Grass } else { BlockType::Dirt }
                } else if y == height && height == 0 {
                    if rng.gen_bool(0.7) { BlockType::Sand } else { BlockType::Dirt }
                } else if y == 0 {
                    if rng.gen_bool(0.05) { BlockType::Obsidian } else { BlockType::Stone }
                } else if y < height - 2 && rng.gen_bool(0.05) {
                    BlockType::Ore
                } else if rng.gen_bool(0.8) {
                    BlockType::Stone
                } else {
                    BlockType::Dirt
                };
                
                // Create water pools in low areas
                let is_water_level = height < 1 && y == 1 && rng.gen_bool(0.4);
                let final_block_type = if is_water_level { BlockType::Water } else { block_type };
                
                game_world.blocks.insert((x, y, z), final_block_type);
                
                let material = match final_block_type {
                    BlockType::Dirt => dirt_material.clone(),
                    BlockType::Stone => stone_material.clone(),
                    BlockType::Wood => wood_material.clone(),
                    BlockType::Grass => grass_material.clone(),
                    BlockType::Sand => sand_material.clone(),
                    BlockType::Water => water_material.clone(),
                    BlockType::Ore => ore_material.clone(),
                    BlockType::Glass => glass_material.clone(),
                    BlockType::Obsidian => obsidian_material.clone(),
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

    // Add some trees and structures
    for _ in 0..20 {
        let x = rng.gen_range(-WORLD_SIZE+2..WORLD_SIZE-2);
        let z = rng.gen_range(-WORLD_SIZE+2..WORLD_SIZE-2);
        
        if let Some(base_height) = game_world.blocks.keys()
            .filter(|(bx, _, bz)| *bx == x && *bz == z)
            .map(|(_, by, _)| *by)
            .max()
        {
            // Decide what to generate - trees or small structures
            let structure_type = rng.gen_range(0..10);
            
            match structure_type {
                // Trees (70% chance)
                0..=6 => {
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
                    
                    // Tree leaves
                    let tree_top = base_height + 5;
                    for leaf_x in x-2..=x+2 {
                        for leaf_z in z-2..=z+2 {
                            // Skip corners for a more natural shape
                            if (leaf_x == x-2 || leaf_x == x+2) && (leaf_z == z-2 || leaf_z == z+2) {
                                continue;
                            }
                            
                            for leaf_y in tree_top-1..=tree_top+1 {
                                // Skip if there's already a block here
                                if game_world.blocks.contains_key(&(leaf_x, leaf_y, leaf_z)) {
                                    continue;
                                }
                                
                                // Add leaf block
                                game_world.blocks.insert((leaf_x, leaf_y, leaf_z), BlockType::Grass);
                                
                                commands.spawn((
                                    Mesh3d(cube_mesh.clone()),
                                    MeshMaterial3d(grass_material.clone()),
                                    Transform::from_xyz(
                                        leaf_x as f32 * BLOCK_SIZE,
                                        leaf_y as f32 * BLOCK_SIZE,
                                        leaf_z as f32 * BLOCK_SIZE,
                                    ),
                                    Block { block_type: BlockType::Grass },
                                    Position { x: leaf_x, y: leaf_y, z: leaf_z },
                                ));
                            }
                        }
                    }
                },
                
                // Stone pillar (20% chance)
                7..=8 => {
                    let height = rng.gen_range(4..8);
                    for y in base_height + 1..base_height + height {
                        game_world.blocks.insert((x, y, z), BlockType::Stone);
                        
                        commands.spawn((
                            Mesh3d(cube_mesh.clone()),
                            MeshMaterial3d(stone_material.clone()),
                            Transform::from_xyz(
                                x as f32 * BLOCK_SIZE,
                                y as f32 * BLOCK_SIZE,
                                z as f32 * BLOCK_SIZE,
                            ),
                            Block { block_type: BlockType::Stone },
                            Position { x, y, z },
                        ));
                        
                        // Add some obsidian at the top
                        if y == base_height + height - 1 {
                            game_world.blocks.insert((x, y+1, z), BlockType::Obsidian);
                            
                            commands.spawn((
                                Mesh3d(cube_mesh.clone()),
                                MeshMaterial3d(obsidian_material.clone()),
                                Transform::from_xyz(
                                    x as f32 * BLOCK_SIZE,
                                    (y+1) as f32 * BLOCK_SIZE,
                                    z as f32 * BLOCK_SIZE,
                                ),
                                Block { block_type: BlockType::Obsidian },
                                Position { x, y: y+1, z },
                            ));
                        }
                    }
                },
                
                // Glass tower (10% chance)
                9 => {
                    let height = rng.gen_range(3..6);
                    for y in base_height + 1..base_height + height {
                        game_world.blocks.insert((x, y, z), BlockType::Glass);
                        
                        commands.spawn((
                            Mesh3d(cube_mesh.clone()),
                            MeshMaterial3d(glass_material.clone()),
                            Transform::from_xyz(
                                x as f32 * BLOCK_SIZE,
                                y as f32 * BLOCK_SIZE,
                                z as f32 * BLOCK_SIZE,
                            ),
                            Block { block_type: BlockType::Glass },
                            Position { x, y, z },
                        ));
                    }
                },
                
                _ => {}
            }
        }
    }
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<MainCamera>)>,
    camera_query: Query<&Transform, With<MainCamera>>,
    time: Res<Time>,
) {
    let mut player_transform = player_query.single_mut();
    let camera_transform = camera_query.single();
    
    let mut direction = Vec3::ZERO;
    let forward = camera_transform.forward();
    let right = camera_transform.right();
    
    // Get forward/backward movement in the XZ plane
    let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let right_xz = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction += forward_xz;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction -= forward_xz;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction -= right_xz;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction += right_xz;
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
    mut crim_query: Query<(&mut Transform, &mut Crim)>,
    time: Res<Time>,
    game_world: Res<GameWorld>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_transform = player_query.single();
    let (mut crim_transform, mut crim) = crim_query.single_mut();
    
    let to_player = player_transform.translation - crim_transform.translation;
    let distance = to_player.length();
    let direction = to_player.normalize();
    
    // Make Crim face the player
    if distance < 20.0 {
        let target_rotation = Quat::from_rotation_arc(Vec3::Z, direction);
        crim_transform.rotation = crim_transform.rotation.slerp(target_rotation, time.delta_secs() * 2.0);
    }
    
    // Advanced line of sight check with raycasting
    let crim_pos = crim_transform.translation;
    let player_pos = player_transform.translation;
    
    // Step from Crim to player position checking for blocks
    let ray_direction = (player_pos - crim_pos).normalize();
    let ray_length = (player_pos - crim_pos).length();
    let ray_steps = (ray_length / 0.5).ceil() as i32; // Check every 0.5 units
    
    let mut can_see_player = true;
    
    // Simple raycast implementation
    'ray_check: for i in 1..ray_steps {
        let check_pos = crim_pos + ray_direction * (i as f32 * 0.5);
        let block_pos = (
            (check_pos.x / BLOCK_SIZE).floor() as i32,
            (check_pos.y / BLOCK_SIZE).floor() as i32,
            (check_pos.z / BLOCK_SIZE).floor() as i32,
        );
        
        if game_world.blocks.contains_key(&block_pos) {
            // Ray hit a block, can't see player
            can_see_player = false;
            break 'ray_check;
        }
    }
    
    // Crim behavior logic
    if can_see_player && distance < 15.0 {
        // Just spotted player
        if !crim.spotted_player {
            crim.spotted_player = true;
            
            // Emit particles when spotting player
            for _ in 0..5 {
                let random_dir = Vec3::new(
                    rng.gen::<f32>() * 2.0 - 1.0,
                    rng.gen::<f32>() * 2.0 - 1.0,
                    rng.gen::<f32>() * 2.0 - 1.0,
                ).normalize();
                
                commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.1))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(1.0, 0.3, 0.0),
                        emissive: Color::srgb(1.0, 0.3, 0.0).into(),
                        ..default()
                    })),
                    Transform::from_translation(crim_pos + Vec3::new(0.0, 0.5, 0.0)),
                    ParticleEffect {
                        lifetime: 1.0,
                        velocity: random_dir * 2.0,
                        created: 0.0,
                    },
                ));
            }
        }
        
        // Chase player
        crim_transform.translation += direction * CRIM_SPEED * time.delta_secs();
        crim.chase_timer = 3.0; // Continue chasing for 3 seconds after losing sight
    } else if crim.chase_timer > 0.0 {
        // Continue chasing for a bit even if player is out of sight
        crim_transform.translation += direction * CRIM_SPEED * 0.7 * time.delta_secs();
        crim.chase_timer -= time.delta_secs();
        
        if crim.chase_timer <= 0.0 {
            crim.spotted_player = false;
        }
    } else {
        // Lost sight of player
        crim.spotted_player = false;
    }
}

fn block_interaction(
    mouse_button: Res<ButtonInput<MouseButton>>,
    player_query: Query<&Transform, With<Player>>,
    camera_query: Query<&Transform, With<MainCamera>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_world: ResMut<GameWorld>,
    blocks_query: Query<(Entity, &Position, &Block)>,
    player_components: Query<&Player>,
) {
    let player = player_components.single();
    
    if !player.has_pickaxe {
        return;
    }
    
    let player_transform = player_query.single();
    let camera_transform = camera_query.single();
    
    // Mining blocks
    if mouse_button.just_pressed(MouseButton::Left) {
        // Use camera position and direction for better aiming
        let camera_pos = camera_transform.translation + player_transform.translation;
        let camera_forward = camera_transform.forward();
        
        // Add mining animation by spawning particles
        commands.spawn((
            Mesh3d(meshes.add(RegularPolygon::new(0.05, 3))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 0.8, 0.0))),
            Transform::from_translation(camera_pos + camera_forward * 1.5),
            ParticleEffect { 
                lifetime: 0.5,
                velocity: camera_forward * 0.5, 
                created: 0.0,
            },
        ));
        
        let max_reach = 5.0;
        let mut closest_block = None;
        let mut closest_distance = max_reach;
        
        for (entity, position, _) in blocks_query.iter() {
            let block_pos = Vec3::new(
                position.x as f32 * BLOCK_SIZE,
                position.y as f32 * BLOCK_SIZE,
                position.z as f32 * BLOCK_SIZE,
            );
            
            let distance = camera_pos.distance(block_pos);
            
            if distance < closest_distance {
                let to_block = (block_pos - camera_pos).normalize();
                
                if camera_forward.dot(to_block) > 0.7 {
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
        // Use camera for block placement
        let camera_pos = camera_transform.translation + player_transform.translation;
        let camera_forward = camera_transform.forward();
        
        // Raycast to find where to place the block
        let ray_start = camera_pos;
        let ray_end = camera_pos + camera_forward * 5.0;
        
        // Simple raycast implementation for block placement
        let mut ray_pos = ray_start;
        let ray_step = camera_forward * 0.1;
        let mut block_to_place = None;
        let mut last_empty_pos = None;
        
        for _ in 0..50 {  // 50 steps for max reach of 5.0
            let block_x = (ray_pos.x / BLOCK_SIZE).floor() as i32;
            let block_y = (ray_pos.y / BLOCK_SIZE).floor() as i32;
            let block_z = (ray_pos.z / BLOCK_SIZE).floor() as i32;
            
            let block_pos = (block_x, block_y, block_z);
            
            // If we hit a block, we want to place in the last empty position
            if game_world.blocks.contains_key(&block_pos) {
                block_to_place = last_empty_pos;
                break;
            }
            
            last_empty_pos = Some(block_pos);
            ray_pos += ray_step;
        }
        
        if let Some(block_pos) = block_to_place {
            // Check if there's already a block at this position
            if !game_world.blocks.contains_key(&block_pos) {
                // Add a new block of the selected type
                game_world.blocks.insert(block_pos, player.selected_block_type);
                
                // Use player's selected block type
                let material = match player.selected_block_type {
                    BlockType::Dirt => materials.add(StandardMaterial {
                        base_color: Color::srgb(0.6, 0.3, 0.1),
                        ..default()
                    }),
                    BlockType::Stone => materials.add(StandardMaterial {
                        base_color: Color::srgb(0.5, 0.5, 0.5),
                        ..default()
                    }),
                    BlockType::Wood => materials.add(StandardMaterial {
                        base_color: Color::srgb(0.6, 0.4, 0.2),
                        ..default()
                    }),
                    BlockType::Grass => materials.add(StandardMaterial {
                        base_color: Color::srgb(0.3, 0.7, 0.2),
                        ..default()
                    }),
                    BlockType::Sand => materials.add(StandardMaterial {
                        base_color: Color::srgb(0.9, 0.8, 0.5),
                        ..default()
                    }),
                    BlockType::Water => materials.add(StandardMaterial {
                        base_color: Color::srgb(0.2, 0.4, 0.8),
                        alpha_mode: AlphaMode::Blend,
                        metallic: 0.0,
                        perceptual_roughness: 0.1,
                        reflectance: 0.5,
                        ..default()
                    }),
                    BlockType::Ore => materials.add(StandardMaterial {
                        base_color: Color::srgb(0.4, 0.2, 0.6),
                        metallic: 0.7,
                        perceptual_roughness: 0.1,
                        ..default()
                    }),
                    BlockType::Glass => materials.add(StandardMaterial {
                        base_color: Color::srgb(0.8, 0.9, 1.0),
                        alpha_mode: AlphaMode::Blend,
                        metallic: 0.0,
                        perceptual_roughness: 0.0,
                        reflectance: 0.5,
                        ..default()
                    }),
                    BlockType::Obsidian => materials.add(StandardMaterial {
                        base_color: Color::srgb(0.1, 0.1, 0.2),
                        metallic: 0.2,
                        perceptual_roughness: 0.1,
                        ..default()
                    }),
                };
                
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::default())),
                    MeshMaterial3d(material),
                    Transform::from_xyz(
                        block_pos.0 as f32 * BLOCK_SIZE,
                        block_pos.1 as f32 * BLOCK_SIZE,
                        block_pos.2 as f32 * BLOCK_SIZE,
                    ),
                    Block { block_type: player.selected_block_type },
                    Position { x: block_pos.0, y: block_pos.1, z: block_pos.2 },
                ));
            }
        }
    }
}

fn camera_control(
    mut windows: Query<&mut Window>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<MainCamera>>,
) {
    let mut rotation_move = Vec2::ZERO;
    
    // Get mouse delta
    for event in mouse_motion_events.read() {
        rotation_move += event.delta;
    }
    
    if rotation_move.length_squared() > 0.0 {
        let mut transform = query.single_mut();
        
        // Apply mouse delta as camera rotation (pitch and yaw)
        let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        
        // Increment yaw and pitch based on mouse movement
        yaw -= rotation_move.x * MOUSE_SENSITIVITY;
        pitch -= rotation_move.y * MOUSE_SENSITIVITY;
        
        // Clamp pitch to avoid camera flipping
        pitch = pitch.clamp(-1.5, 1.5);
        
        // Apply rotations
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
        
        // Grab cursor for continuous rotation
        let mut window = windows.single_mut();
        window.cursor_options.grab_mode = bevy::window::CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

// Apply gravity and handle collisions
fn physics_system(
    time: Res<Time>,
    game_world: Res<GameWorld>,
    mut player_query: Query<(&mut Transform, &mut Player)>,
    game_settings: Res<GameSettings>,
) {
    if !game_settings.gravity_enabled {
        return;
    }

    let gravity = Vec3::new(0.0, -9.8, 0.0);
    let (mut transform, mut player) = player_query.single_mut();
    
    // Apply gravity to velocity
    player.velocity += gravity * time.delta_secs();
    
    // Apply velocity to position
    let potential_position = transform.translation + player.velocity * time.delta_secs();
    
    // Check for collisions
    let block_pos_x = (potential_position.x / BLOCK_SIZE).floor() as i32;
    let block_pos_y = (potential_position.y / BLOCK_SIZE).floor() as i32;
    let block_pos_z = (potential_position.z / BLOCK_SIZE).floor() as i32;
    
    // Check if we're inside a block
    let collision = game_world.blocks.contains_key(&(block_pos_x, block_pos_y, block_pos_z));
    
    // Check if we're standing on a block (grounded)
    let block_below = game_world.blocks.contains_key(&(block_pos_x, block_pos_y - 1, block_pos_z));
    
    // Handle collision response
    if collision {
        // Stop falling
        player.velocity.y = 0.0;
    } else {
        // Apply velocity
        transform.translation = potential_position;
    }
    
    // Update grounded state
    player.is_grounded = block_below;
    
    // Dampen velocity if grounded
    if player.is_grounded && player.velocity.y <= 0.0 {
        player.velocity.y = 0.0;
    }
}

// Switch between block types
fn block_selection_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Player>,
) {
    let mut player = player_query.single_mut();
    
    // Number keys to select different block types
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        player.selected_block_type = BlockType::Dirt;
    } else if keyboard_input.just_pressed(KeyCode::Digit2) {
        player.selected_block_type = BlockType::Stone;
    } else if keyboard_input.just_pressed(KeyCode::Digit3) {
        player.selected_block_type = BlockType::Wood;
    } else if keyboard_input.just_pressed(KeyCode::Digit4) {
        player.selected_block_type = BlockType::Grass;
    } else if keyboard_input.just_pressed(KeyCode::Digit5) {
        player.selected_block_type = BlockType::Sand;
    } else if keyboard_input.just_pressed(KeyCode::Digit6) {
        player.selected_block_type = BlockType::Glass;
    } else if keyboard_input.just_pressed(KeyCode::Digit7) {
        player.selected_block_type = BlockType::Obsidian;
    } else if keyboard_input.just_pressed(KeyCode::Digit8) {
        player.selected_block_type = BlockType::Ore;
    } else if keyboard_input.just_pressed(KeyCode::Digit9) {
        player.selected_block_type = BlockType::Water;
    }
}

// Display UI
fn ui_system(
    mut commands: Commands,
    game_ui: Res<GameUI>,
    player_query: Query<&Player>,
    mut contexts: EguiContexts,
) {
    if game_ui.show_debug {
        let player = player_query.single();
        let ctx = contexts.ctx_mut();
        
        egui::Window::new("Debug Info").show(ctx, |ui| {
            ui.label(format!("Selected Block: {:?}", player.selected_block_type));
            ui.label(format!("Is Grounded: {}", player.is_grounded));
            ui.label(format!("Velocity: {:?}", player.velocity));
        });
    }
    
    if game_ui.show_crosshair {
        // Add crosshair in the center of the screen using Bevy UI
        let ctx = contexts.ctx_mut();
        
        // Simple crosshair in the center
        let screen_size = ctx.screen_rect().size();
        let center = egui::pos2(screen_size.x / 2.0, screen_size.y / 2.0);
        
        ctx.debug_painter().circle_filled(
            center, 
            2.0, 
            egui::Color32::WHITE
        );
    }
}

// System to animate and despawn particles
fn particle_system(
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Transform, &mut ParticleEffect)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle) in particles.iter_mut() {
        // Update age
        if particle.created == 0.0 {
            particle.created = time.elapsed_secs();
        }
        
        let age = time.elapsed_secs() - particle.created;
        
        // Apply velocity
        transform.translation += particle.velocity * time.delta_secs();
        
        // Add some gravity effect to particles
        particle.velocity.y -= 2.0 * time.delta_secs();
        
        // Remove particle if it's too old
        if age > particle.lifetime {
            commands.entity(entity).despawn();
        }
    }
}

// Setup a skybox and ambient lighting
fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Add ambient light
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.6, 0.6, 0.9),
        brightness: 0.3,
    });
    
    // Add a distant directional light for sun effect
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.8),
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 50.0, 10.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CrimCraft - Minecraft-like Demo".to_string(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(bevy_egui::EguiPlugin)
        .insert_resource(GameWorld {
            blocks: HashMap::new(),
        })
        .insert_resource(PlayerStats {
            health: 100.0,
            max_health: 100.0,
            inventory: HashMap::new(),
        })
        .insert_resource(GameUI {
            show_debug: true,
            show_crosshair: true,
        })
        .insert_resource(GameSettings {
            render_distance: 10,
            gravity_enabled: true,
        })
        .add_systems(Startup, (setup, setup_environment))
        .add_systems(Update, (
            player_movement,
            crim_ai,
            block_interaction,
            camera_control,
            physics_system,
            block_selection_system,
            ui_system,
            particle_system
        ))
        .run();
}