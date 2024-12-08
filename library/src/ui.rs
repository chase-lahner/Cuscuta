use bevy::prelude::*;
use bevy::color::palettes::css::{SEA_GREEN, RED, MAROON, BLACK};
use serde::{Serialize, Deserialize};

use crate::cuscuta_resources::{ClientId, Health, TILE_SIZE};
use crate::player::{NetworkId, Player, ItemStatus};


/* stupud to do math like this but basically window is  */
const CARNAGE_BAR_LEFT: f32 = 3.0;
const CARNAGE_BAR_MIDDLE: f32 = CARNAGE_BAR_LEFT + 12.; 
const CARNAGE_BAR_RIGHT: f32 = CARNAGE_BAR_MIDDLE + 12.;

#[derive(Event)]
pub struct CarnageChangeEvent(pub bool);

#[derive(Component, Serialize, Deserialize, Debug, Clone)]
pub struct CarnageBar{
    pub stealth: f32,
    pub carnage: f32
}

#[derive(Component)]
pub struct PotionIcon;

impl CarnageBar{
    pub fn new() -> Self {
        Self{
            stealth: 50.,
            carnage: 50.
        }
    }

    pub fn up_stealth(&mut self, amount:f32){
        self.stealth = (self.stealth + amount).clamp(0., 100.);
        self.carnage = 100. - self.stealth;
    }

    pub fn down_stealth(&mut self, amount:f32){
        self.stealth = (self.stealth - amount).clamp(0. , 100.);
        self.carnage = 100. - self.stealth;
    }
    pub fn up_carnage(&mut self, amount:f32){
        self.carnage = (self.carnage + amount).clamp(0. , 100.);
        self.stealth = 100. - self.carnage
    }
    
    pub fn down_carnage(&mut self, amount:f32){
        self.carnage = (self.carnage - amount).clamp(0. , 100.);
        self.stealth = 100. - self.carnage;
    }

    pub fn get_carnage(&mut self) -> f32 {
       return self.carnage;
    }

    pub fn get_stealth(&mut self) -> f32 {
        return self.stealth;
    }

    pub fn get_overall_percentage(&mut self) -> f32 {
        return self.stealth / 100.;
    }
}

#[derive(Component)]
pub struct Red;

#[derive(Component)]
pub struct Green;


pub fn client_spawn_ui(
    commands: &mut Commands,
    asset_server: & AssetServer
){

    /* carnage bar border */
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(320.0),
                height: Val::Px(TILE_SIZE as f32),
                margin: UiRect{ top: Val::VMin(83.), left: Val::VMax(CARNAGE_BAR_LEFT),..default()},
                ..default()
            },
            z_index: ZIndex::Global(999),
            ..default()
        },
        UiImage::new(asset_server.load("ui/carnage_bar_border.png")),
        CarnageBar{stealth: 50., carnage: 50.}
    ));

    // CARNAGE RED
    commands.spawn(
        (NodeBundle{
            style: Style {
                width: Val::Px(1.),
                height: Val::Px(32.),
                margin: UiRect {
                    top: Val::VMin(83.), left: Val::Px(198.),
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


    // CARNAGE GREEN
    commands.spawn(
        (NodeBundle{
            style: Style {
                width: Val::Px(1.),
                height: Val::Px(32.),
                margin: UiRect {
                    top: Val::VMin(83.), left: Val::Px(48.), // 2.5, left
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

    // HEALTH BAR RED
    commands.spawn(
        (NodeBundle{
            style: Style {
                width: Val::Px(150.),
                height: Val::Px(32.),
                margin: UiRect {
                    top: Val::VMin(90.), left: Val::VMax(3.), // 2.5, 0
                    .. default()
                },
                ..default()
            },
            z_index: ZIndex::Global(5),
            ..default()
        },
        UiImage::solid_color(Color::from(MAROON)),
        Health::new_init(),
    ));

    // HEALTH BAR BLACk
    commands.spawn(
        (NodeBundle{
            style: Style {
                width: Val::Px(150.),
                height: Val::Px(32.),
                margin: UiRect {
                    top: Val::VMin(90.), left: Val::VMax(3.), // 2.5, 0
                    .. default()
                },
                ..default()
            },
            z_index: ZIndex::Global(3),
            ..default()
        },
        UiImage::solid_color(Color::from(BLACK)),
    ));


    // POTION ICON
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(32.0),  
                height: Val::Px(32.0), 
                margin: UiRect {
                    top: Val::VMin(90.),   
                    left: Val::VMax(15.0),  
                    ..default()
                },
                ..default()
            },
            z_index: ZIndex::Global(999),
            ..default()
        },
        UiImage::new(asset_server.load("ui/potion_icon_empty.png")),
        PotionIcon,
    ));
}


pub fn update_ui_elements(
    mut red_q: Query<&mut Style, (With<Red>, Without<Green>, Without<Health>, Without<CarnageBar>)>,
    mut green_q: Query<&mut Style, (With<Green>, Without<Red>, Without<Health>, Without<CarnageBar>)>,
    mut health_bar : Query<&mut Style, (With<Health>, Without<Green>, Without<Red>, Without<CarnageBar>)>,
    mut potion_icon_q: Query<&mut UiImage, With<PotionIcon>>,
    player_q : Query<(&Health, &ItemStatus, &NetworkId), With<Player>>, 
    mut carnage_q: Query<&CarnageBar>,
    asset_server: Res<AssetServer>,
    client_id : Res<ClientId>,
){
    let carnage = carnage_q.single_mut();
    let mut green = green_q.single_mut();
    let mut red = red_q.single_mut();
    let mut healthy = health_bar.single_mut();

    red.width = Val::Px(carnage.carnage * 3. );
    green.width = Val::Px(carnage.stealth * 3. );
    red.margin.left=Val::Px(48.+carnage.stealth * 3.);

    let full_health_width = 150.0;

    for (health, potion_status, id) in player_q.iter(){
        if id.id == client_id.id{
            let health_ratio = health.current / health.max;
            healthy.width = Val::Px(full_health_width * health_ratio);

            // update health icon
            if let Ok(mut potion_icon) = potion_icon_q.get_single_mut() {
                if potion_status.has_potion {
                    potion_icon.texture = asset_server.load("ui/potion_icon_ready.png");
                } else {
                    potion_icon.texture = asset_server.load("ui/potion_icon_empty.png");
                }
            } 
        }
    }
}