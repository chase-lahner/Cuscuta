use bevy::prelude::*;
use flexbuffers::FlexbufferSerializer;
use serde::{Deserialize, Serialize};
use std::{net::UdpSocket, time};
use std::io;
use crate::freshwork::Timestamp;
use crate::player::ServerPlayerBundle;
use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};

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
pub struct PlayerPacket{
    pub id: u8,
    pub transform_x: f32,
    pub transform_y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
}
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct InputPacket{
    header: Header,
    key_pressed: KeyCode,
}

impl InputPacket{
    pub fn new( header: Header, keycode: KeyCode) -> Self {
        Self { header: header , key_pressed: keycode }
    }
    // SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
}

#[derive(Serialize, Deserialize)]
pub struct NewPlayerPacket{
   pub client_bundle: ServerPlayerBundle
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct IdPacket{
    pub id: u8,
}
#[derive(Serialize,Deserialize, PartialEq, Debug)]
pub struct Header{
    pub network_id: u8,
    pub sequence_num: u64,
    pub timestamp: u128

}
#[derive(Serialize,Deserialize,PartialEq,Debug)]
pub struct TimeIdPacket {
    pub header: Header
}

#[derive(Serialize, Deserialize)]
pub enum SendablePacket{
    PlayerPacket(PlayerPacket),
    IdPacket(IdPacket),
    NewPlayerPacket(NewPlayerPacket)
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
