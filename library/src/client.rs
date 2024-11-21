use std::net::SocketAddr;

use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{cuscuta_resources::*, player};
use crate::network::{
    ClientPacket, Header, IdPacket, PlayerC2S, PlayerS2C, Sequence, ServerPacket, Timestamp, UDP,
};
use crate::player::*;

// pub fn recv_id(
//     source_addr: SocketAddr,
//     network_id: &mut NetworkId,
//     ds_struct: IdPacket,
//     mut _commands: Commands,
//     mut id: ResMut<ClientId>
// ) {
//     info!("Recieving ID");
//     /* assign it to the player */
//     id.id = ds_struct.head.network_id;
//     info!("ASSIGNED ID: {:?}", id.id);
// }

/* Sends id request to the server */
pub fn id_request(
    player: Query<&NetworkId, With<Player>>,
    socket: Res<UDP>,
    mut sequence: ResMut<Sequence>,
) {
    let id_packet = IdPacket {
        head: Header {
            network_id: 0,
            sequence_num: sequence.geti(),
            timestamp: 0,
        },
    };

    let to_send: ClientPacket = ClientPacket::IdPacket(id_packet);

    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    /* Serializes, which makes a nice lil u8 array for us */
    to_send.serialize(&mut serializer).unwrap();

    let packet: &[u8] = serializer.view();

    /* beam me up scotty */
    socket.socket.send_to(packet, SERVER_ADR).unwrap();
}

/* Transforms current player state into u8 array that
 * we can then send across the wire to be deserialized once it arrives */
