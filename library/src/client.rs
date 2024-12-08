use std::net::SocketAddr;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{cuscuta_resources::*, player};
use crate::enemies::{ClientEnemy, Enemy, EnemyId, EnemyKind, EnemyMovement};
use crate::network::{
    ClientPacket, ClientPacketQueue, EnemyS2C, Header, IdPacket, KillEnemyPacket, MapS2C, PlayerSendable, Sequence, ServerPacket, UDP
};
use crate::player::*;
use crate::room_gen::{ClientDoor, ClientRoomManager, Door, DoorType, Potion, Room};

/* sends out all clientPackets from the ClientPacketQueue */
pub fn client_send_packets(udp: Res<UDP>, mut packets: ResMut<ClientPacketQueue>) {
    /* for each packet in queue, we send to server*/
    for pack in &packets.packets {
        let mut serializer = flexbuffers::FlexbufferSerializer::new();
        pack.serialize(&mut serializer).unwrap();
        let packet: &[u8] = serializer.view();
        udp.socket.send_to(&packet, SERVER_ADR).unwrap();
    }
    /* i hope this is not fucking our code */
    packets.packets = Vec::new();
}

/* server send us an id so we can know we are we yk */
pub fn recv_id(
    ds_struct: &IdPacket,
    sequence: &mut Sequence,
    mut id: &mut ClientId
) {
    info!("Recieving ID");
    /* assign it to the player */
    id.id = ds_struct.head.network_id;
    /* IMPORTANTE!!! index lets Sequence know
     * what of it's vector values is USSSSS.
     * Seq.index == Player.NetworkId == ClientId
     * for any given client user. Server == 0
     * here we set index*/
    sequence.new_index(ds_struct.head.network_id.into());
    /* here we set the clock values */
    sequence.assign(&ds_struct.head.sequence);
    info!("ASSIGNED ID: {:?}", id.id);
}

/* Sends id request to the server
 * ID PLESASE */
pub fn id_request(
    udp: Res<UDP>,
) {
    /* make an idpacket, server knows if it receives one of these
     * what we really want */
    let id_packet = ClientPacket::IdPacket(IdPacket {
        head: Header {
            network_id: 0,
            sequence: Sequence::new(0),
        }});
    /* stright up sending */
    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    id_packet.serialize(&mut serializer).unwrap();
    let packet: &[u8] = serializer.view();
    udp.socket.send_to(&packet, SERVER_ADR).unwrap();
}


/* client listening function. Takes in a packet, deserializes it
 * into a ServerPacket (client here so from server).
 * Then we match against the packet
 * to figure out what kind it is, passing to another function to properly handle.
 * Important to note that we will Sequence::assign() on every packet
 * within the match, to make sure we Lamport corectly */
