use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::network::{PlayerPacket, UDPHeader, UDP};
use crate::cuscuta_resources::{self, Velocity};
use crate::player::*;

pub fn recv_id(socket: Res<UDP>, mut player: Query<&mut NetworkId, With<Player>> )  // function to recieve ID from server (after we send a request for one)
{
    
    let mut network_id = player.single_mut(); // network id is part of player struct
    let mut buf: [u8; 1024] = [0; 1024]; // buffer
    let (amt, _src) = socket.socket.recv_from(&mut buf).unwrap(); // recieve buffer from server
    let t_buf = &buf[..amt];

    let r = flexbuffers::Reader::get_root(t_buf).unwrap();

    let ds_struct = UDPHeader::deserialize(r).unwrap();

    if ds_struct.opcode == cuscuta_resources::GET_PLAYER_ID_CODE
    {
        network_id.id = ds_struct.id;
    }

    println!("ASSIGNED ID: {:?}", network_id.id);
}

pub fn id_request(
    player: Query<&NetworkId, With<Player>>,
    socket: Res<UDP>,
) {
    let i = player.single();
    let i_d = i.id;
    let mut s = flexbuffers::FlexbufferSerializer::new();
    let to_send = UDPHeader {
        opcode: cuscuta_resources::GET_PLAYER_ID_CODE,
        id: i_d,
    };

    to_send.serialize(&mut s).unwrap();

    socket
        .socket
        .send_to(s.view(), cuscuta_resources::SERVER_ADR)
        .unwrap();
}

/* Transforms current player state into u8 array that
 * we can then send across the wire to be deserialized once it arrives */
 pub fn send_player(
    player : Query<(&Transform, &Velocity, &NetworkId ), With<Player>>,
    socket : Res<UDP>
)
{
    /* Deconstruct out Query. SHould be client side so we can do single */
    let (t, v, i) = player.single();
    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    let header = UDPHeader {opcode: 1, id: i.id};
    let outgoing_state = PlayerPacket {
        head: header, 
        transform_x: t.translation.x,
        transform_y: t.translation.y,
        velocity_x: v.velocity.x,
        velocity_y: v.velocity.y,
    };

    outgoing_state.serialize(&mut serializer).unwrap();

    let r  = flexbuffers::Reader::get_root(serializer.view()).unwrap();
   
    socket.socket.send_to(serializer.view(), cuscuta_resources::SERVER_ADR).unwrap();
} 