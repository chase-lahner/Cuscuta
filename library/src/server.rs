use std::net::{SocketAddr, UdpSocket};

use bevy::{prelude::*, tasks::IoTaskPool};
use flexbuffers::FlexbufferSerializer;
use network::*;
use serde::{Deserialize, Serialize};

use crate::{cuscuta_resources::{self, FlexSerializer, Health, PlayerCount, Velocity, GET_PLAYER_ID_CODE, PLAYER_DATA}, network, player::{Attack, Crouch, NetworkId, Player, Roll, ServerPlayerBundle, Sprint}};

/* Upon request, sends an id to client */
pub fn send_id(
    source_addr : SocketAddr,
    server_socket: &UdpSocket, 
    n_p: &mut PlayerCount,
    mut commands: Commands
) {
    /* assign id, update player count */
    let player_id: u8 = 255 - n_p.count;
    n_p.count += 1;
    commands.spawn(NetworkId{id:player_id,addr: source_addr});

    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    /* lil baby struct to serialize */
    let to_send = IdPacket{ id: player_id};
    to_send.serialize(  &mut serializer ).unwrap();

    /* once serialized, we throw our opcode on the end */
    let opcode: &[u8] = std::slice::from_ref(&GET_PLAYER_ID_CODE);
    let packet_vec  = append_opcode(serializer.view(), opcode);
    let packet: &[u8] = &(&packet_vec);


    /* Send it on over! */
    server_socket.send_to(&packet, source_addr).unwrap();
}

/* Server side listener for packets,  */
pub fn listen(
    udp: Res<UDP>,
    mut commands: Commands,
    mut players: Query<(&mut Velocity, &mut Transform, &mut NetworkId), With<Player>>,
    mut n_p: ResMut<PlayerCount>,
) {
    info!("Listening!!");

    // let task_pool = IoTaskPool::get();
    // let task = task_pool.spawn(async move {

    // })
    /* to hold msg */
    let mut buf: [u8; 1024] = [0;1024];
    let (amt, src) = udp.socket.recv_from(&mut buf).unwrap();
    info!("Read!");
    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    
    /* when we serialize, we throw our opcode on the end, so we know how to
    * de-serialize... jank? maybe.  */
    let opcode = buf[amt -1];
    
    /* trim trailing 0s */
    let t_buf = &buf[..amt-1];

    
    info!("{:?}",buf);
    info!("opcode::{}",&opcode);
    match opcode{
        cuscuta_resources::GET_PLAYER_ID_CODE => {
            info!("sending id to client");
            send_id(src, &udp.socket, n_p.as_mut(), commands)},
        cuscuta_resources::PLAYER_DATA =>
            update_player_state(src, players, t_buf, commands),
        _ => 
            something()//TOTO

    };
}

/* once we have our packeet, we must use it to update
 * the player specified, there's another in client.rs*/
fn update_player_state(
    src: SocketAddr,
    /* fake query, passed from above system */
    mut players: Query<(&mut Velocity, &mut Transform, &mut NetworkId), With<Player>>,
    buf: &[u8],
    mut commands: Commands,
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
        let velo_vec = Vec2::new(player_struct.velocity_x, player_struct.velocity_y);
        commands.spawn(ServerPlayerBundle{
            velo: Velocity::from(velo_vec),
            transform:
                Transform::from_xyz(player_struct.transform_x, player_struct.transform_y, 0.),
            id: NetworkId{
                id: player_struct.id,
                addr: src},
            player: Player,   
            health: Health::new(),
            rolling: Roll::new(),
            crouching: Crouch::new(),
            sprinting: Sprint::new(),
            attacking: Attack::new(),
    });
    }
}

/* Transforms current player state into u8 array that
 * we can then send across the wire to be deserialized once it arrives */
 pub fn send_player(
    player : Query<(&Transform, &Velocity, &NetworkId ), With<Player>>,
    socket : Res<UDP>,
)
{
    /* Deconstruct out Query. SHould be client side so we can do single */
    for (t, v, i)  in player.iter(){
        let outgoing_state = PlayerPacket { 
            id: i.id,
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

        info!("length of player packet:{:?}", packet);
        socket.socket.send_to(&packet, i.addr).unwrap();
    }
}

fn something(){}
