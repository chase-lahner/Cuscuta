use std::net::{SocketAddr, UdpSocket};
use std::collections::HashMap;
use cuscuta_resources::PlayerCount;
use flexbuffers::FlexbufferSerializer;
use serde::{ Deserialize, Serialize};
use library::*;
use bevy::prelude::*;
use network::get_ip_addr;

/* Rate at which we will be sending/recieving packets */
const TICKS_PER_SECOND: f64 = 60.;

fn main() {
    App::new()
    .insert_resource(Time::<Fixed>::from_hz(TICKS_PER_SECOND))
    .insert_resource(PlayerCount{count:0})
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, init::server_setup)
    .add_systems(FixedUpdate, (
        server::listen, 
        ///player::update_player_position.after(server::listen),
        server::send_player
    ))
    .add_systems(FixedUpdate, server::send_player)
    .run();
}


fn old_main() -> std::io::Result<()>{
    /* binding to our little mailbox */
    let raw_ip_str = get_ip_addr();
    let trimmed_ip_str = raw_ip_str.trim(); 
    let socket = UdpSocket::bind(trimmed_ip_str).unwrap(); // "localhost:5001"
    let mut num_players: u8 = 0; // TODO: decrement when disconnect, idk its like connectionless so we need to send a packet saying to dec when we disconnect 
    let mut player_hash: HashMap<String, u8> = HashMap::new();
    let mut s = flexbuffers::FlexbufferSerializer::new();
    /* buffer will be 100 msgs 1024B in length */
    let mut buf: [u8; 1024] = [0;1024];
    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        println!("amount: {:?}", amt);
        let mut t_buf = &buf[..amt];
        // if buf[0] == 255 as u8 // if we recieve a packet requesting an ID
        // {
        //     //print!("sending socket");
        //     let to_send: &[u8;2] = &[255, assign_id(src, player_hash.clone(), &mut num_players)]; // u8 array with code letting client know its an id, and then the id itself
        //     socket.send_to(to_send, "localhost:5000").unwrap(); // send the packet
        // }
        //deserialize_and_delegate(&mut t_buf, src, player_hash.clone(), &mut num_players, &mut s, &socket );
      //  println!("{:?}",&buf);
       // socket.send_to(b"From server", "localhost:5000").unwrap();
    }

}

// pub fn send_id(socket_addr : SocketAddr, mut player_hash : HashMap<String, u8>, n_p: &mut u8, s: &mut FlexbufferSerializer, socket: &UdpSocket ){
//     let arg_ip = socket_addr.ip();
//     let ip_string = arg_ip.to_string();
//     let player_id: u8 = 255 - *n_p;

//     *n_p +=1;

//     player_hash.insert(ip_string, player_id);

//     let to_send = UDPHeader{ opcode: cuscuta_resources::GET_PLAYER_ID_CODE, id: player_id};

//     to_send.serialize(  &mut *s ).unwrap();

   // socket.send_to(s.view(), socket_addr).unwrap(); // localhost:5000

//     println!("SENT!");




    



    
// }

// fn deserialize_and_delegate(packet: &[u8], socket_addr : SocketAddr , player_hash : HashMap<String, u8>, n_p: &mut u8, s:  &mut FlexbufferSerializer, socket: &UdpSocket)
// {
//     println!("{:?}",packet);
//     let r = flexbuffers::Reader::get_root(packet).unwrap();
//     let ds_struct = UDPHeader::deserialize(r).unwrap();
//     if ds_struct.opcode == cuscuta_resources::GET_PLAYER_ID_CODE
//     {
//         send_id(socket_addr, player_hash, n_p, s, socket);
//     }
//     // future deserialization logic for other packets?

// }

// fn deserialize_player_x_y_header(ds_struct : UDPHeader)
// {
//     println!("Deserialized id: {:?}\n", ds_struct.id);
//     println!("Deserialized opcode: {:?}\n", ds_struct.opcode);

// }
