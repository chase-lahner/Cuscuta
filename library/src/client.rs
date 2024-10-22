use std::net::SocketAddr;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::network::{append_opcode, IdPacket, PlayerPacket, UDP};
use crate::cuscuta_resources::*;
use crate::player::*;

pub fn recv_id(
    source_addr: SocketAddr,
    network_id: &mut NetworkId,
    packet: &[u8],
    mut commands: Commands,
    mut id: ResMut<ClientId>
) {
    info!("Recieving ID");
    /* de-serialize the struct IdHeader which contains our id */
    let deserializer = flexbuffers::Reader::get_root(packet).unwrap();
    let ds_struct = IdPacket::deserialize( deserializer).unwrap();

    /* assign it to the player */
    network_id.id = ds_struct.id;
    network_id.addr = source_addr;
    id.id = ds_struct.id;

    /* assign it to the game world */
    commands.insert_resource(ClientId{id:ds_struct.id});

    info!("ASSIGNED ID: {:?}", network_id.id);
}

/* Sends id request to the server */
pub fn id_request(
    player: Query<&NetworkId, With<Player>>,
    socket: Res<UDP>,
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

    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    /* Serializes, which makes a nice lil u8 array for us */
    to_send.serialize(&mut serializer).unwrap();


    /* plop opcode on the end. this is consistent. I would
     * like this to be a fn, but with how you need CONSTANT 
     * to declare array on top of different struct sizes
     * i didn't feel like mallocating anything */
    let opcode: &[u8] = std::slice::from_ref(&GET_PLAYER_ID_CODE);
    let packet_vec  = append_opcode(serializer.view(), opcode);
    let packet: &[u8] = &(&packet_vec);



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
            let mut serializer = flexbuffers::FlexbufferSerializer::new();
            outgoing_state.serialize(&mut serializer).unwrap();
            
            let opcode: &[u8] = std::slice::from_ref(&PLAYER_DATA);
            let packet_vec  = append_opcode(serializer.view(), opcode);
            let packet: &[u8] = &(&packet_vec);

            
            socket.socket.send_to(&packet, SERVER_ADR).unwrap();
        }
    }
} 

/* client listening function */
pub fn listen(
    /* BROOOOOO TOO MANY ARGGGGGGGGGGGGS
     * Would really love to get that spawn player fn out of here, 
     * maybe event or stage??? */
    udp: Res<UDP>,
    mut commands: Commands,
    mut player: Query<(&mut Velocity, &mut Transform, &mut NetworkId), With<Player>>,
    mut asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut id: ResMut<ClientId>
) {
    info!("Listening!!!");
    /* to hold msg */
    let mut buf: [u8; 1024] = [0;1024];
    let packet = udp.socket.recv_from(&mut buf);
    match packet{
        Err(e)=> return,
        _ => info!("read packet!")
    }
    let (amt, src) = packet.unwrap();
    /* opcode is last byte of anything we send */
    let opcode = buf[amt-1];

    /* trim trailing 0s and opcode*/
    let mut packet = &buf[..amt-1];


    match opcode{
        GET_PLAYER_ID_CODE => 
            recv_id(src, player.single_mut().2.as_mut(), packet, commands, id),
        PLAYER_DATA =>
            update_player_state(player, packet, commands, &asset_server, &mut texture_atlases, src),
        _ => something()//TOTO

    };
}

/* once we have our packeet, we must use it to update
 * the player specified, there's another in server.rs */
fn update_player_state(
    /* fake query, passed from above system */
    mut players: Query<(&mut Velocity, &mut Transform, &mut NetworkId), With<Player>>,
    buf: &[u8],
    mut commands: Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    source_ip: SocketAddr
) { 
    let deserializer = flexbuffers::Reader::get_root(buf).unwrap();
    let player_struct = PlayerPacket::deserialize(deserializer).unwrap();
    let mut found = false;
    for (mut velo, mut transform, network_id) in players.iter_mut(){
        if network_id.id == player_struct.id{
            transform.translation.x = player_struct.transform_x;
            transform.translation.y = player_struct.transform_y;
            velo.velocity.x = player_struct.velocity_x;
            velo.velocity.y = player_struct.velocity_y;
            found = true;
        }
    }
    if !found{
        client_spawn_other_player(&mut commands, asset_server, texture_atlases,player_struct, source_ip);
    }
}

fn something(){}

