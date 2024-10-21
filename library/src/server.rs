use std::net::{SocketAddr, UdpSocket};

use bevy::{prelude::*, utils::HashMap};
use flexbuffers::FlexbufferSerializer;
use network::*;
use serde::Serialize;

use crate::{cuscuta_resources::{self, FlexSerializer, PlayerCount, Velocity}, network, player::{NetworkId, Player}};

/* Upon request, sends an id to client */
pub fn send_id(
    source_addr : SocketAddr,
    server_socket: &UdpSocket, 
    mut n_p: ResMut<PlayerCount>,
    serializer: &mut FlexbufferSerializer
) {
    /* assign id, update player count */
    let player_id: u8 = 255 - n_p.count;
    n_p.count +=1;

    /* lil  baby struct to serialize */
    let to_send = IdPacket{ id: player_id};
    to_send.serialize(  &mut *serializer ).unwrap();

    /* once serialized, we throw our opcode on the end */
    const SIZE:usize = size_of::<IdPacket>();
    let mut packet = [0;SIZE+1];
    packet[..SIZE].clone_from_slice(serializer.view());
    packet[SIZE] = cuscuta_resources::GET_PLAYER_ID_CODE;

    /* Send it on over! */
    server_socket.send_to(&packet, source_addr).unwrap();
}

/* Server side listener for packets,  */
pub fn listen(
    udp: Res<UDP>,
    commands: Commands,
    players: Query<(&Velocity, &Transform, &NetworkId), With<Player>>,
    serializer_q: ResMut<FlexSerializer>,
    n_p: ResMut<PlayerCount>,
) -> std::io::Result<()>{// really doesn;t need to return this am lazy see recv_from line
    /* to hold msg */
    let mut buf: [u8; 1024] = [0;1024];
    let (amt, src) = udp.socket.recv_from(&mut buf)?;
    let serializer = &mut serializer_q.into_inner().serializer;
    /* trim trailing 0s */
    let t_buf = &buf[..amt-1];

    /* when we serialize, we throw our opcode on the end, so we know how to
    * de-serialize... jank? maybe.  */
    let opcode = buf[amt];

    match opcode{
        cuscuta_resources::GET_PLAYER_ID_CODE => 
            send_id(src, &udp.socket, n_p,serializer),
        cuscuta_resources::PLAYER_DATA =>
            update_player_state(players, t_buf,),
        _ => 
            something()//TOTO

    };


    Ok(())
}

fn something(){}

pub fn update_player_state(
    /* fake query, passed from above system */
    mut player: Query<(&Velocity, &Transform, &NetworkId), With<Player>>,
    mut buf: &[u8],
) { 


}