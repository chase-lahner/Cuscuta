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
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            grids: Vec::new(),
            current_room: 0,
            room_sizes: Vec::new(),
            max_sizes: Vec::new(), 

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

        self.current_room = self.grids.len() - 1;
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

pub fn spawn_start_room(
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>,
    room_manager: &mut RoomManager,
) {
    let mut rng = rand::thread_rng();

    // generate random integers between 50 and 250, * 32
    let random_width = rng.gen_range(50..=250);
    let random_height = rng.gen_range(50..=250);
    println!("{}",random_width);
    println!("{}",random_height);
    // Room width & height as a multiple of 32
    let room_width = random_width as f32 * TILE_SIZE as f32;  
    let room_height = random_height as f32 * TILE_SIZE as f32;

    let arr_w = (room_width / TILE_SIZE as f32) as usize;
    let arr_h = (room_height / TILE_SIZE as f32) as usize;

    // Add the room and switch to it
    room_manager.add_room(arr_w, arr_h, room_width, room_height);
    room_manager.switch_room(room_manager.grids.len() - 1); 

    let max_x = room_width / 2.;
    let max_y = room_height / 2.;

    // add collision grid for start room
    room_manager.add_room((room_width / TILE_SIZE as f32) as usize,
     (room_height / TILE_SIZE as f32) as usize, room_width, room_height);

    let bg_texture_handle = asset_server.load("tiles/cobblestone_floor/cobblestone_floor.png");
    let north_wall_texture_handle = asset_server.load("tiles/walls/north_wall.png");
    let south_wall_handle = asset_server.load("tiles/walls/bottom_wall.png");
    let east_wall_handle = asset_server.load("tiles/walls/right_wall.png");
    let west_wall_handle = asset_server.load("tiles/walls/left_wall.png");
    let door_handle = asset_server.load("tiles/walls/black_void.png");

    let mut x_offset = -max_x + ((TILE_SIZE / 2) as f32);
    let mut y_offset = -max_y + ((TILE_SIZE / 2) as f32);


    while x_offset < max_x {


        let mut _xcoord: usize;
        let mut _ycoord: usize;

        /* Spawn in north wall */
        commands.spawn((SpriteBundle {
            texture: north_wall_texture_handle.clone(),
            transform: Transform::from_xyz(x_offset, max_y - ((TILE_SIZE / 2) as f32), 1.),
            ..default()
        }, Wall));

        _xcoord = (x_offset - ((TILE_SIZE / 2) as f32) + max_x) as usize;
        _ycoord = (max_y * 2. - ((TILE_SIZE / 2) as f32)) as usize;
        //set_collide(room_manager, xcoord, ycoord, 1);

        /* Spawn in south wall */
        commands.spawn((SpriteBundle {
            texture: south_wall_handle.clone(),
            transform: Transform::from_xyz(x_offset, -max_y + ((TILE_SIZE / 2) as f32), 1.),
            ..default()
        }, Wall));

        _xcoord = (x_offset - ((TILE_SIZE / 2) as f32) + MAX_X) as usize;
        _ycoord = (0) as usize;
        //set_collide(room_manager, xcoord, ycoord, 1);

        while y_offset < max_y + (TILE_SIZE as f32) {

            /* East wall */
            commands.spawn((SpriteBundle {
                texture: east_wall_handle.clone(),
                transform: Transform::from_xyz(max_x - ((TILE_SIZE / 2) as f32), y_offset, 1.),
                ..default()
            }, Wall));

            _xcoord = (MAX_X * 2. - ((TILE_SIZE / 2) as f32)) as usize;
            _ycoord = (y_offset - ((TILE_SIZE / 2) as f32) + MAX_Y - 1.) as usize;
            //set_collide(room_manager, xcoord, ycoord, 1);

            /* West wall */
            commands.spawn((SpriteBundle {
                texture: west_wall_handle.clone(),
                transform: Transform::from_xyz(-max_x + ((TILE_SIZE / 2) as f32), y_offset, 1.),
                ..default()
            }, Wall));

            _xcoord = (0) as usize;
            _ycoord = (y_offset - ((TILE_SIZE / 2) as f32) + MAX_Y - 1.) as usize;
            //set_collide(room_manager, xcoord, ycoord, 1);

            /* Floor tiles */
            commands.spawn(SpriteBundle {
                texture: bg_texture_handle.clone(),
                transform: Transform::from_xyz(x_offset, y_offset, 0.),
                ..default()
            }).insert(Background);

            // door
            if (x_offset == max_x - (3.0 * (TILE_SIZE as f32) / 2.0)) && (y_offset == (TILE_SIZE as f32 / 2.0)) {
                commands.spawn((
                    SpriteBundle {
                        texture: door_handle.clone(),
                        transform: Transform::from_xyz(x_offset, y_offset, 1.),
                        ..default()
                    },
                    Door,
                ));

                _xcoord = (MAX_X * 2. - (3 * TILE_SIZE / 2) as f32) as usize;
                _ycoord = (y_offset - ((TILE_SIZE / 2) as f32) + MAX_Y) as usize;
                //set_collide(room_manager, xcoord, ycoord, 2);
            }
            y_offset += TILE_SIZE as f32;
        }

        y_offset = -max_y + ((TILE_SIZE / 2) as f32);
        x_offset += TILE_SIZE as f32;
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

pub fn transition_map(room: &mut Query<&mut Transform, (Without<Player>, Without<Enemy>)>, pt: &mut Transform) {
    for mut wt in room.iter_mut() {
        wt.translation.z *= -1.;
    }
    let new_pos: Vec3 = pt.translation + Vec3::new(-MAX_X * 1.9, 0., 0.);
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
