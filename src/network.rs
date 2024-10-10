
use bevy::prelude::*;
use std::net::UdpSocket;
use crate::cuscuta_resources::*;

#[derive(Resource)]
pub struct UDP{
    pub socket: UdpSocket
}



pub fn recv_packet(
    socket: Res<UDP>
){
    let mut buf = [0;1024];
    let (_amt, _src) = socket.socket.recv_from(&mut buf).unwrap();
    //println!("{}", String::from_utf8_lossy(&buf));
}

pub fn send_packet(
    socket: Res<UDP>,
) {
    socket.socket.send_to(b"boo!", "localhost:5001").unwrap();
}

pub fn send_movement_info(
    socket: Res<UDP>,
    player: Query<&Transform, With<Player>>,
    
) {
    let pt = player.single();
    let x = pt.translation.x;
    let y = pt.translation.y;
    let x_int = unsafe {x.to_int_unchecked::<u8>()};
    let y_int = unsafe {y.to_int_unchecked::<u8>()};
    let buf:[u8;2] = [x_int, y_int];
    //print!("{:?}", &buf);

    socket.socket.send_to(&buf,"localhost:5001").unwrap();

}