pub fn gather_input(
    mut player: Query<(&NetworkId, &mut InputQueue), With<Player>>,
    socket: Res<UDP>,
    client_id: Res<ClientId>,
    mut sequence: ResMut<Sequence>,
    input: Res<ButtonInput<KeyCode>>,
) {
    /* Deconstruct out Query. SHould be client side so we can do single */
    for (i, mut q) in player.iter_mut() {
        if i.id == client_id.id {
            for key in input.get_pressed() {
                let outgoing_state = PlayerC2S {
                    head: Header {
                        network_id: i.id,
                        sequence_num: sequence.geti(),
                        timestamp: 0, // TODODOODOOO
                    },
                    key: *key,
                };
                q.q.push((Timestamp { time: 0 }, *key));
                let mut serializer = flexbuffers::FlexbufferSerializer::new();
                let to_send: ClientPacket = ClientPacket::PlayerPacket(outgoing_state);
                to_send.serialize(&mut serializer).unwrap();

                let packet: &[u8] = serializer.view();

                socket.socket.send_to(&packet, SERVER_ADR).unwrap();
            }
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
    // mut player: Query<(&mut Velocity, &mut Transform, &mut NetworkId), With<Player>>,
    mut players_new: Query<
        (
            &mut Velocity,
            &mut Transform,
            &mut Player,
            &mut Health,
            &mut Crouch,
            &mut Roll,
            &mut Sprint,
            &mut Attack,
            &mut NetworkId,
        ),
        With<Player>,
    >,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    id: ResMut<ClientId>,
) {
    //info!("Listening!!!");
    /* to hold msg */
    let mut buf: [u8; 1024] = [0; 1024];
    let packet = udp.socket.recv_from(&mut buf);
    match packet {
        Err(_e) => return,
        _ => info!("read packet!"),
    }
    let (amt, src) = packet.unwrap();
    /* opcode is last byte of anything we send */
    // let opcode = buf[amt-1];

    /* trim trailing 0s and opcode*/
    let packet = &buf[..amt];

    let deserializer = flexbuffers::Reader::get_root(packet).unwrap();
    let rec_struct: ServerPacket = ServerPacket::deserialize(deserializer).unwrap();

    match rec_struct {
        ServerPacket::IdPacket(id_packet) => {
            // recv_id(src, id_packet, commands, id);
        }
        ServerPacket::PlayerPacket(player_packet) => {
            info!("Matching Player Struct");
            recieve_player_packet(commands, players_new, &asset_server, player_packet, &mut texture_atlases, id, src);
            //TODODODODOODOOo
            //update_player_state_new(players_new, player_packet, commands, &asset_server, &mut texture_atlases, src);
        }
        ServerPacket::MapPacket(map_packet) => {
            info!("Matching Map Struct");
            receive_map_packet(commands, &asset_server, map_packet.matrix);
        }
        ServerPacket::EntityPacket(player_packet) => {
            //TODO
        }
    }
}

fn recieve_player_packet(
    mut commands: Commands,
    mut players: Query<
        (
            &mut Velocity,
            &mut Transform,
            &mut Player,
            &mut Health,
            &mut Crouch,
            &mut Roll,
            &mut Sprint,
            &mut Attack,
            &mut NetworkId,
        ),
        With<Player>,
    >,
    asset_server: &Res<AssetServer>,
    saranpack: PlayerS2C,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    mut us: ResMut<ClientId>,
    source_ip: SocketAddr
) {
    let mut found = false;
    for (v, t, p, h, c, r, s, a, id) in players.iter_mut() {
        if id.id == us.id {
           
            found = true;
            info!("found");
            // need 2 make this good and not laggy yk

            /*apply state to player pls
             * needs to be some non-actual state (don't apply
             * directly to v) so we can apply reprediction*/
        }
    }

    if !found {
        us.id = saranpack.head.network_id;

        let player_sheet_handle = asset_server.load("player/4x8_player.png");
        let player_layout = TextureAtlasLayout::from_grid(
            UVec2::splat(TILE_SIZE),
            PLAYER_SPRITE_COL,
            PLAYER_SPRITE_ROW,
            None,
            None,
        );
        let player_layout_len = player_layout.textures.len();
        let player_layout_handle = texture_atlases.add(player_layout);

        commands.spawn(ClientPlayerBundle{
             sprite: SpriteBundle{ 
                texture: player_sheet_handle,
                transform: saranpack.transform,
                ..default()
            },
            atlas: TextureAtlas{
                layout: player_layout_handle,
                index: 0,
            },
            animation_timer: AnimationTimer(Timer::from_seconds(ANIM_TIME, TimerMode::Repeating)),
            animation_frames: AnimationFrameCount(player_layout_len),
            velo: Velocity{velocity:saranpack.velocity},
            id: NetworkId{id: saranpack.head.network_id, addr: source_ip},
            player: Player,
            health: saranpack.health,
            crouching: Crouch{crouching: saranpack.crouch},
            rolling: Roll{rolling: saranpack.roll},
            sprinting: Sprint{sprinting: saranpack.sprint},
            attacking: Attack{attacking:saranpack.attack},
            inputs: InputQueue::new(),

        });
    }}

// /* once we have our packeet, we must use it to update
//  * the player specified, there's another in server.rs */
// fn update_player_state(
//     /* fake query, passed from above system */
//     mut players: Query<(&mut Velocity, &mut Transform, &mut NetworkId), With<Player>>,
//     player_struct: PlayerPacket,
//     mut commands: Commands,
//     asset_server: &Res<AssetServer>,
//     texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
//     source_ip: SocketAddr
// ) {
//     // let deserializer = flexbuffers::Reader::get_root(buf).unwrap();
//     // let player_struct = PlayerPacket::deserialize(deserializer).unwrap();
//     let mut found = false;
//     for (mut velo, mut transform, network_id) in players.iter_mut(){
//         info!("REc: {}  Actual:: {}", player_struct.id, network_id.id);
//         if network_id.id == player_struct.id{
//             transform.translation.x = player_struct.transform_x;
//             transform.translation.y = player_struct.transform_y;
//             velo.velocity.x = player_struct.velocity_x;
//             velo.velocity.y = player_struct.velocity_y;
//             found = true;
//         }
//     }
//     if !found{
//         info!("new player!");
//         client_spawn_other_player(&mut commands, asset_server, texture_atlases,player_struct, source_ip);
//     }
// }

// fn update_player_state_new(
//     mut players: Query<(&mut Velocity, &mut Transform, &mut Player, &mut Health, &mut Crouch, &mut Roll, &mut Sprint, &mut Attack, &mut NetworkId), With<Player>>,
//     player_struct: NewPlayerPacket,
//     mut commands: Commands,
//     asset_server: &Res<AssetServer>,
//     texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
//     source_ip: SocketAddr
// ){
//     let mut found = false;
//     for(mut velocity, mut transform,mut player,mut health, mut crouch, mut roll, mut sprint, mut attack, mut network_id) in players.iter_mut(){
//         if network_id.id == player_struct.client_bundle.id.id{
//            // *transform = player_struct.client_bundle.transform;
//             transform.translation.x = player_struct.client_bundle.transform.translation.x;
//             transform.translation.y = player_struct.client_bundle.transform.translation.y;
//             velocity.velocity.x = player_struct.client_bundle.velo.velocity.x;
//             velocity.velocity.y = player_struct.client_bundle.velo.velocity.y;
//             health.current = player_struct.client_bundle.health.current;
//             crouch.crouching = player_struct.client_bundle.crouching.crouching;
//             roll.rolling = player_struct.client_bundle.rolling.rolling;
//             sprint.sprinting = player_struct.client_bundle.sprinting.sprinting;
//             attack.attacking = player_struct.client_bundle.attacking.attacking;
//            // *velocity = player_struct.client_bundle.velo;
//             // *health = player_struct.client_bundle.health;
//             // *crouch = player_struct.client_bundle.crouching;
//             // *roll = player_struct.client_bundle.rolling;
//             // *sprint = player_struct.client_bundle.sprinting;
//             // *attack = player_struct.client_bundle.attacking;
//             found = true;
//         }
//     }
//     if !found {
//         info!("new player!");
//         let v = player_struct.client_bundle.velo;
//         client_spawn_other_player_new(&mut commands, asset_server, texture_atlases, player_struct, source_ip);
//     }
// }


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
fn receive_map_packet (
    mut commands: Commands,
    asset_server: &Res<AssetServer>,
    map_array: Vec<Vec<u8>>
) {
    let mut vertical = -((map_array.len() as f32) / 2.0) + (TILE_SIZE as f32 / 2.0);
    let mut horizontal = -((map_array[0].len() as f32) / 2.0) + (TILE_SIZE as f32 / 2.0);

    for a in 0..map_array.len() {
        for b in 0..map_array[0].len() {
            let val = map_array[a][b];
            match val {
                0 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/cobblestone_floor/cobblestone_floor.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, 0.0),
                    ..default() },)),
                1 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/walls/left_wall.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, 0.1),
                    ..default() },)),
                2 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/walls/right_wall.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, 0.1),
                    ..default() },)),
                3 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/1x2_pot.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, 0.1),
                    ..default() },)),
                4 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/solid_floor/solid_floor.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, 0.2),
                    ..default() },)),
                5 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/solid_floor/solid_floor.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, 0.3),
                    ..default() },)),
                6 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/solid_floor/solid_floor.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, 0.4),
                    ..default() },)),
                7 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/solid_floor/solid_floor.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, 0.5),
                    ..default() },)),
                8 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/walls/north_wall.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, 0.2),
                    ..default() },)),
                9 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/walls/bottom_wall.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, 0.2),
                    ..default() },)),
                _ => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/walls/bottom_wall.png").clone(),
                    transform: Transform::from_xyz(-10000.0, -10000.0, 0.2),
                    ..default() },)),
            };
            horizontal = horizontal + TILE_SIZE as f32;
        }
        vertical = vertical + TILE_SIZE as f32;
    }
}
