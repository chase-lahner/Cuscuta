use std::{collections::HashSet, net::{SocketAddr, UdpSocket}};
use std::collections::HashMap;
/* Rate at which we will be sending/recieving packets */
const _TICKS_PER_SECOND: u32 = 60;




fn main() -> std::io::Result<()>{
    /* binding to our little mailbox */
    let socket = UdpSocket::bind("localhost:5001").unwrap();
    let mut num_players: u8 = 0; // TODO: decrement when disconnect, idk its like connectionless so we need to send a packet saying to dec when we disconnect 
    let mut player_hash: HashMap<String, u8> = HashMap::new();
    /* buffer will be 100 msgs 1024B in length */
    let mut buf: [u8; 1024] = [0;1024];
    let mut sources:HashSet<SocketAddr> = HashSet::new(); // hashset assigning ip's to id's
    loop {
        let (_amt, src) = socket.recv_from(&mut buf)?;
        if buf[0] == 255 as u8 // if we recieve a packet requesting an ID
        {
            //print!("sending socket");
            let to_send: &[u8;2] = &[255, assign_id(src, player_hash.clone(), &mut num_players)]; // u8 array with code letting client know its an id, and then the id itself
            socket.send_to(to_send, "localhost:5000").unwrap(); // send the packet
        }
      //  println!("{:?}",&buf);
       // socket.send_to(b"From server", "localhost:5000").unwrap();
    }



}

fn assign_id(socket_addr : SocketAddr, mut player_hash : HashMap<String, u8>, n_p: &mut u8) -> u8{
    let arg_ip = socket_addr.ip();
    let ip_string = arg_ip.to_string();
    let player_id: u8 = 255 - *n_p;

    *n_p +=1;


    player_hash.insert(ip_string, player_id);

    player_id
}