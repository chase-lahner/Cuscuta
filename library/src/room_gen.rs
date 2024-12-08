use bevy::prelude::*;
use rand::{Rng, distributions::{Distribution, WeightedIndex}};
use crate::collision::*;
use crate::cuscuta_resources::*;
use crate::network::Sequence;
use crate::player::*;
use crate::enemies::*;
use crate::markov_chains::*;
use crate::server::send_player_to_self;
use crate::ui::*;
use crate::network::UDP;

#[derive(Event)]
pub struct RoomChangeEvent(pub bool);

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
            width: 40. * TILE_SIZE as f32,
            height: 40. * TILE_SIZE as f32,
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
impl InnerWallStartPos{
    pub fn new() -> Self{
        Self{
            x: 0,
            y: 0,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct InnerWall {
    pub start_pos: InnerWallStartPos,
    pub length_direction_vector: (i32, i32),
}
impl InnerWall{
    pub fn new() -> Self{
        Self{
            start_pos: InnerWallStartPos::new(),
            length_direction_vector: (0,0)
        }
    }
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
        println!("add_room_to_map_from_top_door - finding room bounds of z index: {}", z_index);
        if let Some((left_x, right_x, top_y, _bottom_y)) = self.find_room_bounds(z_index) {
            let old_x = (left_x + right_x) / 2;
            let old_y = top_y;

            let start_x = old_x - (new_width / 2);
            println!("Old_x:{} Old_y:{} width:{} height:{}", old_x, old_y, new_width, new_height);
            let start_y = old_y - new_height;
    
            // Loop through the dimensions of the room and place the z_index in the grid
            for x in start_x..(start_x + new_width) {
                for y in start_y..(start_y + new_height) {
                    self.room_map[x][y] = new_z_index;
                }
            }
            println!("add_room_to_map_from_left_door - ADDED ROOM z index: {}", new_z_index);

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
        println!("add_room_to_map_from_bottom_door - finding room bounds of z index: {}", z_index);
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
            println!("add_room_to_map_from_left_door - ADDED ROOM z index: {}", new_z_index);
    
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
        println!("add_room_to_map_from_left_door - finding room bounds of z index: {}", z_index);
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
            println!("add_room_to_map_from_left_door - ADDED ROOM z index: {}", new_z_index);
    
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
        println!("add_room_to_map_from_right_door - finding room bounds of z index: {}", z_index);
        if let Some((_left_x, right_x, top_y, bottom_y)) = self.find_room_bounds(z_index) {
            let old_y = (top_y + bottom_y) / 2;
            let old_x = right_x + 1;
            let start_y = old_y - (new_height / 2);
            let start_x = old_x ;

            // Loop through the dimensions of the room and place the z_index in the grid
            for x in start_x..(start_x + new_width) {
                for y in start_y..(start_y + new_height) {
                    self.room_map[x][y] = new_z_index;
                }
            }
            println!("add_room_to_map_from_left_door - ADDED ROOM z index: {}", new_z_index);

    
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


#[derive(Component, Clone, Debug, Resource)]
pub struct RoomConfig {
    states: Vec<StateConfig>,
}

#[derive(Component, Clone, Debug)]
pub struct StateConfig {
    width_range: (usize, usize),
    height_range: (usize, usize),
    inner_wall_count: (usize, usize),
    enemy_count: (usize, usize),
    enemy_type: (usize, usize),
    item_count: (usize, usize),
}


impl RoomConfig {
    pub fn new() -> Self {
        RoomConfig {
            states: vec![
                StateConfig {
                    width_range: (40, 59),
                    height_range: (40, 59),
                    inner_wall_count: (1, 3),
                    enemy_count: (1, 2),
                    enemy_type: (1, 2),
                    item_count: (1, 2),
                },
                StateConfig {
                    width_range: (60, 69),
                    height_range: (60, 69),
                    inner_wall_count: (2, 4),
                    enemy_count: (2, 3),
                    enemy_type: (2, 3),
                    item_count: (2, 3),
                },
                StateConfig {
                    width_range: (70, 79),
                    height_range: (70, 79),
                    inner_wall_count: (3, 5),
                    enemy_count: (3, 5),
                    enemy_type: (3, 4),
                    item_count: (3, 4),
                },
            ],
        }
    }

    pub fn get_width_range(&self, state: u8) -> (usize, usize) {
        self.states.get(state as usize).expect("Invalid state").width_range
    }

    pub fn get_height_range(&self, state: u8) -> (usize, usize) {
        self.states.get(state as usize).expect("Invalid state").height_range
    }

    pub fn get_inner_wall_count(&self, state: u8) -> (usize, usize) {
        self.states.get(state as usize).expect("Invalid state").inner_wall_count
    }

    pub fn get_enemy_count(&self, state: u8) -> (usize, usize) {
        self.states.get(state as usize).expect("Invalid state").enemy_count
    }

    pub fn get_enemy_type(&self, state: u8) -> (usize, usize) {
        self.states.get(state as usize).expect("Invalid state").enemy_type
    }

    pub fn get_item_count(&self, state: u8) -> (usize, usize) {
        self.states.get(state as usize).expect("Invalid state").item_count
    }
}

pub fn spawn_items_in_room(
    commands: &mut Commands,
    room_manager: &RoomManager,
    last_attribute_array: &LastAttributeArray,
    room_config: &RoomConfig,
) {
    // Get room boundaries for spawning
    let (room_width, room_height) = room_manager.current_room_size();
    let max_x = room_width / 2.0;
    let max_y = room_height / 2.0;
    let z_index = room_manager.current_room_z_index();

    let mut rng = rand::thread_rng();

    let item_count_attribute_value = last_attribute_array.get_attribute(4).unwrap_or(1);

    // get spawn range from last attribute
    let num_items_to_spawn_range = room_config.get_item_count(item_count_attribute_value);

    // get random count from roon config
    let potion_count = rng.gen_range(num_items_to_spawn_range.0..=num_items_to_spawn_range.1);
    let coin_pot_count = rng.gen_range(num_items_to_spawn_range.0..=num_items_to_spawn_range.1);

    // spawn potions
    for _ in 0..potion_count {
        // Randomize position within room boundaries
        let x_position = rng.gen_range(-max_x + TILE_SIZE as f32..max_x - TILE_SIZE as f32);
        let y_position = rng.gen_range(-max_y + TILE_SIZE as f32..max_y - TILE_SIZE as f32);

        // Spawn the potion
        commands.spawn((
            Transform::from_xyz(x_position, y_position, z_index + 0.1),
            Potion,
        ));
    }


    // for _ in 0..coin_pot_count {
    //     // Randomize position within room boundaries
    //     let x_position = rng.gen_range(-max_x + TILE_SIZE as f32..max_x - TILE_SIZE as f32);
    //     let y_position = rng.gen_range(-max_y + TILE_SIZE as f32..max_y - TILE_SIZE as f32);

    //     // Spawn the coin pot
    //     commands.spawn((
    //         SpriteBundle {
    //             texture: pot_handle.clone(),
    //             transform: Transform::from_xyz(x_position, y_position, z_index + 0.1),
    //             ..default()
    //         },
    //         TextureAtlas {
    //             layout: pot_layout_handle.clone(),
    //             index:0,
    //         },
    //         Pot{
    //             touch: 0
    //         }
    //     ));
    // }
}


#[derive(Component)]
pub struct Room;

pub fn spawn_start_room(
    commands: &mut Commands, 
    room_manager: &mut RoomManager,
    carnage_percent: f32,
    last_attribute_array: &mut LastAttributeArray,
    room_config: &RoomConfig,
) {
    println!("SPAWNING THE FUCKING START ROOM");
    // repeat for rest
    let mut rng = rand::thread_rng();

    // initialize the next attributes array
    let mut next_attribute_array = NextAttributeArray::new();


    // iterate through each attribute
    for i in 0..5 {
        // map index to Room_Attributes enum
        let room_attribute = match i {
            0 => Room_Attributes::Room_Size,
            1 => Room_Attributes::Inner_Walls,
            2 => Room_Attributes::Enemy_Count,
            3 => Room_Attributes::Enemy_Type,
            4 => Room_Attributes::Item_Count,
            _ => panic!("Invalid attribute index!"),
        };

        // get the last attribute value (1)
        let last_attribute_value = last_attribute_array.get_attribute(i).unwrap_or(1);

        // retrieve the corresponding matrix for the attribute
        let base_matrix = Room_Attributes::get_matrix_for_attribute(&room_attribute);

        // apply skew to the matrix
        let skewed_matrix = Skew_Row(base_matrix, carnage_percent, last_attribute_value as usize);

        let rand_percent: f32 = rng.gen_range(0.0..1.0);

        let next_state = if rand_percent < skewed_matrix[0] {
            //stealth state
            0
        } else if rand_percent < skewed_matrix[1] {
            //normal
            1
        } else {
            //carnage
            2
        };

        next_attribute_array.set_next_attribute(i, next_state);
    }

    // copy values from next attribute array into last attribute array
    last_attribute_array.attributes = next_attribute_array.attributes;

    // Room width & height as a multiple of 32
    // * 32d = pixel count
    let random_width = rng.gen_range(40..=40);
    let random_height = rng.gen_range(40..=40);
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

    // GET WALL COUNT FROM MARKOV CHAIN

    let wall_count_attribute_value = last_attribute_array.get_attribute(1).unwrap_or(1);

    // get spawn range from last attribute
    let num_walls_spawn_range = room_config.get_item_count(wall_count_attribute_value);

    // get random count from roon config
    let inner_wall_count = rng.gen_range(num_walls_spawn_range.0..=num_walls_spawn_range.1);

    for _ in 0..inner_wall_count {
        create_inner_walls(commands, room_manager, random_width, random_height, z_index as isize);
    }

    // end new fn
    generate_doors(
        commands,
        room_manager,
        max_x,
        max_y,
        z_index,
    );

    spawn_items_in_room(commands, &room_manager, &last_attribute_array, &room_config);

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
    let start_pos_x = rng.gen_range(2..=room_width - 5 );
    let start_pos_y = rng.gen_range(2..=room_height - 5);

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
    inner_wall: &InnerWall,
    z_index: usize,
    room_width: usize,
    room_height: usize,
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
                ));
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
                    Transform::from_xyz(current_x, current_y, z_index as f32),
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
                    Transform::from_xyz(current_x, current_y, z_index as f32),
                    Wall,
                    Room,
                ));
                current_y -= TILE_SIZE as f32;
            }
        }
    }
}

