use bevy::prelude::*;

#[derive(Component)]
pub struct CarnageBar{
    pub stealth: f32,
    pub carnage: f32
}

pub fn client_spawn_carnage_bar(
    commands: &mut Commands,
    asset_server: & AssetServer
){
    /* carnage bar spawn */
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(320.0),
                height: Val::Px(32.0),
                margin: UiRect{ top: Val::VMin(2.5), left: Val::VMax(37.5),..default()},
                ..default()
            },
            z_index: ZIndex::Global(999),
            ..default()
        },
        UiImage::new(asset_server.load("ui/carnage_bar_border.png")),
        CarnageBar{stealth: 0., carnage: 0.}
    ));

    // commands.spawn((
    //     NodeBundle{
    //         style: Style {

    //         }
    //     }
    // ))
}
