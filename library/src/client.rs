use bevy::prelude::*;
use flexbuffers::FlexbufferSerializer;
use serde::{Deserialize, Serialize};

use crate::network::{IdPacket, PlayerPacket, UDP};
use crate::cuscuta_resources::{self, FlexSerializer, Velocity};
use crate::player::*;

pub fn recv_id(
    socket: Res<UDP>, 
    mut player: Query<&mut NetworkId, With<Player>>,
    serializre: &mut FlexbufferSerializer,
    
) {
    
    let mut network_id = player.single_mut(); // network id is part of player struct
    let mut buf: [u8; 1024] = [0; 1024]; // buffer
    let (amt, _src) = socket.socket.recv_from(&mut buf).unwrap(); // recieve buffer from server
    let t_buf = &buf[..amt];

    let ds_struct = IdPacket::deserialize(r).unwrap();

    if ds_struct.opcode == cuscuta_resources::GET_PLAYER_ID_CODE
    {
        network_id.id = ds_struct.id;
    }

    println!("ASSIGNED ID: {:?}", network_id.id);
}

pub fn id_request(
    player: Query<&NetworkId, With<Player>>,
    socket: Res<UDP>,
    mut serializer: ResMut<FlexSerializer>,
) {
    let i = player.single();
    let i_d = i.id;
    let to_send = IdPacket {
        id: i_d,
    };

    to_send.serialize(&mut serializer.serializer).unwrap();
    const SIZE:usize = size_of::<IdPacket>();
    let mut packet = [0;SIZE+1];
    packet[..SIZE].clone_from_slice(serializer.serializer.view());
    packet[SIZE] = cuscuta_resources::GET_PLAYER_ID_CODE;

    socket
        .socket
        .send_to(&packet, cuscuta_resources::SERVER_ADR)
        .unwrap();
}

/* Transforms current player state into u8 array that
 * we can then send across the wire to be deserialized once it arrives */
 pub fn send_player(
    player : Query<(&Transform, &Velocity, &NetworkId ), With<Player>>,
    socket : Res<UDP>,
    mut serializer: ResMut<FlexSerializer>
)
{
    /* Deconstruct out Query. SHould be client side so we can do single */
    let (t, v, i) = player.single();
    let outgoing_state = PlayerPacket { 
        transform_x: t.translation.x,
        transform_y: t.translation.y,
        velocity_x: v.velocity.x,
        velocity_y: v.velocity.y,
    };

    outgoing_state.serialize(&mut serializer.serializer).unwrap();
    const SIZE:usize = size_of::<PlayerPacket>();
    let mut packet = [0;SIZE+1];
    packet[..SIZE].clone_from_slice(serializer.serializer.view());
    packet[SIZE] = cuscuta_resources::PLAYER_DATA;
    socket.socket.send_to(&packet, cuscuta_resources::SERVER_ADR).unwrap();
} 

pub fn listen(
    udp: Res<UDP>,
    commands: Commands,
    mut player: Query<(&Velocity, &Transform, &NetworkId), With<Player>>,
    mut serializer: ResMut<FlexSerializer>,
) -> std::io::Result<()>{// really doesn;t need to return this am lazy see recv_from line
    /* to hold msg */
    let mut buf: [u8; 1024] = [0;1024];
    let (amt, src) = udp.socket.recv_from(&mut buf)?;
    /* trim trailing 0s */
    let mut t_buf = &buf[..amt];

    /* when we serialize, we throw our opcode on the end, so we know how to
    * de-serialize... jank? maybe.  */
    let opcode = buf[amt];

    match opcode{
        cuscuta_resources::GET_PLAYER_ID_CODE => 
            recv_id(src, &udp.socket, n_p,serializer),
        cuscuta_resources::PLAYER_DATA =>
            update_player(player, t_buf,),
        
        _ => something()//TOTO

    };


    Ok(())
}