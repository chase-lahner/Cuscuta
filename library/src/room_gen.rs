use bevy::prelude::*;
use rand::Rng;
use crate::collision::*;
use crate::cuscuta_resources::*;
use crate::player::*;
use crate::enemies::*;
use crate::markov_chains::*;
use crate::ui::*;

#[derive(Resource)]
pub struct ClientRoomManager{
    pub max_x: f32,
    pub max_y: f32,
    pub width: f32,
    pub height: f32,
}

impl ClientRoomManager{
    pub fn new() -> Self{
        Self {
            width: 40.,
            height: 40.,
            max_x: 0.,
            max_y: 0.,
        }
    }
}


#[derive(Component)]
pub struct Potion;

// dimensions for remebering rooms array
#[derive(Debug, Clone)] 
pub struct RoomDimensions {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone)]
pub struct InnerWallStartPos {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Component)]
pub struct InnerWall {
    pub start_pos: InnerWallStartPos,
    pub length_direction_vector: (i32, i32),
}

#[derive(Debug, Clone)]
pub struct InnerWallList {
    pub walls: Vec<Vec<InnerWall>>, 
}

// array that remembers rooms and their z indexes
#[derive(Debug)]
pub struct RoomArray {
    pub rooms: Vec<Option<RoomDimensions>>,
}

impl RoomArray {
    fn new() -> Self {
        RoomArray {
           rooms: Vec::new(),
        }
    }

    // convert z index to absolute value
    fn z_to_index(z: f32) -> usize {
        z.abs() as usize
    }

    // add room to array with width and height
    pub fn add_room_to_storage(&mut self, z_index: f32, width: usize, height: usize) {
        let index = Self::z_to_index(z_index);

        // ensure array is large enough to hold index
        if index >= self.rooms.len() {
            // resize array to fit new room
            self.rooms.resize(index + 1, None);
        }

        // store room at correct index
        self.rooms[index] = Some(RoomDimensions { width, height })
    }

    // get room dimensions at given index
    pub fn get_room_from_storage(&self, z_index: f32) -> Option<&RoomDimensions> {
        let index = Self::z_to_index(z_index);
        if index < self.rooms.len() {
            return self.rooms[index].as_ref();
        }
        None
    }
    
    // Get room dimensions in PIXEL SIZE
    pub fn get_room_from_storage_in_pixels(&self, z_index: f32) -> Option<RoomDimensions> {
        let index = Self::z_to_index(z_index);
        if index < self.rooms.len() {
            if let Some(dimensions) = &self.rooms[index] {
                return Some(RoomDimensions {
                    width: dimensions.width * 32,
                    height: dimensions.height * 32,
                });
            }
        }
        None
    }

    


    
}

#[derive(Component)]
pub struct Door {
    pub next: Option<f32>,
    pub door_type: DoorType,
}

#[derive(Component)]
pub struct ClientDoor {
    pub door_type: DoorType,
}

// enum to represent different door types
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorType {
    Right,
    Left,
    Top,
    Bottom,
}

#[derive(Resource)]
pub struct RoomManager {
    // 2D array to store the room placement
    pub room_map: Vec<Vec<i32>>,
    pub grids: Vec<Vec<Vec<u32>>>,
    pub current_room: usize,
    pub room_sizes: Vec<(f32, f32)>,
    pub room_array: RoomArray,
    pub max_sizes: Vec<(f32, f32)>,  
    //MARKOV impl 2
    pub state_vector: Vec<(usize)>,
    // z of room that player is currently in
    pub current_z_index: f32,  
    // z of room that was most recently generated (used so we can backtrack w/o screwing everything up)
    pub global_z_index: f32,  
    pub inner_wall_list: InnerWallList,
}

impl RoomManager {
    pub fn new() -> Self {
        // initialize the 200x200 grid with 1s
        let room_map = vec![vec![1; 400]; 400];

        Self {
            room_map,
            grids: Vec::new(),
            current_room: 0,
            room_array: RoomArray::new(),
            room_sizes: Vec::new(),
            max_sizes: Vec::new(), 
            state_vector: Vec::new(),
            current_z_index: -2.0,
            global_z_index: -2.0,
            inner_wall_list: InnerWallList { walls: vec![Vec::new(); 100] },
        }

    }

     // Getter for current Z index
    pub fn get_current_z_index(&self) -> f32 {
        self.current_z_index
    }

    // Getter for global Z index
    pub fn get_global_z_index(&self) -> f32 {
        self.global_z_index
    }

    //MARKOV impl 3
    pub fn get_state_vector(&self) -> &Vec<usize> {
        &self.state_vector
    }

