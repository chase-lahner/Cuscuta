mod sprint_mechanic;
mod title_sequence;

use bevy::{prelude::*, render::extract_component::ExtractComponent, window::PresentMode};
use rand::Rng;

const TITLE: &str = "Cuscuta Demo";// window title
const WIN_W: f32 = 1280.;// window width
const WIN_H: f32 = 720.;// window height

const PLAYER_SPEED: f32 = 480.; 
const ACCEL_RATE: f32 = 4800.; 
const SPRINT_MULTIPLIER: f32 = 2.0;

const TILE_SIZE: u32 = 32; 

const LEVEL_W: f32 = 4800.; 

const LEVEL_H: f32 = 1600.; 

const ARR_W: usize = (LEVEL_W as usize) / 32;

const ARR_H: usize = (LEVEL_H as usize) / 32;

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

#[derive(Component)]
struct Enemy; // danger lurks ahead

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

struct Aabb {
    min: Vec2,
    max: Vec2,
}

impl Aabb {
    fn new(center: Vec3, size: Vec2) -> Self {
        let half_size = size / 2.0;
        Self {
            min: Vec2::new(center.x - half_size.x, center.y - half_size.y),
            max: Vec2::new(center.x + half_size.x, center.y + half_size.y),
        }
    }

    fn intersects(&self, other: &Aabb) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
    }
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

static mut grid1: [[u32; ARR_H]; ARR_W] = [[0; ARR_H]; ARR_W];
static mut grid2: [[u32; ARR_H]; ARR_W] = [[0; ARR_H]; ARR_W];

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
    mut commands: Commands, // to spawn in entities
    asset_server: Res<AssetServer>, // to access images
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>, // used in animation
) {
    // spawn the starting room
    SpawnStartRoom(&mut commands, &asset_server);

    // spawn camera
    commands.spawn(Camera2dBundle::default());

    let player_sheet_handle = asset_server.load("berry_rat.png");
    let player_layout = TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE), 4, 1, None, None);
    let player_layout_len = player_layout.textures.len();
    let player_layout_handle = texture_atlases.add(player_layout);

    // spawn player at origin
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

    // enemy spawning
    let skeleton_asset = asset_server.load("Skelly.png");

    // spawn enemy near player spawn
    commands.spawn((
        SpriteBundle {
            texture: skeleton_asset,
            transform: Transform::from_xyz(100., 50., 900.),
            ..default()
        },
        Enemy,
    ));
}

fn set_collide(room: u32, x: &usize, y: &usize)
{
    if room == 1 {unsafe{grid1[x/TILE_SIZE as usize][y/TILE_SIZE as usize] = 1;}}
    if room == 2 {unsafe{grid2[x/TILE_SIZE as usize][y/TILE_SIZE as usize] = 1;}}
}

