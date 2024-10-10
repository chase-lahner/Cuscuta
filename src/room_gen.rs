use bevy::prelude::*;

use crate::cuscuta_resources::*;


pub struct Aabb {
    min: Vec2,
    max: Vec2,
}

impl Aabb {
    pub fn new(center: Vec3, size: Vec2) -> Self {
        let half_size = size / 2.0;
        Self {
            min: Vec2::new(center.x - half_size.x, center.y - half_size.y),
            max: Vec2::new(center.x + half_size.x, center.y + half_size.y),
        }
    }

    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }
}
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