    // Setter for state_vector (replaces the entire vector)
    pub fn set_state_vector(&mut self, new_state_vector: Vec<usize>) {
        self.state_vector = new_state_vector;
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

    // add rooms dimensions to map with z index at a random position (for start room)
    pub fn add_start_room_to_map(&mut self, z_index: i32, width: usize, height: usize){
        let mut rng = rand::thread_rng();

        let upper_width = 400 - width;
        let upper_height = 400 - height;

        // Define the top-left corner for the start room placement randomly
        let start_x = rng.gen_range(0..upper_width);
        let start_y = rng.gen_range(0..upper_height);

        // Loop through the dimensions of the room and place the z_index in the grid
        for x in start_x..(start_x + width) {
            for y in start_y..(start_y + height) {
                self.room_map[x][y] = z_index;
            }
        }
    }


    pub fn add_room_to_map_from_top_door(
        &mut self, 
        z_index: i32, 
        new_z_index: i32, 
        new_width: usize, 
        new_height: usize
    ) {
        // Find the bounds of the current room
        
        if let Some((left_x, right_x, top_y, _bottom_y)) = self.find_room_bounds(z_index) {
            let old_x = (left_x + right_x) / 2;
            let old_y = top_y;

            let start_x = old_x - (new_width / 2);
            let start_y = old_y - new_height;
    
            // Loop through the dimensions of the room and place the z_index in the grid
            for x in start_x..(start_x + new_width) {
                for y in start_y..(start_y + new_height) {
                    self.room_map[x][y] = new_z_index;
                }
            }

        } else {
            println!("Error: TOP Could not find bounds for the current room with z_index {}", z_index);
        }
    }

    pub fn add_room_to_map_from_bottom_door(
        &mut self, 
        z_index: i32, 
        new_z_index: i32, 
        new_width: usize, 
        new_height: usize
    ) {
        // Find the bounds of the current room
        if let Some((left_x, right_x, _top_y, bottom_y)) = self.find_room_bounds(z_index) {
            let old_x = (left_x + right_x) / 2;
            let old_y = bottom_y + 1;

            let start_x = old_x - (new_width / 2);
            let start_y = old_y;

            // Loop through the dimensions of the room and place the z_index in the grid
            for x in start_x..(start_x + new_width) {
                for y in start_y..(start_y + new_height) {
                    self.room_map[x][y] = new_z_index;
                }
            }
    
        } else {
            println!("Error: BOTTOM Could not find bounds for the current room with z_index {}", z_index);
        }
    }

    pub fn add_room_to_map_from_left_door(
        &mut self, 
        z_index: i32, 
        new_z_index: i32, 
        new_width: usize, 
        new_height: usize
    ) {
        // Find the bounds of the current room
        if let Some((left_x, _right_x, top_y, bottom_y)) = self.find_room_bounds(z_index) {
            let old_y = (top_y + bottom_y) / 2;
            let old_x = left_x;
            let start_y = old_y - (new_height / 2);
            let start_x = old_x - new_width;

            // Loop through the dimensions of the room and place the z_index in the grid
            for x in start_x..(start_x + new_width) {
                for y in start_y..(start_y + new_height) {
                    self.room_map[x][y] = new_z_index;
                }
            }
    
        } else {
            println!("Error: LEFT Could not find bounds for the current room with z_index {}", z_index);
        }
    }

    pub fn add_room_to_map_from_right_door(
        &mut self, 
        z_index: i32, 
        new_z_index: i32, 
        new_width: usize, 
        new_height: usize
    ) {
        // Find the bounds of the current room
        if let Some((_left_x, right_x, top_y, bottom_y)) = self.find_room_bounds(z_index) {
            let old_y = (top_y + bottom_y) / 2;
            let old_x = right_x + 1;
            let start_y = old_y - (new_height / 2);
            let start_x = old_x;

            // Loop through the dimensions of the room and place the z_index in the grid
            for x in start_x..(start_x + new_width) {
                for y in start_y..(start_y + new_height) {
                    self.room_map[x][y] = new_z_index;
                }
            }
    
        } else {
            println!("Error: RIGHT Could not find bounds for the current room with z_index {}", z_index);
        }
    }

    // method to find room bounds based on the current room z index
    pub fn find_room_bounds(&self, z_index: i32) -> Option<(usize, usize, usize, usize)> {
        let mut left_x = usize::MAX;
        let mut right_x = 0;
        let mut top_y = usize::MAX;
        let mut bottom_y = 0;

        for x in 0..self.room_map.len(){
            for y in 0..self.room_map[x].len(){
                if self.room_map[x][y] == z_index {
                    if x < left_x {left_x = x; }
                    if x > right_x {right_x = x; }
                    if y < top_y {top_y = y; }
                    if y > bottom_y {bottom_y = y; }
                }
            }
        }

        if left_x != usize::MAX && right_x > 0 && top_y != usize::MAX && bottom_y > 0 {
            Some((left_x, right_x, top_y, bottom_y))
        } else {
            None
        }
    }

    pub fn get_room_value(&self, x: usize, y: usize) -> Option<i32> {
        // Check if indices are within bounds
        if x < self.room_map.len() && y < self.room_map[x].len() {
            Some(self.room_map[x][y])
        } else {
            None // Return None if indices are out of bounds
        }
    }

    pub fn set_current_z_index(&mut self, new_z_index: f32) {
        self.current_z_index = new_z_index;
    }

    // Print the 200x200 grid for debugging
    pub fn print_room_map(&self) {
        for row in &self.room_map {
            println!("{:?}", row);
        }
    }

    pub fn current_room_z_index(&self) -> f32 {
        self.current_z_index
    }

    // Get the Z-index for the next room
    pub fn next_room_z_index(&mut self) -> f32 {
        self.global_z_index -= 2.0; // Always decrement the global Z by 2 for a new room
        self.current_z_index = self.get_global_z_index();
        self.global_z_index
    }


    // Get mutable reference to the current grid
    pub fn current_grid(&mut self) -> &mut Vec<Vec<u32>> {
        &mut self.grids[self.current_room]
    }
    
    // Get the size of the current room (width, height)
    pub fn current_room_size(&self) -> (f32, f32) {
        let current_player_z = self.get_current_z_index();
        if let Some(room) = self.room_array.get_room_from_storage(current_player_z) {
            let width = room.width as f32 * TILE_SIZE as f32;
            let height = room.height as f32 * TILE_SIZE as f32;
            return (width, height);
        }else{
            self.room_sizes[self.current_room]
        }
    }

    pub fn current_room_max(&self) -> (f32, f32) {
        self.max_sizes[self.current_room]
    }

    pub fn add_inner_wall(&mut self, index: usize, wall: InnerWall) {
        if index < self.inner_wall_list.walls.len() {
            self.inner_wall_list.walls[index].push(wall);
        } else {
            println!("Error: Index {} is out of bounds for InnerWallList.", index);
        }
    }

    pub fn get_inner_walls(&self, index: usize) -> Option<&Vec<InnerWall>> {
        self.inner_wall_list.walls.get(index)
    }
}

pub fn spawn_potions_in_room(
    commands: &mut Commands,
    room_manager: &RoomManager,
    num_potions: usize,
) {
    // Get room boundaries
    let (room_width, room_height) = room_manager.current_room_size();
    let max_x = room_width / 2.0;
    let max_y = room_height / 2.0;
    let z_index = room_manager.current_room_z_index();

    let mut rng = rand::thread_rng();

    for _ in 0..num_potions {
        // Randomize position within room boundaries
        let x_position = rng.gen_range(-max_x + TILE_SIZE as f32..max_x - TILE_SIZE as f32);
        let y_position = rng.gen_range(-max_y + TILE_SIZE as f32..max_y - TILE_SIZE as f32);

        // Spawn the potion
        commands.spawn((
            Transform::from_xyz(x_position, y_position, z_index + 0.1),
            Potion,
        ));
    }
}


#[derive(Component)]
pub struct Room;

pub fn spawn_start_room(
    mut commands: &mut Commands, 
    mut room_manager: &mut RoomManager,
    mut last_attribute_array: &mut LastAttributeArray, // Add reference here
) {

    // get a reference to the last attribute array
    // Access last attribute array
    println!("Last Attribute Array: {:?}", last_attribute_array.attributes);

    // Example: Update an attribute
    //last_attribute_array.set_attribute(0, 2); // Set Room_Size to "low"

    let mut rng = rand::thread_rng();

    let enemy_type_matrix = Room_Attributes::get_preset_matrix()[3].clone(); // Get the Room_Size matrix
    println!("Room Size Matrix: {:?}", enemy_type_matrix);
    
    //MARKOV impl 4
    room_manager.set_state_vector(vec![1; 5]);
    // generate random integers between 50 and 250, * 32
    let random_width = rng.gen_range(40..=40);
    let random_height = rng.gen_range(40..=40);
    // Room width & height as a multiple of 32
    // * 32d = pixel count
    let room_width = random_width as f32 * TILE_SIZE as f32;  
    let room_height = random_height as f32 * TILE_SIZE as f32;

    // Add the room to room manager
    room_manager.add_room(random_width, random_height, room_width, room_height);

    // max room bounds
    let max_x = room_width / 2.0;
    let max_y = room_height / 2.0;

    // get current room z index
    let z_index = room_manager.current_room_z_index();

    // ADD THIS TO FIX SPAWN ROOM
    room_manager.room_array.add_room_to_storage(z_index as f32, random_width as usize, random_height as usize);

    // add start room to map at a random position
    room_manager.add_start_room_to_map(z_index as i32, random_width as usize, random_height as usize);


    // find the bounds of the start room and print them
    if let Some(_room) = room_manager.find_room_bounds(z_index as i32) {
        //println!("Start room bounds: Left: {}, Right: {}, Top: {}, Bottom: {}", left_x, right_x, top_y, bottom_y);
    } else {
        println!("Error: Could not find bounds for the start room.");
    }

    // offset for spawning tiles
    let mut x_offset = -max_x + ((TILE_SIZE / 2) as f32);
    let mut y_offset = -max_y + ((TILE_SIZE / 2) as f32);
    

    // spawn floors & walls
    while x_offset < max_x {
        let xcoord: usize = ((x_offset + max_x) / TILE_SIZE as f32).floor() as usize;

         /* Spawn in north wall */
         commands.spawn((
            Transform::from_xyz(x_offset, max_y - ((TILE_SIZE / 2) as f32), z_index),
            Wall, 
            Room,
        ));
        set_collide(room_manager, xcoord, (max_y / TILE_SIZE as f32).floor() as usize, 1);

        /* Spawn in south wall */
        commands.spawn((
            Transform::from_xyz(x_offset, -max_y + ((TILE_SIZE / 2) as f32), z_index),
            Wall, 
            Room,
        ));
        set_collide(room_manager, xcoord, (-max_y / TILE_SIZE as f32).floor() as usize, 1);

        while y_offset < max_y + (TILE_SIZE as f32) {
            let ycoord: usize = ((y_offset + max_y) / TILE_SIZE as f32).floor() as usize;

            /* East wall */
            commands.spawn((
                Transform::from_xyz(max_x - ((TILE_SIZE / 2) as f32), y_offset, z_index - 0.1),
                Wall, 
                Room,
            ));
            set_collide(room_manager, (max_x / TILE_SIZE as f32).floor() as usize, ycoord, 1);

            /* West wall */
            commands.spawn((
                Transform::from_xyz(-max_x + ((TILE_SIZE / 2) as f32), y_offset, z_index - 0.2),
                Wall, 
                Room,
            ));
            set_collide(room_manager, (-max_x / TILE_SIZE as f32).floor() as usize, ycoord, 1);

            /* Floor tiles */
            commands.spawn((
                Transform::from_xyz(x_offset, y_offset, z_index - 0.3),
                Room,
                Background,
            ));

            y_offset += TILE_SIZE as f32;
        }

        y_offset = -max_y + ((TILE_SIZE / 2) as f32);
        x_offset += TILE_SIZE as f32;
    }

    let current_z_index = room_manager.current_room_z_index();

    // BEGIN EDITING HERE:
    let wall_count = rng.gen_range(1..=3);

    for _ in 0..wall_count {
        create_inner_walls(&mut commands, &mut room_manager, random_width, random_height, current_z_index as isize);
    }

    // end new fn

    generate_doors(
        &mut commands,
        &mut room_manager,
        max_x,
        max_y,
        z_index,
    );

    spawn_potions_in_room(&mut commands, &room_manager, 2);

}

fn create_inner_walls(
    commands: &mut Commands, 
    room_manager: &mut RoomManager,
    room_width: usize,
    room_height: usize,
    z_index: isize,
){
    let z_abs = z_index.abs() as usize;
    let mut rng = rand::thread_rng();
    let start_pos_x = rng.gen_range(1..=room_width - 1);
    let start_pos_y = rng.gen_range(1..=room_height - 1);

    // horizontal or vertical wall
    let horizon_or_vert = rng.gen_range(0..=1);

    // HORIZONTAL WALL
    let wall = if horizon_or_vert == 0 {
        // get wall length
        let wall_length = rng.gen_range(3..=(room_width / 2) - 1);

        // get room mid point
        let mid_point = room_width / 2;

        // if closer to right wall
        let length_direction_vector = if start_pos_x >= mid_point {
            (-(wall_length as i32), 1)
        } else {
            (wall_length as i32, 1)
        };
        
        // create a new inner wall
        InnerWall {
            start_pos: InnerWallStartPos { x: start_pos_x, y: start_pos_y },
            length_direction_vector,
        }
    } 

    // VERTICAL WALL
    else 
    {
        // get wall height
        let wall_height = rng.gen_range(3..=(room_height / 2) - 1);

        // get room mid point
        let mid_point = room_height / 2;

        // if closer to right wall
        let length_direction_vector = if start_pos_y >= mid_point {
            (1, -(wall_height as i32))
        } else {
            (1, wall_height as i32)
        };
        
        // create a new inner wall
        InnerWall {
            start_pos: InnerWallStartPos { x: start_pos_x, y: start_pos_y },
            length_direction_vector,
        }
    };
    // add inner wall to inner wall list
    room_manager.add_inner_wall(z_abs, wall);


    // loop through inner wall list at current z index
    if let Some(walls) = room_manager.get_inner_walls(z_abs) {
        let walls_to_draw: Vec<_> = walls.clone(); // Clone to avoid mutable borrowing issues
        for wall in walls_to_draw.iter() {
            draw_inner_wall(commands, wall, z_abs, room_width, room_height, room_manager);
        }
    } else {
        println!("No inner walls found for Z index {}", z_abs);
    }
    
}


fn draw_inner_wall(
    commands: &mut Commands,
    inner_wall: &InnerWall,
    z_index: usize,
    room_width: usize,
    room_height: usize,
    room_manager: &mut RoomManager,
){

    // get start pos out of inner wall
    let mut current_x = inner_wall.start_pos.x as f32 * TILE_SIZE as f32 - ((room_width * 32) / 2) as f32 - (TILE_SIZE / 2) as f32;
    let mut current_y = inner_wall.start_pos.y as f32 * TILE_SIZE as f32 - ((room_height * 32) / 2) as f32 - (TILE_SIZE / 2) as f32;

    // get direction length vector out of inner wall
    let (dir_x, dir_y) = inner_wall.length_direction_vector;

    // horizontal wall
    if dir_y == 1 {
        // draw to the right
        if dir_x > 0 {
            let end_value = current_x + (dir_x as f32 - 3.) * 32.;

            while current_x <= end_value {
                commands.spawn((
                    Transform::from_xyz(current_x, current_y, z_index as f32),
                    Wall,
                    Room,
                    inner_wall.clone(),
                ));
                 // call set_collide for this wall segment
                 let grid_x = ((current_x + (room_width as f32 * TILE_SIZE as f32) / 2.0) / TILE_SIZE as f32).floor() as usize;
                 let grid_y = ((current_y + (room_height as f32 * TILE_SIZE as f32) / 2.0) / TILE_SIZE as f32).floor() as usize;
                 set_collide(room_manager, grid_x, grid_y, 1);

                current_x += TILE_SIZE as f32;
            }
        } 
        // draw to the left
        else {

            let end_value = current_x + (dir_x as f32 + 3.) * 32.;

            while current_x >= end_value {
                commands.spawn((
                    Transform::from_xyz(current_x, current_y, z_index as f32),
                    Wall,
                    Room,
                    inner_wall.clone(),
                ));
                 // Call set_collide for this wall segment
                 let grid_x = ((current_x + (room_width as f32 * TILE_SIZE as f32) / 2.0) / TILE_SIZE as f32).floor() as usize;
                 let grid_y = ((current_y + (room_height as f32 * TILE_SIZE as f32) / 2.0) / TILE_SIZE as f32).floor() as usize;
                 set_collide(room_manager, grid_x, grid_y, 1);

                current_x -= TILE_SIZE as f32;
            }
        }

    }



    // vertical wall
    if dir_x == 1 {
        // draw down
        if dir_y > 0 {
            let end_value = current_y + (dir_y as f32 - 3.) * 32.;

            while current_y <= end_value {
                commands.spawn((
                    Transform::from_xyz(current_x, current_y, z_index as f32),
                    Wall,
                    Room,
                    inner_wall.clone(),
                ));
                // Call set_collide for this wall segment
                let grid_x = ((current_x + (room_width as f32 * TILE_SIZE as f32) / 2.0) / TILE_SIZE as f32).floor() as usize;
                let grid_y = ((current_y + (room_height as f32 * TILE_SIZE as f32) / 2.0) / TILE_SIZE as f32).floor() as usize;
                set_collide(room_manager, grid_x, grid_y, 1);
 

                current_y += TILE_SIZE as f32;
            }
        } 
        // draw up
        else {

            let end_value = current_y + (dir_y as f32 + 3.) * 32.;

            while current_y >= end_value {
                commands.spawn((
                    Transform::from_xyz(current_x, current_y, z_index as f32),
                    Wall,
                    Room,
                    inner_wall.clone(),
                ));
                // Call set_collide for this wall segment
                let grid_x = ((current_x + (room_width as f32 * TILE_SIZE as f32) / 2.0) / TILE_SIZE as f32).floor() as usize;
                let grid_y = ((current_y + (room_height as f32 * TILE_SIZE as f32) / 2.0) / TILE_SIZE as f32).floor() as usize;
                set_collide(room_manager, grid_x, grid_y, 1);
 

                current_y -= TILE_SIZE as f32;
            }
        }
    }
}

fn regen_draw_inner_wall(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    inner_wall: &InnerWall,
    z_index: usize,
    room_width: usize,
    room_height: usize,
){
    let north_wall_texture_handle = asset_server.load("tiles/walls/north_wall.png");

    // get start pos out of inner wall
    let mut current_x = inner_wall.start_pos.x as f32 * TILE_SIZE as f32 - ((room_width * 32) / 2) as f32 - (TILE_SIZE / 2) as f32;
    let mut current_y = inner_wall.start_pos.y as f32 * TILE_SIZE as f32 - ((room_height * 32) / 2) as f32 - (TILE_SIZE / 2) as f32;

    // get direction length vector out of inner wall
    let (dir_x, dir_y) = inner_wall.length_direction_vector;

    // horizontal wall
    if dir_y == 1 {
        // draw to the right
        if dir_x > 0 {
            let end_value = current_x + (dir_x as f32 - 3.) * 32.;

            while current_x <= end_value {
                commands.spawn((
                    SpriteBundle {
                        texture: north_wall_texture_handle.clone(),
                        transform: Transform::from_xyz(current_x, current_y, z_index as f32),
                        ..default()
                    },
                    Wall,
                    Room,
                ));
                current_x += TILE_SIZE as f32;
            }
        } 
        // draw to the left
        else {
            let end_value = current_x + (dir_x as f32 + 3.) * 32.;

            while current_x >= end_value {
                commands.spawn((
                    SpriteBundle {
                        texture: north_wall_texture_handle.clone(),
                        transform: Transform::from_xyz(current_x, current_y, z_index as f32),
                        ..default()
                    },
                    Wall,
                    Room,
                ));
                current_x -= TILE_SIZE as f32;
            }
        }

    }



    // vertical wall
    if dir_x == 1 {
        // draw down
        if dir_y > 0 {
            let end_value = current_y + (dir_y as f32 - 3.) * 32.;

            while current_y <= end_value {
                commands.spawn((
                    SpriteBundle {
                        texture: north_wall_texture_handle.clone(),
                        transform: Transform::from_xyz(current_x, current_y, z_index as f32),
                        ..default()
                    },
                    Wall,
                    Room,
                ));
                current_y += TILE_SIZE as f32;
            }
        } 
        // draw up
        else {
            let end_value = current_y + (dir_y as f32 + 3.) * 32.;

            while current_y >= end_value {
                commands.spawn((
                    SpriteBundle {
                        texture: north_wall_texture_handle.clone(),
                        transform: Transform::from_xyz(current_x, current_y, z_index as f32),
                        ..default()
                    },
                    Wall,
                    Room,
                ));
                current_y -= TILE_SIZE as f32;
            }
        }
    }
}


/// Generates random room boundaries and adds the room to the room manager.
/// Returns the room width, room height, max x, max y, and z-index.
fn generate_room_boundaries(
    room_manager: &mut RoomManager,
    mut carnage_query: Query<&mut CarnageBar>, 
) -> (f32, f32, f32, f32, f32) {
    let mut rng = rand::thread_rng();

    // GET CARNAGE PERCENT FROM UI VALUE
    let carnage_percent: f32 = carnage_query.single_mut().get_overall_percentage();

    println!("carnage percent: {}", carnage_percent);

    // ADD REFERENCE TO MARKOV CHAIN FILE HERE THAT WE CAN USE
    //let room_size_original_matrix = Room_Attributes::Room_Size.get_preset_vector();
    //let skewed_matrix = Skew(room_size_original_matrix, carnage_percent);
    //                                  |
    //                                  |
    //MARKOV impl 1 THE GENERATOR       V

    let current_states = room_manager.get_state_vector();
    let mut future_states = current_states.clone();

    //LOOP THRU CURRENT STATES TO GET TO INDIVIDUAL MATRICES AND PROCEEDS TO SKEW ONLY THE REQUIRED ROW
    //
    for (index, state) in current_states.iter().enumerate() {
        println!("State {}: {}", index + 1, state);
            
        //let current_row = Skew_Row(Room_Attributes::get_matrix_by_index(index),carnage_percent,current_states[index]); DEPRECATED?!?!?!?
        
        if let Some(matrix) = Room_Attributes::get_matrix_by_index(index) {
            let current_row = Skew_Row(matrix, carnage_percent, current_states[index]);
            let rand_percent: f32 = rng.gen_range(0.0..1.0);

            //Deciding the fate of the state
            future_states[index] = if rand_percent < current_row[0] {
                //stealth state
                0
            } else if rand_percent < current_row[1] {
                //normal
                1
            } else {
                //carnage
                2
            };
        } else {
            println!("Invalid index: {}", index);
        }
    }
    // randomly select size based on skewed values

    // ONLY FUTURE STATES FROM HERE ON OUT

    // Generate random width and height between 40 and 80 tiles
    let random_width = rng.gen_range(40..=80);
    let random_height = rng.gen_range(40..=80);

    // Convert to pixel sizes
    let room_width = random_width as f32 * TILE_SIZE as f32;
    let room_height = random_height as f32 * TILE_SIZE as f32;

    // Add the room to the room manager
    room_manager.add_room(random_width, random_height, room_width, room_height);

    // Get z-index for this room
    let z_index = room_manager.get_global_z_index() - 2.0;

    // add room to rooms array
    room_manager.room_array.add_room_to_storage(z_index, random_width, random_height);

    // Calculate maximum x and y coordinates (room boundaries)
    let max_x = room_width / 2.0;
    let max_y = room_height / 2.0;

    (room_width, room_height, max_x, max_y, z_index)
}

/// Generates walls and floors for the room.
fn generate_walls_and_floors(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    _room_width: f32,
    _room_height: f32,
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
// take in the correct door type
fn generate_doors(
    commands: &mut Commands,
    room_manager: &mut RoomManager,
    max_x: f32,
    max_y: f32,
    z_index: f32,
) {
    if let Some((left_x, right_x, top_y, bottom_y)) = room_manager.find_room_bounds(z_index as i32) {
        let height = bottom_y - top_y;
        let width = right_x - left_x;
        let half_height = height / 2;
        let half_width = width / 2;


        // LEFT DOOR
        if room_manager.get_room_value(left_x - 1, top_y + half_height) != Some(1) {
            // Left door
            let door_left_x = -max_x + (3.0 * TILE_SIZE as f32 / 2.0) - TILE_SIZE as f32;
            let door_left_y = TILE_SIZE as f32 / 2.0;
            commands.spawn((
                Transform::from_xyz(door_left_x, door_left_y, z_index + 0.1),
                Door {
                    next: Some(room_manager.global_z_index),
                    door_type: DoorType::Left,
                },
                Room,
            ));

            let xcoord_left = ((-max_x * 2.0 + (3.0 * TILE_SIZE as f32 / 2.0)) - TILE_SIZE as f32) as usize;
            let ycoord_left = (door_left_y + max_y) as usize;
            set_collide(room_manager, xcoord_left, ycoord_left, 2);

        } else if (left_x as i32 - 81 > 0) && ((top_y + half_height) as i32 + 41 < 400) && ((bottom_y - half_height) as i32 - 41 > 0) {
             // Left door
             let door_left_x = -max_x + (3.0 * TILE_SIZE as f32 / 2.0) - TILE_SIZE as f32;
             let door_left_y = TILE_SIZE as f32 / 2.0;
             commands.spawn((
                Transform::from_xyz(door_left_x, door_left_y, z_index + 0.1),
                Door {
                     next: Some(room_manager.global_z_index),
                     door_type: DoorType::Left,
                },
                Room,
             ));
 
             let xcoord_left = ((-max_x * 2.0 + (3.0 * TILE_SIZE as f32 / 2.0)) - TILE_SIZE as f32) as usize;
             let ycoord_left = (door_left_y + max_y) as usize;
             set_collide(room_manager, xcoord_left, ycoord_left, 2);
        }
      

        // RIGHT DOOR
        if room_manager.get_room_value(right_x + 1, top_y + half_height) != Some(1) {
            // Right door
            let door_x = max_x - (3.0 * (TILE_SIZE as f32) / 2.0) + TILE_SIZE as f32;
            let door_y = TILE_SIZE as f32 / 2.0;  
            commands.spawn((
                Transform::from_xyz(door_x, door_y, z_index + 0.1),
                Door {
                    next: Some(room_manager.global_z_index),
                    door_type: DoorType::Right,
                },
                Room,
            ));
            
            let xcoord_right = ((max_x * 2.0 - (3.0 * TILE_SIZE as f32 / 2.0)) + TILE_SIZE as f32) as usize;
            let ycoord_right = (door_y + max_y) as usize;
            set_collide(room_manager, xcoord_right, ycoord_right, 2);
        } 
        else if (right_x as i32 + 81 < 400) && ((top_y + half_height) as i32 + 41 < 400) && (bottom_y - half_height) as i32 - 41 > 0
        {
             // Right door
             let door_x = max_x - (3.0 * (TILE_SIZE as f32) / 2.0) + TILE_SIZE as f32;
             let door_y = TILE_SIZE as f32 / 2.0;  
             commands.spawn((
                Transform::from_xyz(door_x, door_y, z_index + 0.1),
                Door {
                     next: Some(room_manager.global_z_index),
                     door_type: DoorType::Right,
                },
                Room,
             ));
             
             let xcoord_right = ((max_x * 2.0 - (3.0 * TILE_SIZE as f32 / 2.0)) + TILE_SIZE as f32) as usize;
             let ycoord_right = (door_y + max_y) as usize;
             set_collide(room_manager, xcoord_right, ycoord_right, 2);
        }

        // TOP DOOR
        if room_manager.get_room_value(left_x + half_width, top_y - 1) != Some(1) {
             // Top door
             let door_top_x = TILE_SIZE as f32 / 2.0;
             let door_top_y = max_y - (3.0 * TILE_SIZE as f32 / 2.0) + TILE_SIZE as f32;
             commands.spawn((
                Transform::from_xyz(door_top_x, door_top_y, z_index + 0.1),
                
                 Door {
                     next: Some(room_manager.global_z_index),
                     door_type: DoorType::Top,
                 },
                 Room,
             ));
 
             let xcoord_top = (door_top_x + max_x) as usize;
             let ycoord_top = ((max_y * 2.0 - (3.0 * TILE_SIZE as f32 / 2.0)) + TILE_SIZE as f32) as usize;
             set_collide(room_manager, xcoord_top, ycoord_top, 2);
        }
        else if top_y as i32 - 81 > 0 && ((left_x + half_width) as i32 + 41 < 400) && (right_x - half_width) as i32 - 41> 0{
            // Top door
            let door_top_x = TILE_SIZE as f32 / 2.0;
            let door_top_y = max_y - (3.0 * TILE_SIZE as f32 / 2.0) + TILE_SIZE as f32;
            commands.spawn((
                Transform::from_xyz(door_top_x, door_top_y, z_index + 0.1),
               
                Door {
                    next: Some(room_manager.global_z_index),
                    door_type: DoorType::Top,
                },
                Room,
            ));

            let xcoord_top = (door_top_x + max_x) as usize;
            let ycoord_top = ((max_y * 2.0 - (3.0 * TILE_SIZE as f32 / 2.0)) + TILE_SIZE as f32) as usize;
            set_collide(room_manager, xcoord_top, ycoord_top, 2); 
        }

        // BOTTOM DOOR
        if room_manager.get_room_value(left_x + half_width, bottom_y + 1) != Some(1) {
            // Bottom door
            let door_bottom_x = TILE_SIZE as f32 / 2.0;
            let door_bottom_y = -max_y + (3.0 * TILE_SIZE as f32 / 2.0) - TILE_SIZE as f32;
            commands.spawn((
                Transform::from_xyz(door_bottom_x, door_bottom_y, z_index + 0.1),
                
                Door {
                    next: Some(room_manager.global_z_index),
                    door_type: DoorType::Bottom,
                },
                Room,
            ));
            let xcoord_bottom = (door_bottom_x + max_x) as usize;
            let ycoord_bottom = ((-max_y * 2.0 - (3.0 * TILE_SIZE as f32 / 2.0)) - TILE_SIZE as f32) as usize;
            set_collide(room_manager, xcoord_bottom, ycoord_bottom, 2);
        } else if bottom_y as i32 + 81 < 400 && ((left_x + half_width) as i32 + 41 < 400) && (right_x - half_width) as i32 - 41 > 0{
            // Bottom door
            let door_bottom_x = TILE_SIZE as f32 / 2.0;
            let door_bottom_y = -max_y + (3.0 * TILE_SIZE as f32 / 2.0) - TILE_SIZE as f32;
            commands.spawn((
                Transform::from_xyz(door_bottom_x, door_bottom_y, z_index + 0.1),
                
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
    }
}

pub fn generate_random_room_with_bounds(
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>,
    room_manager: &mut RoomManager,
    width: usize,
    height: usize,
) {
    // Manually calculate the room width and height in pixels
    let room_width = width as f32 * TILE_SIZE as f32;
    let room_height = height as f32 * TILE_SIZE as f32;
    let max_x = room_width / 2.0;
    let max_y = room_height / 2.0;

    // Get z-index for this room
    let next_z_index = room_manager.next_room_z_index();

    let _current_z_index = room_manager.current_z_index;

    // global
    let global_z_index = room_manager.get_global_z_index();

    // Add the room to the room manager
    room_manager.add_room(width, height, room_width, room_height);

    // Generate walls and floors
    generate_walls_and_floors(
        commands,
        asset_server,
        room_width,
        room_height,
        max_x,
        max_y,
        next_z_index,
    );

    let mut rng = rand::thread_rng();

    let wall_count = rng.gen_range(1..=3);
    
    // add inner walls
    for _ in 0..wall_count {
        create_inner_walls(commands, room_manager, width, height, global_z_index as isize);
    }

    // Generate doors
    generate_doors(
        commands,
        room_manager,
        max_x,
        max_y,
        next_z_index,
    );

    // **NEW**: Find and print the room bounds after generating the room
    if let Some((left_x, right_x, top_y, bottom_y)) = room_manager.find_room_bounds(global_z_index as i32) {
        println!("Generated room bounds: Left: {}, Right: {}, Top: {}, Bottom: {}, z_index: {}", left_x, right_x, top_y, bottom_y, global_z_index);
    } else {
        println!("Error: Could not find bounds for the newly generated room. {}", global_z_index);
    }
}

pub fn regenerate_existing_room(
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>,
    room_manager: &mut RoomManager,
    width: usize,
    height: usize,
    z_for_regen: f32,
) {
    let z_abs = z_for_regen.abs() as usize;
    // Manually calculate the room width and height in pixels
    let room_width = width as f32 * TILE_SIZE as f32;
    let room_height = height as f32 * TILE_SIZE as f32;
    let max_x = room_width / 2.0;
    let max_y = room_height / 2.0;

    // Get z-index for this room
    let next_z_index = z_for_regen as f32;

    let current_z_index = z_for_regen;
    room_manager.set_current_z_index(current_z_index);

    // global
    let global_z_index = room_manager.get_global_z_index();

    // Generate walls and floors
    generate_walls_and_floors(
        commands,
        asset_server,
        room_width,
        room_height,
        max_x,
        max_y,
        next_z_index,
    );


    // Generate doors
    generate_doors(
        commands,
        room_manager,
        max_x,
        max_y,
        next_z_index,
    );

    // retrieve and spawn inner walls for the current room from `InnerWallList`
    if let Some(walls) = room_manager.get_inner_walls(z_abs as usize) {
        let walls_to_draw: Vec<_> = walls.clone(); // Clone walls to a temporary variable
        for wall in walls_to_draw.iter() {
            draw_inner_wall(commands, wall, z_abs, width, height, room_manager);
        }
    } else {
        println!("No inner walls found for Z index {}", z_abs);
    }
}

pub fn transition_map(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    room_manager: &mut RoomManager,
    mut room_query: Query<Entity, With<Room>>, 
    _door_query: Query<(&Transform, &Door), (Without<Player>, Without<Enemy>)>,  
    pt: &mut Transform,
    door_type: DoorType, 
    mut carnage_query: Query<&mut CarnageBar>, 
) {
    let mut right_x_out = 0;
    let mut left_x_out = 0;
    let mut top_y_out = 0;
    let mut bottom_y_out = 0;

    // Despawn old room
    for entity in room_query.iter_mut() {
        commands.entity(entity).despawn();
    }
    
    let z_in = room_manager.get_current_z_index();
    if let Some((left_x, right_x, top_y, bottom_y)) = room_manager.find_room_bounds(z_in as i32) {
        right_x_out = right_x;
        left_x_out = left_x;
        top_y_out = top_y;
        bottom_y_out = bottom_y;
    } else {
        println!("Error: Could not find bounds for the newly generated room. {}", z_in);
    }

    

    let _max_x = room_manager.current_room_max().0;
    let _max_y = room_manager.current_room_max().1;

    // generate random room boundaries for upcoming room
    let (room_width, room_height, max_x, max_y, _z_index) = generate_room_boundaries(room_manager, carnage_query);

    // Adjust the player's position based on the door they entered
    match door_type {
        DoorType::Right => {
            // get new z index
            let x_to_check = right_x_out + 1;
            let y_to_check =(top_y_out + bottom_y_out) / 2;
            let room_val = room_manager.get_room_value(x_to_check,y_to_check);
            if room_val == Some(1) {
                let new_z_index = room_manager.get_global_z_index() - 2.0;

                let current_z = room_manager.get_current_z_index();
                let _global_z = room_manager.get_global_z_index();

                // add new room to map relative to current room top door
                room_manager.add_room_to_map_from_right_door(
                    current_z as i32,
                    new_z_index as i32,
                    room_width as usize / TILE_SIZE as usize,
                    room_height as usize / TILE_SIZE as usize,
                );

                // generate the room with random bounds
                generate_random_room_with_bounds(
                    commands,
                    &asset_server,
                    room_manager,
                    room_width as usize / TILE_SIZE as usize,
                    room_height as usize / TILE_SIZE as usize,
                );
                // pass in right door
                //generate_random_room(commands, &asset_server, room_manager);

                // Spawn the player a little away from the right door
                pt.translation = Vec3::new(-max_x + TILE_SIZE as f32 * 2.0, TILE_SIZE as f32 / 2.0, room_manager.current_z_index);
            }else{
                if let Some(room_val_unwrapped) = room_val {
                    // room_manager.room_array.get_room_from_storage(room_val_unwrapped as f32);
                    if let Some(room_dimensions) = room_manager.room_array.get_room_from_storage(room_val_unwrapped as f32) {
                        let width = room_dimensions.width;
                        let height = room_dimensions.height;
                        //println!("Room width: {}, height: {}", width, height);
                        //generate room with set bounds

                        regenerate_existing_room(
                            commands,
                            &asset_server,
                            room_manager,
                            width as usize,
                            height as usize,
                            room_val_unwrapped as f32,
                        );
                        pt.translation = Vec3::new(-max_x + TILE_SIZE as f32 * 2.0, TILE_SIZE as f32 / 2.0, room_val_unwrapped as f32);
                    } else {
                        println!("Error: Room not found in storage.");
                    }
                } else {
                    println!("Error: room_val is None, cannot retrieve room dimensions.");
                }
                // room_manager.room_array.get_room_from_storage(room_val);
            }
        },
        DoorType::Left => {

            let x_to_check = left_x_out - 1;
            let y_to_check =(top_y_out + bottom_y_out) / 2;
            let room_val = room_manager.get_room_value(x_to_check,y_to_check);
            if room_val == Some(1) {
                let new_z_index = room_manager.get_global_z_index() - 2.0;

                let current_z = room_manager.get_current_z_index();
                let _global_z = room_manager.get_global_z_index();

                // add new room to map relative to current room top door
                room_manager.add_room_to_map_from_left_door(
                    current_z as i32,
                    new_z_index as i32,
                    room_width as usize / TILE_SIZE as usize,
                    room_height as usize / TILE_SIZE as usize,
                );

                // generate the room with random bounds
                generate_random_room_with_bounds(
                    commands,
                    &asset_server,
                    room_manager,
                    room_width as usize / TILE_SIZE as usize,
                    room_height as usize / TILE_SIZE as usize,
                );
                // pass in right door
                //generate_random_room(commands, &asset_server, room_manager);

                // Spawn the player a little away from the right door
                pt.translation = Vec3::new(max_x - TILE_SIZE as f32 * 2.0, TILE_SIZE as f32 / 2.0, room_manager.current_z_index);
            }else{
                if let Some(room_val_unwrapped) = room_val {
                    // room_manager.room_array.get_room_from_storage(room_val_unwrapped as f32);
                    if let Some(room_dimensions) = room_manager.room_array.get_room_from_storage(room_val_unwrapped as f32) {
                        let width = room_dimensions.width;
                        let height = room_dimensions.height;
                        //println!("Room width: {}, height: {}", width, height);
                        //generate room with set bounds

                        regenerate_existing_room(
                            commands,
                            &asset_server,
                            room_manager,
                            width as usize,
                            height as usize,
                            room_val_unwrapped as f32,
                        );
                        pt.translation = Vec3::new(max_x - TILE_SIZE as f32 * 2.0, TILE_SIZE as f32 / 2.0, room_manager.current_z_index);
                    } else {
                        println!("Error: Room not found in storage.");
                    }
                } else {
                    println!("Error: room_val is None, cannot retrieve room dimensions.");
                }
                // room_manager.room_array.get_room_from_storage(room_val);
            }

            // Spawn the player a little away from the right door
            
        },
        DoorType::Top => {
            let x_to_check = (left_x_out + right_x_out) / 2;
            let y_to_check = top_y_out - 1;
            let room_val = room_manager.get_room_value(x_to_check,y_to_check);
            if room_val == Some(1) {
                let new_z_index = room_manager.get_global_z_index() - 2.0;

                let current_z = room_manager.get_current_z_index();
                let _global_z = room_manager.get_global_z_index();

                // add new room to map relative to current room top door
                room_manager.add_room_to_map_from_top_door(
                    current_z as i32,
                    new_z_index as i32,
                    room_width as usize / TILE_SIZE as usize,
                    room_height as usize / TILE_SIZE as usize,
                );

                // generate the room with random bounds
                generate_random_room_with_bounds(
                    commands,
                    &asset_server,
                    room_manager,
                    room_width as usize / TILE_SIZE as usize,
                    room_height as usize / TILE_SIZE as usize,
                );
                // pass in right door
                //generate_random_room(commands, &asset_server, room_manager);

                // Spawn the player a little away from the right door
                pt.translation = Vec3::new(TILE_SIZE as f32 / 2.0, -max_y + TILE_SIZE as f32 * 2.0, room_manager.current_z_index);
            }else{
                if let Some(room_val_unwrapped) = room_val {
                    // room_manager.room_array.get_room_from_storage(room_val_unwrapped as f32);
                    if let Some(room_dimensions) = room_manager.room_array.get_room_from_storage(room_val_unwrapped as f32) {
                        let width = room_dimensions.width;
                        let height = room_dimensions.height;
                        //println!("Room width: {}, height: {}", width, height);
                        //generate room with set bounds

                        regenerate_existing_room(
                            commands,
                            &asset_server,
                            room_manager,
                            width as usize,
                            height as usize,
                            room_val_unwrapped as f32,
                        );
                        pt.translation = Vec3::new(TILE_SIZE as f32 / 2.0, -max_y + TILE_SIZE as f32 * 2.0, room_manager.current_z_index);
                    } else {
                        println!("Error: Room not found in storage.");
                    }
                } else {
                    println!("Error: room_val is None, cannot retrieve room dimensions.");
                }
                // room_manager.room_array.get_room_from_storage(room_val);
            }

            // Spawn the player a little below the top door
            //pt.translation = Vec3::new(TILE_SIZE as f32 / 2.0, -max_y + TILE_SIZE as f32 * 2.0, room_manager.current_z_index);


        },
        DoorType::Bottom => { 
            let x_to_check = (left_x_out + right_x_out) / 2;
            let y_to_check = bottom_y_out + 1;
            let room_val = room_manager.get_room_value(x_to_check,y_to_check);
            if room_val == Some(1) {
                let new_z_index = room_manager.get_global_z_index() - 2.0;

                let current_z = room_manager.get_current_z_index();
                let _global_z = room_manager.get_global_z_index();

                // add new room to map relative to current room top door
                room_manager.add_room_to_map_from_bottom_door(
                    current_z as i32,
                    new_z_index as i32,
                    room_width as usize / TILE_SIZE as usize,
                    room_height as usize / TILE_SIZE as usize,
                );

                // generate the room with random bounds
                generate_random_room_with_bounds(
                    commands,
                    &asset_server,
                    room_manager,
                    room_width as usize / TILE_SIZE as usize,
                    room_height as usize / TILE_SIZE as usize,
                );
                // pass in right door
                //generate_random_room(commands, &asset_server, room_manager);

                // Spawn the player a little away from the right door
                pt.translation = Vec3::new(TILE_SIZE as f32 / 2.0, max_y - TILE_SIZE as f32 * 2.0, room_manager.current_z_index);
            }else{
                if let Some(room_val_unwrapped) = room_val {
                    // room_manager.room_array.get_room_from_storage(room_val_unwrapped as f32);
                    if let Some(room_dimensions) = room_manager.room_array.get_room_from_storage(room_val_unwrapped as f32) {
                        let width = room_dimensions.width;
                        let height = room_dimensions.height;
                        //println!("Room width: {}, height: {}", width, height);
                        //generate room with set bounds

                        regenerate_existing_room(
                            commands,
                            &asset_server,
                            room_manager,
                            width as usize,
                            height as usize,
                            room_val_unwrapped as f32,
                        );
                        pt.translation = Vec3::new(TILE_SIZE as f32 / 2.0, max_y - TILE_SIZE as f32 * 2.0, room_manager.current_z_index);
                    } else {
                        println!("Error: Room not found in storage.");
                    }
                } else {
                    println!("Error: room_val is None, cannot retrieve room dimensions.");
                }
                // room_manager.room_array.get_room_from_storage(room_val);
            }

            // Spawn the player a little above the bottom door
            //pt.translation = Vec3::new(TILE_SIZE as f32 / 2.0, max_y - TILE_SIZE as f32 * 2.0, room_manager.current_z_index);
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
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
){
    let pot_handle = asset_server.load("tiles/1x2_pot.png");
    let pot_layout = TextureAtlasLayout::from_grid(
        UVec2::splat(TILE_SIZE),
         1,
         2,
        None,
        None
    );
    let _pot_layout_len = pot_layout.textures.len();
    let pot_layout_handle = texture_atlases.add(pot_layout);
    info!("spawning pot");
    commands.spawn((
        SpriteBundle{
            texture: pot_handle,
            transform: Transform::from_xyz(200.,200.,1.),
            ..default()
        },
        TextureAtlas {
            layout: pot_layout_handle,
            index:0,
        },
        Pot{
            touch: 0
        }
    ));
}

