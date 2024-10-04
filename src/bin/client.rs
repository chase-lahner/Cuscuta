use std::net::UdpSocket;
use std::{thread,time};
fn main() -> std::io::Result<()>{
     /* binding to our little mailbox */
     let socket = UdpSocket::bind("0.0.0.0:2022")?;
     /* buffer will be 100 msgs 1024B in length */
     let mut buf = [0;1024];
 
     loop {
         socket.send_to(&[1;20],"0.0.0.0:2021")?;
    }

}