/// Generates walls and floors for the room.
fn generate_walls_and_floors(
    commands: &mut Commands,
    _room_width: f32,
    _room_height: f32,
    max_x: f32,
    max_y: f32,
    z_index: f32,
) {

    // Offset for spawning tiles
    let mut x_offset = -max_x + ((TILE_SIZE / 2) as f32);
    let mut y_offset = -max_y + ((TILE_SIZE / 2) as f32);

    // Spawn walls and floors
    while x_offset < max_x {
        /* Spawn north and south walls */
        commands.spawn((
            Transform::from_xyz(x_offset, max_y - ((TILE_SIZE / 2) as f32), z_index),
            Wall,
            Room,
        ));
        commands.spawn((
            Transform::from_xyz(x_offset, -max_y + ((TILE_SIZE / 2) as f32), z_index),
            Wall,
            Room,
        ));

        /* Spawn east and west walls */
        while y_offset < max_y + (TILE_SIZE as f32) {
            commands.spawn((
                Transform::from_xyz(max_x - ((TILE_SIZE / 2) as f32), y_offset, z_index - 0.1),
                Wall,
                Room,
            ));
            commands.spawn((
                Transform::from_xyz(-max_x + ((TILE_SIZE / 2) as f32), y_offset, z_index - 0.2),
                Wall,
                Room,
            ));

            /* Spawn floor tiles */
            commands.spawn((
                Transform::from_xyz(x_offset, y_offset, z_index - 0.3),
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
    room_manager: &mut RoomManager,
    carnage_query: &mut Query<&mut CarnageBar>, 
    last_attribute_array: &mut LastAttributeArray, 
    room_config: &RoomConfig,
) -> (usize,usize, f32, f32, f32) {
    let mut rng = rand::thread_rng();

    let mut next_attribute_array = NextAttributeArray::new();

    // GET CARNAGE PERCENT FROM UI VALUE
    let carnage_percent: f32 = carnage_query.single_mut().get_overall_percentage();

    // Determine the next state for each attribute
    let mut next_state: u8 = 0;

    // MARKOV CHAIN
    // iterate through each attribute
    for i in 0..5 {
        // map index to Room_Attributes enum
        let room_attribute = match i {
            0 => Room_Attributes::Room_Size,
            1 => Room_Attributes::Inner_Walls,
            2 => Room_Attributes::Enemy_Count,
            3 => Room_Attributes::Enemy_Type,
            4 => Room_Attributes::Item_Count,
            _ => panic!("Invalid attribute index!"),
        };

        // get the last attribute value (1)
        let last_attribute_value = last_attribute_array.get_attribute(i).unwrap_or(1);

        // retrieve the corresponding matrix for the attribute
        let base_matrix = Room_Attributes::get_matrix_for_attribute(&room_attribute);


        // apply skew to the matrix
        let skewed_matrix = Skew_Row(base_matrix, carnage_percent, last_attribute_value as usize);

        let rand_percent: f32 = rng.gen_range(0.0..1.0);

        // determine next state
        if rand_percent < skewed_matrix[0] {
            //stealth state
            next_state = 0;
        } else if rand_percent < skewed_matrix[1] {
            //normal
            next_state = 1;
        } else {
            //carnage
            next_state = 2;
        };


        next_attribute_array.set_next_attribute(i, next_state);
    }

    // copy values from next attribute array into last attribute array
    last_attribute_array.attributes = next_attribute_array.attributes;

    // ROOM SIZE RANGE
    let width_range = room_config.get_width_range(next_state);
    let height_range = room_config.get_height_range(next_state);

    // generate random size within the bounds
    let random_width = rng.gen_range(width_range.0..=width_range.1);
    let random_height = rng.gen_range(height_range.0..=height_range.1);


    let room_width = random_width as f32 * TILE_SIZE as f32;  
    let room_height = random_height as f32 * TILE_SIZE as f32;

    // MULTIPLY TO PIXELS
    let max_x = room_width / 2.;
    let max_y = room_height / 2.;

    // add the room to the room manager
    room_manager.add_room(random_width, random_height, room_width, room_height);

    // get z-index for this room
    let z_index = room_manager.next_room_z_index();
    println!("z_inex for new room i hop hi hokhpohkhpoikh {}", z_index);

    // add room to rooms array
    println!("Adding room z:{} to storage", z_index);
    room_manager.room_array.add_room_to_storage(z_index, random_width, random_height);
    

    // GET CURRENT Z
    let current_z_index = room_manager.current_z_index;

    // global Z
    let global_z_index = room_manager.get_global_z_index();

    // Generate walls and floors
    generate_walls_and_floors(
        commands,
        random_width as f32,
        random_height as f32,
        max_x as f32,
        max_y as f32,
        current_z_index,
    );

    let wall_count = rng.gen_range(1..=3);
    
    // add inner walls
    for _ in 0..wall_count {
        create_inner_walls(commands, room_manager, random_width, random_height, global_z_index as isize);
    }

   return (random_width, random_height, max_x as f32, max_y as f32, current_z_index);

}


pub fn regenerate_existing_room(
    commands: &mut Commands, 
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

    room_manager.set_current_z_index(z_for_regen);

    // Generate walls and floors
    generate_walls_and_floors(
        commands,
        room_width,
        room_height,
        max_x,
        max_y,
        z_for_regen,
    );

    generate_doors(
        commands,
        room_manager,
        max_x,
        max_y,
        z_for_regen,
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

/* transitions room to room. I have made this an amalgamous mess. Lots going
 * on here, we despawn old room, check which door we're on to generate a new room,
 * and send out player packets to let the youngins know where they should be */
pub fn transition_map(
    commands: &mut Commands,
    room_manager: &mut RoomManager,
    room_query: &mut Query<Entity, With<Room>>, 
    door_type: DoorType, 
    mut carnage_query: &mut Query<&mut CarnageBar>, 
    last_attribute_array: &mut LastAttributeArray, 
    room_config: &RoomConfig,
    player : &mut Query<(&mut Transform), With<Player>>,
) {
    println!("global z: {}", room_manager.global_z_index);
    if room_manager.global_z_index > 20.0 {
        println!("SPAWN BOSS BWAHAHAHAHA");
        return;
    }
    let mut right_x_out = 0;
    let mut left_x_out = 0;
    let mut top_y_out = 0;
    let mut bottom_y_out = 0;

    // Despawn old room
    for entity in room_query.iter_mut() {
        commands.entity(entity).despawn();
    }
    
    // get carnage percent from carnage query

    let z_in: f32 = room_manager.get_current_z_index();
    if let Some((left_x, right_x, top_y, bottom_y)) = room_manager.find_room_bounds(z_in as i32) {
        right_x_out = right_x;
        left_x_out = left_x;
        top_y_out = top_y;
        bottom_y_out = bottom_y;
    } else {
        println!("Error: Could not find bounds for the current room. {}", z_in);
    }

    // Adjust the player's position based on the door they entered
    match door_type {
        DoorType::Right => {
            // get new z index
            let x_to_check = right_x_out + 1;
            let y_to_check =(top_y_out + bottom_y_out) / 2;
            let room_val = room_manager.get_room_value(x_to_check, y_to_check);
            if room_val == Some(1) {
                
                
                // generate the room with random bounds
                let (room_width, room_height, max_x, max_y, _z_index) = generate_random_room_with_bounds(
                    commands,
                    room_manager,
                    &mut carnage_query, 
                    last_attribute_array, 
                    &room_config,
                );
                
                let new_z_index: f32 = room_manager.get_global_z_index();
                let current_z = room_manager.get_current_z_index()+2.;

                println!("Room width going into add map: {} room height: {}", room_width, room_height);
                // add new room to map relative to current room top door
                room_manager.add_room_to_map_from_right_door(
                    current_z as i32,
                    new_z_index as i32,
                    room_width as usize / TILE_SIZE as usize,
                    room_height as usize/ TILE_SIZE as usize,
                );

                println!("z index looking for current room: {} z index for room to add: {}", current_z, new_z_index);

                println!("before generate doors");

                // generate doors
                generate_doors(
                    commands,
                    room_manager,
                    max_x as f32,
                    max_y as f32,
                    current_z,
                );


                println!("before spawn items");
                //spawn_items_in_room(commands, &room_manager, &last_attribute_array, &room_config);


                println!("room_manager.current_z_index: {}", room_manager.get_current_z_index());
               println!("room_manager.global: {}", room_manager.get_global_z_index());
                // Spawn at left door

                info!("player successfully moved to new room... but maybe not.");

                for mut transform in player.iter_mut(){
                    transform.translation = Vec3::new(-max_x + TILE_SIZE as f32 * 2.0, TILE_SIZE as f32 / 2.0, room_manager.current_z_index);
                }
            }else{
                if let Some(room_val_unwrapped) = room_val {
                    // room_manager.room_array.get_room_from_storage(room_val_unwrapped as f32);
                    if let Some(room_dimensions) = room_manager.room_array.get_room_from_storage(room_val_unwrapped as f32) {
                        let width = room_dimensions.width;
                        let height = room_dimensions.height;
                        let max_x = room_dimensions.width as f32 / 2.0;
                        let max_y = room_dimensions.height as f32 / 2.0;
                        //generate room with set bounds

                        regenerate_existing_room(
                            commands,
                            room_manager,
                            width as usize,
                            height as usize,
                            room_val_unwrapped as f32,
                        );

                        // spawn at left door
                        for mut transform in player.iter_mut(){

                            transform.translation = Vec3::new(-max_x + TILE_SIZE as f32 * 2.0, TILE_SIZE as f32 / 2.0, room_val_unwrapped as f32);
                        }
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
                
                // generate the room with random bounds
                let (room_width, room_height, max_x, max_y, _z_index) = generate_random_room_with_bounds(
                    commands,
                    room_manager,
                    &mut carnage_query, 
                    last_attribute_array, 
                    &room_config,
                );
                
                let new_z_index: f32 = room_manager.get_global_z_index();
                let current_z = room_manager.get_current_z_index()+2.;

                println!("Room width going into add map: {} room height: {}", room_width, room_height);
                // add new room to map relative to current room top door
                room_manager.add_room_to_map_from_left_door(
                    current_z as i32,
                    new_z_index as i32,
                    room_width as usize / TILE_SIZE as usize,
                    room_height as usize / TILE_SIZE as usize,
                );

                println!("before generate doors");

                // generate doors
                generate_doors(
                    commands,
                    room_manager,
                    max_x as f32,
                    max_y as f32,
                    current_z,
                );

                println!("before spawn items");
               // spawn_items_in_room(commands, &room_manager, &last_attribute_array, &room_config);


               println!("room_manager.current_z_index: {}", room_manager.get_current_z_index());
               println!("room_manager.global: {}", room_manager.get_global_z_index());
                // Spawn the player a little away from the right door
                for mut transform in player.iter_mut(){
                    transform.translation = Vec3::new(max_x - TILE_SIZE as f32 * 2.0, TILE_SIZE as f32 / 2.0, room_manager.current_z_index);
                }
            }else{
                if let Some(room_val_unwrapped) = room_val {
                    // room_manager.room_array.get_room_from_storage(room_val_unwrapped as f32);
                    if let Some(room_dimensions) = room_manager.room_array.get_room_from_storage(room_val_unwrapped as f32) {
                        let width = room_dimensions.width;
                        let height = room_dimensions.height;
                        let max_x = room_dimensions.width as f32 / 2.0;
                        let max_y = room_dimensions.height as f32 / 2.0;
                        println!("Room width: {}, height: {}", width, height);
                        println!("max x: {} max y: {}", max_x, max_y);


                        regenerate_existing_room(
                            commands,
                            room_manager,
                            width as usize,
                            height as usize,
                            room_val_unwrapped as f32,
                        );
                        for mut transform in player.iter_mut(){
                            transform.translation = Vec3::new(max_x - TILE_SIZE as f32 * 2.0, TILE_SIZE as f32 / 2.0, room_manager.current_z_index);
                        }
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
                
                
                // generate the room with random bounds
                let (room_width, room_height, max_x, max_y, _z_index) = generate_random_room_with_bounds(
                    commands,
                    room_manager,
                    &mut carnage_query, 
                    last_attribute_array, 
                    &room_config,
                );
                
                let new_z_index: f32 = room_manager.get_global_z_index();
                let current_z = room_manager.get_current_z_index()+2.;

                println!("Room width going into add map: {} room height: {}", room_width, room_height);
                // add new room to map relative to current room top door
                room_manager.add_room_to_map_from_top_door(
                    current_z as i32,
                    new_z_index as i32,
                    room_width as usize / TILE_SIZE as usize,
                    room_height as usize / TILE_SIZE as usize,
                );

                println!("before generate doors");

                // generate doors
                generate_doors(
                    commands,
                    room_manager,
                    max_x as f32,
                    max_y as f32,
                    current_z,
                );

                println!("before spawn items");
                //spawn_items_in_room(commands, &room_manager, &last_attribute_array, &room_config);


                println!("room_manager.current_z_index: {}", room_manager.current_z_index);
                // Spawn the player a little away from the bottom door
                for mut transform in player.iter_mut(){
                    transform.translation = Vec3::new(TILE_SIZE as f32 / 2.0, -max_y + TILE_SIZE as f32 * 2.0, room_manager.current_z_index);
                }
            }else{
                if let Some(room_val_unwrapped) = room_val {
                    // room_manager.room_array.get_room_from_storage(room_val_unwrapped as f32);
                    if let Some(room_dimensions) = room_manager.room_array.get_room_from_storage(room_val_unwrapped as f32) {
                        let width = room_dimensions.width;
                        let height = room_dimensions.height;
                        let max_x = room_dimensions.width as f32 / 2.0;
                        let max_y = room_dimensions.height as f32 / 2.0;
                        println!("Room width: {}, height: {}", width, height);
                        println!("max x: {} max y: {}", max_x, max_y);
    
                        regenerate_existing_room(
                            commands,
                            room_manager,
                            width as usize,
                            height as usize,
                            room_val_unwrapped as f32,
                        );
                        for mut transform in player.iter_mut(){
                            transform.translation = Vec3::new(TILE_SIZE as f32 / 2.0, -max_y + TILE_SIZE as f32 * 2.0, room_manager.current_z_index);
                        }
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
                
                // generate the room with random bounds
                let (room_width, room_height, max_x, max_y, _z_index) = generate_random_room_with_bounds(
                    commands,
                    room_manager,
                    &mut carnage_query, 
                    last_attribute_array, 
                    &room_config,
                );
                
                let new_z_index: f32 = room_manager.get_global_z_index();
                let current_z = room_manager.get_current_z_index()+2.;

                println!("Room width going into add map: {} room height: {}", room_width, room_height);
                // add new room to map relative to current room top door
                room_manager.add_room_to_map_from_bottom_door(
                    current_z as i32,
                    new_z_index as i32,
                    room_width as usize / TILE_SIZE as usize,
                    room_height as usize / TILE_SIZE as usize,
                );

                
                println!("before generate doors");
                // generate doors
                generate_doors(
                    commands,
                    room_manager,
                    max_x as f32,
                    max_y as f32,
                    current_z,
                );

                println!("before spawn items");
               // spawn_items_in_room(commands, &room_manager, &last_attribute_array, &room_config);

               println!("room_manager.current_z_index: {}", room_manager.get_current_z_index());
               println!("room_manager.global: {}", room_manager.get_global_z_index());
                // Spawn the player a little away from the right door
                for mut transform in player.iter_mut(){
                    transform.translation = Vec3::new(TILE_SIZE as f32 / 2.0, max_y - TILE_SIZE as f32 * 2.0, room_manager.current_z_index);
                }
            }else{
                if let Some(room_val_unwrapped) = room_val {
                    // room_manager.room_array.get_room_from_storage(room_val_unwrapped as f32);
                    if let Some(room_dimensions) = room_manager.room_array.get_room_from_storage(room_val_unwrapped as f32) {
                        let width = room_dimensions.width;
                        let height = room_dimensions.height;
                        let max_x = room_dimensions.width as f32 / 2.0;
                        let max_y = room_dimensions.height as f32 / 2.0;

                        regenerate_existing_room(
                            commands,
                            room_manager,
                            width as usize,
                            height as usize,
                            room_val_unwrapped as f32,
                        );
                        for mut transform in player.iter_mut(){
                            transform.translation = Vec3::new(TILE_SIZE as f32 / 2.0, max_y - TILE_SIZE as f32 * 2.0, room_manager.current_z_index);
                        }
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