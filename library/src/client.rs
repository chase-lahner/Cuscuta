use std::net::SocketAddr;

use bevy::prelude::*;
use serde::Deserialize;

use crate::enemies::{ClientEnemy, Enemy, EnemyKind, EnemyMovement};
use crate::cuscuta_resources::*;
use crate::network::{
    client_seq_update, ClientPacket, ClientPacketQueue, EnemyS2C, Header, IdPacket, PlayerS2C, Sequence, ServerPacket, Timestamp, UDP
};
use crate::player::*;


pub fn send_packets(
udp: Res<UDP>,
mut packets: ResMut<ClientPacketQueue>
){
    for pack in &packets.packets{

    }

}

/* server send us an id so we can know we are we yk */
pub fn recv_id(
    ds_struct: &IdPacket,
    mut sequence: ResMut<Sequence>,
    mut id: ResMut<ClientId>
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
    mut packet_q: ResMut<ClientPacketQueue>
) {
    /* make an idpacket, server knows if it receives one of these
     * what we really want */
    let id_packet = ClientPacket::IdPacket(IdPacket {
        head: Header {
            network_id: 0,
            sequence: Sequence::new(0),
        }});
    /* plop into 'to-send' list */
    packet_q.packets.push(id_packet);//sneed
}

/* Parse player input, and apply it*/
pub fn gather_input(
    mut player: Query<(&NetworkId, &mut InputQueue), With<Player>>,
    client_id: Res<ClientId>,
    mut sequence: ResMut<Sequence>,
    input: Res<ButtonInput<KeyCode>>,
) {
    /* Deconstruct out Query. */
    for (i, mut q) in player.iter_mut() {
        /* are we on us????? */
        if i.id == client_id.id {
            /* create a vec of keypresses as our 'this tick'
             * inputs */
            let mut keys: Vec<KeyCode> = vec![];
            for key in input.get_pressed() {
                keys.push(*key);
            }

            /* add to input queue @ timestamp */

            /* if last element in InputQueue has same sequence#, append
             * lists together so we have 1 per stamp.
             * This can happen because we gather_input() at
             * an unfixed rate, however the game progresess it 
             * progresses, while we only send/increment seq
             * on fixedupdate, which is when we send. It's possible to have
             * two++ gathers per send, we must make sure we are aware of this
             * possibility. Maybe there's a better way to handle it, i'm
             * down 2 adjust 
             * 
             * LONG STORY SHORT WE NEED CLIENT/SERVER CONSISTENCY,
             * SO WE MUST PREEDICT HOW THE SERVER WILL. admittedly,
             * this loses us some accuracy in movement. we will survive, currently
             * @ 60hz that's not very human noticable. This means we will
             * have descepancies in intantaneous prediction, the time @ which
             * you press UP within the frame does have an effect, 
             * although negligible (@max I think like 15ms for 64hz but then half
             * that fo 7.5ms ohhh no whatever shall we do {GAH SUBTICK [i'd be down]}).
             * Our reprediction should be pretty tho, as long as the server
             * isn't missing out on packets, as any enforced state we should
             * have already propely predicted!! Ideally we want the InputQueue
             * of client and servers snapshot of client @ time t to be the same 
             * - rorto */
            
            let len = q.q.len();
            if q.q.get_mut(len).unwrap().0 == sequence.get(){
                let (q_timey, mut q_keys) = q.q.pop().unwrap();
                q_keys.append(&mut keys);
                q_keys.dedup();
                q.q.push((q_timey, q_keys));
            }else{
                q.q.push((sequence.get(), keys));
            }

            /* so now it's in our input queue!!! what do we want to do from here?
             * 1. We need to make sure we send this tick's input next time we 
             *      do a fixed update...
             *  One thing I am thinking about as a potential error right now is
             * what if we have an inputqueue made above^, and then end up recieving a 
             * packet from the server with a higher sequence number. This would cause
             * our sequence number to update to the higher value, throwing off our input
             * queue. Maybe we should keep this in mind when changing the sequence number, so
             * we have the ability to adjust within our input queue right now. We could also
             * put in a "deprecated sequence" values, that we use to pair against good ones.
             * Pair or maybe even just immediately change (i want to immediate teebs, more
             * reasoning around Sequence impl) */

            /* TODODO what?!? do we want another list? just query the inputqueue for
             * sequence.get()?? Think we can really just do the latter. also key that
             * we update player state right after this input gather happens.*/

            
        }
    }
}

/* client listening function. Takes in a packet, deserializes it
 * into a ServerPacket (client here so from server). 
 * Then we match against the packet
 * to figure out what kind it is, passing to another function to properly handle.
 * Important to note that we will Sequence::assign() on every packet
 * within the match, to make sure we Lamport corectly */
