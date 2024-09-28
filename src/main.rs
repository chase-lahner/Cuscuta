use bevy::{prelude::*, window::PresentMode};

#[derive(Component, Deref, DerefMut)]
struct PopupTimer(Timer);

const TITLE: &str = "Cuscuta Demo";
const WIN_W: f32 = 1280.;
const WIN_H: f32 = 720.;

const PLAYER_SPEED: f32 = 480.; //500
const ACCEL_RATE: f32 = 4800.; //5000
const SPRINT_MULTIPLIER: f32 = 2.0;

const TILE_SIZE: u32 = 32; //100

const LEVEL_LEN: f32 = 4800.; //5000

const LEVEL_HEIGHT: f32 = 1600.; //2000

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
                title: TITLE.into(),
                present_mode: PresentMode::Fifo,
                ..default() // Name and present mode all we need for now
             }),
             ..default()
         }))
         .add_systems(Startup,setup)
         .add_systems(Update, move_player)
         .add_systems(Update, animate_player.after(move_player))
         .add_systems(Update, move_camera.after(animate_player))
         .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2dBundle::default());

    let bg_texture_handle = asset_server.load("tiles/cobblestone_floor/cobblestone_floor.png");

    let mut x_offset = -WIN_H/2.; //0.
    let mut y_offset = -WIN_W/2.; //0.
    while x_offset < LEVEL_LEN {
        while y_offset < LEVEL_HEIGHT {
        commands
            .spawn(SpriteBundle {
                texture: bg_texture_handle.clone(),
                transform: Transform::from_xyz(x_offset, y_offset, 0.),
                ..default()
            })
            .insert(Background);
            y_offset += 32 as f32; //WIN_H
        }
        y_offset = -WIN_W/2.; //0.
        x_offset += 32 as f32; //WIN_W
    }
   
    /*let mut y_offset = 0.;
    while y_offset < LEVEL_HEIGHT {
        commands.spawn(SpriteBundle {
            texture: bg_texture_handle.clone(),
            transform: Transform::from_xyz(0.,y_offset,0.),
            ..default()
        })
        .insert(Background);
        
        y_offset += (32 as f32);
    }*/



    let player_sheet_handle = asset_server.load("walking.png");
    let player_layout = TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE), 4, 1, None, None);
    let player_layout_len = player_layout.textures.len();
    let player_layout_handle = texture_atlases.add(player_layout);
    commands.spawn((
        SpriteBundle {
            texture: player_sheet_handle,
            transform: Transform::from_xyz(0., -(WIN_H / 2.) + ((TILE_SIZE as f32) * 1.5), 900.),
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

    /* let brick_sheet_handle = asset_server.load("bricks.png");
    let brick_layout = TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE), 4, 1, None, None);
    let brick_layout_len = brick_layout.len();
    let brick_layout_handle = texture_atlases.add(brick_layout);

    let mut i = 0;
    let mut t = Vec3::new(
        -WIN_W / 2. + (TILE_SIZE as f32) / 2.,
        -WIN_H / 2. + (TILE_SIZE as f32) / 2.,
        0.,
    );
    while i * TILE_SIZE < (LEVEL_LEN as u32) {
        commands.spawn((
            SpriteBundle {
                texture: brick_sheet_handle.clone(),
                transform: Transform {
                    translation: t,
                    ..default()
                },
                ..default()
            },
            TextureAtlas {
                layout: brick_layout_handle.clone(),
                index: (i as usize) % brick_layout_len,
            },
            Brick,
        ));

        i += 1;
        t += Vec3::new(TILE_SIZE as f32, 0., 0.);
    } */
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
    if input.pressed(KeyCode::KeyW){
        deltav.y += 1.;
    }
    if input.pressed(KeyCode:: KeyS){
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
        (pv.velocity + (deltav.normalize_or_zero() * acc)).clamp_length_max(max_speed) // heres where we use the max_speed
    } else if pv.velocity.length() > acc {
        pv.velocity + (pv.velocity.normalize_or_zero() * -acc)
    } else {
        Vec2::splat(0.)
    };
    let change = pv.velocity * deltat;

    let new_pos: Vec3 = pt.translation + Vec3::new(change.x, 0., 0.);
    if new_pos.x >= -(WIN_W / 2.) + (TILE_SIZE as f32) / 2.
        && new_pos.x <= LEVEL_LEN - (WIN_W / 2. + (TILE_SIZE as f32) / 2.)
    {
        pt.translation = new_pos;
    }

    let new_pos = pt.translation + Vec3::new(0., change.y, 0.);
    //if new_pos.y >= -(WIN_H / 2.) + ((TILE_SIZE as f32) * 1.5)
    /* FOR ABOVE ^^^ I think it would be better for us to first set the border 
    as the screen, then use detection where all wall tiles are present - Lukas */
    if new_pos.y >= -(WIN_H / 2.) + (TILE_SIZE as f32) / 2.
        && new_pos.y <= LEVEL_HEIGHT - (WIN_H / 2. - (TILE_SIZE as f32) / 2.) - (TILE_SIZE as f32)
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
    let (v, mut texture_atlas, mut timer, frame_count) = player.single_mut();
    if v.velocity.cmpne(Vec2::ZERO).any() {
        timer.tick(time.delta());

        // if timer.just_finished() {
        texture_atlas.index = (texture_atlas.index + 1) % **frame_count;
        // }
    }
}

fn move_camera(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (Without<Player>, With<Camera>)>,
) {
    let pt = player.single();
    let mut ct = camera.single_mut();

    ct.translation.x = pt.translation.x.clamp(0., LEVEL_LEN - WIN_W);
    ct.translation.y = pt.translation.y.clamp(0.,LEVEL_HEIGHT - WIN_H);
}
