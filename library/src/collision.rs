use bevy::prelude::*;

use crate::{cuscuta_resources::*, room_gen::*};


pub fn set_collide(room_manager: &mut RoomManager, x: usize, y: usize, val: u32) {
    // convert world coordinates to grid indices
    let grid_x = x / (TILE_SIZE as usize);
    let grid_y= y / (TILE_SIZE as usize);

    // get current grid
    let current_grid = room_manager.current_grid();

    // get width & height
    let arr_w_limit = current_grid.len();
    let arr_h_limit = current_grid[0].len();

    if grid_x < arr_w_limit && grid_y < arr_h_limit {
        current_grid[grid_x][grid_y] = val;
    } else {
       //println!("Error: index out of bounds for collision at ({}, {})", grid_x, grid_y);
    }

}
pub struct Aabb {
    pub min: Vec2,
    pub max: Vec2,
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
pub fn aabb_collision(player_aabb: &Aabb, enemy_aabb: &Aabb) -> bool {
    player_aabb.intersects(&enemy_aabb)
}