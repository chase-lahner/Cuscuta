use std::{collections::HashSet, net::{SocketAddr, UdpSocket}};
/* Rate at which we will be sending/recieving packets */
const _TICKS_PER_SECOND: u32 = 60;

fn main() -> std::io::Result<()>{
    /* binding to our little mailbox */
    let socket = UdpSocket::bind("localhost:2021").unwrap();
    /* buffer will be 100 msgs 1024B in length */
    let mut buf: [u8; 1024] = [0;1024];
    let mut sources:HashSet<SocketAddr> = HashSet::new();
    loop {
        let (_amt, src) = socket.recv_from(&mut buf)?;
        sources.insert(src);
        println!("{}",String::from_utf8_lossy(&buf));
        socket.send_to(b"From server", "localhost:2022").unwrap();
    }



}
