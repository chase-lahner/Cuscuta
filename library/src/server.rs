use std::net::{SocketAddr, UdpSocket};

use bevy::prelude::*;
use network::*;
use serde::{Deserialize, Serialize};

use crate::{cuscuta_resources::{self, AddressList, ClientId, Health, PlayerCount, Velocity, GET_PLAYER_ID_CODE, PLAYER_DATA}, enemies::Enemy, network, player::{self, Attack, Crouch, NetworkId, Player, Roll, ServerPlayerBundle, Sprint}};

/* Upon request, sends an id to client */
pub fn send_id(
    source_addr : SocketAddr,
    server_socket: &UdpSocket, 
    n_p: &mut PlayerCount,
    mut commands: Commands,
    mut addresses: ResMut<AddressList>,
    mut server_seq: ResMut<ServerSequence>
) {
    /* assign id, update player count */
    let player_id: u8 = 255 - n_p.count;
    n_p.count += 1;
    addresses.list.push(source_addr);
    commands.spawn(NetworkId{id:player_id, addr:source_addr});

    let mut serializer = flexbuffers::FlexbufferSerializer::new();

    let id_packet = IdPacket{head: Header{network_id: player_id, sequence_num: server_seq.get(), timestamp: 0 }};

    let to_send = ServerPacket::IdPacket(id_packet);

    to_send.serialize(&mut serializer).unwrap();

    /* once serialized, we throw our opcode on the end */
    // let opcode: &[u8] = std::slice::from_ref(&GET_PLAYER_ID_CODE);
    // let packet_vec  = append_opcode(serializer.view(), opcode);
    // let packet: &[u8] = &(&packet_vec);

    let packet: &[u8] = serializer.view();


    /* Send it on over! */
    server_socket.send_to(packet, source_addr).unwrap(); // maybe &packet
}

/* Server side listener for packets,  */
pub fn listen(
    udp: Res<UDP>,
    mut commands: Commands,
    // mut players: Query<(&mut Velocity, &mut Transform, &mut NetworkId), With<Player>>,
    mut players_new: Query<(&mut Velocity, &mut Transform, &mut Health, &mut Crouch, &mut Roll, &mut Sprint, &mut Attack, &mut NetworkId), (With<Player>, Without<Enemy>)>,
    mut n_p: ResMut<PlayerCount>,
    addresses: ResMut<AddressList>,
    mut server_seq: ResMut<ServerSequence>

) {
    /* to hold msg */
    let mut buf: [u8; 1024] = [0;1024];
    // pseudo poll. nonblocking, gives ERR on no read tho
    let packet = udp.socket.recv_from(&mut buf);
    match packet{
        Err(e)=> return,
        _ => ()
    }
    let (amt, src) = packet.unwrap();
    

    /* trim trailing 0s */
    let t_buf = &buf[..amt]; // / -1


    let deserializer = flexbuffers::Reader::get_root(t_buf).unwrap();
    // this shoulddd be a client packet right?
    let player_struct: ClientPacket = ClientPacket::deserialize(deserializer).unwrap();

    match player_struct {
        ClientPacket::IdPacket(_id_packet) => {
            info!("sending id to client");
            send_id(src, &udp.socket, n_p.as_mut(), commands, addresses, server_seq)},
        ClientPacket::PlayerPacket(player_packet) => {
            // TODO: Fix this
           // update_player_state(src, players, player_packet, commands);
            recieve_input(player_packet);
        }
    }


}

fn recieve_input(player_struct: PlayerC2S){
    // TODO this needs to check inputs and move player, check for collisions, basically everything we are doing onv the client side idk
}

/* once we have our packeet, we must use it to update
 * the player specified, there's another in client.rs*/
