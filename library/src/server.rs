use std::net::{SocketAddr, UdpSocket};

use bevy::prelude::*;
use network::*;
use serde::{Deserialize, Serialize};

use crate::{cuscuta_resources::{AddressList, Health, PlayerCount, Velocity}, enemies::{Enemy, EnemyId, EnemyMovement}, network, player::{Attack, Crouch, InputQueue, NetworkId, Player, Roll, ServerPlayerBundle, Sprint}};

/* Upon request, sends an id to client, spawns a player, and
 * punts player state off to client */
pub fn send_id(
    source_addr : SocketAddr,
    server_socket: &UdpSocket, 
    n_p: &mut PlayerCount,
    mut commands: Commands,
    mut addresses: ResMut<AddressList>,
    mut server_seq: ResMut<Sequence>
) {
    /* assign id, update player count */
    let player_id: u8 = 255 - n_p.count;
    n_p.count += 1;
    addresses.list.push(source_addr);
    commands.spawn(NetworkId::new_s(player_id, source_addr));

    /* to send packets over wire we must serialize */
    let mut id_serializer = flexbuffers::FlexbufferSerializer::new();

    let id_send = ServerPacket::IdPacket(IdPacket{
        head: Header::new(player_id,server_seq.geti())});

    id_send.serialize(&mut id_serializer).unwrap();
    
    let id_packet: &[u8] = id_serializer.view();
    
    /* Send it on over! */
    server_socket.send_to(id_packet, source_addr).unwrap(); // maybe &packet

    /* now we must spawn in a new player */
    commands.spawn(ServerPlayerBundle{
        id: NetworkId::new_s(player_id, source_addr),
        velo: Velocity::new(),
        transform: Transform{
            translation: Vec3 { x: 0., y: 0., z: 900. },
            ..default()},
        health: Health::new(),
        crouching: Crouch::new(),
        rolling: Roll::new(),
        sprinting: Sprint::new(),
        attacking: Attack::new(),
        inputs: InputQueue::new(),
        time: Timestamp::new(0),//TODO set time properly
    });
    /* same shit but now we sending off to the cleint */
    let playa = ServerPacket::PlayerPacket(PlayerS2C{
        head: Header::new(player_id,server_seq.geti(),0),//TODO TIMESTAMPS
        transform: Transform{
            translation: Vec3{
                x: 0., y: 0., z: 900.,
            },
            ..default()},
        velocity: Vec2::new(0.,0.),
        health: Health::new(),
        crouch: false,
        attack: false,
        roll: false,
        sprint: false,
    });

    /* another serialize rq rq rq */
    let mut player_serializer = flexbuffers::FlexbufferSerializer::new();
    playa.serialize(&mut player_serializer).unwrap();
    let player_packet = player_serializer.view();
    server_socket.send_to(player_packet, source_addr).unwrap();
}

/* Server side listener for packets,  */
pub fn listen(
    udp: Res<UDP>,
    commands: Commands,
    // mut players: Query<(&mut Velocity, &mut Transform, &mut NetworkId), With<Player>>,
    players_q: Query<(&mut Velocity, &mut Transform, &mut Health,
         &mut Crouch, &mut Roll, &mut Sprint, &mut Attack, &mut NetworkId,
          &mut InputQueue, &Timestamp), (With<Player>, Without<Enemy>)>,//eek a lot
    mut n_p: ResMut<PlayerCount>,
    addresses: ResMut<AddressList>,
    server_seq: ResMut<Sequence>

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
            recieve_input(player_packet, players_q);
        }
    }


}

//TOTOTOODODODODODODODO--------------------------------
fn recieve_input(
    client_pack: PlayerC2S,
    mut players_q: Query<(&mut Velocity, &mut Transform, &mut Health,
         &mut Crouch, &mut Roll, &mut Sprint, &mut Attack, &mut NetworkId,
          &mut InputQueue, &Timestamp), (With<Player>, Without<Enemy>)>,
){
    // TODO this needs to check inputs and move player, check for collisions, basically everything we are doing onv the client side idk
    /* for all players in server game world */
    for (v, t, h, c, r, s, a, id, mut iq, time) in players_q.iter_mut(){
        /* if we find the one corresponding to our packet */
        if client_pack.head.network_id == id.id {
            /* for all the keys passed on the clients update */
            for key in &client_pack.key {
                /* append that hoe !!!!! */
                iq.q.push((Timestamp::new(client_pack.head.timestamp), *key)); 
            }
            /* ok if we want to update immediately then we od it right here
             * buuuuut the fn takes in diff args than we have (odd query). TBH
             * i am down to plop in the main logic loop for now, no reaason to use
             * any data longer than we have to, right?? (is not in main logic loop as of
             * 11/19 3:31pm*/
        }
    }
}

pub fn send_enemies(
    enemies: Query<(& EnemyId, & EnemyMovement), 
        (With<Enemy>, Without<Player>)>,
    addresses: Res<AddressList>,
    mut server_seq: ResMut<Sequence>,
    server_socket: Res<UDP>, 
){
    for (id, movement) in enemies.iter(){
        let enemy: EnemyS2C = EnemyS2C{
            head: Header::new(0,server_seq.geti(), 0),//TODO FIX TIME
            movement: movement.clone(),
            enemytype: id.clone(),
        };
        let mut serializer = flexbuffers::FlexbufferSerializer::new();
        let to_send: ServerPacket = ServerPacket::EnemyPacket(enemy);
        to_send.serialize(&mut serializer).unwrap();

        let packet: &[u8] = serializer.view();
        for addr in addresses.list.iter() {  
            server_socket.socket.send_to(&packet, *addr).unwrap();
        }

    }

    
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
    mut server_seq: ResMut<Sequence>
)
{
    /* Deconstruct out Query. */
    for (v, t, i, p, h, c, r, s, a,)  in player.iter(){
        for addressi in addresses.list.iter(){
            if *addressi != i.addr && (v.velocity.x != 0. || v.velocity.y != 0.)
            {
                let outgoing_state: PlayerS2C = PlayerS2C {
                    transform: *t,
                    head: Header::new(i.id,server_seq.geti(), 0),
                    attack: a.attacking,
                    velocity: v.velocity,
                    health: *h,
                    crouch: c.crouching,
                    roll: r.rolling,
                    sprint: s.sprinting,

                    
                };
                let mut serializer = flexbuffers::FlexbufferSerializer::new();
                let to_send: ServerPacket = ServerPacket::PlayerPacket(outgoing_state);
                to_send.serialize(&mut serializer).unwrap();
    
                let packet: &[u8] = serializer.view();
    
                /* one for you, one for you, one for you, another for yough */
                for address in &addresses.list {
                        socket.socket.send_to(&packet, address).unwrap();
                    
                }
            }
        }   
    }
}
