use crate::cuscuta_resources::{self, FlexSerializer, Velocity, PLAYER_DATA};
use crate::player::*;
use bevy::prelude::*;
use flexbuffers::FlexbufferSerializer;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::net::UdpSocket;
use std::{collections::HashMap, net::SocketAddr};
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
   pub client_bundle: ServerPlayerBundle
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct IdPacket{
    pub id: u8
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum SendablePacket{
    PlayerPacket(PlayerPacket),
    IdPacket(IdPacket),
    NewPlayerPacket(NewPlayerPacket)
}

pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    // will slice anything into u8 array !! https://stackoverflow.com/questions/28127165/how-to-convert-struct-to-u8
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}

pub unsafe fn u8_to_f32(input_arr: &[u8]) -> (&[u8], &[f32], &[u8]) {
    // prefix, actual stuff, suffix
    input_arr.align_to::<f32>()
}

/* Transforms current player state into u8 array that
 * we can then send across the wire to be deserialized once it arrives */
 //DEPRECATED
// pub fn serialize_player(
//     player : Query<(&Transform, &Velocity, &NetworkId ), With<Player>>,
//     socket : Res<UDP>,
//     mut serializer: ResMut<FlexSerializer>,
// )
// {
//     /* Deconstruct out Query. SHould be client side so we can do single */
//     let (t, v, i) = player.single();
//     let outgoing_state = PlayerPacket {
//         id: i.id,
//         transform_x: t.translation.x,
//         transform_y: t.translation.y,
//         velocity_x: v.velocity.x,
//         velocity_y: v.velocity.y,
//     };

//     let to_send: SendablePacket = SendablePacket::PlayerPacket(outgoing_state);

//     to_send.serialize(&mut serializer.serializer).unwrap();

//     /* slices are gross and ugl and i need +1 sooooo vec back to slice ig */
//     // let opcode: &[u8] = std::slice::from_ref(&PLAYER_DATA);
//     // let packet_vec  = append_opcode(serializer.serializer.view(), opcode);
//     // let packet: &[u8] = &(&packet_vec);
//     let packet: &[u8] = serializer.serializer.view();
//     /* beam him up scotty */
//     socket.socket.send_to(&packet, cuscuta_resources::SERVER_ADR).unwrap();
// } 

pub fn append_opcode(
    slice: &[u8],
    opcode: &[u8],
) -> Vec<u8> {
    let mut vec: Vec<u8> = Vec::with_capacity(slice.len() + 1);
    vec.write(slice).unwrap();
    vec.write(opcode).unwrap();
    vec
}

pub fn get_ip_addr() -> String {
    print!("Enter the IP Address  + enter then port number + enter you would like your socket to bind to: \n");
    let mut buffer = String::new();
    let _ = io::stdin().read_line(&mut buffer); // read from stdin

    buffer // return buffer

}
