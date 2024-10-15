use bevy::prelude::*;

#[derive(Component)]
pub struct carnage_bar{
    pub stealth: u8,
    pub carnage: u8
}

pub fn spawn_carnage_bar(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>
){
    let carnage_handle: Handle<Image> = asset_server.load("carnage/carnage_bar_border.png");

}
