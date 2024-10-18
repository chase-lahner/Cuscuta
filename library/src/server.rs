pub fn recieve_packets(
    udp: Res<UDP>
)
{
    let mut buf: [u8;1024] = [0;1024];
    loop{
        let (amt, src) = udp.socket.recv_from(&mut buf).unwrap();
        /* TODO need to deseralize first  */
        let opcode = buf[0];
        

    }
}