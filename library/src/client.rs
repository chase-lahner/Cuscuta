use std::net::SocketAddr;
use std::ops::Deref;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::enemies::{ClientEnemy, Enemy, EnemyKind, EnemyMovement};
use crate::cuscuta_resources::*;
use crate::network::{
    client_seq_update, ClientPacket, ClientPacketQueue, EnemyS2C, Header, IdPacket, PlayerSendable, Sequence, ServerPacket, Timestamp, UDP
};
use crate::player::*;


/* sends out all clientPackets from the ClientPacketQueue */
pub fn client_send_packets(
    udp: Res<UDP>,
    mut packets: ResMut<ClientPacketQueue>,
){
    /* for each packet in queue, we send to server*/
    for pack in &packets.packets{
        let mut serializer = flexbuffers::FlexbufferSerializer::new();
        pack.serialize(&mut serializer).unwrap();
        let packet: &[u8] = serializer.view();
        udp.socket.send_to(&packet, SERVER_ADR).unwrap();
    }
    /* i hope this is not fucking our code */
    packets.packets = Vec::new();

}


/* what could have beennnnn.............. goodbye client side prediction */
// /* to be called right b4 sending packets */
// pub fn pack_up_input(
//     mut packets: ResMut<ClientPacketQueue>,
//     player_q: Query<(&NetworkId, &InputQueue), With<Player>>,
//     client_id: Res<ClientId>,
//     sequence: ResMut<Sequence>
// ){
//      /* for the input queue, we check to make sure that the last item
//      * in it is the right inputs for this sequence value and send off if so */

//      /* for all players */
//      for (id, iq) in player_q.iter(){
//         /* if we are us */
//         if id.id == client_id.id{
//             /* grab last input, and check if correct seq */
//             let (seq, keys) = &iq.q[iq.q.len()];

//             let pack:ClientPacket;
//             if *seq != sequence.get(){
//                 pack = ClientPacket::PlayerPacket(
//                     PlayerSendable{
//                         head: Header::new(client_id.id, sequence.clone()),
//                         key: Vec::new(),
//                     }
//                 )
//             }else{
//                 pack = ClientPacket::PlayerPacket(
//                     PlayerC2S{
//                         head: Header::new(client_id.id, sequence.clone()),
//                         key: keys.clone(),
//                     }
//                 );
//             }
//             packets.packets.push(pack);
//         }
//     }
// }

