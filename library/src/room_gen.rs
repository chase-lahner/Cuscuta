use bevy::prelude::*;
use rand::Rng;
use crate::collision::*;
use crate::cuscuta_resources::*;
use crate::player::*;
use crate::enemies::*;


// enum to represent different door types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorType {
    Right,
    Left,
    Top,
    Bottom,
}

#[derive(Resource)]
pub struct RoomManager {
    pub grids: Vec<Vec<Vec<u32>>>,
    pub current_room: usize,
    pub room_sizes: Vec<(f32, f32)>,
    pub max_sizes: Vec<(f32, f32)>,  
    // z of room that player is currently in
    pub current_z_index: f32,  
    // z of room that was most recently generated (used so we can backtrack w/o screwing everything up)
    pub global_z_index: f32,  
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            grids: Vec::new(),
            current_room: 0,
            room_sizes: Vec::new(),
            max_sizes: Vec::new(), 
            current_z_index: 0.0,
            global_z_index: 0.0,
        }
    }

    // add new grid for new room 
    pub fn add_room(&mut self, width: usize, height: usize, room_width: f32, room_height: f32) {
        let new_grid = vec![vec![0; height]; width];
        self.grids.push(new_grid);
        self.room_sizes.push((room_width, room_height));
        
        // Calculate and store the max_x and max_y based on room size
        let max_x = room_width / 2.0;
        let max_y = room_height / 2.0;
        self.max_sizes.push((max_x, max_y));

        // Set the current room to the new one
        self.current_room = self.grids.len() - 1;
    }

    pub fn current_room_z_index(&self) -> f32 {
        self.current_z_index
    }
    // Get the Z-index for the next room
    pub fn next_room_z_index(&mut self) -> f32 {
        self.global_z_index -= 2.0; // Always decrement the global Z by 2 for a new room
        self.global_z_index
    }


    // Get mutable reference to the current grid
    pub fn current_grid(&mut self) -> &mut Vec<Vec<u32>> {
    &mut self.grids[self.current_room]
    }
    
    // Get the size of the current room (width, height)
    pub fn current_room_size(&self) -> (f32, f32) {
        self.room_sizes[self.current_room]
    }

    pub fn switch_room(&mut self, room_index: usize) {
        if room_index < self.grids.len() {
            self.current_room = room_index;
        }
    }

    pub fn current_room_max(&self) -> (f32, f32) {
        self.max_sizes[self.current_room]
    }
}

#[derive(Component)]
pub struct Door {
    pub next: Option<f32>,
    pub door_type: DoorType,
}



#[derive(Component)]
pub struct Room;