pub fn listen(
    udp: Res<UDP>,
    mut commands: Commands,
    mut players_q: Query<
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
            &mut Visibility
        ),
        With<Player>,
    >,
    mut enemy_q: Query<(Entity, &mut Transform, &mut EnemyMovement, &mut EnemyId, &mut EnemyPastStateQueue, &mut Health),(With<Enemy>, Without<Player>)>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut client_id: ResMut<ClientId>,
    mut sequence: ResMut<Sequence>,
    mut room_query: Query<Entity, With<Room>>,
    mut room_manager: ResMut<ClientRoomManager>,
    mut idstore: ResMut<'_, EnemyIdChecker>
) {
    //info!("Listening!!!");
    loop{
    /* to hold msg */
    let mut buf: [u8; 10000] = [0; 10000];
    /* grab dat shit */
    let packet = udp.socket.recv_from(&mut buf);
    match packet {
        Err(_e) => return,
        _ => {}
    }
    let (amt, src) = packet.unwrap();

    /* trim trailing 0s */
    let packet = &buf[..amt];

    /* deserialize and turn into a ServerPacket */
    let deserializer = flexbuffers::Reader::get_root(packet).unwrap();
    let rec_struct: ServerPacket = ServerPacket::deserialize(deserializer).unwrap();

    /* match to figure out. MAKE SURE WE SEQUENCE::ASSIGN() on every
     * packet!! is essential for lamportaging */
    match rec_struct {
        ServerPacket::IdPacket(id_packet) => {
            info!("matching idpacket");
            recv_id(&id_packet, &mut sequence, &mut client_id);
            sequence.assign(&id_packet.head.sequence);
        }
        ServerPacket::PlayerPacket(player_packet) => {
            info!("Matching Player  {}", player_packet.head.network_id);
            /*  gahhhh sequence borrow checker is giving me hell */
            /* if we encounter porblems, it's herer fs */ 
            receive_player_packet( &mut commands, &mut players_q, &asset_server, &player_packet, &mut texture_atlases, src,);
            sequence.assign(&player_packet.head.sequence);
        }
        ServerPacket::MapPacket(map_packet) => {
            info!("Matching Map Struct");
            receive_map_packet(&mut commands, &asset_server, &map_packet, &mut room_query, &mut room_manager);
            sequence.assign(&map_packet.head.sequence);
        }
        ServerPacket::EnemyPacket(enemy_packet) => {
           // info!{"Matching Enemy Struct"};
            recv_enemy(&enemy_packet, &mut commands, &mut enemy_q, &asset_server, &mut texture_atlases, &mut idstore);
            sequence.assign(&enemy_packet.head.sequence);
        }
        ServerPacket::DespawnPacket(despawn_packet) => {
            info!("Matching Despawn Packet");
            despawn_enemy(&mut commands, &mut enemy_q, &despawn_packet.enemy_id);

        }
        ServerPacket::MonkeyPacket(monkey_packet) => {
            info!("Matching Monkey Packet");
            player::spawn_other_monkey(&mut commands, monkey_packet.transform, &asset_server, &mut texture_atlases,);
        }
    }
}// stupid loop
}

fn receive_player_packet(
    mut commands: &mut Commands,
    mut players: &mut Query<
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
            &mut Visibility
        ),
        With<Player>,
    >,
    asset_server: &AssetServer,
    saranpack: &PlayerSendable,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    source_ip: SocketAddr,
) {
    /* need to know if we were sent a player we don't currently have */
    let mut found_packet = false;
    /* for all players, find what was sent */
    for (mut v, mut t, _p, mut h, mut c, mut r, mut s, mut a, id, mut visibility) in players.iter_mut() {
        if id.id == saranpack.head.network_id {
            /* we found! */
            found_packet = true;
            /* set player */
            v.set(&saranpack.velocity);
            /* dam u transform */
            *t = saranpack.transform;
            //info!("TRANSFORM: {:?}", saranpack.transform);

            h.set(&saranpack.health);
            c.set(saranpack.crouch);
            s.set(saranpack.sprint);
            a.set(saranpack.attack);
            r.rolling = saranpack.roll;

        }
        if h.current <= 0.{
            *visibility = Visibility::Hidden;
        }
        else
        {
            *visibility = Visibility::Visible;
        }
    }

    /* ohno. he doesnt exist... what. */
    if !found_packet {
        info!("creating new player {}", saranpack.head.network_id);
        let player_sheet_handle = asset_server.load("player/4x12_player.png");
        let player_layout = TextureAtlasLayout::from_grid(
            UVec2::splat(TILE_SIZE),
            PLAYER_SPRITE_COL,
            PLAYER_SPRITE_ROW,
            None,
            None,
        );
        let player_layout_len = player_layout.textures.len();
        let player_layout_handle = texture_atlases.add(player_layout);
        info!("SPAWN SPAWN SPAWNNNN");// SPAWN SPAWN SPAWN
        commands.spawn(ClientPlayerBundle {
            sprite: SpriteBundle {
                texture: player_sheet_handle,
                transform: saranpack.transform,
                ..default()
            },
            atlas: TextureAtlas {
                layout: player_layout_handle,
                index: 0,
            },
            animation_timer: AnimationTimer(Timer::from_seconds(ANIM_TIME, TimerMode::Repeating)),
            animation_frames: AnimationFrameCount(player_layout_len),
            velo: Velocity {
                velocity: saranpack.velocity,
            },
            id: NetworkId {
                id: saranpack.head.network_id,
                addr: source_ip,
            },
            player: Player,
            health: saranpack.health,
            crouching: Crouch {
                crouching: saranpack.crouch,
            },
            rolling: Roll {
                rolling: saranpack.roll,
            },
            sprinting: Sprint {
                sprinting: saranpack.sprint,
            },
            attacking: Attack {
                attacking: saranpack.attack,
            },
            inputs: InputQueue::new(),
            states: PastStateQueue::new(),
            potion_status: ItemStatus::new(),
        });
    }
}

