use std::net::UdpSocket;
use std::{thread,time};
fn main() -> std::io::Result<()>{
     /* binding to our little mailbox */
     let socket = UdpSocket::bind("localhost:5001")?;
     /* buffer will be 100 msgs 1024B in length */
     let mut buf = [0;1024];
 
     loop {
         socket.send_to(&[1;20],"localhost:5000")?;
    }

}