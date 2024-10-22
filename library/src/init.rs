use std::{net::{ Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket}, str::FromStr};

use bevy::{prelude::*, tasks::IoTaskPool};
use flexbuffers::FlexbufferSerializer;

use crate::{camera::spawn_camera, carnage::*, cuscuta_resources::{self, AddressList, ClientId}, network::*, player::*, room_gen::*};

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
    //let socket = SocketAddrV4::new(Ipv4Addr::new(sendable[0], sendable[1],sendable[2],sendable[3]), split_u16);


    // spawn the starting room & next room
    spawn_start_room(&mut commands, &asset_server, &mut room_manager);

    commands.insert_resource(ClientId{id:0});

    // spawn camera
    spawn_camera(&mut commands, &asset_server);

    client_spawn_carnage_bar(&mut commands, &asset_server);
    /* spawn pot to play with */
    client_spawn_pot(&mut commands, &asset_server);
    // spawn player, id 0 because it will be set later on
    client_spawn_user_player(&mut commands, &asset_server, &mut texture_atlases, 0);
}


pub fn server_setup(
    mut commands: Commands
){
    info!("entered setup");
    let socket = UdpSocket::bind(cuscuta_resources::SERVER_ADR).unwrap();
    socket.set_nonblocking(true).unwrap();
    commands.insert_resource(UDP{socket:socket});
    
    commands.insert_resource(AddressList::new());
    info!("done setup");
}
