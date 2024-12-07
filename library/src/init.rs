use std::net::UdpSocket;

use bevy::prelude::*;
use serde::Deserialize;


use crate::client::*;
use crate::cuscuta_resources::*;
use crate::player::{Attack, Crouch, NetworkId, Player, Roll, Sprint};
use crate::ui::CarnageBar;
use crate::{camera::spawn_camera, cuscuta_resources::{self, AddressList, ClientId, EnemiesToKill, PlayerCount, TICKS_PER_SECOND}, enemies::{EnemyId, EnemyKind}, markov_chains::*, network::*, room_gen::{self, *}, ui::client_spawn_ui

};

pub fn ip_setup(
    mut commands: Commands
)
{
    let binding = get_ip_addr(); // call fn in network and to get ip from user
    let ip_string = binding.trim(); // trim extra whitespce

     /* initializes our networking socket */
     let socket = UdpSocket::bind(ip_string).unwrap(); // string has a toSocketAddr implementation so this works
     socket.set_nonblocking(true).unwrap();

     commands.insert_resource(UDP {socket: socket}); // insert socket resource
}


pub fn client_setup(
    mut commands: Commands, // to spawn in entities
    asset_server: Res<AssetServer>, // to access images
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>, // used in animation
    mut room_manager: ResMut<RoomManager>,
    mut last_attribute_array: ResMut<LastAttributeArray>,
    room_config: Res<RoomConfig>,
) {


    // spawn the starting room & next room
    spawn_start_room(&mut commands, &asset_server, &mut room_manager, 0.2, &mut last_attribute_array, &room_config, &mut texture_atlases);

    /* initialize to 0. works for single player!
     * will be assigned when given one from server */
    commands.insert_resource(ClientId::new());
    
    /* sequence number! gives us a lil ordering... we put 0
     * for now, which is the server's id but we will reassign
     * when we recv a packet from the server */
    commands.insert_resource(Sequence::new(0));

    commands.insert_resource(PlayerDeathTimer::new());

    // spawn camera
    spawn_camera(&mut commands);

    client_spawn_ui(&mut commands, &asset_server);
    /* spawn pot to play with */
    //client_spawn_pot(&mut commands, &asset_server, &mut texture_atlases);

   // commands.insert_resource(ClientRoomManager::new());
    
}


pub fn server_setup(
    mut commands: Commands,
){
    info!("entered setup");
    /* send from where ?*/
    let socket = UdpSocket::bind(cuscuta_resources::SERVER_ADR).unwrap();
    /* fuck you soket. */
    socket.set_nonblocking(true).unwrap();
    commands.insert_resource(UDP{socket:socket});

    
    commands.insert_resource(RoomConfig::new());
    /* who we connected to again?*/
    commands.insert_resource(AddressList::new());
    /* lilk ordering action. 0 is server's Sequence index/id */
    commands.insert_resource(Sequence::new(0));
    /* tha rate ehhh this could need to be called before init idk*/
    commands.insert_resource(Time::<Fixed>::from_hz(TICKS_PER_SECOND));
    /* bum ass no friend ass lonely ahh */
    commands.insert_resource(PlayerCount{count:0});
    /* to hold mid frame packeets, sent every tick */
    commands.insert_resource(ServerPacketQueue::new());

    commands.insert_resource(EnemiesToKill::new());

    commands.insert_resource(EnemyId::new(0, EnemyKind::skeleton()));

    commands.spawn((CarnageBar::new()));
    
    let mut room_manager = RoomManager::new();
    let mut last_attribute_array = LastAttributeArray::new();


    spawn_start_room(&mut commands, &mut room_manager, &mut last_attribute_array, 0.);
    commands.insert_resource(room_manager);
    commands.insert_resource(last_attribute_array);

    

    info!("done setup");
}
