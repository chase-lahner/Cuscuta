use std::net::SocketAddr;

use bevy:: prelude::*;
use network::*;
use serde::{Deserialize, Serialize};


use crate::{cuscuta_resources::{self, AddressList, Background, EnemiesToKill, Health, PlayerCount, Velocity, Wall}, enemies::{Enemy, EnemyId, EnemyMovement}, network, player::{Attack, Crouch, NetworkId, Player, Roll, ServerPlayerBundle, Sprint, Trackable}, room_gen::{Door, DoorType, Potion, Room}};


/* Upon request, sends an id to client, spawns a player, and
 * punts player state off to client via the packet queue */
pub fn send_id(
    source_addr : SocketAddr,
    n_p: &mut PlayerCount,
    mut commands: &mut Commands,
    mut addresses: &mut AddressList,
    mut server_seq: &mut Sequence,
    udp: & UDP
) {
    /* assign id, update player count */
    n_p.count += 1;
    let player_id: u8 = n_p.count;
    addresses.list.push(source_addr);
    info!("pushing addresss");
    commands.spawn(NetworkId::new_s(player_id, source_addr));

    server_seq.nums.push(0);
    let id_send = ServerPacket::IdPacket(IdPacket{
        head: Header::new(player_id,server_seq.clone())});

    /* put idpacket into 'to-send' queue */
    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    id_send.serialize(&mut serializer).unwrap();
    let packet: &[u8] = serializer.view();
    udp.socket.send_to(&packet, source_addr).unwrap();

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
        player: Player,
        track: Trackable
    });
    /* same shit but now we sending off to the cleint */
    let playa = ServerPacket::PlayerPacket(PlayerSendable{
        head: Header::new(player_id,server_seq.clone()),//TODO TIMESTAMPS
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

    /* usually we want to send later, but for this we don't want to send
     * player over and over and over, so we do it extra here */
    let mut serial = flexbuffers::FlexbufferSerializer::new();
    playa.serialize(&mut serial).unwrap();
    let packet: &[u8] = serial.view();
    udp.socket.send_to(&packet, source_addr).unwrap();
}

/* Server side listener for packets,  */
// go thru again and make sure that every function fits within new framework
pub fn listen(
    udp: Res<UDP>,
    mut commands: Commands,
    // mut players: Query<(&mut Velocity, &mut Transform, &mut NetworkId), With<Player>>,
    mut players_q: Query<(&mut Velocity, &mut Transform, &mut Health,
         &mut Crouch, &mut Roll, &mut Sprint, &mut Attack, &mut NetworkId), (With<Player>, Without<Enemy>)>,//eek a lot
    mut n_p: ResMut<PlayerCount>,
    mut addresses: ResMut<AddressList>,
    mut server_seq: ResMut<Sequence>,

    mut enemies_to_kill: ResMut<EnemiesToKill>,
    mut enemies: Query<(Entity, &mut EnemyId, &mut EnemyMovement, &mut Transform), (With<Enemy>, Without<Player>)>,
) {
    loop{
   /* to hold msg */
        let mut buf: [u8; 1024] = [0;1024];
        // pseudo poll. nonblocking, gives ERR on no read tho
        let packet = udp.socket.recv_from(&mut buf);
        match packet{
            Err(_e)=> return,
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
                send_id(src,  n_p.as_mut(), &mut commands, &mut addresses, &mut server_seq,&udp)},
            ClientPacket::PlayerPacket(player_packet) => {
                // TODO: Fix this
                update_player_state(src, &mut players_q, player_packet, &mut commands);
            }
            ClientPacket::KillEnemyPacket(kill_enemy) => {
                println!("recieved kill enemy packet");
                update_despawn(kill_enemy, &mut enemies_to_kill, &mut commands, &mut enemies); 
            }
            
        }
    }
}

    /* uses items in packetQueue to send to all clients,
    * and removes them from the list.  */
    pub fn server_send_packets(
        mut packet_q: ResMut<ServerPacketQueue>,
        udp: Res<UDP>,
        addresses: Query<&NetworkId>,

    ){
        /* for all packets in queue */
        for packet in packet_q.packets.iter(){
            let mut serializer = flexbuffers::FlexbufferSerializer::new();
            packet.serialize(&mut serializer).unwrap();
            let packet_chunk: &[u8] = serializer.view();
            /* send to all users */
            'adds: for address in addresses.iter()
            {
                /* buuuut only id for the id'd, and player 1 not to player 1 again,
                * instaed off to p2 */
                match packet {
                    ServerPacket::PlayerPacket(playa) =>{
                        if address.id == playa.head.network_id{
                            continue 'adds;
                        }
                    }
                    ServerPacket::IdPacket(id)=> {
                        if address.id != id.head.network_id{
                            continue 'adds;
                        }
                    }
                    ServerPacket::EnemyPacket(enemy)=>{
                    //  info!("sending enemy packet");
                        if address.id == enemy.head.network_id {
                            continue 'adds;
                        }
                    }
                    _ => {}
                }
                udp.socket.send_to(&packet_chunk, address.addr).unwrap();

            }

            /* I want to deleteteeeeeee. What's rust's free thing? We
            * all good to just like make a new one? Or is that grim */
        }
        packet_q.packets = Vec::new();
    }

    // //TOTOTOODODODODODODODO--------------------------------
    // fn recieve_input(
    //     client_pack: PlayerC2S,
    //     mut players_q: Query<(&mut Velocity, &mut Transform, &mut Health,
    //          &mut Crouch, &mut Roll, &mut Sprint, &mut Attack, &mut NetworkId,
    //           &mut InputQueue, &Timestamp), (With<Player>, Without<Enemy>)>,
    // ){
    //     // TODO this needs to check inputs and move player, check for collisions, basically everything we are doing onv the client side idk
    //     /* for all players in server game world */
    //     for (v, t, h, c, r, s, a, id, mut iq, time) in players_q.iter_mut(){
    //         /* if we find the one corresponding to our packet */
    //         if client_pack.head.network_id == id.id {
    //             /* for all the keys passed on the clients update */
    //             iq.q.push((client_pack.head.sequence.get(), client_pack.key.clone()));
                
    //             /* ok if we want to update immediately then we od it right here
    //              * buuuuut the fn takes in diff args than we have (odd query). TBH
    //              * i am down to plop in the main logic loop for now, no reaason to use
    //              * any data longer than we have to, right?? (is not in main logic loop as of
    //              * 11/19 3:31pm*/
    //         }
    //     }
    // }

