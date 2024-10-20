use bevy::prelude::*;
use rand::Rng;
use crate::collision::*;
use crate::cuscuta_resources::*;
use crate::player::*;
use crate::enemies::*;


#[derive(Resource)]
pub struct RoomManager {
    pub grids: Vec<Vec<Vec<u32>>>,
    pub current_room: usize,
    pub room_sizes: Vec<(f32, f32)>,
    pub max_sizes: Vec<(f32, f32)>,  
    pub z_index: f32,  
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            grids: Vec::new(),
            current_room: 0,
            room_sizes: Vec::new(),
            max_sizes: Vec::new(), 
            z_index: 0.0,
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

        // Set the z index for the new room and increment it by 5 for the next room
        self.z_index = self.z_index - 2.0;

        // Set the current room to the new one
        self.current_room = self.grids.len() - 1;
    }

    pub fn current_room_z_index(&self) -> f32 {
        self.z_index
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
    // sends pixel count & indexes
    room_manager.add_room(random_width, random_height, room_width, room_height);

    // set current room to newly added room
    room_manager.switch_room(room_manager.grids.len() - 1); 

    // max room bounds
    let max_x = room_width / 2.;
    let max_y = room_height / 2.;

    // get current room z index
    let z_index = room_manager.current_room_z_index();
    println!("spawn room z index: {}", z_index);

    // texture inputs
    let bg_texture_handle = asset_server.load("tiles/cobblestone_floor/cobblestone_floor.png");
    let north_wall_texture_handle = asset_server.load("tiles/walls/north_wall.png");
    let south_wall_handle = asset_server.load("tiles/walls/bottom_wall.png");
    let east_wall_handle = asset_server.load("tiles/walls/right_wall.png");
    let west_wall_handle = asset_server.load("tiles/walls/left_wall.png");
    let door_handle = asset_server.load("tiles/walls/black_void.png");

    // offset for spawning tiles
    let mut x_offset = -max_x + ((TILE_SIZE / 2) as f32);
    let mut y_offset = -max_y + ((TILE_SIZE / 2) as f32);

    // loop thru indexes
    while x_offset <= max_x {
        let mut xcoord: usize;
        let mut ycoord: usize;

        /* Spawn in north wall */
        commands.spawn((SpriteBundle {
            texture: north_wall_texture_handle.clone(),
            transform: Transform::from_xyz(x_offset, max_y - ((TILE_SIZE / 2) as f32), z_index),
            ..default()
        }, Wall, Room,));

        /* Spawn in south wall */
        commands.spawn((SpriteBundle {
            texture: south_wall_handle.clone(),
            transform: Transform::from_xyz(x_offset, -max_y + ((TILE_SIZE / 2) as f32), z_index),
            ..default()
        }, Wall, Room,));


        while y_offset <= max_y + (TILE_SIZE as f32) {
            /* East wall */
            commands.spawn((SpriteBundle {
                texture: east_wall_handle.clone(),
                transform: Transform::from_xyz(max_x - ((TILE_SIZE / 2) as f32), y_offset, z_index - 1.0),
                ..default()
            }, Wall, Room,));

            /* West wall */
            commands.spawn((SpriteBundle {
                texture: west_wall_handle.clone(),
                transform: Transform::from_xyz(-max_x + ((TILE_SIZE / 2) as f32), y_offset, z_index - 1.0),
                ..default()
            }, Wall, Room,));

            /* Floor tiles */
            commands.spawn(SpriteBundle {
                texture: bg_texture_handle.clone(),
                transform: Transform::from_xyz(x_offset, y_offset, z_index - 1.0),
                ..default()
            }).insert(Room).insert(Background);

            // door
            if (x_offset == max_x - (3.0 * (TILE_SIZE as f32) / 2.0)) && (y_offset == (TILE_SIZE as f32 / 2.0)) {
                commands.spawn((
                    SpriteBundle {
                        texture: door_handle.clone(),
                        transform: Transform::from_xyz(x_offset, y_offset, z_index - 1.0),
                        ..default()
                    },
                    Door, Room,
                ));

                xcoord = (max_x * 2. - (3 * TILE_SIZE / 2) as f32) as usize;
                ycoord = (y_offset + max_y) as usize;
                set_collide(room_manager, xcoord, ycoord, 2);

            }
            y_offset += TILE_SIZE as f32;
        }

        y_offset = -max_y + ((TILE_SIZE / 2) as f32);
        x_offset += TILE_SIZE as f32;
    }

}