/*
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡀⠴⠤⠤⠴⠄⡄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⣠⠄⠒⠉⠀⠀⠀⠀⠀⠀⠀⠀⠁⠃⠆⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⢀⡜⠁⠀⠀⠀⢠⡄⠀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠑⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⢈⠁⠀⠀⠠⣿⠿⡟⣀⡹⠆⡿⣃⣰⣆⣤⣀⠀⠀⠹⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⣼⠀⠀⢀⣀⣀⣀⣀⡈⠁⠙⠁⠘⠃⠡⠽⡵⢚⠱⠂⠛⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠈⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⡆⠀⠀⠀⠀⢐⣢⣤⣵⡄⢀⠀⢀⢈⣉⠉⠉⠒⠤⠀⠿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠘⡇⠀⠀⠀⠀⠀⠉⠉⠁⠁⠈⠀⠸⢖⣿⣿⣷⠀⠀⢰⡆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⢀⠃⠀⡄⠀⠈⠉⠀⠀⠀⢴⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⢈⣇⠀⠀⠀⠀⠀⠀⠀⢰⠉⠀⠀⠱⠀⠀⠀⠀⠀⢠⡄⠀⠀⠀⠀⠀⣀⠔⠒⢒⡩⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⣴⣿⣤⢀⠀⠀⠀⠀⠀⠈⠓⠒⠢⠔⠀⠀⠀⠀⠀⣶⠤⠄⠒⠒⠉⠁⠀⠀⠀⢸⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⡄⠤⠒⠈⠈⣿⣿⣽⣦⠀⢀⢀⠰⢰⣀⣲⣿⡐⣤⠀⠀⢠⡾⠃⠀⠀⠀⠀⠀⠀⠀⣀⡄⣠⣵⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠘⠏⢿⣿⡁⢐⠶⠈⣰⣿⣿⣿⣿⣷⢈⣣⢰⡞⠀⠀⠀⠀⠀⠀⢀⡴⠋⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠈⢿⣿⣍⠀⠀⠸⣿⣿⣿⣿⠃⢈⣿⡎⠁⠀⠀⠀⠀⣠⠞⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠈⢙⣿⣆⠀⠀⠈⠛⠛⢋⢰⡼⠁⠁⠀⠀⠀⢀⠔⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠚⣷⣧⣷⣤⡶⠎⠛⠁⠀⠀⠀⢀⡤⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠁⠈⠁⠀⠀⠀⠀⠀⠠⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
*/