pub fn send_enemies(
    enemies: Query<(& EnemyId, & EnemyMovement, &Transform), 
        (With<Enemy>, Without<Player>)>,
    server_seq: ResMut<Sequence>,
    mut packet_q: ResMut<ServerPacketQueue>,
    addresses: Res<AddressList>,
    udp: Res<UDP>
){
    //info!("sending enemies");
    
    /* for each enemy in the game world */
    for (id, movement, transform) in enemies.iter(){
        /* packet-ify it */
        let enemy  = ServerPacket::EnemyPacket(
        EnemyS2C{
            transform: *transform,
            head: Header::new(0,server_seq.clone()),
            movement: movement.clone(),
            enemytype: id.clone(),
        });
        //info!("actually entered for loop lmfao crazy if this was what was broken loll");

        /* send off to our clients  */
        let mut serializer = flexbuffers::FlexbufferSerializer::new();
        enemy.serialize(&mut serializer).unwrap();
        let packet: &[u8] = serializer.view();
        for addr in addresses.list.iter(){
            udp.socket.send_to(&packet, addr).unwrap();
        }
    }
}



// /* once we have our packeet, we must use it to update
//  * the player specified, there's another in client.rs*/
// fn update_player_state_OLD_AND_BROKEN(
//     src: SocketAddr,
//     /* fake query, passed from above system */
//     mut players: Query<(&mut Velocity, &mut Transform, &mut NetworkId), With<Player>>,
//     player_struct: PlayerPacket,
//     mut commands: Commands,
// ) { 
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