pub fn spawn_start_room(
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>,
    room_manager: &mut RoomManager,
) {
    let mut rng = rand::thread_rng();

    // generate random integers between 50 and 250, * 32
    let random_width = rng.gen_range(40..=40);
    let random_height = rng.gen_range(40..=40);
    // Room width & height as a multiple of 32
    // * 32d = pixel count
    let room_width = random_width as f32 * TILE_SIZE as f32;  
    let room_height = random_height as f32 * TILE_SIZE as f32;

    // Add the room to room manager
    room_manager.add_room(random_width, random_height, room_width, room_height);
    room_manager.switch_room(room_manager.grids.len() - 1); 

    // max room bounds
    let max_x = room_width / 2.0;
    let max_y = room_height / 2.0;

    // get current room z index
    let z_index = room_manager.current_room_z_index();

    // texture inputs
    let bg_texture_handle = asset_server.load("tiles/solid_floor/solid_floor.png");
    let north_wall_texture_handle = asset_server.load("tiles/walls/north_wall.png");
    let south_wall_handle = asset_server.load("tiles/walls/bottom_wall.png");
    let east_wall_handle = asset_server.load("tiles/walls/right_wall.png");
    let west_wall_handle = asset_server.load("tiles/walls/left_wall.png");

    // offset for spawning tiles
    let mut x_offset = -max_x + ((TILE_SIZE / 2) as f32);
    let mut y_offset = -max_y + ((TILE_SIZE / 2) as f32);

    // spawn floors & walls
    while x_offset < max_x {
        let xcoord: usize = ((x_offset + max_x) / TILE_SIZE as f32).floor() as usize;

         /* Spawn in north wall */
         commands.spawn((
            SpriteBundle {
                texture: north_wall_texture_handle.clone(),
                transform: Transform::from_xyz(x_offset, max_y - ((TILE_SIZE / 2) as f32), z_index),
                ..default()
            }, 
            Wall, 
            Room,
        ));
        set_collide(room_manager, xcoord, (max_y / TILE_SIZE as f32).floor() as usize, 1);

        /* Spawn in south wall */
        commands.spawn((
            SpriteBundle {
                texture: south_wall_handle.clone(),
                transform: Transform::from_xyz(x_offset, -max_y + ((TILE_SIZE / 2) as f32), z_index),
                ..default()
            }, 
            Wall, 
            Room,
        ));
        set_collide(room_manager, xcoord, (-max_y / TILE_SIZE as f32).floor() as usize, 1);

        while y_offset < max_y + (TILE_SIZE as f32) {
            let ycoord: usize = ((y_offset + max_y) / TILE_SIZE as f32).floor() as usize;

            /* East wall */
            commands.spawn((
                SpriteBundle {
                    texture: east_wall_handle.clone(),
                    transform: Transform::from_xyz(max_x - ((TILE_SIZE / 2) as f32), y_offset, z_index - 0.1),
                    ..default()
                }, 
                Wall, 
                Room,
            ));
            set_collide(room_manager, (max_x / TILE_SIZE as f32).floor() as usize, ycoord, 1);

            /* West wall */
            commands.spawn((
                SpriteBundle {
                    texture: west_wall_handle.clone(),
                    transform: Transform::from_xyz(-max_x + ((TILE_SIZE / 2) as f32), y_offset, z_index - 0.2),
                    ..default()
                }, 
                Wall, 
                Room,
            ));
            set_collide(room_manager, (-max_x / TILE_SIZE as f32).floor() as usize, ycoord, 1);

            /* Floor tiles */
            commands.spawn(SpriteBundle {
                texture: bg_texture_handle.clone(),
                transform: Transform::from_xyz(x_offset, y_offset, z_index - 0.3),
                ..default()
            }).insert(Room).insert(Background);

            y_offset += TILE_SIZE as f32;
        }

        y_offset = -max_y + ((TILE_SIZE / 2) as f32);
        x_offset += TILE_SIZE as f32;
    }

    generate_doors(
        commands,
        asset_server,
        room_manager,
        max_x,
        max_y,
        z_index,
    );
}


pub fn generate_random_room(
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>,
    room_manager: &mut RoomManager,
) {
    let (room_width, room_height, max_x, max_y, z_index) = generate_room_boundaries(room_manager);

    // Generate walls and floors
    generate_walls_and_floors(
        commands,
        asset_server,
        room_width,
        room_height,
        max_x,
        max_y,
        z_index,
    );

    // Generate doors
    generate_doors(
        commands,
        asset_server,
        room_manager,
        max_x,
        max_y,
        z_index,
    );
}

/// Generates random room boundaries and adds the room to the room manager.
/// Returns the room width, room height, max x, max y, and z-index.
fn generate_room_boundaries(
    room_manager: &mut RoomManager
) -> (f32, f32, f32, f32, f32) {
    let mut rng = rand::thread_rng();

    // Generate random width and height between 40 and 80 tiles
    let random_width = rng.gen_range(40..=80);
    let random_height = rng.gen_range(40..=80);

    // Convert to pixel sizes
    let room_width = random_width as f32 * TILE_SIZE as f32;
    let room_height = random_height as f32 * TILE_SIZE as f32;

    // Add the room to the room manager
    room_manager.add_room(random_width, random_height, room_width, room_height);

    // Get z-index for this room
    let z_index = room_manager.next_room_z_index();

    // Calculate maximum x and y coordinates (room boundaries)
    let max_x = room_width / 2.0;
    let max_y = room_height / 2.0;

    (room_width, room_height, max_x, max_y, z_index)
}

