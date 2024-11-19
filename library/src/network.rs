use bevy::prelude::*;
use flexbuffers::FlexbufferSerializer;
use serde::{Deserialize, Serialize};
use std::{net::UdpSocket, time};
use std::io;
use crate::enemies::{EnemyId, EnemyMovement};
use crate::player::{InputQueue, ServerPlayerBundle};
use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};
use crate::cuscuta_resources::Health;

#[derive(Component, Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
pub struct Timestamp{
    pub time: u64
}
impl Timestamp {
    pub fn new(time:u64) -> Self {
        Self{
            time:time
        }
    }

}

#[derive(Resource)]
pub struct Sequence{
    num: u64
}

impl Sequence{
    /* everytime we use a sequence # we should increment */ 
    pub fn geti(&mut self) -> u64{
        self.num += 1;
        self.num - 1
    }
    
    pub fn new() -> Self{
        Self{
            num: 0
        }
    }
}

#[derive(Resource)]
pub struct ServerSequence{
    num: u64
}

impl ServerSequence{
    pub fn up(&mut self){
        self.num +=1;
    }

    pub fn get(&mut self) -> u64{
        self.num
    }
}

#[derive(Resource, Component)]
pub struct UDP {
    pub socket: UdpSocket,
}

pub struct UserInputAddr { 
    pub user_string: String,
}

#[derive(Resource)]
pub struct BufSerializer {
    pub serializer: FlexbufferSerializer,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PlayerC2S{
    pub head: Header,
    pub key: Vec<KeyCode>,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PlayerS2C{
    pub head: Header,
    pub transform: Transform,
    pub velocity: Vec2,
    pub health: Health,
    pub crouch: bool,
    pub attack: bool,
    pub roll: bool,
    pub sprint: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct MapS2C{
    pub head: Header,
    pub matrix: Vec<Vec<u8>>,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct EnemyS2C{
    pub head: Header,
    pub enemytype: EnemyId,
    pub movement: EnemyMovement,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct IdPacket{
    pub head: Header
}
#[derive(Component, Serialize,Deserialize, PartialEq, Debug)]
pub struct Header{
    pub network_id: u8,
    pub sequence_num: u64,
    pub timestamp: u64
}
impl Header{
    pub fn new(id: u8, seq: u64, time: u64)-> Self{
        Self{
            network_id: id,
            sequence_num: seq,
            timestamp: time,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum ClientPacket{
    PlayerPacket(PlayerC2S),
    IdPacket(IdPacket),
}

#[derive(Serialize, Deserialize)]
pub enum ServerPacket{
    PlayerPacket(PlayerS2C),
    MapPacket(MapS2C),
    IdPacket(IdPacket),
    EnemyPacket(EnemyS2C),
}

pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] { // will slice anything into u8 array 
    
    ::core::slice::from_raw_parts((p as *const T) as *const u8,
     ::core::mem::size_of::<T>())
}

pub unsafe fn u8_to_f32(input_arr: &[u8]) -> (&[u8], &[f32], &[u8]) {
    // prefix, actual stuff, suffix
    input_arr.align_to::<f32>()
}


pub fn get_ip_addr() -> String {
    print!("Enter the IP Address  + enter then port number + enter you would like your socket to bind to: \n");
    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer); // read from stdin

    buffer // return buffer

}