pub fn generate_random_room(
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>,
    room_manager: &mut RoomManager,
) {
    let mut rng = rand::thread_rng();

    // generate random integers between 50 and 250, * 32
    let random_width = rng.gen_range(40..=80);
    let random_height = rng.gen_range(40..=80);

    // Room width & height as a multiple of 32
    // * 32d = pixel count
    let room_width = random_width as f32 * TILE_SIZE as f32;  
    let room_height = random_height as f32 * TILE_SIZE as f32;

    // // Add the room to room manager
    // // sends pixel count & indexes
    room_manager.add_room(random_width, random_height, room_width, room_height);
    let z_index = room_manager.current_room_z_index();
    println!("2nd room z index: {}", z_index);

    // max room bounds
    let max_x = room_width / 2.;
    let max_y = room_height / 2.;

    // texture inputs
    let bg_texture_handle = asset_server.load("tiles/solid_floor/solid_floor.png");
    let north_wall_texture_handle = asset_server.load("tiles/walls/north_wall.png");
    let south_wall_handle = asset_server.load("tiles/walls/bottom_wall.png");
    let east_wall_handle = asset_server.load("tiles/walls/right_wall.png");
    let west_wall_handle = asset_server.load("tiles/walls/left_wall.png");
    let door_handle = asset_server.load("tiles/walls/black_void.png");

    // offset for spawning tiles
    let mut x_offset = -max_x + ((TILE_SIZE / 2) as f32);
    let mut y_offset = -max_y + ((TILE_SIZE / 2) as f32);

    // loop thru indexes
    while x_offset < max_x {
        let mut xcoord: usize;
        let mut ycoord: usize;

         /* Spawn in north wall */
         commands.spawn((SpriteBundle {
            texture: north_wall_texture_handle.clone(),
            transform: Transform::from_xyz(x_offset, max_y - ((TILE_SIZE / 2) as f32), z_index),
            ..default()
        }, Wall, Room,));

        /* Spawn in south wall */
        commands.spawn((SpriteBundle {
            texture: south_wall_handle.clone(),
            transform: Transform::from_xyz(x_offset, -max_y + ((TILE_SIZE / 2) as f32), z_index),
            ..default()
        }, Wall, Room,));

        while y_offset < max_y + (TILE_SIZE as f32) {
            /* East wall */
            commands.spawn((SpriteBundle {
                texture: east_wall_handle.clone(),
                transform: Transform::from_xyz(max_x - ((TILE_SIZE / 2) as f32), y_offset, z_index - 1.0),
                ..default()
            }, Wall, Room,));

            /* West wall */
            commands.spawn((SpriteBundle {
                texture: west_wall_handle.clone(),
                transform: Transform::from_xyz(-max_x + ((TILE_SIZE / 2) as f32), y_offset, z_index - 1.0),
                ..default()
            }, Wall, Room,));

            /* Floor tiles */
            commands.spawn(SpriteBundle {
                texture: bg_texture_handle.clone(),
                transform: Transform::from_xyz(x_offset, y_offset, z_index - 1.0),
                ..default()
            }).insert(Room).insert(Background);

        
            y_offset += TILE_SIZE as f32;
        }

        y_offset = -max_y + ((TILE_SIZE / 2) as f32);
        x_offset += TILE_SIZE as f32;
    }

    // doors
    // Place the door in a similar way as in the start room
    let door_x = max_x - (3.0 * (TILE_SIZE as f32) / 2.0);
    let door_y = TILE_SIZE as f32 / 2.0;

    // Spawn the door
    commands.spawn((
        SpriteBundle {
            texture: door_handle.clone(),
            transform: Transform::from_xyz(door_x, door_y, z_index - 1.0),
            ..default()
        },
        Door, Room,
    ));

    // Set collision for the door in the room grid
    let xcoord = (max_x * 2. - (3 * TILE_SIZE / 2) as f32) as usize;
    let ycoord = (door_y + max_y) as usize;
    set_collide(room_manager, xcoord, ycoord, 2);
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

pub fn transition_map(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    room_manager: &mut RoomManager,
    mut room_query: Query<Entity, With<Room>>, 
    pt: &mut Transform,
) {
    // despawn old room
    for entity in room_query.iter_mut() {
        commands.entity(entity).despawn();
    }

    // generate a new room with the updated z-index
    generate_random_room(commands, &asset_server, room_manager);

    // adjust the player's position for the new room (if necessary)
    let new_pos: Vec3 = pt.translation + Vec3::new(-MAX_X * 1.9, 0., -1.0);  // Adjust the offset as needed
    pt.translation = new_pos;
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
