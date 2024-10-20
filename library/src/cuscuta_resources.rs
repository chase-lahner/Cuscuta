/* Constants for Cuscuta
 * use `mod constants;` to grab.
 * I hope this dead_code isn't package wide... */
#![allow(dead_code)]
use bevy::prelude::*;
use serde::Serialize;







pub const SERVER_ADR: &str = "localhost:5001";
pub const GET_PLAYER_ID_CODE: u8 = 255;

pub const TITLE: &str = "Cuscuta Demo";// window title
pub const WIN_W: f32 = 1280.;// window width
pub const WIN_H: f32 = 720.;// window height

pub const PLAYER_SPEED: f32 = 480.; 
pub const ACCELERATION_RATE: f32 = 4800.; 
pub const SPRINT_MULTIPLIER: f32 = 2.0;
pub const CROUCH_MULTIPLIER: f32 = 0.5;

pub const ENEMY_SPEED: f32 = 0.;
pub const NUMBER_OF_ENEMIES: u32 = 10;

pub const TILE_SIZE: u32 = 32; 

pub const LEVEL_W: f32 = 4800.; 

pub const LEVEL_H: f32 = 1600.; 

pub const ARR_W: usize = (LEVEL_W as usize) / 32;

pub const ARR_H: usize = (LEVEL_H as usize) / 32;

/* (0,0) is center level,          
 * this gives us easy coordinate usage */
pub const MAX_X: f32 = LEVEL_W / 2.;
pub const MAX_Y: f32 = LEVEL_H / 2.;

pub const ANIM_TIME: f32 = 0.2;


#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);// for switching through animation frames

#[derive(Component, Deref, DerefMut)]
pub struct AnimationFrameCount(pub usize);

//struct Brick;

#[derive(Component)]
pub struct Background;

#[derive(Component)]
pub struct Pot{
    pub touch: u8
}

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Door;

#[derive(Component)]
pub struct Health{
    pub max: f32,
    pub current: f32
}

#[derive(Component, Serialize)]
pub struct Velocity {
    pub velocity: Vec2,
}
impl Velocity {
    pub fn new() -> Self {
        Self {
            velocity: Vec2::splat(0.),
        }
    }
}

impl From<Vec2> for Velocity {
    fn from(velocity: Vec2) -> Self {
        Self { velocity }
    }
}
