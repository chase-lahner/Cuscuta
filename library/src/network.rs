use bevy::prelude::*;
use flexbuffers::FlexbufferSerializer;
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::io;

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
pub struct NewPlayerPacket{
    pub id: u8,
    pub key: KeyCode,
    pub room: u8,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct IdPacket{
    pub id: u8
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum SendablePacket{
    PlayerPacket(PlayerPacket),
    IdPacket(IdPacket)
}

pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    // will slice anything into u8 array !! https://stackoverflow.com/questions/28127165/how-to-convert-struct-to-u8
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
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