/// Generates walls and floors for the room.
fn generate_walls_and_floors(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    room_width: f32,
    room_height: f32,
    max_x: f32,
    max_y: f32,
    z_index: f32,
) {
    let bg_texture_handle = asset_server.load("tiles/solid_floor/solid_floor.png");
    let north_wall_texture_handle = asset_server.load("tiles/walls/north_wall.png");
    let south_wall_handle = asset_server.load("tiles/walls/bottom_wall.png");
    let east_wall_handle = asset_server.load("tiles/walls/right_wall.png");
    let west_wall_handle = asset_server.load("tiles/walls/left_wall.png");

    // Offset for spawning tiles
    let mut x_offset = -max_x + ((TILE_SIZE / 2) as f32);
    let mut y_offset = -max_y + ((TILE_SIZE / 2) as f32);

    // Spawn walls and floors
    while x_offset < max_x {
        /* Spawn north and south walls */
        commands.spawn((
            SpriteBundle {
                texture: north_wall_texture_handle.clone(),
                transform: Transform::from_xyz(x_offset, max_y - ((TILE_SIZE / 2) as f32), z_index),
                ..default()
            },
            Wall,
            Room,
        ));
        commands.spawn((
            SpriteBundle {
                texture: south_wall_handle.clone(),
                transform: Transform::from_xyz(x_offset, -max_y + ((TILE_SIZE / 2) as f32), z_index),
                ..default()
            },
            Wall,
            Room,
        ));

        /* Spawn east and west walls */
        while y_offset < max_y + (TILE_SIZE as f32) {
            commands.spawn((
                SpriteBundle {
                    texture: east_wall_handle.clone(),
                    transform: Transform::from_xyz(max_x - ((TILE_SIZE / 2) as f32), y_offset, z_index - 0.1),
                    ..default()
                },
                Wall,
                Room,
            ));
            commands.spawn((
                SpriteBundle {
                    texture: west_wall_handle.clone(),
                    transform: Transform::from_xyz(-max_x + ((TILE_SIZE / 2) as f32), y_offset, z_index - 0.2),
                    ..default()
                },
                Wall,
                Room,
            ));

            /* Spawn floor tiles */
            commands.spawn((
                SpriteBundle {
                    texture: bg_texture_handle.clone(),
                    transform: Transform::from_xyz(x_offset, y_offset, z_index - 0.3),
                    ..default()
                },
                Room,
                Background,
            ));

            y_offset += TILE_SIZE as f32;
        }

        // Reset y_offset for the next column
        y_offset = -max_y + ((TILE_SIZE / 2) as f32);
        x_offset += TILE_SIZE as f32;
    }
}

/// Generates doors for the room and sets up their collisions.
fn generate_doors(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    room_manager: &mut RoomManager,
    max_x: f32,
    max_y: f32,
    z_index: f32,
) {
    let door_handle = asset_server.load("tiles/walls/black_void.png");

    // Right door
    let door_x = max_x - (3.0 * (TILE_SIZE as f32) / 2.0) + TILE_SIZE as f32;
    let door_y = TILE_SIZE as f32 / 2.0;  
    commands.spawn((
        SpriteBundle {
            texture: door_handle.clone(),
            transform: Transform::from_xyz(door_x, door_y, z_index + 0.4),
            ..default()
        },
        Door {
            next: Some(room_manager.global_z_index),
            door_type: DoorType::Right,
        },
        Room,
    ));
    
    let xcoord_right = ((max_x * 2.0 - (3.0 * TILE_SIZE as f32 / 2.0)) + TILE_SIZE as f32) as usize;
    let ycoord_right = (door_y + max_y) as usize;
    set_collide(room_manager, xcoord_right, ycoord_right, 2);

    // Left door
    let door_left_x = -max_x + (3.0 * TILE_SIZE as f32 / 2.0) - TILE_SIZE as f32;
    let door_left_y = TILE_SIZE as f32 / 2.0;
    commands.spawn((
        SpriteBundle {
            texture: door_handle.clone(),
            transform: Transform::from_xyz(door_left_x, door_left_y, z_index + 0.4),
            ..default()
        },
        Door {
            next: Some(room_manager.global_z_index),
            door_type: DoorType::Left,
        },
        Room,
    ));
    let xcoord_left = ((-max_x * 2.0 + (3.0 * TILE_SIZE as f32 / 2.0)) - TILE_SIZE as f32) as usize;
    let ycoord_left = (door_left_y + max_y) as usize;
    set_collide(room_manager, xcoord_left, ycoord_left, 2);

    // Top door
    let door_top_x = TILE_SIZE as f32 / 2.0;
    let door_top_y = max_y - (3.0 * TILE_SIZE as f32 / 2.0) + TILE_SIZE as f32;
    commands.spawn((
        SpriteBundle {
            texture: door_handle.clone(),
            transform: Transform::from_xyz(door_top_x, door_top_y, z_index + 0.4),
            ..default()
        },
        Door {
            next: Some(room_manager.global_z_index),
            door_type: DoorType::Top,
        },
        Room,
    ));
    let xcoord_top = (door_top_x + max_x) as usize;
    let ycoord_top = ((max_y * 2.0 - (3.0 * TILE_SIZE as f32 / 2.0)) + TILE_SIZE as f32) as usize;
    set_collide(room_manager, xcoord_top, ycoord_top, 2);

    // Bottom door
    let door_bottom_x = TILE_SIZE as f32 / 2.0;
    let door_bottom_y = -max_y + (3.0 * TILE_SIZE as f32 / 2.0) - TILE_SIZE as f32;
    commands.spawn((
        SpriteBundle {
            texture: door_handle.clone(),
            transform: Transform::from_xyz(door_bottom_x, door_bottom_y, z_index + 0.4),
            ..default()
        },
        Door {
            next: Some(room_manager.global_z_index),
            door_type: DoorType::Bottom,
        },
        Room,
    ));
    let xcoord_bottom = (door_bottom_x + max_x) as usize;
    let ycoord_bottom = ((-max_y * 2.0 - (3.0 * TILE_SIZE as f32 / 2.0)) - TILE_SIZE as f32) as usize;
    set_collide(room_manager, xcoord_bottom, ycoord_bottom, 2);
}

