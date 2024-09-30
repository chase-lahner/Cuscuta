mod sprint_mechanic;
mod title_sequence;

use bevy::{prelude::*, render::extract_component::ExtractComponent, window::PresentMode};

const TITLE: &str = "Cuscuta Demo";// window title
const WIN_W: f32 = 1280.;// window width
const WIN_H: f32 = 720.;// window height

const PLAYER_SPEED: f32 = 480.; 
const ACCEL_RATE: f32 = 4800.; 
const SPRINT_MULTIPLIER: f32 = 2.0;

const TILE_SIZE: u32 = 32; 

const LEVEL_W: f32 = 4800.; 

const LEVEL_H: f32 = 1600.; 

/* (0,0) is center level,          
 * this gives us easy coordinate usage */
const MAX_X: f32 = LEVEL_W / 2.;
const MAX_Y: f32 = LEVEL_H / 2.;

const ANIM_TIME: f32 = 0.2;

//const NUM_WALL: u32 = 10;

#[derive(Component)]
struct Player;// wow! it is he!

// #[derive(Component, Deref, DerefMut)]
// struct PopupTimer(Timer);// not currently in use

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);// for switching through animation frames

#[derive(Component, Deref, DerefMut)]
struct AnimationFrameCount(usize);

//struct Brick;

#[derive(Component)]
struct Background;

#[derive(Component)]

struct Wall;

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
            primary_window: Some(Window {
                // need window!
                title: TITLE.into(),
                present_mode: PresentMode::Fifo,
                ..default() // Name and present mode all we need for now
             }),
             ..default()
         }))
         .add_systems(Startup,setup)// runs once, sets up scene
         .add_systems(Update, move_player)// every frame, takes in WASD for movement
         .add_systems(Update, animate_player.after(move_player))
         .add_systems(Update, move_camera.after(animate_player))// follow character
         .run();
}

