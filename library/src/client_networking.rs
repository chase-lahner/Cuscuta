use std::net::SocketAddr;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use bevy::input::keyboard::KeyboardInput;

use crate::network::{ IdPacket, PlayerC2S, SendablePacket, UDP};
use crate::cuscuta_resources::*;
use crate::player::*;

pub fn recv_id(
    source_addr: SocketAddr,
    network_id: &mut NetworkId,
    ds_struct: IdPacket,
    mut _commands: Commands,
    mut id: ResMut<ClientId>
) {
    info!("Recieving ID");
    /* de-serialize the struct IdHeader which contains our id */
    //let deserializer = flexbuffers::Reader::get_root(packet).unwrap();
    //let ds_struct = IdPacket::deserialize( deserializer).unwrap();

    /* assign it to the player */
    network_id.id = ds_struct.id;
    network_id.addr = source_addr;
    id.id = ds_struct.id;

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
    let id_packet = IdPacket {
        id: i.id,
    };

    let to_send: SendablePacket = SendablePacket::IdPacket(id_packet);

    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    /* Serializes, which makes a nice lil u8 array for us */
    to_send.serialize(&mut serializer).unwrap();


    /* plop opcode on the end. this is consistent. I would
     * like this to be a fn, but with how you need CONSTANT 
     * to declare array on top of different struct sizes
     * i didn't feel like mallocating anything */
    // let opcode: &[u8] = std::slice::from_ref(&GET_PLAYER_ID_CODE);
    // let packet_vec  = append_opcode(serializer.view(), opcode);
    // let packet: &[u8] = &(&packet_vec);

    let packet: &[u8] = serializer.view();



    /* beam me up scotty */
    socket
        .socket
        .send_to(packet, SERVER_ADR)
        .unwrap();

}

/* Transforms current player state into u8 array that
 * we can then send across the wire to be deserialized once it arrives */
 pub fn send_player(
    socket : Res<UDP>,
    client_id: Res<ClientId>,
    key: KeyCode,
    door: u8,
)
{
    /* Deconstruct out Query. SHould be client side so we can do single */
    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    let to_send: SendablePacket = SendablePacket::PlayerPacket(PlayerC2S {id: client_id.id, key: key, door: door});
    to_send.serialize(&mut serializer).unwrap();
    
    // let opcode: &[u8] = std::slice::from_ref(&PLAYER_DATA);
    // let packet_vec  = append_opcode(serializer.view(), opcode);
    // let packet: &[u8] = &(&packet_vec);

    let packet: &[u8] = serializer.view();

    socket.socket.send_to(&packet, SERVER_ADR).unwrap();
} 

/* client listening function */
pub fn listen(
    /* BROOOOOO TOO MANY ARGGGGGGGGGGGGS
     * Would really love to get that spawn player fn out of here, 
     * maybe event or stage??? */
    udp: Res<UDP>,
    mut commands: Commands,
    // mut player: Query<(&mut Velocity, &mut Transform, &mut NetworkId), With<Player>>,
    mut players_new: Query<(&mut Velocity, &mut Transform, &mut Player, &mut Health, &mut Crouch, &mut Roll, &mut Sprint, &mut Attack, &mut NetworkId), With<Player>>,
    mut asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    id: ResMut<ClientId>
) {
    //info!("Listening!!!");
    /* to hold msg */
    let mut buf: [u8; 1024] = [0;1024];
    let packet = udp.socket.recv_from(&mut buf);
    match packet{
        Err(_e)=> return,
        _ =>  info!("read packet!")
    }
    let (amt, src) = packet.unwrap();
    /* opcode is last byte of anything we send */
    // let opcode = buf[amt-1];

    /* trim trailing 0s and opcode*/
    let packet = &buf[..amt];


    let deserializer = flexbuffers::Reader::get_root(packet).unwrap();
    let player_struct: SendablePacket = SendablePacket::deserialize(deserializer).unwrap();

    match player_struct{
        

        SendablePacket::IdPacket(id_packet) => {
            recv_id(src, players_new.single_mut().8.as_mut(), id_packet, commands, id);
        }
        SendablePacket::PlayerPacket(player_packet)=> {
            // update_player_state(player, player_packet, commands, &asset_server, &mut texture_atlases, src);

        }
        SendablePacket::NewPlayerPacket(player_packet) => {
            info!("Matching Player Struct");
            update_player_state_new(players_new, player_packet, commands, &asset_server, &mut texture_atlases, src);
        }
    }

    // match opcode{
    //     GET_PLAYER_ID_CODE => 
    //         recv_id(src, player.single_mut().2.as_mut(), packet, commands, id),
    //     PLAYER_DATA =>
    //         update_player_state(player, packet, commands, &asset_server, &mut texture_atlases, src),
    //     _ => something()//TOTO

    // };
}