fn recv_enemy(
    pack: &EnemyS2C,
    mut commands: &mut Commands,
    mut enemy_q: &mut Query<(Entity, &mut Transform, &mut EnemyMovement, &mut EnemyId, &mut EnemyPastStateQueue, &mut Health),(With<Enemy>, Without<Player>)>,//TODO make ecs
    asset_server: &AssetServer,
    tex_atlas: &mut ResMut<Assets<TextureAtlasLayout>>,
    idstore: &mut ResMut<EnemyIdChecker>
){
  //  info!("rec'd enemy");
    let mut found = false;
    if idstore.idstore.contains(&pack.enemytype.id) {
        for (mut _entity, mut t, _m, i, mut q, mut health) in enemy_q.iter_mut(){
        // info!("in enemy for");
            if pack.enemytype.id == i.id{
            // info!("here!"); 
                //info!("enemy queue length: {}", q.q.len());
                if(q.q.len() > 2)
                {
                    while(q.q.len() > 2)
                    {
                        q.q.pop_back();
                    }
                }
                //info!("enemy transform: {:?} player transform {:?}", t.translation.x, pack.transform.translation.x);
                q.q.push_back(EnemyPastState{
                    transform: t.clone(),
                });
                t.translation.x = pack.transform.translation.x;
                t.translation.y = pack.transform.translation.y;
                health.current = pack.health.current;
                // enemy.movement = pack.movement;
            //  enemy.movement.push(pack.movement.clone());
                break;
            }
        }
        found = true;
    }

    if !found {
        let the_enemy: &Enemy;
        match &pack.enemytype.kind {
            EnemyKind::Skeleton(enemy) => the_enemy = enemy,
            EnemyKind::BerryRat(enemy) => the_enemy = enemy,
            EnemyKind::Ninja(enemy) => the_enemy = enemy,
            EnemyKind::SplatMonkey(enemy) => the_enemy = enemy,
            EnemyKind::Boss(enemy) => the_enemy = enemy,
        };

        let enemy_layout = TextureAtlasLayout::from_grid(
            UVec2::splat(the_enemy.size),
            the_enemy.sprite_column,
            the_enemy.sprite_row,
            None,
            None,
        );

        let enemy_layout_handle = tex_atlas.add(enemy_layout);

        // let mut vec: Vec<EnemyMovement> = Vec::new();
        // vec.push(pack.movement.clone());
        let x = pack.transform.translation.x;
        let y = pack.transform.translation.y;
        //info!("x: {} y: {}", x, y);
        let transform_to_use = Transform::from_xyz(x, y, 900.);
        commands.spawn(ClientEnemy {
            sprite: SpriteBundle {
                texture: asset_server.load(the_enemy.filepath.clone()).clone(),
                transform: transform_to_use,
                ..default()
            },
            atlas: TextureAtlas {
                layout: enemy_layout_handle,
                index: 0,
            },
            animation_timer: AnimationTimer(Timer::from_seconds(ANIM_TIME, TimerMode::Repeating)),
            animation_frames: AnimationFrameCount(
                the_enemy.sprite_column as usize * the_enemy.sprite_row as usize,
            ),
            enemy: the_enemy.clone(),
            movement: pack.movement.clone(),
            id: pack.enemytype.clone(),
            past: EnemyPastStateQueue::new(),
            health: Health::new(&the_enemy.health),
        });
        let ind = idstore.index as usize;
        //print!("adding id {}", pack.enemytype.id);
        idstore.idstore[ind] = pack.enemytype.id;
        idstore.index = idstore.index + 1;
    };
}

fn despawn_enemy(
    mut commands: &mut Commands,
    mut enemy_q: &mut Query<(Entity, &mut Transform, &mut EnemyMovement, &mut EnemyId, &mut EnemyPastStateQueue, &mut Health),(With<Enemy>, Without<Player>)>,
    id: &EnemyId
){
    for (entity, _, _, enemy_id, _, _) in enemy_q.iter_mut(){
        if enemy_id.id == id.id{
            commands.entity(entity).despawn();
            info!("killed dat");
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
fn receive_map_packet (
    mut commands: &mut Commands,
    asset_server: &AssetServer,
    mut map_packet: &MapS2C,
    mut room_query: &mut Query<Entity, With<Room>>,
    mut room_manager: &mut ClientRoomManager,
) {
    /* setters for clientside room stats
     * Is there a one liner? probabaly. idk im lazy */
    let (new_width, new_height) = map_packet.size;
    room_manager.width = new_width;
    room_manager.height = new_height;

    let map_array = &map_packet.matrix;
    let mut horizontal = -(new_width / 2.0) + (TILE_SIZE as f32 / 2.0);
    let mut vertical = -(new_height / 2.0) + (TILE_SIZE as f32 / 2.0);
    /* ye ol sliding room problem. Kinda funny, never
     * reset so we made a slinky */
    let og_horizontal = horizontal;
    let og_vertical = vertical;
    let og_vertical = vertical;
    let z_index = map_packet.z;



    /* get rid of room */
    for tile in room_query.iter_mut()
    {
        commands.entity(tile).despawn();
    }

    info!("starting ({}, {})",horizontal, vertical);
    for a in 0..map_array.len() {
        for b in 0..map_array[0].len() {
            let val = map_array[a][b];
            info!("[{}][{}] = ({}, {})",a,b,horizontal,vertical);
            match val {
                0 => commands.spawn((SpriteBundle {
                    texture: asset_server
                        .load("tiles/cobblestone_floor/cobblestone_floor.png")
                        .clone(),
                    transform: Transform::from_xyz(horizontal, vertical, 0.0),
                    ..default() },Background,Room,)),
                1 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/walls/left_wall.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, z_index),
                    ..default() },Wall,Room,)),
                2 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/walls/right_wall.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, z_index),
                    ..default() },Wall,Room,)),
