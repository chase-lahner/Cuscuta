/* Constants for Cuscuta
 * use `mod constants;` to grab.
 * I hope this dead_code isn't package wide... */
#![allow(dead_code)]
use std::net::SocketAddr;
use bevy::prelude::*;
#[derive(Component)]
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
