use std::net::UdpSocket;

use bevy::prelude::*;

use crate::{camera::spawn_camera, cuscuta_resources::{self, AddressList, ClientId, PlayerCount, TICKS_PER_SECOND}, network::*, player::*, room_gen::*, ui::client_spawn_ui};

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
) {


    // spawn the starting room & next room
    spawn_start_room(&mut commands, &asset_server, &mut room_manager);

    /* initialize to 0. works for single player!
     * will be assigned when given one from server */
    commands.insert_resource(ClientId{id:0});
    
    /* sequence number! gives us a lil ordering */
    commands.insert_resource(Sequence::new());

    // spawn camera
    spawn_camera(&mut commands, &asset_server);

    client_spawn_ui(&mut commands, &asset_server);
    /* spawn pot to play with */
    client_spawn_pot(&mut commands, &asset_server, &mut texture_atlases);
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
    /* lilk ordering action */
    commands.insert_resource(Sequence::new());
    /* tha rate ehhh this could need to be called before init idk*/
    commands.insert_resource(Time::<Fixed>::from_hz(TICKS_PER_SECOND));
    /* bum ass no friend ass lonely ahh */
    commands.insert_resource(PlayerCount{count:0});
    info!("done setup");
}