fn setup(
    mut commands: Commands,// to spawn in enities
    asset_server: Res<AssetServer>,// to access images
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,// used in animation
) {
    commands.spawn(Camera2dBundle::default());

    let bg_texture_handle = asset_server.load("tiles/cobblestone_floor/cobblestone_floor.png");

    let north_wall_texture_handle = asset_server.load("tiles/walls/north_wall.png");
    let south_wall_handle = asset_server.load("tiles/walls/bottom_wall.png");
    let east_wall_handle:Handle<Image> = asset_server.load("tiles/walls/right_wall.png");
    let west_wall_handle:Handle<Image> = asset_server.load("tiles/walls/left_wall.png");
    /* We want (0,0) to be center stage, *
     * this will start us in bottom left *
     * for spawning in tiles             */
    let mut x_offset = -MAX_X; 
    let mut y_offset = -MAX_Y; 

    while x_offset < MAX_X + (TILE_SIZE as f32){
        /* Spawn in north wall */
        commands.spawn((SpriteBundle {
            texture: north_wall_texture_handle.clone(),
            /* Top of level, minus tile size/2 for center spawning yk */
            transform: Transform::from_xyz(x_offset, MAX_Y - ((TILE_SIZE / 2)as f32), 1.),
            ..default()
        },
        Wall,
        ));
        /* Spawn in south wall */
        commands.spawn((SpriteBundle {
            texture: south_wall_handle.clone(),
            /* bottom of level, plus tile size/2 for center spawning yk */
            transform: Transform::from_xyz(x_offset, -MAX_Y + ((TILE_SIZE/2)as f32), 1.),
            ..default()
        },
        Wall,
        ));
        while y_offset < MAX_Y + (TILE_SIZE as f32){
            /* floor tiles */
            commands
                .spawn(SpriteBundle {
                    texture: bg_texture_handle.clone(),
                    transform: Transform::from_xyz(x_offset, y_offset, 0.),
                    ..default()
                })
                .insert(Background);
            /* east wall */
            commands.spawn((SpriteBundle {
                texture: east_wall_handle.clone(),
                /* right side of level, minus tile size/2 for center spawning yk */
                transform: Transform::from_xyz(MAX_X - ((TILE_SIZE/2)as f32), y_offset, 1.),
                ..default()
            },
            Wall,
            ));

            commands.spawn((SpriteBundle {
                texture: west_wall_handle.clone(),
                /* left side of level, minus tile size/2 for center spawning yk */
                transform: Transform::from_xyz(-MAX_X + ((TILE_SIZE/2)as f32), y_offset, 1.),
                ..default()
            },
            Wall,
            ));
           
            y_offset += 32 as f32; 
        }
        y_offset = -MAX_Y; 
        x_offset += 32 as f32; 
    }

    let player_sheet_handle = asset_server.load("berry_rat.png");
    let player_layout = TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE), 4, 1, None, None);
    let player_layout_len = player_layout.textures.len();
    let player_layout_handle = texture_atlases.add(player_layout);
    commands.spawn((
        SpriteBundle {
            texture: player_sheet_handle,
            transform: Transform::from_xyz(0., 0., 900.),
            ..default()
        },
        TextureAtlas {
            layout: player_layout_handle,
            index: 0,
        },
        AnimationTimer(Timer::from_seconds(ANIM_TIME, TimerMode::Repeating)),
        AnimationFrameCount(player_layout_len),
        Velocity::new(),
        Player,
    ));

}
fn move_player(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut Transform, &mut Velocity), (With<Player>, Without<Background>)>,
) {
    let (mut pt, mut pv) = player.single_mut();

    let mut deltav = Vec2::splat(0.);

    if input.pressed(KeyCode::KeyA) {
        deltav.x -= 1.;
    }

    if input.pressed(KeyCode::KeyD) {
        deltav.x += 1.;
    }
    if input.pressed(KeyCode::KeyW) {
        deltav.y += 1.;
    }
    if input.pressed(KeyCode::KeyS) {
        deltav.y -= 1.;
    }

    let deltat = time.delta_seconds();
    let acc = ACCEL_RATE * deltat;

    // sprint - check if shift is pressed
    let speed_multiplier = if input.pressed(KeyCode::ShiftLeft) {
        SPRINT_MULTIPLIER
    } else {
        1.0
    };

    // set new max speed
    let max_speed = PLAYER_SPEED * speed_multiplier;

    pv.velocity = if deltav.length() > 0. {
        (pv.velocity + (deltav.normalize_or_zero() * acc)).clamp_length_max(max_speed)
    // heres where we use the max_speed
    } else if pv.velocity.length() > acc {
        pv.velocity + (pv.velocity.normalize_or_zero() * -acc)
    } else {
        Vec2::splat(0.)
    };
    let change = pv.velocity * deltat;

    let new_pos: Vec3 = pt.translation + Vec3::new(change.x, 0., 0.);
    if (new_pos.x >= -MAX_X + (TILE_SIZE as f32) / 2.
       && new_pos.x <= MAX_X - (TILE_SIZE as f32) / 2.)
    {
        pt.translation = new_pos;
    }

    let new_pos = pt.translation + Vec3::new(0., change.y, 0.);
    //if new_pos.y >= -(WIN_H / 2.) + ((TILE_SIZE as f32) * 1.5)
    /* FOR ABOVE ^^^ I think it would be better for us to first set the border
    as the screen, then use detection where all wall tiles are present - Lukas */
    /* Werd - ROry */
    if new_pos.y >= -MAX_Y + (TILE_SIZE as f32) / 2.
        && new_pos.y <= MAX_Y - (TILE_SIZE as f32) / 2.
    {
        pt.translation = new_pos;
    }
}

fn animate_player(
    time: Res<Time>,
    mut player: Query<
        (
            &Velocity,
            &mut TextureAtlas,
            &mut AnimationTimer,
            &AnimationFrameCount,
        ),
        With<Player>,
    >,
) {
    /* In texture atlas for ratatta:
     * 0 = left
     * 1 = right
     * 2 = up
     * 3 = down 
     * ratlas. heh. get it.*/
    let (v, mut ratlas, mut timer, frame_count) = player.single_mut();
    if v.velocity.cmpne(Vec2::ZERO).any() {
        timer.tick(time.delta());

        if v.velocity.x > 0.{
            ratlas.index = 1;
        }
        else if v.velocity.x < 0. {
            ratlas.index = 0;
        }

        if v.velocity.y > 0.{
            ratlas.index = 2;
        }
        else if v.velocity.y < 0. {
            ratlas.index = 3;
        }
    }
}

fn move_camera(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (Without<Player>, With<Camera>)>,
) {
    let pt = player.single();
    let mut ct = camera.single_mut();

    ct.translation.x = pt.translation.x.clamp(-MAX_X + (WIN_W/2.), MAX_X - (WIN_W/2.));
    ct.translation.y = pt.translation.y.clamp(-MAX_Y + (WIN_H/2.), MAX_Y - (WIN_H/2.));
}
