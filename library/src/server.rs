use std::net::{SocketAddr, UdpSocket};

use bevy::{prelude::*, utils::HashMap};
use flexbuffers::FlexbufferSerializer;
use network::*;
use serde::Serialize;

use crate::{cuscuta_resources, network};


pub fn recieve_packets(
    udp: Res<UDP>
)
{
    let mut buf: [u8;1024] = [0;1024];
    loop{
        let (amt, src) = udp.socket.recv_from(&mut buf).unwrap();
        /* TODO need to deseralize first  */
        let opcode = buf[0];
        

    }
}

pub fn send_id(socket_addr : SocketAddr, mut player_hash : HashMap<String, u8>, n_p: &mut u8, s: &mut FlexbufferSerializer, socket: &UdpSocket ){
    let arg_ip = socket_addr.ip();
    let ip_string = arg_ip.to_string();
    let player_id: u8 = 255 - *n_p;

    *n_p +=1;

    player_hash.insert(ip_string, player_id);

    let to_send = UDPHeader{ opcode: cuscuta_resources::GET_PLAYER_ID_CODE, id: player_id};

    to_send.serialize(  &mut *s ).unwrap();

    socket.send_to(s.view(), "localhost:5000").unwrap();

    println!("SENT!");




    



    
}