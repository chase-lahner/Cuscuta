use bevy::prelude::*;
use bevy::color::palettes::css::{SEA_GREEN, RED, MAROON, BLACK};

use crate::cuscuta_resources::{ClientId, Health, TILE_SIZE};
use crate::player::{NetworkId, Player};


/* stupud to do math like this but basically window is  */
const CARNAGE_BAR_LEFT: f32 = 15./40. * 100.;
const _CARNAGE_BAR_RIGHT: f32 = 25./40. * 100.;
const CARNAGE_BAR_MIDDLE: f32 = 20./40. * 100.;


#[derive(Component)]
pub struct CarnageBar{
    pub stealth: f32,
    pub carnage: f32
}

#[derive(Component)]
pub struct Red;

#[derive(Component)]
pub struct Green;


pub fn client_spawn_ui(
    commands: &mut Commands,
    asset_server: & AssetServer
){

    /* carnage bar spawn */
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(320.0),
                height: Val::Px(TILE_SIZE as f32),
                margin: UiRect{ top: Val::VMin(2.5), left: Val::VMax(CARNAGE_BAR_LEFT),..default()},
                ..default()
            },
            z_index: ZIndex::Global(999),
            ..default()
        },
        UiImage::new(asset_server.load("ui/carnage_bar_border.png")),
        CarnageBar{stealth: 0., carnage: 0.}
    ));
    /* 'carnage' aspect of bar */
    commands.spawn(
        (NodeBundle{
            style: Style {
                width: Val::Px(1.),
                height: Val::Px(32.),
                margin: UiRect {
                    top: Val::VMin(2.5), left: Val::VMax(CARNAGE_BAR_MIDDLE),
                    .. default()
                },
                ..default()
            },
            z_index: ZIndex::Global(5),
            ..default()
        },
        UiImage::solid_color(Color::from(RED)),
        Red,
    ));
    /* 'stealth' side of carnage bar */
    commands.spawn(
        (NodeBundle{
            style: Style {
                width: Val::Px(1.),
                height: Val::Px(32.),
                margin: UiRect {
                    top: Val::VMin(2.5), left: Val::VMax(CARNAGE_BAR_LEFT),
                    .. default()
                },
                ..default()
            },
            z_index: ZIndex::Global(5),
            ..default()
        },
        UiImage::solid_color(Color::from(SEA_GREEN)),
        Green,
    ));
    /* carnage bar underlay */
    commands.spawn(
        (NodeBundle{
            style: Style {
                width: Val::Px(320.0),
                height: Val::Px(TILE_SIZE as f32),
                margin: UiRect{ top: Val::VMin(2.5), left: Val::VMax(CARNAGE_BAR_LEFT),..default()},
                ..default()
            },
            z_index: ZIndex::Global(3),
            ..default()
        },
        UiImage::solid_color(Color::from(BLACK)),
    ));
    /* health bar spawn */
    commands.spawn(
        (NodeBundle{
            style: Style {
                width: Val::Px(100.),
                height: Val::Px(32.),
                margin: UiRect {
                    top: Val::VMin(2.5), left: Val::VMax(0.),
                    .. default()
                },
                ..default()
            },
            z_index: ZIndex::Global(5),
            ..default()
        },
        UiImage::solid_color(Color::from(MAROON)),
        Health::new(),
    ));
    /* Health underlay */
    commands.spawn(
        (NodeBundle{
            style: Style {
                width: Val::Px(100.),
                height: Val::Px(32.),
                margin: UiRect {
                    top: Val::VMin(2.5), left: Val::VMax(0.),
                    .. default()
                },
                ..default()
            },
            z_index: ZIndex::Global(3),
            ..default()
        },
        UiImage::solid_color(Color::from(BLACK)),
    ));

}


pub fn update_ui_elements(
    mut red_q: Query<&mut Style, (With<Red>, Without<Green>, Without<Health>, Without<CarnageBar>)>,
    mut green_q: Query<&mut Style, (With<Green>, Without<Red>, Without<Health>, Without<CarnageBar>)>,
    mut health_bar : Query<&mut Style, (With<Health>, Without<Green>, Without<Red>, Without<CarnageBar>)>,
    player_q : Query<(&Health, &NetworkId), With<Player>>,
    mut carnage_q: Query<&CarnageBar>,
    client_id : Res<ClientId>,
){
    let carnage = carnage_q.single_mut();
    let mut green = green_q.single_mut();
    let mut red = red_q.single_mut();
    let mut healthy = health_bar.single_mut();

    red.width = Val::Px(carnage.carnage * 2. );
    green.width = Val::Px(carnage.stealth * 2. );
    for (health, id) in player_q.iter(){
        if id.id == client_id.id{
            healthy.width = Val::Px(health.current)
        }
    }
}