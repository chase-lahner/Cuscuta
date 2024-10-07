use std::{collections::HashSet, net::{SocketAddr, UdpSocket}};
/* Rate at which we will be sending/recieving packets */
const _TICKS_PER_SECOND: u32 = 60;

fn main() -> std::io::Result<()>{
    /* binding to our little mailbox */
    let socket = UdpSocket::bind("localhost:5001").unwrap();
    /* buffer will be 100 msgs 1024B in length */
    let mut buf: [u8; 1024] = [0;1024];
    let mut sources:HashSet<SocketAddr> = HashSet::new();
    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        println!("{:?}",&buf);
        socket.send_to(b"From server", "localhost:5000").unwrap();
    }



}