/* server send us an id so we can know we are we yk */
pub fn recv_id(
    ds_struct: &IdPacket,
    sequence: &mut Sequence,
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
    sequence: ResMut<Sequence>,
    input: Res<ButtonInput<KeyCode>>,
) {
    /* Deconstruct out Query. */
    for (i, mut iq) in player.iter_mut() {
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
            
            let len = iq.q.len();
            if len == 0{
                iq.q.push((sequence.get(), keys));
            }
            else if iq.q.get_mut(len-1).unwrap().0 == sequence.get(){
                let (q_timey, mut q_keys) = iq.q.pop().unwrap();
                q_keys.append(&mut keys);
                q_keys.dedup();
                iq.q.push((q_timey, q_keys));
            }else{
                iq.q.push((sequence.get(), keys));
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
            &mut InputQueue,
            &mut PastStateQueue
        ),
        With<Player>,
    >,
    mut enemy_q: Query<&mut ClientEnemy>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    client_id: ResMut<ClientId>,
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

    // /*GOD todo AS FUCK. I want to grab the input queue of US... but then also
    //  * need to still be able to query later in recv_player_packeet...
    //  * damn you borrow checker!!!!! */
    // for (v,t,p,h,c,r,s,a,id,iq, psq) in players_q.iter_mut(){
    //     if id.id == client_id.id{
    //         inputs = iq.into_inner();
    //     }
    // }

    /* match to figure out. MAKE SURE WE SEQUENCE::ASSIGN() on every
     * packet!! is essential for lamportaging */
    match rec_struct {
        ServerPacket::IdPacket(id_packet) => {
            info!("matching idpacket");
            recv_id(&id_packet, &mut sequence, client_id);
            client_seq_update(&id_packet.head.sequence, sequence, packets);
        }
        ServerPacket::PlayerPacket(player_packet) => {
            info!("Matching Player Struct");
            /*  gahhhh sequence borrow checker is giving me hell */
            /* if we encounter porblems, it's herer fs */ 
            receive_player_packet(commands, players_q, &asset_server, &player_packet, &mut texture_atlases, client_id, src, &mut sequence);
            client_seq_update(&player_packet.head.sequence, sequence, packets);
        }
        ServerPacket::MapPacket(map_packet) => {
            info!("Matching Map Struct");
            receive_map_packet(commands, &asset_server, map_packet.matrix);
            client_seq_update(&map_packet.head.sequence, sequence, packets);
        }
        ServerPacket::EnemyPacket(enemy_packet) => {
            info!{"Matching Enemy Struct"};
            recv_enemy(&enemy_packet, commands, enemy_q, asset_server, &mut texture_atlases);
            client_seq_update(&enemy_packet.head.sequence, sequence, packets);
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
            &mut PastStateQueue
        ),
        With<Player>,
    >,
    asset_server: &Res<AssetServer>,
    saranpack: &PlayerSendable,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    mut us: ResMut<ClientId>,
    source_ip: SocketAddr,
    sequence: &mut ResMut<Sequence>
) {
    /* need to know if we were sent a player we don't currently have */
    let mut found_packet = false;
    let mut found_us = false;
    /* for all players, find what was sent */
    for (mut v, mut t, p, mut h, mut c, mut r, mut s, mut a, id, iq, mut psq) in players.iter_mut() {
        if id.id == saranpack.head.network_id {
            /* we found! */
            found_packet = true;
            /* set player */
            v.set(&saranpack.velocity);
            /* dam u transform */
            *t = saranpack.transform;
            h.set(&saranpack.health);
            c.set(saranpack.crouch);
            s.set(saranpack.sprint);
            a.set(saranpack.attack);
        }

        /* do we even exist?!?! */
        if id.id == us.id{
            found_us = true;
        }
    }

    /* we don't have this player!!!!!! Oh no!! whatever
     * shall we do?!?!
     * 
     * Actually a qustion. there are three scenarios here. So, when we
     * ask the server for an id, it will send us an establishing id packet,
     * and then also punt over a newly spawned player.
     * Scenario 1: We already have the userplayer, this is someone else.
     *          In this case, we need to create a new clientplayerbundle
     * Scenario 2: We recvieve userplayer, and have recieved the id packet first
     *              Id is all good, we can check against the 'us' variable of id
     * Scenario 3: We recv player **before** the id packet. lil iffy.
     *              I think the only way to know of this is to  check if clientID
     *              'us' is still @ default value (0).
     * 
     * 
     * We have a lil check above to see if we have found 'us' in our
     * query of the game world. if we did not find, we can lowk merge 
     * scenarios 2&3, with just doin a lil 'make sure we set our id'
     * in scenario 3
     * 
     * 
     * GAHHHH all the scenarios are the same we must just do some setting (to be sure that
     * shit works even if we failed to get a id packet) */
    if !found_packet {
        info!("spawning a new player: {}!", saranpack.head.network_id);
        /* ok lowk all good just do the recv_id sets if its us */
        if !found_us{
            us.id = saranpack.head.network_id;
            sequence.new_index(us.id as usize);
            /* here we set the clock values */
            sequence.assign(&saranpack.head.sequence);
        }
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
            states: PastStateQueue::new(),
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

pub fn send_player(
    player_q: Query<(&NetworkId, &Velocity, &Transform, &Health, &Crouch, &Roll, &Sprint, &Attack), With<Player>>,
    mut packet_queue: ResMut<ClientPacketQueue>,
    seq: Res<Sequence>,
    clientid: Res<ClientId>
){
    for (id, velo, trans, heal, crouch, roll, sprint, attack) in player_q.iter(){
        if id.id == clientid.id{
            let to_send = ClientPacket::PlayerPacket(PlayerSendable{
                head: Header{ network_id: id.id, sequence: seq.clone() },
                transform: trans.clone(),
                velocity: velo.velocity,
                health: heal.clone(),
                crouch: crouch.crouching,
                attack: attack.attacking,
                roll: roll.rolling,
                sprint: sprint.sprinting,
            });
            packet_queue.packets.push(to_send);
        }
    }
}


/* interpolate player/enemy

use Res<ClientId> to find players that are not us. From there, use the
PastStateQueue to get the average movement yk. We gotta do some queue squashing to
make sure we don't have a bunch of repeats

I dont think you need to worry about sequence, just use the implied order
from our vec.push()



can do same with enemy but a paststatequeue needs creted for their stuff yk yk yk*/