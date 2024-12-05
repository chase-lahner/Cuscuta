/* Constants for Cuscuta
 * use `mod constants;` to grab.
 * I hope this dead_code isn't package wide... */
#![allow(dead_code)]
use std::net::SocketAddr;
use bevy::prelude::*;
use flexbuffers::FlexbufferSerializer;
use serde::{Deserialize, Serialize};

use crate::network::KillEnemyPacket;


pub const CLIENT_ID_DEFAULT: u8 = 0;

pub const TICKS_PER_SECOND: f64 = 60.;

pub const SERVER_ADR: &str = "127.0.0.1:5001"; //136.142.159.86:5001



pub const GET_PLAYER_ID_CODE: u8 = 255;
pub const PLAYER_DATA: u8 = 254;
/* end opcode!! */

pub const TITLE: &str = "Cuscuta Demo";// window title
pub const WIN_W: f32 = 1280.;// window width
pub const WIN_H: f32 = 720.;// window height

pub const PLAYER_SPEED: f32 = 480.; 
pub const ACCELERATION_RATE: f32 = 4800.; 
pub const SPRINT_MULTIPLIER: f32 = 2.0;
pub const CROUCH_MULTIPLIER: f32 = 0.25;

pub const PLAYER_SPRITE_COL: u32 = 4;
pub const PLAYER_SPRITE_ROW: u32 = 16;

pub const ENEMY_SPEED: f32 = 160.;
pub const NUMBER_OF_ENEMIES: u32 = 10;
pub const ENEMY_SPOT_DISTANCE: f32 = 192.;

/* for cymbal monkey item */
pub const MONKEY_HANDLE: &str = "monkey/cymbal_monkey1x2.png";
pub const MONKEY_SPRITE_COL: u32 = 1;
pub const MONKEY_SPRITE_ROW: u32 = 2;

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

impl Pot{
    pub fn new() -> Self{
        Self{
            touch: 0
        }
    }
}
#[derive(Component)]
pub struct Wall;

#[derive(Component, Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct Health{
    pub max: f32,
    pub current: f32
}
impl Health {
    pub fn new() -> Self {
        Self {
            max: 100.,
            current: 100.
        }
    }

    pub fn set(&mut self, heal: &Health){
        self.current = heal.current;
        self.max = heal.max;
    }
}

#[derive(Resource)]
pub struct PlayerCount{
    pub count: u8
}

#[derive(Resource)]
pub struct FlexSerializer{
    pub serializer: FlexbufferSerializer
}

#[derive(Resource)]
pub struct ClientId{
    pub id: u8
}

impl ClientId{
    pub fn new() -> Self{
        Self{
            id: CLIENT_ID_DEFAULT
        }
    }
}

#[derive(Resource)]
pub struct AddressList{
    pub list: Vec<SocketAddr>,
}
impl AddressList{
    pub fn new() -> Self{
        Self{
            list: Vec::new()
        }
    }
}

#[derive(Resource)]
pub struct EnemiesToKill{
    pub list: Vec<KillEnemyPacket>,
}

impl EnemiesToKill{
    pub fn new() -> Self{
        Self{
            list: Vec::new()
        }
    }
}



#[derive(Component, Serialize, Deserialize, Clone, Copy)]
pub struct Velocity {
    pub velocity: Vec2,
}
impl Velocity {
    pub fn new() -> Self {
        Self {
            velocity: Vec2::splat(0.),
        }
    }

    pub fn set(&mut self, velo: &Vec2){
        self.velocity = *velo;
    }
}

impl From<Vec2> for Velocity {
    fn from(velocity: Vec2) -> Self {
        Self { velocity }
    }
}