fn update_player_state(
    src: SocketAddr,
    mut players_q: &mut Query<(&mut Velocity, &mut Transform, &mut Health,
        &mut Crouch, &mut Roll, &mut Sprint, &mut Attack, &mut NetworkId), 
            (With<Player>, Without<Enemy>)>,
    player_struct: PlayerSendable,
    mut commands: &mut Commands
){
    let mut found = false;
    for (mut vel,mut trans,mut health, mut crouching, mut rolling, mut sprinting, mut attacking, id) in players_q.into_iter(){

        if id.id == player_struct.head.network_id {
         //   info!("updaetd");
            trans.translation.x = player_struct.transform.translation.x;
            trans.translation.y = player_struct.transform.translation.y;
            vel.velocity.x = player_struct.velocity.x;
            vel.velocity.y = player_struct.velocity.y;
            health.current = player_struct.health.current;
            crouching.crouching = player_struct.crouch;
            rolling.rolling = player_struct.roll;
            sprinting.sprinting = player_struct.sprint;
            attacking.attacking = player_struct.attack;
            // *trans = player_struct.client_bundle.transform;
            // *vel = player_struct.client_bundle.velo;
            // *health = player_struct.client_bundle.health;
            // *crouching = player_struct.client_bundle.crouching;
            // *rolling = player_struct.client_bundle.rolling;
            // *sprinting = player_struct.client_bundle.sprinting;
            // *attacking = player_struct.client_bundle.attacking;
            found = true;
        }

    }
    if !found {
        info!("spawning anew with id{}", player_struct.head.network_id);
        let v = cuscuta_resources::Velocity { velocity: player_struct.velocity };
        commands.spawn(ServerPlayerBundle {
            velo: v,
            transform: player_struct.transform,
            id: NetworkId::new_s(player_struct.head.network_id, src),
            health: player_struct.health,
            rolling: Roll::new_set(player_struct.roll),
            crouching: Crouch::new_set(player_struct.crouch),
            sprinting: Sprint::new_set(player_struct.sprint),
            attacking: Attack::new_set(player_struct.attack), 
            player: Player,
            track: Trackable,
        });
    }
}

fn update_despawn(
    kill_enemy: KillEnemyPacket,
    enemies_to_kill: &mut EnemiesToKill,
    commands: &mut Commands,
    enemies: &mut Query<(Entity, &mut EnemyId, &mut EnemyMovement, &mut Transform), (With<Enemy>, Without<Player>)>,
){
    enemies_to_kill.list.push(kill_enemy.clone());
    for(entity, id, _movement, _transform) in enemies.iter(){
        if id.id == kill_enemy.enemy_id.id{
            commands.entity(entity).despawn();
            println!("despawning enemy");
        }
    }
}

/* runs to send off 'despawn this hoe' messages to clients
 * ensures that if p1 kills a player, it shows for p2 */
