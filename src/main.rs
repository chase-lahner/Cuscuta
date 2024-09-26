use bevy::{prelude::*, window::PresentMode};

#[derive(Component, Deref, DerefMut)]
struct PopupTimer(Timer);

const TITLE: &str = "Cuscuta Demo";
const WIN_W: f32 = 1280.;
const WIN_H: f32 = 720.;

const PLAYER_SPEED: f32 = 500.;
const ACCEL_RATE: f32 = 5000.;

const TILE_SIZE: u32 = 100;

const LEVEL_LEN: f32 = 5000.;

const ANIM_TIME: f32 = 0.2;

#[derive(Component)]
struct Player;

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
struct AnimationFrameCount(usize);

#[derive(Component)]
struct Brick;

#[derive(Component)]
struct Background;

#[derive(Component)]
struct Velocity {
    velocity: Vec2,
}

impl Velocity {
    fn new() -> Self {
        Self {
            velocity: Vec2::splat(0.),
        }
    }
}

impl From<Vec2> for Velocity {
    fn from(velocity: Vec2) -> Self {
        Self { velocity }
    }
}





fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {// need window!
                title: "Credit Sequence!".into(),
                present_mode: PresentMode::Fifo,
                ..default() // Name and present mode all we need for now
             }),
             ..default()
         }))
         .add_systems(Startup,setup)
         .add_systems(Update, move_player)
         //.add_systems(Update, animate_player.after(move_player))
         //.add_systems(Update, move_camera.after(animate_player))
         .run();
}

fn setup(

){}

fn move_player(){}

fn animate_player(){}

fn move_camera() {}