fn SpawnStartRoom( /* First Room */
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>
) {
    let bg_texture_handle = asset_server.load("tiles/cobblestone_floor/cobblestone_floor.png");

    let north_wall_texture_handle = asset_server.load("tiles/walls/north_wall.png");
    let south_wall_handle = asset_server.load("tiles/walls/bottom_wall.png");
    let east_wall_handle: Handle<Image> = asset_server.load("tiles/walls/right_wall.png");
    let west_wall_handle: Handle<Image> = asset_server.load("tiles/walls/left_wall.png");

    let mut x_offset = -MAX_X + ((TILE_SIZE / 2) as f32);
    let mut y_offset = -MAX_Y + ((TILE_SIZE / 2) as f32);

    while x_offset < MAX_X {

        let mut xcoord: usize;
        let mut ycoord: usize;

        /* Spawn in north wall */
        commands.spawn((SpriteBundle {
            texture: north_wall_texture_handle.clone(),
            transform: Transform::from_xyz(x_offset, MAX_Y - ((TILE_SIZE / 2) as f32), 1.),
            ..default()
        }, Wall));

        xcoord = (x_offset - ((TILE_SIZE / 2) as f32) + MAX_X) as usize;
        ycoord = (MAX_Y * 2. - ((TILE_SIZE / 2) as f32)) as usize;
        set_collide(1, &xcoord, &ycoord);
        //unsafe{grid1[xcoord/TILE_SIZE as usize][ycoord/TILE_SIZE as usize] = 1;}

        /* Spawn in south wall */
        commands.spawn((SpriteBundle {
            texture: south_wall_handle.clone(),
            transform: Transform::from_xyz(x_offset, -MAX_Y + ((TILE_SIZE / 2) as f32), 1.),
            ..default()
        }, Wall));

        xcoord = (x_offset - ((TILE_SIZE / 2) as f32) + MAX_X) as usize;
        ycoord = (0) as usize;
        set_collide(1, &xcoord, &ycoord);
        //unsafe{grid1[xcoord/TILE_SIZE as usize][ycoord/TILE_SIZE as usize] = 1;}

        while y_offset < MAX_Y + (TILE_SIZE as f32) {

            /* East wall */
            commands.spawn((SpriteBundle {
                texture: east_wall_handle.clone(),
                transform: Transform::from_xyz(MAX_X - ((TILE_SIZE / 2) as f32), y_offset, 1.),
                ..default()
            }, Wall));

            xcoord = (MAX_X * 2. - ((TILE_SIZE / 2) as f32)) as usize;
            ycoord = (y_offset - ((TILE_SIZE / 2) as f32) + MAX_Y -1.) as usize;
            set_collide(1, &xcoord, &ycoord);
            //unsafe{grid1[xcoord/TILE_SIZE as usize][ycoord/TILE_SIZE as usize] = 1;}

            /* West wall */
            commands.spawn((SpriteBundle {
                texture: west_wall_handle.clone(),
                transform: Transform::from_xyz(-MAX_X + ((TILE_SIZE / 2) as f32), y_offset, 1.),
                ..default()
            }, Wall));

            xcoord = (0) as usize;
            ycoord = (y_offset - ((TILE_SIZE / 2) as f32) + MAX_Y - 1.) as usize;
            set_collide(1, &xcoord, &ycoord);
            //unsafe{grid1[xcoord/TILE_SIZE as usize][ycoord/TILE_SIZE as usize] = 1;}

            /* Floor tiles */
            commands.spawn(SpriteBundle {
                texture: bg_texture_handle.clone(),
                transform: Transform::from_xyz(x_offset, y_offset, 0.),
                ..default()
            }).insert(Background);

            y_offset += TILE_SIZE as f32;
        }
        y_offset = -MAX_Y + ((TILE_SIZE / 2) as f32);
        x_offset += TILE_SIZE as f32;
    }
    for a in 0..150
    {
        for b in 0..50
        {
            unsafe{print!("{}", grid1[a][b])}
        }
        println!()
    }
}

fn SpawnNextRoom( /* Second Room */
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>
) {
    let bg_texture_handle = asset_server.load("tiles/solid_floor/solid_floor.png");

    let north_wall_texture_handle = asset_server.load("tiles/walls/north_wall.png");
    let south_wall_handle = asset_server.load("tiles/walls/bottom_wall.png");
    let east_wall_handle: Handle<Image> = asset_server.load("tiles/walls/right_wall.png");
    let west_wall_handle: Handle<Image> = asset_server.load("tiles/walls/left_wall.png");

    let mut x_offset = -MAX_X + ((TILE_SIZE / 2) as f32);
    let mut y_offset = -MAX_Y + ((TILE_SIZE / 2) as f32);

    while x_offset < MAX_X + (TILE_SIZE as f32) {
        /* Spawn in north wall */
        commands.spawn((SpriteBundle {
            texture: north_wall_texture_handle.clone(),
            transform: Transform::from_xyz(x_offset, MAX_Y - ((TILE_SIZE / 2) as f32), 3.),
            ..default()
        }, Wall));
        
        /* Spawn in south wall */
        commands.spawn((SpriteBundle {
            texture: south_wall_handle.clone(),
            transform: Transform::from_xyz(x_offset, -MAX_Y + ((TILE_SIZE / 2) as f32), 3.),
            ..default()
        }, Wall));

        while y_offset < MAX_Y + (TILE_SIZE as f32) {
            /* Floor tiles */
            commands.spawn(SpriteBundle {
                texture: bg_texture_handle.clone(),
                transform: Transform::from_xyz(x_offset, y_offset, 2.),
                ..default()
            }).insert(Background);

            /* East wall */
            commands.spawn((SpriteBundle {
                texture: east_wall_handle.clone(),
                transform: Transform::from_xyz(MAX_X - ((TILE_SIZE / 2) as f32), y_offset, 3.),
                ..default()
            }, Wall));

            /* West wall */
            commands.spawn((SpriteBundle {
                texture: west_wall_handle.clone(),
                transform: Transform::from_xyz(-MAX_X + ((TILE_SIZE / 2) as f32), y_offset, 3.),
                ..default()
            }, Wall));

            y_offset += TILE_SIZE as f32;
        }
        y_offset = -MAX_Y + ((TILE_SIZE / 2) as f32);
        x_offset += TILE_SIZE as f32;
    }
}

