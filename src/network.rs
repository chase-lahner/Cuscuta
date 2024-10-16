use bevy::a11y::accesskit::CustomAction;
use bevy::prelude::*;
use std::str::from_utf8;
use std::{any, net::UdpSocket};
use crate::{player, NetworkId, Player};
use serde::{Serialize, Deserialize};
use crate:: cuscuta_resources;
use flexbuffers;


#[derive(Resource)]
pub struct UDP{
    pub socket: UdpSocket
}


// UNUSED as of now
pub fn recv_packet(
    socket: Res<UDP>
){
    let mut buf = [0;1024];
    let (_amt, _src) = socket.socket.recv_from(&mut buf).unwrap();
    //println!("{}", String::from_utf8_lossy(&buf));
}


pub fn recv_id(socket: Res<UDP>, mut player: Query<&mut NetworkId, With<Player>> )  // function to recieve ID from server (after we send a request for one)
{
 //   print!("RECIEVING");
    let mut network_id = player.single_mut(); // network id is part of player struct
    let mut buf: [u8; 1024] = [0;1024]; // buffer 
    let (_amt , _src )  = socket.socket.recv_from(&mut buf).unwrap(); // recieve buffer from server
  //  print!("{:?}",buf[0]);
    if buf[0] == cuscuta_resources::GET_PLAYER_ID_CODE { // 255, if first val of udp packet (like key ig) is 255, we know it's a server response packet
     //   print!("ID!: {:?}", buf[1]);
        network_id.id = buf[1]; // assign player id to id we get from server
      //  print!("DID IT WORK: {:?}", network_id.id);
    }
    else{
        return;
    }
}

pub fn send_id_packet( // function that sends a packet telling server we want an id
    socket: Res<UDP>,
) {
   // print!("sending to server!");
    let buf: [u8; 1] = [cuscuta_resources::GET_PLAYER_ID_CODE]; // code to tell server we want an ID
    socket.socket.send_to(&buf, cuscuta_resources::SERVER_ADR).unwrap(); // send to server
}


pub fn send_movement_info(
    socket: Res<UDP>,
    player: Query<&Transform, With<Player>>,
    
) {
    let pt = player.single();
    let x = pt.translation.x;
    let y = pt.translation.y;
    let x_int = unsafe {any_as_u8_slice(&x)};
    let y_int = unsafe {any_as_u8_slice(&y)};
    let buf:[u8;2] = [x_int[0], y_int[0]];
    //print!("{:?}", &buf);

    socket.socket.send_to(&buf,"localhost:5001").unwrap();

}


pub fn get_id(socket: Res<UDP>){
    let buf: [u8;1] = [cuscuta_resources::GET_PLAYER_ID_CODE];
    socket.socket.send_to(&buf, cuscuta_resources::SERVER_ADR).unwrap();
}

pub fn serialize_player(
    player: Query<&Player>
){
    let p = player.single();
    let mut s = flexbuffers::FlexbufferSerializer::new();
    println!("Player before: {:?}\n", p);
    p.serialize(&mut s).unwrap();

    let r = flexbuffers::Reader::get_root(s.view()).unwrap();

    println!("Stored in: {:?} bytes. \n", s.view().len());

    let p2 = Player::deserialize(r).unwrap();

    assert_eq!(*p, p2);

    

    



}

pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] { // will slice anything into u8 array !! https://stackoverflow.com/questions/28127165/how-to-convert-struct-to-u8
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::core::mem::size_of::<T>(),
    )
}

pub unsafe fn u8_to_f32(input_arr : &[u8]) -> (&[u8], &[f32], &[u8]) { // prefix, actual stuff, suffix
    input_arr.align_to::<f32>()
}