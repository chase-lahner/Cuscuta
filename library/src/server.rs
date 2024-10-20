use std::net::{SocketAddr, UdpSocket};

use bevy::{prelude::*, utils::HashMap};
use flexbuffers::FlexbufferSerializer;
use network::*;
use serde::Serialize;

use crate::{cuscuta_resources::{self, FlexSerializer, PlayerCount, Velocity}, network, player::{NetworkId, Player}};


pub fn send_id(
    source_addr : SocketAddr,
    mut n_p: ResMut<PlayerCount>,
    server_socket: &UdpSocket, 
    mut serializer: ResMut<FlexSerializer>
) {
    let arg_ip = source_addr.ip();
    let ip_string = arg_ip.to_string();
    let player_id: u8 = 255 - n_p.count;

    n_p.count +=1;

    let to_send = IdPacket{ id: player_id};


    to_send.serialize(  &mut serializer.serializer ).unwrap();
    const SIZE:usize = size_of::<IdPacket>();
    let mut packet = [0;SIZE+1];
    packet[..SIZE].clone_from_slice(serializer.serializer.view());
    packet[SIZE] = cuscuta_resources::PLAYER_DATA;
    server_socket.send_to(packet, source_addr).unwrap();

    println!("SENT!");
}

/* Server side listener for packets,  */
pub fn listen(
    udp: Res<UDP>,
    commands: Commands,
    mut player: Query<(&Velocity, &Transform, &NetworkId), With<Player>>
) -> std::io::Result<()>{// really doesn;t need to return this am lazy see recv_from line
    /* to hold msg */
    let mut buf: [u8; 1024] = [0;1024];
    let (amt, src) = udp.socket.recv_from(&mut buf)?;
    /* trim trailing 0s */
    let mut t_buf = &buf[..amt];

    let mut serializer: FlexbufferSerializer = flexbuffers::FlexbufferSerializer::new();
    /* when we serialize, we throw our opcode on the end, so we know how to
    * de-serialize... jank? maybe.  */
    let opcode = buf[amt];

    match opcode{
        cuscuta_resources::GET_PLAYER_ID_CODE => send_id(src, udp)
        _ => //TOTO

    }


    Ok(())
}