pub fn send_despawn_command(
    mut commands: Commands,
    addresses: Res<AddressList>,
    udp: Res<UDP>,
    mut enemies_to_kill: ResMut<EnemiesToKill>,
    enemies: Query<(Entity, & EnemyId, & EnemyMovement, &Transform), 
        (With<Enemy>, Without<Player>)>,
){
        for enemy in enemies_to_kill.list.iter(){
                let mut serializer = flexbuffers::FlexbufferSerializer::new();
                let to_send: ServerPacket = ServerPacket::DespawnPacket(enemy.clone());
                to_send.serialize(&mut serializer).unwrap();
                let packet: &[u8] = serializer.view();
                for address in addresses.list.iter(){
                    udp.socket.send_to(&packet, address).unwrap();
                }
        }
        enemies_to_kill.list = Vec::new();
        
        for(entity, id, _movement, _transform) in enemies.iter(){
            for kill_enemy in enemies_to_kill.list.iter(){
                if id.id == kill_enemy.enemy_id.id{
                    commands.entity(entity).despawn();
                    println!("despawning enemy");
                }
            }
        }
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
    player : Query<(&Velocity, &Transform, &NetworkId, &Health, &Crouch, &Roll, &Sprint, &Attack), With<Player>>,
    server_seq: ResMut<Sequence>,
    mut packet_q: ResMut<ServerPacketQueue>,
    addresses: Res<AddressList>,
    udp: Res<UDP>
)
{
    /* For each player in the game*/
    for (v, t, i, h, c, r, s, a,)  in player.iter(){
        /* packet-ify it */
        info!("Sending {}", i.id);
        let outgoing_state  = ServerPacket::PlayerPacket(PlayerSendable{
            transform: *t,
            head: Header::new(i.id,server_seq.clone()),
            attack: a.attacking,
            velocity: v.velocity,
            health: *h,
            crouch: c.crouching,
            roll: r.rolling,
            sprint: s.sprinting,
        });
        /* push onto the 'to-send' queue */
        
        let mut serializer = flexbuffers::FlexbufferSerializer::new();
        outgoing_state.serialize(&mut serializer).unwrap();
        let packet: &[u8] = serializer.view();
        for addr in addresses.list.iter(){
            if *addr != i.addr {
                udp.socket.send_to(&packet, addr).unwrap();

            }
        }
    }
}   

/** INDEX TO USE
0 - floor
1 - left wall
2 - right wall
3 - chest/pot
4 - left door
5 - right door
6 - top door
7 - bottom door 
8 - top wall
9 - bottom wall */
fn send_map_packet (
    mut _commands: Commands,
    door_query: Query<(&Transform, &DoorType), With<Door>>, 
    wall_query: Query<&Transform, With<Wall>>, 
    background_query: Query<&Transform, With<Background>>,
    potion_query: Query<&Transform, With<Potion>>,
    mut packet_q: ResMut<ServerPacketQueue>,
    server_seq: ResMut<Sequence>,
) {
    let mut map_array: Vec<Vec<u8>> = vec![];
    let room_w = 10; //need to grab these values from roomgen fn()
    let _room_h = 5;

    for tile in background_query.iter()
    {
        let arr_x:usize = (tile.translation.x - 16.0) as usize / 32;
        let arr_y:usize = (tile.translation.y - 16.0) as usize / 32;
        map_array[arr_x][arr_y] = 0;
    }

    for tile in door_query.iter()
    {
        let arr_x:usize = (tile.0.translation.x - 16.0) as usize / 32;
        let arr_y:usize = (tile.0.translation.y - 16.0) as usize / 32;
        match tile.1{
            DoorType::Right => map_array[arr_x][arr_y] = 5,
            DoorType::Left => map_array[arr_x][arr_y] = 4,
            DoorType::Top => map_array[arr_x][arr_y] = 6,
            DoorType::Bottom => map_array[arr_x][arr_y] = 7
        }
    }

    for tile in wall_query.iter()
    {
        let arr_x:i32 = (tile.translation.x - 16.0) as i32 / 32;
        let arr_y:i32 = (tile.translation.y - 16.0) as i32 / 32;
        if arr_x == 0 {map_array[arr_x as usize][arr_y as usize] = 1;}
        else if arr_y == 0 {map_array[arr_x as usize][arr_y as usize] = 9;}
        else if arr_x == room_w/32 {map_array[arr_x as usize][arr_y as usize] = 2;}
        else {map_array[arr_x as usize][arr_y as usize] = 8;}
    }

    for tile in potion_query.iter()
    {
        let arr_x: usize = (tile.translation.x - 16.0) as usize / 32;
        let arr_y: usize = (tile.translation.y - 16.0) as usize / 32;
        map_array[arr_x][arr_y] = 3;
    }

    let mappy = ServerPacket::MapPacket(MapS2C{
        head: Header::new(0,server_seq.clone()),// server id == 0
        matrix: map_array
    });

    packet_q.packets.push(mappy);
}