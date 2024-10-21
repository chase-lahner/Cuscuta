use bevy::prelude::*;
use flexbuffers::FlexbufferSerializer;
use serde::{Deserialize, Serialize};

use crate::network::{IdPacket, PlayerPacket, UDP};
use crate::cuscuta_resources::*;
use crate::player::*;

pub fn recv_id(
    mut player: Query<&mut NetworkId, With<Player>>,
    packet: &[u8],
    mut commands: Commands,
) {
    /* recv id is first, dont have another player so dont worry about player count */
    let mut network_id = player.single_mut();

    /* de-serialize the struct IdHeader which contains our id */
    let deserializer = flexbuffers::Reader::get_root(packet).unwrap();
    let ds_struct = IdPacket::deserialize( deserializer).unwrap();

    /* assign it to the player */
    network_id.id = ds_struct.id;
    /* assign it to the game world */
    commands.insert_resource(ClientId{id:ds_struct.id});

    info!("ASSIGNED ID: {:?}", network_id.id);
}

/* Sends id request to the server */
pub fn id_request(
    player: Query<&NetworkId, With<Player>>,
    socket: Res<UDP>,
    mut serializer: ResMut<FlexSerializer>,
) {

    /* plop network id into struct for serialization.
     * Can assume no other players, as this is first
     * networking communication, aka can't have told about
     * any others yet AND BRO THIS ID AINT EVEN REAL
     * WE HERE TO ASK FOR ONE ANYWAYS
     * 
     * still need something to shove over tho */
    let i = player.single();
    let to_send = IdPacket {
        id: i.id,
    };

    /* Serializes, which makes a nice lil u8 array for us */
    to_send.serialize(&mut serializer.serializer).unwrap();


    /* plop opcode on the end. this is consistent. I would
     * like this to be a fn, but with how you need CONSTANT 
     * to declare array on top of different struct sizes
     * i didn't feel like mallocating anything */
    const SIZE:usize = size_of::<IdPacket>();
    let mut packet = [0;SIZE+1];
    packet[..SIZE].clone_from_slice(serializer.serializer.view());
    packet[SIZE] = GET_PLAYER_ID_CODE;

    /* beam me up scotty */
    socket
        .socket
        .send_to(&packet, SERVER_ADR)
        .unwrap();
}

/* Transforms current player state into u8 array that
 * we can then send across the wire to be deserialized once it arrives */
 pub fn send_player(
    player : Query<(&Transform, &Velocity, &NetworkId ), With<Player>>,
    socket : Res<UDP>,
    mut serializer: ResMut<FlexSerializer>,
    client_id: Res<ClientId>,

)
{
    /* Deconstruct out Query. SHould be client side so we can do single */
    for (t, v, i)  in player.iter(){
        if i.id == client_id.id{
            let outgoing_state = PlayerPacket { 
                id: client_id.id,
                transform_x: t.translation.x,
                transform_y: t.translation.y,
                velocity_x: v.velocity.x,
                velocity_y: v.velocity.y,
            };
        
            outgoing_state.serialize(&mut serializer.serializer).unwrap();
            const SIZE:usize = size_of::<PlayerPacket>();
            let mut packet = [0;SIZE+1];
            packet[..SIZE].clone_from_slice(serializer.serializer.view());
            packet[SIZE] = PLAYER_DATA;
            socket.socket.send_to(&packet, SERVER_ADR).unwrap();
        }
    }
} 

/* client listening function */
pub fn listen(
    udp: Res<UDP>,
    mut commands: Commands,
    mut player: Query<(&Velocity, &Transform, &NetworkId), With<Player>>,
    mut serializer: ResMut<FlexSerializer>,
) -> std::io::Result<()>{// really doesn;t need to return this am lazy see recv_from line
    /* to hold msg */
    let mut buf: [u8; 1024] = [0;1024];
    let (amt, src) = udp.socket.recv_from(&mut buf)?;
    
    /* opcode is last byte of anything we send */
    let opcode = buf[amt];

    /* trim trailing 0s and opcode*/
    let mut packet = &buf[..amt-1];


    match opcode{
        GET_PLAYER_ID_CODE => 
            recv_id(player, packet, commands),
        PLAYER_DATA =>
            update_player(player, packet,),
        
        _ => something()//TOTO

    };


    Ok(())
}