fn aabb_collision(player_aabb: &Aabb, enemy_aabb: &Aabb) -> bool {
    player_aabb.intersects(&enemy_aabb)
}


fn move_player(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut Transform, &mut Velocity), (With<Player>, Without<Background>)>,
    enemy_query: Query<&Transform, (With<Enemy>, Without<Player>)>,
) {
    let (mut pt, mut pv) = player.single_mut();
    let enemy = enemy_query.single();

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
    } else if pv.velocity.length() > acc {
        pv.velocity + (pv.velocity.normalize_or_zero() * -acc)
    } else {
        Vec2::splat(0.)
    };
    let change = pv.velocity * deltat;

    let mut player_aabb = Aabb::new(pt.translation, Vec2::splat(TILE_SIZE as f32));
    let enemy_aabb = Aabb::new(enemy.translation, Vec2::splat(TILE_SIZE as f32));

    // array coords
    let mut topleft: u32;
    let mut topright: u32;
    let mut bottomleft: u32;
    let mut bottomright: u32;

    // horizontal movement and collision detection
    let new_pos: Vec3 = pt.translation + Vec3::new(change.x, 0., 0.);
    player_aabb = Aabb::new(new_pos, Vec2::splat(TILE_SIZE as f32)); // Update AABB for new position

    //translate pixel coords into array coords
    let arrymax: f32 = player_aabb.max.y / 32.0 + (ARR_H as f32 / 2.);
    let arrymin: f32 = player_aabb.min.y / 32.0 + (ARR_H as f32 / 2.);
    let arrxmax: f32 = player_aabb.max.x /32.0 + (ARR_W as f32 / 2.);
    let arrxmin: f32 = player_aabb.min.x /32.0 + (ARR_W as f32 / 2.);
    unsafe{topleft = grid1[arrxmin as usize][arrymax as usize];}
    unsafe{topright = grid1[arrxmax as usize][arrymax as usize];}
    unsafe{bottomleft = grid1[arrxmin as usize][arrymin as usize];}
    unsafe{bottomright = grid1[arrxmax as usize][arrymin as usize];}

    if !aabb_collision(&player_aabb, &enemy_aabb)
        && new_pos.x >= -MAX_X + (TILE_SIZE as f32) / 2.
        && new_pos.x <= MAX_X - (TILE_SIZE as f32) / 2.
        && topleft != 1 && topright != 1 && bottomleft != 1 && bottomright != 1
    {
        pt.translation = new_pos;
    }

    // vertical movement & collision detection
    let new_pos = pt.translation + Vec3::new(0., change.y, 0.);
    player_aabb = Aabb::new(new_pos, Vec2::splat(TILE_SIZE as f32)); // Update AABB for new position

    //translate pixel coords into array coords
    let arrymax: f32 = player_aabb.max.y / 32.0 + (ARR_H as f32 / 2.);
    let arrymin: f32 = player_aabb.min.y / 32.0 + (ARR_H as f32 / 2.);
    let arrxmax: f32 = player_aabb.max.x /32.0 + (ARR_W as f32 / 2.);
    let arrxmin: f32 = player_aabb.min.x /32.0 + (ARR_W as f32 / 2.);
    unsafe{topleft = grid1[arrxmin as usize][arrymax as usize];}
    unsafe{topright = grid1[arrxmax as usize][arrymax as usize];}
    unsafe{bottomleft = grid1[arrxmin as usize][arrymin as usize];}
    unsafe{bottomright = grid1[arrxmax as usize][arrymin as usize];}

    //if new_pos.y >= -(WIN_H / 2.) + ((TILE_SIZE as f32) * 1.5)
    /* FOR ABOVE ^^^ I think it would be better for us to first set the border
    as the screen, then use detection where all wall tiles are present - Lukas */
    /* Werd - ROry */
    if !aabb_collision(&player_aabb, &enemy_aabb)
        && new_pos.y >= -MAX_Y + (TILE_SIZE as f32) / 2.
        && new_pos.y <= MAX_Y - (TILE_SIZE as f32) / 2.
        && topleft != 1 && topright != 1 && bottomleft != 1 && bottomright != 1
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