pub fn listen(
    udp: Res<UDP>,
    commands: Commands,
    players_q: Query<
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
            &mut InputQueue,
        ),
        With<Player>,
    >,
    mut enemy_q: Query<&mut ClientEnemy>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    res_id: ResMut<ClientId>,
    mut sequence: ResMut<Sequence>,
    mut packets: ResMut<ClientPacketQueue>
) {
    //info!("Listening!!!");
    /* to hold msg */
    let mut buf: [u8; 1024] = [0; 1024];
    /* grab dat shit */
    let packet = udp.socket.recv_from(&mut buf);
    match packet {
        Err(_e) => return,
        _ => info!("read packet!")}
    let (amt, src) = packet.unwrap();
  
    /* trim trailing 0s */
    let packet = &buf[..amt];

    /* deserialize and turn into a ServerPacket */
    let deserializer = flexbuffers::Reader::get_root(packet).unwrap();
    let rec_struct: ServerPacket = ServerPacket::deserialize(deserializer).unwrap();

    /* we need the inputqueue of US, OUR PLAYER for an update when we recv
     * a new sequence number. Might as well find that now to not pass an ugly
     * Query */
    let mut inputs: &mut InputQueue = &mut InputQueue::new();
    for (v,t,p,h,c,r,s,a,id,iq) in players_q.iter_mut(){
        if id.id == res_id.id{
            inputs = iq.into_inner();2
        }
    }
    /* match to figure out. MAKE SURE WE SEQUENCE::ASSIGN() on every
     * packet!! is essential for lamportaging */
    match rec_struct {
        ServerPacket::IdPacket(id_packet) => {
            recv_id(&id_packet, res_id);
            sequence.assign(&id_packet.head.sequence);
            client_seq_update(&id_packet.head.sequence, sequence, inputs, packets);
        }
        ServerPacket::PlayerPacket(player_packet) => {
            info!("Matching Player Struct");
            receive_player_packet(commands, players_q, &asset_server, &player_packet, &mut texture_atlases, id, src);
            sequence.assign(&player_packet.head.sequence);
            client_seq_update(&player_packet.head.sequence, sequence, inputs, packets);
        }
        ServerPacket::MapPacket(map_packet) => {
            info!("Matching Map Struct");
            receive_map_packet(commands, &asset_server, map_packet.matrix);
            sequence.assign(&map_packet.head.sequence);
            client_seq_update(&map_packet.head.sequence, sequence, inputs, packets);
        }
        ServerPacket::EnemyPacket(enemy_packet) => {
            recv_enemy(&enemy_packet,commands,enemy_q,asset_server,&mut texture_atlases);
            sequence.assign(&enemy_packet.head.sequence);
            client_seq_update(&enemy_packet.head.sequence, sequence, inputs, packets);
        }
    }
}

fn receive_player_packet(
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
            &mut InputQueue,
        ),
        With<Player>,
    >,
    asset_server: &Res<AssetServer>,
    saranpack: &PlayerS2C,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    mut us: ResMut<ClientId>,
    source_ip: SocketAddr
) {
    let mut found = false;
    for (v, t, p, h, c, r, s, a, id, iq) in players.iter_mut() {
        if id.id == us.id {
            found = true;
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
    mut commands: Commands,
    mut enemy_q: Query<&mut ClientEnemy>,//TODO make ecs
    asset_server: Res<AssetServer>,
    tex_atlas: &mut ResMut<Assets<TextureAtlasLayout>>
){
    let mut found = false;
    for mut enemy in enemy_q.iter_mut(){
        if pack.enemytype.get_id() == enemy.id.id{ 
            enemy.movement.push(pack.movement.clone());
            found = true;
            break;
        }
    }

    if !found {
        let the_enemy: &Enemy;
        match &pack.enemytype.kind{
            EnemyKind::Skeleton(enemy) => the_enemy = enemy,
            EnemyKind::BerryRat(enemy) => the_enemy = enemy,
            EnemyKind::Ninja(enemy) => the_enemy = enemy,
            EnemyKind::SplatMonkey(enemy) => the_enemy = enemy,
            EnemyKind::Boss(enemy) => the_enemy = enemy,
        };

        let enemy_layout = 
        TextureAtlasLayout::from_grid(
            UVec2::splat(the_enemy.size),
            the_enemy.sprite_column,
            the_enemy.sprite_row,
            None,
            None
        );

        let enemy_layout_handle = tex_atlas.add(enemy_layout);

        let mut vec: Vec<EnemyMovement> = Vec::new();
        vec.push(pack.movement.clone());
        let x = pack.movement.direction.x;
        let y = pack.movement.direction.y;
        commands.spawn(
            (SpriteBundle{
                transform: Transform::from_xyz(x, y, 900.),
                texture: asset_server.load(the_enemy.filepath.clone()),
                
                ..default()
            },
            TextureAtlas{
                layout: enemy_layout_handle,
                index:0
            },
            ClientEnemy{
                id: pack.enemytype.clone(),
                movement: vec,
            })

        );

    };
}



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
