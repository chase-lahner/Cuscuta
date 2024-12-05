use std::net::UdpSocket;

use bevy::prelude::*;


use crate::{camera::spawn_camera, cuscuta_resources::{self, AddressList, ClientId, EnemiesToKill, PlayerCount, TICKS_PER_SECOND}, enemies::{EnemyId, EnemyKind}, markov_chains::*, network::*, room_gen::*, ui::client_spawn_ui
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
    last_attribute_array: ResMut<LastAttributeArray>, // LastAttributeArray as a mutable resource
) {


    // spawn the starting room & next room
    spawn_start_room(&mut commands, &asset_server, &mut room_manager, last_attribute_array);

    /* initialize to 0. works for single player!
     * will be assigned when given one from server */
    commands.insert_resource(ClientId::new());
    
    /* sequence number! gives us a lil ordering... we put 0
     * for now, which is the server's id but we will reassign
     * when we recv a packet from the server */
    commands.insert_resource(Sequence::new(0));

    // spawn camera
    spawn_camera(&mut commands);

    client_spawn_ui(&mut commands, &asset_server);
    /* spawn pot to play with */
    client_spawn_pot(&mut commands, &asset_server, &mut texture_atlases);

    commands.insert_resource(ClientPacketQueue::new());
    // spawn player, id 0 because it will be set later on
   //  client_spawn_other_player_new(&mut commands, &asset_server, &mut texture_atlases, 0);
   // WHAT DO WE WANT TO DO WITH THIS?
}


pub fn server_setup(
    mut commands: Commands
){
    info!("entered setup");
    /* send from where ?*/
    let socket = UdpSocket::bind(cuscuta_resources::SERVER_ADR).unwrap();
    /* fuck you soket. */
    socket.set_nonblocking(true).unwrap();
    commands.insert_resource(UDP{socket:socket});

    
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
    info!("done setup");
}