/* once we have our packeet, we must use it to update
 * the player specified, there's another in server.rs */
fn update_player_state(
    /* fake query, passed from above system */
    mut players: Query<(&mut Velocity, &mut Transform, &mut NetworkId), With<Player>>,
    player_struct: PlayerPacket,
    mut commands: Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    source_ip: SocketAddr
) { 
    // let deserializer = flexbuffers::Reader::get_root(buf).unwrap();
    // let player_struct = PlayerPacket::deserialize(deserializer).unwrap();
    let mut found = false;
    for (mut velo, mut transform, network_id) in players.iter_mut(){
        info!("REc: {}  Actual:: {}", player_struct.id, network_id.id);
        if network_id.id == player_struct.id{
            transform.translation.x = player_struct.transform_x;
            transform.translation.y = player_struct.transform_y;
            velo.velocity.x = player_struct.velocity_x;
            velo.velocity.y = player_struct.velocity_y;
            found = true;
        }
    }
    if !found{
        info!("new player!");
        client_spawn_other_player(&mut commands, asset_server, texture_atlases,player_struct, source_ip);
    }
}

fn update_player_state_new(
    mut players: Query<(&mut Velocity, &mut Transform, &mut Player, &mut Health, &mut Crouch, &mut Roll, &mut Sprint, &mut Attack, &mut NetworkId), With<Player>>,
    player_struct: NewPlayerPacket,
    mut commands: Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    source_ip: SocketAddr
){
    let mut found = false;
    for(mut velocity, mut transform,mut player,mut health, mut crouch, mut roll, mut sprint, mut attack, mut network_id) in players.iter_mut(){
        if network_id.id == player_struct.client_bundle.id.id{
           // *transform = player_struct.client_bundle.transform;
            transform.translation.x = player_struct.client_bundle.transform.translation.x;
            transform.translation.y = player_struct.client_bundle.transform.translation.y;
            velocity.velocity.x = player_struct.client_bundle.velo.velocity.x;
            velocity.velocity.y = player_struct.client_bundle.velo.velocity.y;
            health.current = player_struct.client_bundle.health.current;
            crouch.crouching = player_struct.client_bundle.crouching.crouching;
            roll.rolling = player_struct.client_bundle.rolling.rolling;
            sprint.sprinting = player_struct.client_bundle.sprinting.sprinting;
            attack.attacking = player_struct.client_bundle.attacking.attacking;
           // *velocity = player_struct.client_bundle.velo;
            // *health = player_struct.client_bundle.health;
            // *crouch = player_struct.client_bundle.crouching;
            // *roll = player_struct.client_bundle.rolling;
            // *sprint = player_struct.client_bundle.sprinting;
            // *attack = player_struct.client_bundle.attacking;
            found = true;
        }
    }
    if !found {
        info!("new player!");
        let v = player_struct.client_bundle.velo;
        client_spawn_other_player_new(&mut commands, asset_server, texture_atlases, player_struct, source_ip);
    }
}

fn something(){}