/*poton */      3 => {commands.spawn(( SpriteBundle {
                    texture: asset_server.load("items/potion.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, z_index),
                    ..default() },Potion,Room,));
                    commands.spawn((SpriteBundle {
                    texture: asset_server
                        .load("tiles/cobblestone_floor/cobblestone_floor.png")
                        .clone(),
                    transform: Transform::from_xyz(horizontal, vertical, z_index),
                    ..default() },Background,Room,))}
                4 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/solid_floor/solid_floor.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, z_index),
                    ..default() },ClientDoor{door_type: DoorType::Left,},Room,)),
                5 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/solid_floor/solid_floor.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, z_index),
                    ..default() },ClientDoor{door_type: DoorType::Right,},Room,)),
                6 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/solid_floor/solid_floor.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, z_index),
                    ..default() },ClientDoor{door_type: DoorType::Top,},Room,)),
                7 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/solid_floor/solid_floor.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, z_index),
                    ..default() },ClientDoor{door_type: DoorType::Bottom,},Room,)),
                8 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/walls/north_wall.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, z_index),
                    ..default() },Wall,Room,)),
                9 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/walls/bottom_wall.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, z_index),
                    ..default() },Wall,Room,)),
                10 => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/1x2_pot.png").clone(),
                    transform: Transform::from_xyz(horizontal, vertical, z_index),
                    ..default() },Pot::new(),Room,)),
                _ => commands.spawn(( SpriteBundle {
                    texture: asset_server.load("tiles/walls/bottom_wall.png").clone(),
                    transform: Transform::from_xyz(-10000.0, -10000.0, z_index),
                    ..default() },Wall,Room,)),
            };
            vertical += TILE_SIZE as f32;
        }
        horizontal += TILE_SIZE as f32;
        vertical = og_vertical;
    }
}

pub fn send_player(
    player_q: Query<
        (
            &NetworkId,
            &Velocity,
            &Transform,
            &Health,
            &Crouch,
            &Roll,
            &Sprint,
            &Attack,
        ),
        With<Player>,
    >,
    seq: Res<Sequence>,
    clientid: Res<ClientId>,
    udp: Res<UDP>
){
    'playa: for (id, velo, trans, heal, crouch, roll, sprint, attack) in player_q.iter(){
        if id.id == clientid.id{
            /* we don't want to send if we arent doing anything, no use...
             * same goes for server!!!!! */
            if velo.velocity.y == 0. && velo.velocity.x == 0. {
                continue 'playa;
            }
            let to_send = ClientPacket::PlayerPacket(PlayerSendable {
                head: Header {
                    network_id: id.id,
                    sequence: seq.clone(),
                },
                transform: trans.clone(),
                velocity: velo.velocity,
                health: heal.clone(),
                crouch: crouch.crouching,
                attack: attack.attacking,
                roll: roll.rolling,
                sprint: sprint.sprinting,
            });
            let mut serializer = flexbuffers::FlexbufferSerializer::new();
            to_send.serialize(&mut serializer).unwrap();
            let packet: &[u8] = serializer.view();
            udp.socket.send_to(&packet, SERVER_ADR).unwrap();
        }
    }
}


