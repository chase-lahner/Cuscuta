use std::net::UdpSocket;

/* Rate at which we will be sending/recieving packets */
const TICKS_PER_SECOND: u32 = 60;

fn main() -> std::io::Result<()>{
    /* binding to our little mailbox */
    let socket = UdpSocket::bind("0.0.0.0:2021").unwrap();
    /* buffer will be 100 msgs 1024B in length */
    let mut buf = [0;1024];

    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        println!("{:?}",buf);
    }
}
