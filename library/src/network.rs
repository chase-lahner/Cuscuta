use bevy::prelude::*;
use flexbuffers::FlexbufferSerializer;
use serde::{Deserialize, Serialize};
use std::{net::UdpSocket, time};
use std::io;
use crate::player::{InputQueue, ServerPlayerBundle};
use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};


#[derive(Component, Serialize, Deserialize, Copy, Clone, PartialEq, Debug)]
pub struct Timestamp{
    pub time: u128
}

#[derive(Resource)]
pub struct Sequence{
    num: u64
}

impl Sequence{
    pub fn up(&mut self){
        self.num += 1;
    }
    
    pub fn get(&mut self) -> u64{
        self.num
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


/*#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PlayerPacket{
    pub id: u8,
    pub transform_x: f32,
    pub transform_y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
}**/

/**#[derive(Serialize, Deserialize)]
pub struct NewPlayerC2S{
   pub client_bundle: ServerPlayerBundle
}


#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct NewPlayerPacket{
    pub id: u8,
    pub key: KeyCode,
    pub room: u8,
}*/

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PlayerC2S{
    pub head: Header,
    pub key: KeyCode,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PlayerS2C{
    pub head: Header,
    pub transform: Transform,
    pub velocity: Vec2,
    pub health: f32,
    pub max_health: f32,
    pub crouch: bool,
    pub attack: bool,
    pub roll: bool,
    pub sprint: bool,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct EntityS2C{
    pub head: Header,
    pub entid: u8,
    pub transform: Transform,
    pub velocity: Vec2,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct MapS2C{
    pub head: Header,
    pub matrix: Vec<Vec<u8>>,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct IdPacket{
    pub head: Header
}
#[derive(Component, Serialize,Deserialize, PartialEq, Debug)]
pub struct Header{
    pub network_id: u8,
    pub sequence_num: u64,
    pub timestamp: u128

}

/**#[derive(Serialize,Deserialize,PartialEq,Debug)]
pub struct TimeIdPacket {
    pub header: Header
}*/

/**#[derive(Component, Serialize, Deserialize, PartialEq, Debug)]
pub struct InputPacket{
    header: Header,
    key_pressed: KeyCode,
}*/

/**impl InputPacket{
    pub fn new( header: Header, keycode: KeyCode) -> Self {
        Self { header: header , key_pressed: keycode }
    }
    // SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
}*/

#[derive(Serialize, Deserialize)]
pub enum ClientPacket{
    PlayerPacket(PlayerC2S),
    IdPacket(IdPacket),
}

#[derive(Serialize, Deserialize)]
pub enum ServerPacket{
    PlayerPacket(PlayerS2C),
    EntityPacket(EntityS2C),
    MapPacket(MapS2C),
    IdPacket(IdPacket),
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