fn send_enemies_killed(
    mut commands: Commands,
    mut enemy_q: Query<(&Transform, &EnemyId), With<Enemy>>,
    mut packets: ResMut<ClientPacketQueue>,
    seq: Res<Sequence>,
    clientid: Res<ClientId>,
    udp: Res<UDP>,
){
    
}

/* interpolate player/enemy

use Res<ClientId> to find players that are not us. From there, use the
PastStateQueue to get the average movement yk. We gotta do some queue squashing to
make sure we don't have a bunch of repeats

I dont think you need to worry about sequence, just use the implied order
from our vec.push()



can do same with enemy but a paststatequeue needs creted for their stuff yk yk yk*/








/* STUPID. we sorta need a room to look at..... we may not get the before we go to
 * game loop. so, here we have a listen, who will listen, until it gets a room 
 * hacky as fuck. maybe not idk.... maybe a ack would be nice we could potentially
 * miss the first one and just be stuck.......*/
 pub fn init_listen(
    udp: Res<UDP>,
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
            &mut Visibility
        ),
        With<Player>,
    >,
    mut enemy_q: Query<(Entity, &mut Transform, &mut EnemyMovement, &mut EnemyId, &mut EnemyPastStateQueue, &mut Health),(With<Enemy>, Without<Player>)>,//TODO make ecs
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    mut client_id: ResMut<ClientId>,
    mut sequence: ResMut<Sequence>,
    mut room_query: Query<Entity, With<Room>>,
    mut room_manager: ResMut<ClientRoomManager>,
    mut idstore: ResMut<'_, EnemyIdChecker>
) {
    //info!("Listening!!!");
    loop{
    /* to hold msg */
    let mut buf: [u8; 10000] = [0; 10000];
    /* grab dat shit */
    let packet = udp.socket.recv_from(&mut buf);
    match packet {
        Err(_e) => {continue},//usually return, but for this fn we want to stick around
        _ => {}
    }
    let (amt, src) = packet.unwrap();

    /* trim trailing 0s */
    let packet = &buf[..amt];

    /* deserialize and turn into a ServerPacket */
    let deserializer = flexbuffers::Reader::get_root(packet).unwrap();
    let rec_struct: ServerPacket = ServerPacket::deserialize(deserializer).unwrap();

    /* match to figure out. MAKE SURE WE SEQUENCE::ASSIGN() on every
     * packet!! is essential for lamportaging */
    match rec_struct {
        ServerPacket::IdPacket(id_packet) => {
            info!("matching idpacket");
            recv_id(&id_packet, &mut sequence, &mut client_id);
            sequence.assign(&id_packet.head.sequence);
        }
        ServerPacket::PlayerPacket(player_packet) => {
            /*  gahhhh sequence borrow checker is giving me hell */
            /* if we encounter porblems, it's herer fs */ 
            receive_player_packet( &mut commands, &mut players, &asset_server, &player_packet, &mut texture_atlases, src);
            sequence.assign(&player_packet.head.sequence);
        }
        ServerPacket::MapPacket(map_packet) => {
            info!("Matching Map Struct");
            receive_map_packet(&mut commands, &asset_server, &map_packet, &mut room_query, &mut room_manager);
            sequence.assign(&map_packet.head.sequence);
            return;
        }
        ServerPacket::EnemyPacket(enemy_packet) => {
           // info!{"Matching Enemy Struct"};
            recv_enemy(&enemy_packet, &mut commands, &mut enemy_q, &asset_server, &mut texture_atlases, &mut idstore);
            sequence.assign(&enemy_packet.head.sequence);
        }
        ServerPacket::DespawnPacket(despawn_packet) => {
            //info!("Matching Despawn Packet");
            // despawn_enemy(&mut commands, &despawn_packet.enemy_id);
        }
        _ => info!("Got some weirdness")
    }
}// stupid loop
}