fn update_player_state(
    // src: SocketAddr,
    // /* fake query, passed from above system */
    // mut players: Query<(&mut Velocity, &mut Transform, &mut NetworkId), With<Player>>,
    // player_struct: PlayerPacket,
    // mut commands: Commands,
) { 
//     // let deserializer = flexbuffers::Reader::get_root(buf).unwrap();
//     // let player_struct = PlayerPacket::deserialize(deserializer).unwrap();
//     let mut found = false;
//     for (mut velo, mut transform, network_id) in players.iter_mut(){
//         if network_id.id == player_struct.id{
//             transform.translation.x = player_struct.transform_x;
//             transform.translation.y = player_struct.transform_y;
//             velo.velocity.x = player_struct.velocity_x;
//             velo.velocity.y = player_struct.velocity_y;
//             found = true;
//         }
//     }
//     if !found{
//         let velo_vec = Vec2::new(player_struct.velocity_x, player_struct.velocity_y);
//         commands.spawn(ServerPlayerBundle{
//             velo: Velocity::from(velo_vec),
//             transform:
//                 Transform::from_xyz(player_struct.transform_x, player_struct.transform_y, 0.),
//             id: NetworkId{
//                 id: player_struct.id,
//                 addr: src},
//             player: Player,   
//             health: Health::new(),
//             rolling: Roll::new(),
//             crouching: Crouch::new(),
//             sprinting: Sprint::new(),
//             attacking: Attack::new(),
//     });
//     }
// }

// fn update_player_state_new(
//     src: SocketAddr,
//     mut players: Query<(&mut Velocity,  &mut Transform, &mut Health, &mut Crouch, &mut Roll, &mut Sprint, &mut Attack, &mut NetworkId), (With<Player>, Without<Enemy>)>,
//     player_struct: NewPlayerPacket,
//     mut commands: Commands
// ){
//     let mut found = false;
//     for (mut vel,mut trans,mut health, mut crouching, mut rolling, mut sprinting, mut attacking, id) in players.iter_mut(){

//         if id.id == player_struct.client_bundle.id.id {
//             trans.translation.x = player_struct.client_bundle.transform.translation.x;
//             trans.translation.y = player_struct.client_bundle.transform.translation.y;
//             vel.velocity.x = player_struct.client_bundle.velo.velocity.x;
//             vel.velocity.y = player_struct.client_bundle.velo.velocity.y;
//             health.current = player_struct.client_bundle.health.current;
//             crouching.crouching = player_struct.client_bundle.crouching.crouching;
//             rolling.rolling = player_struct.client_bundle.rolling.rolling;
//             sprinting.sprinting = player_struct.client_bundle.sprinting.sprinting;
//             attacking.attacking = player_struct.client_bundle.attacking.attacking;
//             // *trans = player_struct.client_bundle.transform;
//             // *vel = player_struct.client_bundle.velo;
//             // *health = player_struct.client_bundle.health;
//             // *crouching = player_struct.client_bundle.crouching;
//             // *rolling = player_struct.client_bundle.rolling;
//             // *sprinting = player_struct.client_bundle.sprinting;
//             // *attacking = player_struct.client_bundle.attacking;
//             found = true;
//         }

//     }
//     if !found {
//         let v = player_struct.client_bundle.velo;
//         commands.spawn(ServerPlayerBundle{
//             velo: v,
//             transform: player_struct.client_bundle.transform,
//             id: player_struct.client_bundle.id,
//             player: player_struct.client_bundle.player,
//             health: player_struct.client_bundle.health,
//             rolling: player_struct.client_bundle.rolling,
//             crouching: player_struct.client_bundle.crouching,
//             sprinting: player_struct.client_bundle.sprinting,
//             attacking: player_struct.client_bundle.attacking
            
//         });
//     }
}

// /* Transforms current player state into u8 array that
//  * we can then send across the wire to be deserialized once it arrives */
//  pub fn send_player(
//     player : Query<(&Transform, &Velocity, &NetworkId ), With<Player>>,
//     socket : Res<UDP>,
//     addresses: ResMut<AddressList>
// ) {
//     /* Deconstruct out Query. SHould be client side so we can do single */
//     for (t, v, i)  in player.iter(){
//         for addressi in addresses.list.iter(){
//             if *addressi != i.addr && (v.velocity.x != 0. || v.velocity.y != 0.){
//                 let outgoing_state = PlayerPacket { 
//                     id: i.id,
//                     transform_x: t.translation.x,
//                     transform_y: t.translation.y,
//                     velocity_x: v.velocity.x,
//                     velocity_y: v.velocity.y,
//                 };
            
//                 let mut serializer = flexbuffers::FlexbufferSerializer::new();
//                 let to_send: SendablePacket = SendablePacket::PlayerPacket(outgoing_state);
//                 to_send.serialize(&mut serializer).unwrap();
                
//                 // let opcode: &[u8] = std::slice::from_ref(&PLAYER_DATA);
//                 // let packet_vec  = append_opcode(serializer.view(), opcode);
//                 // let packet: &[u8] = &(&packet_vec);

//                 let packet: &[u8] = serializer.view();

//                 for address in &addresses.list{
//                     if *address != i.addr{
//                         // info!("{}: id:{}",address, outgoing_state.id);
//                     socket.socket.send_to(&packet, address).unwrap();
//                     }
//                 }
//             }
//         }
//     }
// }

/* Transforms current player state into u8 array that
 * we can then send across the wire to be deserialized once it arrives */
 pub fn send_player(
    player : Query<(&Velocity, &Transform, &NetworkId, &Player, &Health, &Crouch, &Roll, &Sprint, &Attack), With<Player>>,
    socket : Res<UDP>,
    addresses: ResMut<AddressList>,
    mut server_seq: ResMut<ServerSequence>
)
{
    /* Deconstruct out Query. */
    for (v, t, i, p, h, c, r, s, a)  in player.iter(){
        for addressi in addresses.list.iter(){
            if *addressi != i.addr && (v.velocity.x != 0. || v.velocity.y != 0.)
            {
                let outgoing_state: PlayerS2C = PlayerS2C {
                    xcoord: t.translation.x,
                    ycoord: t.translation.y,
                    head: Header{network_id: i.id, sequence_num: server_seq.get(), timestamp: 0},
                    attack: a.attacking,
                    velocity: v.velocity,
                    health: h.current,
                    crouch: c.crouching,
                    roll: r.rolling,
                    sprint: s.sprinting

                    
                };
                server_seq.up();
                let mut serializer = flexbuffers::FlexbufferSerializer::new();
                let to_send: ServerPacket = ServerPacket::PlayerPacket(outgoing_state);
                to_send.serialize(&mut serializer).unwrap();
                
                // let opcode: &[u8] = std::slice::from_ref(&PLAYER_DATA);
                // let packet_vec  = append_opcode(serializer.view(), opcode);
                // let packet: &[u8] = &(&packet_vec);
    
                let packet: &[u8] = serializer.view();
    
                for address in &addresses.list {
                    if *address != i.addr{
                        info!("sending to stuff");
                        socket.socket.send_to(&packet, address).unwrap();
                    }
                }
                

            }
        }
        
            
        
    }
}
