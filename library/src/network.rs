
use bevy::a11y::accesskit::CustomAction;
use bevy::prelude::*;
use flexbuffers::FlexbufferSerializer;
use flexbuffers::{Reader, ReaderError};
use serde::ser::SerializeStruct;
use serde::{ Deserialize, Serialize};
use std::str::from_utf8;
use std::{any, net::UdpSocket};
use crate::player::*;
use crate:: cuscuta_resources::{self, Velocity};
use std::{collections::HashMap, net::SocketAddr};


#[derive(Resource)]
pub struct UDP{
    pub socket: UdpSocket
}

#[derive(Resource)]
pub struct BufSerializer{
    pub serializer: FlexbufferSerializer
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct UDPHeader{
    id: u8,
    p_x: f32,
    p_y: f32,
}

// impl Serialize for UDPHeader {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer, {

//             let mut state = serializer.serialize_struct("UDPHeader", 3)?;
//             state.serialize_field("id", &self.id)?;
//             state.serialize_field("p_x", &self.p_x)?;
//             state.serialize_field("p_y", &self.p_y)?;
//             state.end()
       
//     }
// }


// UNUSED as of now
pub fn recv_packet(
    socket: Res<UDP>
){
    let mut buf = [0;1024];
    let (_amt, _src) = socket.socket.recv_from(&mut buf).unwrap();
    //println!("{}", String::from_utf8_lossy(&buf));
}

// pub fn send_packet(
//     socket: Res<UDP>,
//     buf: [u8]
// ){
// }


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


// pub fn send_movement_info(
//     socket: Res<UDP>,
//     player: Query<&Transform, With<Player>>,
    
// ) {
//     let pt = player.single();
//     let x = pt.translation.x;
//     let y = pt.translation.y;
//     let x_int = unsafe {any_as_u8_slice(&x)};
//     let y_int = unsafe {any_as_u8_slice(&y)};
//     let buf:[u8;2] = [x_int[0], y_int[0]];
//     //print!("{:?}", &buf);

//     socket.socket.send_to(&buf,"localhost:5001").unwrap();

    

// }


pub fn get_id(socket: Res<UDP>)
{
    let buf: [u8;1] = [cuscuta_resources::GET_PLAYER_ID_CODE];
    socket.socket.send_to(&buf, cuscuta_resources::SERVER_ADR).unwrap();
}

pub unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] 
{ // will slice anything into u8 array !! https://stackoverflow.com/questions/28127165/how-to-convert-struct-to-u8
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::core::mem::size_of::<T>(),
    )
}

pub unsafe fn u8_to_f32(input_arr : &[u8]) -> (&[u8], &[f32], &[u8]) 
{ // prefix, actual stuff, suffix
    input_arr.align_to::<f32>()
}

pub fn serialize_player(
    player : Query<(&Transform, &Velocity, &NetworkId ), With<Player>>,
    socket : Res<UDP>
)
{
    let (t, v, i) = player.single();
    let p_x = t.translation.x;
    let p_y = t.translation.y;
    let i_d = i.id;
    let mut s = flexbuffers::FlexbufferSerializer::new();
    let to_send = UDPHeader {id: i_d, p_x: p_x, p_y: p_y};

    to_send.serialize(&mut s).unwrap();

    let r  = flexbuffers::Reader::get_root(s.view()).unwrap();
   
   // println!("data type: {:?}", data_type);
    if let flexbuffers::Blob(blob) = r.as_blob() {
    let byte_array: &[u8] = blob; // Access the byte slice directly
   // println!("{:?}", byte_array);
    let send_two = UDPHeader::deserialize(r).unwrap();

    println!("{}", send_two.id);

    socket.socket.send_to(, cuscuta_resources::SERVER_ADR);
} else {
    println!("not blob");
    
}

}



fn get_data_from_reader(reader: Reader<&[u8]>) -> Result<&[u8], ReaderError> {
    // Extract the byte slice from the Reader
    let data: &[u8] = reader.as_slice();

    // Wrap the byte slice in Ok to return the expected Result type
    Ok(data)
}


pub fn assign_id(socket_addr : SocketAddr, mut player_hash : HashMap<String, u8>, n_p: &mut u8) -> u8{
    let arg_ip = socket_addr.ip();
    let ip_string = arg_ip.to_string();
    let player_id: u8 = 255 - *n_p;

    *n_p +=1;


    player_hash.insert(ip_string, player_id);

    player_id
}