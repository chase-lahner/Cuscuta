use crate::cuscuta_resources::{self, Velocity};
use crate::player::*;
use bevy::prelude::*;
use flexbuffers::FlexbufferSerializer;
use serde::{Deserialize, Serialize};
use std::mem;
use std::net::UdpSocket;
use std::{collections::HashMap, net::SocketAddr};

#[derive(Resource)]
pub struct UDP {
    pub socket: UdpSocket,
}

#[derive(Resource)]
pub struct BufSerializer {
    pub serializer: FlexbufferSerializer,
}


#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PlayerPacket{
    pub transform_x: f32,
    pub transform_y: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct IdPacket{
    pub id: u8
}

pub fn send_movement_info(
    socket: Res<UDP>, 
    player: Query<&Transform, With<Player>>
) {
    let pt = player.single();
    let x = pt.translation.x;
    let y = pt.translation.y;
    let x_int = unsafe { any_as_u8_slice(&x) };
    let y_int = unsafe { any_as_u8_slice(&y) };
    let buf: [u8; 2] = [x_int[0], y_int[0]];
    //print!("{:?}", &buf);

    socket.socket.send_to(&buf, "localhost:5001").unwrap();
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
pub fn serialize_player(
    player : Query<(&Transform, &Velocity, &NetworkId ), With<Player>>,
    socket : Res<UDP>
)
{
    /* Deconstruct out Query. SHould be client side so we can do single */
    let (t, v, i) = player.single();
    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    let outgoing_state = PlayerPacket {
        transform_x: t.translation.x,
        transform_y: t.translation.y,
        velocity_x: v.velocity.x,
        velocity_y: v.velocity.y,
    };

    outgoing_state.serialize(&mut serializer).unwrap();

    const SIZE:usize = size_of::<PlayerPacket>();
    let mut packet: [u8;SIZE+1] = [0;SIZE+1];
    packet[..SIZE].clone_from_slice(serializer.view());
    packet[SIZE] = cuscuta_resources::PLAYER_DATA;
    socket.socket.send_to(&packet, cuscuta_resources::SERVER_ADR).unwrap();
} 



pub fn server_assign_id(socket_addr : SocketAddr, mut player_hash : HashMap<String, u8>, n_p: &mut u8) -> u8{
    let arg_ip = socket_addr.ip();
    let ip_string = arg_ip.to_string();
    let player_id: u8 = 255 - *n_p;

    *n_p += 1;

    player_hash.insert(ip_string, player_id);

    player_id
}
