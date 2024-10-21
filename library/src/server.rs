use std::net::{SocketAddr, UdpSocket};

use bevy::{prelude::*, utils::HashMap};
use flexbuffers::FlexbufferSerializer;
use network::*;
use serde::Serialize;

use crate::{cuscuta_resources::{self, FlexSerializer, PlayerCount, Velocity, GET_PLAYER_ID_CODE}, network, player::{NetworkId, Player}};

/* Upon request, sends an id to client */
pub fn send_id(
    source_addr : SocketAddr,
    server_socket: &UdpSocket, 
    n_p: &mut PlayerCount,
    serializer: &mut FlexbufferSerializer
) {
    /* assign id, update player count */
    let player_id: u8 = 255 - n_p.count;
    n_p.count += 1;

    /* lil baby struct to serialize */
    let to_send = IdPacket{ id: player_id};
    to_send.serialize(  &mut *serializer ).unwrap();

    /* once serialized, we throw our opcode on the end */
    let opcode: &[u8] = std::slice::from_ref(&GET_PLAYER_ID_CODE);
    let packet_vec  = append_opcode(serializer.view(), opcode);
    let packet: &[u8] = &(&packet_vec);


    /* Send it on over! */
    server_socket.send_to(&packet, source_addr).unwrap();
}

/* Server side listener for packets,  */
pub fn listen(
    udp: Res<UDP>,
    mut commands: Commands,
    players: Query<(&mut Velocity, &mut Transform, &mut NetworkId), With<Player>>,
    mut serializer_q: ResMut<FlexSerializer>,
    mut n_p: ResMut<PlayerCount>,
) {
    info!("Listening!!");
    /* to hold msg */
    let mut buf: [u8; 1024] = [0;1024];
    let (amt, src) = udp.socket.recv_from(&mut buf).unwrap();
    info!("Read!");
    let serializer = &mut serializer_q.as_mut().serializer;
    
    /* when we serialize, we throw our opcode on the end, so we know how to
    * de-serialize... jank? maybe.  */
    let opcode = buf[amt -1];
    
    /* trim trailing 0s */
    let t_buf = &buf[..amt-1];

    
    info!("{:?}",buf);
    info!("opcode::{}",&opcode);
    match opcode{
        cuscuta_resources::GET_PLAYER_ID_CODE => {
            info!("sending id to client");
            send_id(src, &udp.socket, n_p.as_mut(),serializer)},
        cuscuta_resources::PLAYER_DATA =>
            update_player_state(players, t_buf),
        _ => 
            something()//TOTO

    };
}

fn something(){}