pub fn transition_map(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    room_manager: &mut RoomManager,
    mut room_query: Query<Entity, With<Room>>, 
    door_query: Query<(&Transform, &Door), (Without<Player>, Without<Enemy>)>,  
    pt: &mut Transform,
    door_type: DoorType,  // Accept the door type as an argument
) {
    // Despawn old room
    for entity in room_query.iter_mut() {
        commands.entity(entity).despawn();
    }

    // Generate a new room with the updated z-index
    generate_random_room(commands, &asset_server, room_manager);

    let max_x = room_manager.current_room_max().0;
    let max_y = room_manager.current_room_max().1;

    // Adjust the player's position based on the door they entered
    match door_type {
        DoorType::Right => {
            // Spawn the player a little away from the left door to avoid getting stuck
            pt.translation = Vec3::new(-max_x + TILE_SIZE as f32 * 2.0, TILE_SIZE as f32 / 2.0, room_manager.current_z_index);
        },
        DoorType::Left => {
            // Spawn the player a little away from the right door
            pt.translation = Vec3::new(max_x - TILE_SIZE as f32 * 2.0, TILE_SIZE as f32 / 2.0, room_manager.current_z_index);
        },
        DoorType::Top => {
            // Spawn the player a little below the top door
            pt.translation = Vec3::new(TILE_SIZE as f32 / 2.0, -max_y + TILE_SIZE as f32 * 2.0, room_manager.current_z_index);
        },
        DoorType::Bottom => {
            // Spawn the player a little above the bottom door
            pt.translation = Vec3::new(TILE_SIZE as f32 / 2.0, max_y - TILE_SIZE as f32 * 2.0, room_manager.current_z_index);
        },
    }
}

pub fn translate_coords_to_grid(aabb: &Aabb, room_manager: &mut RoomManager) -> (u32, u32, u32, u32){
    // get the current room's grid size
    let current_grid = room_manager.current_grid();
    let room_width = current_grid.len() as f32 * TILE_SIZE as f32;
    let room_height = current_grid[0].len() as f32 * TILE_SIZE as f32;

    let max_x = room_width / 2.0;
    let max_y = room_height / 2.0;

    // Calculate the grid indices for the player's bounding box corners
    let arr_x_max = ((aabb.max.x + max_x) / TILE_SIZE as f32).floor().clamp(0., (current_grid.len() - 1) as f32);
    let arr_x_min = ((aabb.min.x + max_x) / TILE_SIZE as f32).floor().clamp(0., (current_grid.len() - 1) as f32);
    let arr_y_max = ((aabb.max.y + max_y) / TILE_SIZE as f32).floor().clamp(0., (current_grid[0].len() - 1) as f32);
    let arr_y_min = ((aabb.min.y + max_y) / TILE_SIZE as f32).floor().clamp(0., (current_grid[0].len() - 1) as f32);

    let topleft = current_grid[arr_x_min as usize][arr_y_max as usize];
    let topright = current_grid[arr_x_max as usize][arr_y_max as usize];
    let bottomleft = current_grid[arr_x_min as usize][arr_y_min as usize];
    let bottomright = current_grid[arr_x_max as usize][arr_y_min as usize];

    (topleft, topright, bottomleft, bottomright)
}


pub fn client_spawn_pot(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>
){
    let pot_handle = asset_server.load("tiles/pot.png");
    commands.spawn((
        SpriteBundle{
            texture: pot_handle,
            transform: Transform::from_xyz(200.,200.,1.),
            ..default()
        },
        Pot{
            touch: 0
        }
    ));
}
