use std::net::SocketAddr;

use bevy:: prelude::*;
use network::*;
use serde::{Deserialize, Serialize};

use crate::markov_chains::LastAttributeArray;
use crate::room_gen::{InnerWall, RoomChangeEvent, RoomConfig};
use crate::{cuscuta_resources::{self, AddressList, Background, EnemiesToKill, Health, PlayerCount, Pot, Velocity, Wall, TILE_SIZE}, enemies::{Enemy, EnemyId, EnemyMovement}, network, player::{check_door_collision, Attack, Crouch, NetworkId, Player, Roll, ServerPlayerBundle, Sprint, Trackable}, room_gen::{transition_map, Door, DoorType, Potion, Room, RoomManager}, ui::CarnageBar};


/* Upon request, sends an id to client, spawns a player, and
 * punts player state off to client via the packet queue */
pub fn send_id(
    source_addr : SocketAddr,
    n_p: &mut PlayerCount,
    commands: &mut Commands,
    addresses: &mut AddressList,
    server_seq: &mut Sequence,
    udp: & UDP
) {
    /* assign id, update player count */
    n_p.count += 1;
    let player_id: u8 = n_p.count;
    addresses.list.push(source_addr);
    println!("pushing addresss");
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
        health: Health::new_init(),
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
        health: Health::new_init(),
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
         &mut Crouch, &mut Roll, &mut Sprint, &mut Attack, &mut NetworkId), 
         (With<Player>, Without<Enemy>, Without<Potion>, Without<Door>, Without<Wall>, Without<Background>, Without<DoorType>, Without<Pot>,Without<InnerWall>)>,//eek a lot
    mut n_p: ResMut<PlayerCount>,
    mut door_query: Query<(&mut Transform, &Door), (Without<Wall>, Without<Background>, Without<Potion>, Without<Enemy>, Without<Pot>,Without<InnerWall>)>,  
    mut wall_query: Query<&mut Transform, (With<Wall>, Without<Door>, Without<Background>, Without<Potion>, Without<Enemy>, Without<Pot>,Without<InnerWall>)>, 
    mut background_query: Query<&mut Transform, (With<Background>, Without<Potion>, Without<Enemy>, Without<Pot>,Without<InnerWall>)>,
    mut potion_query: Query<&mut Transform, (With<Potion>, Without<Pot>, Without<Enemy>,Without<InnerWall>)>,
    mut pot_query: Query<&mut Transform, (With<Pot>, Without<Enemy>,Without<InnerWall>)>,
    mut roomman: ResMut<RoomManager>,
    mut addresses: ResMut<AddressList>,
    mut server_seq: ResMut<Sequence>,
    mut enemies_to_kill: ResMut<EnemiesToKill>,
    mut enemies: Query<(Entity, &mut EnemyId, &mut EnemyMovement, &mut Transform, &mut Health), (With<Enemy>, Without<Player>, Without<InnerWall>)>,
    mut inner_wall_query: Query<&mut Transform, With<InnerWall>>,
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
                send_id(src,  &mut n_p, &mut commands, &mut addresses, &mut server_seq, &udp);
                println!("{:?}", addresses.list);
                send_map_packet(&mut door_query, &mut wall_query,
                    &mut background_query, &mut potion_query,
                    &mut pot_query, &mut inner_wall_query,
                    &mut server_seq,
                    &mut roomman, &udp, & addresses);
            },
            ClientPacket::PlayerPacket(player_packet) => {
                // TODO: Fix this
                update_player_state(src, &mut players_q, player_packet, &mut commands);
            }  
            ClientPacket::KillEnemyPacket(kill_enemy) => {
                println!("recieved kill enemy packet");
                update_despawn(kill_enemy, &mut enemies_to_kill, &mut commands, &mut enemies); 
            }

            ClientPacket::DecreaseEnemyHealthPacket(decrease_enemy_health_packet) => {
                println!("recieved decrease enemy health packet");
                // TODO: decrease serverside health :p
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
    enemies: Query<(& EnemyId, & EnemyMovement, &Transform, &Health), 
        (With<Enemy>, Without<Player>)>,
    server_seq: ResMut<Sequence>,
    mut packet_q: ResMut<ServerPacketQueue>,
    addresses: Res<AddressList>,
    udp: Res<UDP>
){
    
    /* for each enemy in the game world */
    for (id, movement, transform, health) in enemies.iter(){
        /* packet-ify it */
       // println!("created enemy packet");
        let enemy  = ServerPacket::EnemyPacket(
        EnemyS2C{
            transform: *transform,
            head: Header::new(0,server_seq.clone()),
            movement: movement.clone(),
            enemytype: id.clone(),
            health: health.clone()
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
        (With<Player>, Without<Enemy>, Without<Potion>, Without<Door>, Without<Wall>, Without<Background>, Without<DoorType>, Without<Pot>,Without<InnerWall>)>,//eek a lot
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
    enemies: &mut Query<(Entity, &mut EnemyId, &mut EnemyMovement, &mut Transform, &mut Health), (With<Enemy>, Without<Player>, Without<InnerWall>)>,
){
    enemies_to_kill.list.push(kill_enemy.clone());
    for(entity, id, _movement, _transform, _health) in enemies.iter(){
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
    enemies: Query<(Entity, & EnemyId, & EnemyMovement, &Transform, &mut Health), 
        (With<Enemy>, Without<Player>)>,
){
        for enemy in enemies_to_kill.list.iter(){
                let mut serializer = flexbuffers::FlexbufferSerializer::new();
                let to_send: ServerPacket = ServerPacket::DespawnPacket(enemy.clone());
                to_send.serialize(&mut serializer).unwrap();
                let packet: &[u8] = serializer.view();
                for address in addresses.list.iter(){
                    udp.socket.send_to(&packet, address).unwrap();
                    println!("sending despawn packet");
                }
        }
        enemies_to_kill.list = Vec::new();
        
        for(entity, id, _movement, _transform, mut _health) in enemies.iter(){
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
    addresses: Res<AddressList>,
    udp: Res<UDP>
)
{
    /* For each player in the game*/
    for (v, t, i, h, c, r, s, a,)  in player.iter(){
        /* packet-ify it */
        info!("Sending {}", i.id);
        let mut better_z = *t;
        better_z.translation.z = 100.;
        let outgoing_state  = ServerPacket::PlayerPacket(PlayerSendable{
            transform: better_z,
            head: Header::new(i.id,server_seq.clone()),
            attack: a.attacking,
            velocity: v.velocity,
            health: *h,
            crouch: c.crouching,
            roll: r.rolling,
            sprint: s.sprinting,
        });
        /* push onto the 'to-send' queue */
        
        /* send to everyone but self, let client movement happen */
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

pub fn send_player_to_self(
    player : &Query<(&Velocity, &mut Transform, &NetworkId, &Health, &Crouch, &Roll, &Sprint, &Attack), 
        (With<Player>, Without<Door>, Without<Wall>, Without<Background>, Without<Potion>, Without<Enemy>, Without<Pot>,Without<InnerWall>)>,
    server_seq: &mut Sequence,
    addresses: &AddressList,
    udp: &UDP
)
{
    /* For each player in the game*/
    for (v, t, i, h, c, r, s, a,)  in player.iter(){
        /* packet-ify it */
        info!("Sending {}", i.id);
        let mut transform_to_send = *t;
        transform_to_send.translation.z = 100.;
        let outgoing_state  = ServerPacket::PlayerPacket(PlayerSendable{
            transform: transform_to_send,
            head: Header::new(i.id,server_seq.clone()),
            attack: a.attacking,
            velocity: v.velocity,
            health: *h,
            crouch: c.crouching,
            roll: r.rolling,
            sprint: s.sprinting,
        });
        /* push onto the 'to-send' queue */
        
        /* send to everyone but self, let client movement happen */
        let mut serializer = flexbuffers::FlexbufferSerializer::new();
        outgoing_state.serialize(&mut serializer).unwrap();
        let packet: &[u8] = serializer.view();
        for addr in addresses.list.iter(){
            udp.socket.send_to(&packet, addr).unwrap();
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
9 - bottom wall 
10 - pot */
fn send_map_packet (
    door_query: &mut Query<(&mut Transform, &Door), (Without<Wall>, Without<Background>, Without<Potion>, Without<Enemy>, Without<Pot>,Without<InnerWall>)>,  
    wall_query: &mut Query<&mut Transform, (With<Wall>, Without<Door>, Without<Background>, Without<Potion>, Without<Enemy>, Without<Pot>,Without<InnerWall>)>, 
    background_query: &mut Query<&mut Transform, (With<Background>, Without<Potion>, Without<Enemy>, Without<Pot>,Without<InnerWall>)>,
    potion_query: &mut Query<&mut Transform, (With<Potion>, Without<Pot>, Without<Enemy>,Without<InnerWall>)>,
    pot_query: &mut Query<&mut Transform, (With<Pot>, Without<Enemy>,Without<InnerWall>)>,
    inner_wall_query: &mut Query<&mut Transform, (With<InnerWall>)>,
    server_seq: &Sequence,
    roomman: &mut RoomManager,
    udp: &UDP,
    addresses: &AddressList,
) {

    let (room_w,room_h):(f32, f32) = RoomManager::current_room_size(&roomman);
    let room_tile_w = room_w / TILE_SIZE as f32;
    let room_tile_h = room_h / TILE_SIZE as f32;
    let mut map_array: Vec<Vec<u8>> =vec![vec![0; room_tile_h as usize + 1]; room_tile_w as usize + 1];
    

    let max_x = room_w / 2.0 ;
    let max_y = room_h / 2.0 ;
    for tile in background_query.iter()
    {
        let arr_x:usize = (tile.translation.x + max_x - 16.0) as usize / 32;
        let arr_y:usize = (tile.translation.y + max_y - 16.0) as usize / 32;
        map_array[arr_x][arr_y] = 0;
    }

    for tile in wall_query.iter()
    {
        let arr_x:usize = (tile.translation.x + max_x - 16.0) as usize / 32;
        let arr_y:usize = (tile.translation.y + max_y - 16.0) as usize / 32;
        if arr_x == 0 {map_array[arr_x][arr_y as usize] = 1;}
        else if arr_y == 0 {map_array[arr_x][arr_y] = 9;}
        else if arr_x == (room_w as usize/32)-1 {map_array[arr_x][arr_y] = 2;}
        else {map_array[arr_x][arr_y] = 8;}
    }

    for tile in inner_wall_query.iter(){
        let arr_x:usize = (tile.translation.x + max_x - 16.0) as usize / 32;
        let arr_y:usize = (tile.translation.y + max_y - 16.0) as usize / 32;
        map_array[arr_x][arr_y as usize] = 11;
    }

    for tile in potion_query.iter()
    {
        let arr_x: usize = (tile.translation.x + max_x - 16.0) as usize / 32;
        let arr_y: usize = (tile.translation.y + max_y - 16.0) as usize / 32;
        map_array[arr_x][arr_y] = 3;
    }

    for tile in pot_query.iter()
    {
        let arr_x: usize = (tile.translation.x + max_x - 16.0) as usize / 32;
        let arr_y: usize = (tile.translation.y + max_y - 16.0) as usize / 32;
        map_array[arr_x][arr_y] = 10;
    }

    /* grab doors */
    for tile in door_query.iter()
    {
        let arr_x:usize = (tile.0.translation.x + max_x - 16.0) as usize / 32;
        let arr_y:usize = (tile.0.translation.y + max_y - 16.0) as usize / 32;
        println!("Matching door! @ ({},{})", arr_x, arr_y);
        match tile.1.door_type
        {
            DoorType::Right => map_array[arr_x][arr_y] = 5, 
            DoorType::Left => map_array[arr_x][arr_y] = 4,
            DoorType::Top => map_array[arr_x][arr_y] = 6,
            DoorType::Bottom => map_array[arr_x][arr_y] = 7
        }
    }
    println!("{:?},", map_array);
    let mappy = ServerPacket::MapPacket(MapS2C{
        head: Header::new(0,server_seq.clone()),// server id == 0
        matrix: map_array,
        size: RoomManager::current_room_size(&roomman),
        max: RoomManager::current_room_max(&roomman),
        z: roomman.current_z_index,
    });

    

    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    mappy.serialize(&mut serializer).unwrap();
    let packet: &[u8] = serializer.view();
    for addr in addresses.list.iter(){
        let drawn = udp.socket.send_to(&packet, addr).unwrap();
        println!("wrote {} bytes", drawn);
        println!("sending map packet yippee for {}", addr);
    }
    
}

pub fn check_door(
    mut player : Query<(&mut Transform), With<Player>>,
    door_query: Query<(&Transform, &Door), (Without<Player>, Without<Enemy>)>,
    mut carnage: Query<&mut CarnageBar>,
    mut commands: Commands,
    mut room_manager: ResMut<RoomManager>,
    mut room_query: Query<Entity, With<Room>>,
    mut room_change: EventWriter<RoomChangeEvent>,
    mut last_attribute_array: ResMut<LastAttributeArray>,
    room_config: Res<RoomConfig>,
){
    /* are allthe players standing on a door? */
    let mut all_hit = true;
    let mut have_player = false;
    let mut final_door = None;
    /* for all players */
    for transform in player.iter(){
        let (door_hit, door_type) = check_door_collision(&door_query, transform);
        /* ah boolean. ensures if we get false, it'll
         * stay false. do need to make sure we have a player lol.. */
        all_hit = all_hit && door_hit;
        have_player = true;
        /* also set door. If the final door is None,
         * we will set (lower). If final_door is something,
         * we check if its the same as what we just got. if it's not,
         * the players are on different doors, abort mission */
        if let Some(final_door) = final_door{
            /* final door is something! Is door_type? */
            if let Some(door_type) = door_type{
                /* yippe! if same, we dont care, if not
                 * boooo killll tomato */
                if door_type != final_door{
                    all_hit = false;
                }
            }
        } else{
            final_door = door_type;
        }
    }

    // If a door was hit, handle the transition
    if all_hit && have_player{
        let mut carnage_bar = carnage.single_mut();
        carnage_bar.stealth += 10.;
        if let Some(final_door) = final_door {
            transition_map(
                &mut commands,
                &mut room_manager,
                &mut room_query,
                &mut player,
                final_door,
                &mut carnage,
                &mut last_attribute_array,
                &room_config,
            );
            room_change.send(RoomChangeEvent(all_hit));
        }
    }
}   

pub fn room_change_infodump(
    mut event_listener: EventReader<RoomChangeEvent>,
    udp: Res<UDP>,
    mut addresses: Res<AddressList>,
    player : Query<(&Velocity, &mut Transform, &NetworkId, &Health, &Crouch, &Roll, &Sprint, &Attack), 
        (With<Player>, Without<Door>, Without<Wall>, Without<Background>, Without<Potion>, Without<Enemy>, Without<Pot>,Without<InnerWall>)>,
    mut server_seq: ResMut<Sequence>,
    mut door_query: Query<(&mut Transform, &Door), 
        (Without<Wall>, Without<Background>, Without<Potion>, Without<Enemy>, Without<Pot>,Without<InnerWall>)>,  
    mut wall_query: Query<&mut Transform, 
        (With<Wall>, Without<Door>, Without<Background>, Without<Potion>, Without<Enemy>, Without<Pot>,Without<InnerWall>)>, 
    mut background_query: Query<&mut Transform, 
        (With<Background>, Without<Potion>, Without<Enemy>, Without<Pot>,Without<InnerWall>)>,
    mut potion_query: Query<&mut Transform, 
        (With<Potion>, Without<Pot>, Without<Enemy>,Without<InnerWall>)>,
    mut pot_query: Query<&mut Transform, 
        (With<Pot>, Without<Enemy>,Without<InnerWall>)>,
    mut inner_wall_query: Query<&mut Transform, With<InnerWall>>,
    mut room_manager: ResMut<RoomManager>,
){
    for event in event_listener.read(){
        if !event.0{continue};
        send_map_packet(&mut door_query, &mut wall_query,
             &mut background_query, &mut potion_query,
              &mut pot_query, &mut inner_wall_query,
              &server_seq,
               &mut room_manager, &udp, & addresses);
        send_player_to_self(&player, &mut server_seq, &addresses, &udp);


    }
}