

use std::{mem::transmute, net::UdpSocket};
use bevy::{ecs::query::QueryIter, log::tracing_subscriber::fmt::format, prelude::*, render::extract_component::ExtractComponent, window::PresentMode};

use rand::Rng;

const TITLE: &str = "Cuscuta Demo";// window title
const WIN_W: f32 = 1280.;// window width
const WIN_H: f32 = 720.;// window height

const PLAYER_SPEED: f32 = 480.; 
const ACCEL_RATE: f32 = 4800.; 
const SPRINT_MULTIPLIER: f32 = 2.0;

const ENEMY_SPEED: f32 = 200.;
const NUMBER_OF_ENEMIES: u32 = 10;

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


#[derive(Component)]
pub struct Player;// wow! it is he!


#[derive(Component)]
pub struct Enemy {
    pub direction: Vec2,
} 

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
struct Door;

#[derive(Component)]
struct Velocity {
    velocity: Vec2,
}

#[derive(Resource)]
struct UDP{
    socket: UdpSocket
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

static mut GRID1: [[u32; ARR_H]; ARR_W] = [[0; ARR_H]; ARR_W];
static mut GRID2: [[u32; ARR_H]; ARR_W] = [[0; ARR_H]; ARR_W];

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
         .add_systems(Startup, spawn_enemies)
         .add_systems(Update, move_player)// every frame, takes in WASD for movement
         //.add_systems(Update, send_packet)
         .add_systems(Update, recv_packet)
         .add_systems(Update, send_movement_info.after(move_player))
         .add_systems(Update, enemy_movement.after(move_player))
         .add_systems(Update, animate_player.after(move_player)) // animates player, duh
         .add_systems(Update, move_camera.after(animate_player))// follow character
         .run();
}

fn setup(
    mut commands: Commands, // to spawn in entities
    asset_server: Res<AssetServer>, // to access images
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>, // used in animation
) {
    // spawn the starting room & next room
    spawn_start_room(&mut commands, &asset_server);
    spawn_next_room(&mut commands, &asset_server);

    /* initializes our networking socket */
    let socket = UdpSocket::bind("localhost:5000").unwrap();

    commands.insert_resource(UDP {socket: socket});


    // spawn camera
    commands.spawn(Camera2dBundle::default());

    // spawn player
 spawn_player(&mut commands, &asset_server, &mut texture_atlases);
}

fn set_collide(room: u32, x: &usize, y: &usize, val: u32)
{
    if room == 1 {unsafe{GRID1[x/TILE_SIZE as usize][y/TILE_SIZE as usize] = val;}}
    if room == 2 {unsafe{GRID2[x/TILE_SIZE as usize][y/TILE_SIZE as usize] = val;}}
}

fn spawn_player(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>
) {
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
}

fn spawn_start_room( /* First Room */
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>
) {
    let bg_texture_handle = asset_server.load("tiles/cobblestone_floor/cobblestone_floor.png");

    let north_wall_texture_handle = asset_server.load("tiles/walls/north_wall.png");
    let south_wall_handle = asset_server.load("tiles/walls/bottom_wall.png");
    let east_wall_handle = asset_server.load("tiles/walls/right_wall.png");
    let west_wall_handle = asset_server.load("tiles/walls/left_wall.png");
    let door_handle = asset_server.load("tiles/walls/black_void.png");

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
        set_collide(1, &xcoord, &ycoord, 1);
        //unsafe{GRID1[xcoord/TILE_SIZE as usize][ycoord/TILE_SIZE as usize] = 1;}

        /* Spawn in south wall */
        commands.spawn((SpriteBundle {
            texture: south_wall_handle.clone(),
            transform: Transform::from_xyz(x_offset, -MAX_Y + ((TILE_SIZE / 2) as f32), 1.),
            ..default()
        }, Wall));

        xcoord = (x_offset - ((TILE_SIZE / 2) as f32) + MAX_X) as usize;
        ycoord = (0) as usize;
        set_collide(1, &xcoord, &ycoord, 1);
        //unsafe{GRID1[xcoord/TILE_SIZE as usize][ycoord/TILE_SIZE as usize] = 1;}

        while y_offset < MAX_Y + (TILE_SIZE as f32) {

            /* East wall */
            commands.spawn((SpriteBundle {
                texture: east_wall_handle.clone(),
                transform: Transform::from_xyz(MAX_X - ((TILE_SIZE / 2) as f32), y_offset, 1.),
                ..default()
            }, Wall));

            xcoord = (MAX_X * 2. - ((TILE_SIZE / 2) as f32)) as usize;
            ycoord = (y_offset - ((TILE_SIZE / 2) as f32) + MAX_Y -1.) as usize;
            set_collide(1, &xcoord, &ycoord, 1);
            //unsafe{GRID1[xcoord/TILE_SIZE as usize][ycoord/TILE_SIZE as usize] = 1;}

            /* West wall */
            commands.spawn((SpriteBundle {
                texture: west_wall_handle.clone(),
                transform: Transform::from_xyz(-MAX_X + ((TILE_SIZE / 2) as f32), y_offset, 1.),
                ..default()
            }, Wall));

            xcoord = (0) as usize;
            ycoord = (y_offset - ((TILE_SIZE / 2) as f32) + MAX_Y - 1.) as usize;
            set_collide(1, &xcoord, &ycoord, 1);
            //unsafe{GRID1[xcoord/TILE_SIZE as usize][ycoord/TILE_SIZE as usize] = 1;}


            /* Floor tiles */
            commands.spawn(SpriteBundle {
                texture: bg_texture_handle.clone(),
                transform: Transform::from_xyz(x_offset, y_offset, 0.),
                ..default()
            }).insert(Background);


            if (x_offset == MAX_X - (3 * TILE_SIZE/2) as f32) && (y_offset == (TILE_SIZE / 2) as f32)
            {
                commands.spawn((SpriteBundle {
                    texture: door_handle.clone(),
                    transform: Transform::from_xyz(x_offset, y_offset, 1.),
                    ..default()
                }, Door));
                xcoord = (MAX_X * 2. - (3 * TILE_SIZE/2) as f32) as usize;
                ycoord = (y_offset - ((TILE_SIZE / 2) as f32) + MAX_Y) as usize;
                set_collide(1, &xcoord, &ycoord, 2);
            }

            y_offset += TILE_SIZE as f32;
        }
        y_offset = -MAX_Y + ((TILE_SIZE / 2) as f32);
        x_offset += TILE_SIZE as f32;
    }
    /*for a in 0..150
    {
        for b in 0..50
        {
            unsafe{print!("{}", GRID1[a][b])}
        }
        println!()
    }*/
}

fn spawn_next_room( /* Second Room */
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>
) {
    let bg_texture_handle = asset_server.load("tiles/solid_floor/solid_floor.png");

    let north_wall_texture_handle = asset_server.load("tiles/walls/north_wall.png");
    let south_wall_handle = asset_server.load("tiles/walls/bottom_wall.png");
    let east_wall_handle: Handle<Image> = asset_server.load("tiles/walls/right_wall.png");
    let west_wall_handle: Handle<Image> = asset_server.load("tiles/walls/left_wall.png");
    let door_handle: Handle<Image> = asset_server.load("tiles/walls/black_void.png");

    let mut x_offset = -MAX_X + ((TILE_SIZE / 2) as f32);
    let mut y_offset = -MAX_Y + ((TILE_SIZE / 2) as f32);

    while x_offset < MAX_X {

        let mut xcoord: usize;
        let mut ycoord: usize;

        /* Spawn in north wall */
        commands.spawn((SpriteBundle {
            texture: north_wall_texture_handle.clone(),
            transform: Transform::from_xyz(x_offset, MAX_Y - ((TILE_SIZE / 2) as f32), -3.),
            ..default()
        }, Wall));

        xcoord = (x_offset - ((TILE_SIZE / 2) as f32) + MAX_X) as usize;
        ycoord = (MAX_Y * 2. - ((TILE_SIZE / 2) as f32)) as usize;
        set_collide(2, &xcoord, &ycoord, 1);

        /* Spawn in south wall */
        commands.spawn((SpriteBundle {
            texture: south_wall_handle.clone(),
            transform: Transform::from_xyz(x_offset, -MAX_Y + ((TILE_SIZE / 2) as f32), -3.),
            ..default()
        }, Wall));

        xcoord = (x_offset - ((TILE_SIZE / 2) as f32) + MAX_X) as usize;
        ycoord = (0) as usize;
        set_collide(2, &xcoord, &ycoord, 1);

        while y_offset < MAX_Y + (TILE_SIZE as f32) {

            /* East wall */
            commands.spawn((SpriteBundle {
                texture: east_wall_handle.clone(),
                transform: Transform::from_xyz(MAX_X - ((TILE_SIZE / 2) as f32), y_offset, -3.),
                ..default()
            }, Wall));

            xcoord = (MAX_X * 2. - ((TILE_SIZE / 2) as f32)) as usize;
            ycoord = (y_offset - ((TILE_SIZE / 2) as f32) + MAX_Y -1.) as usize;
            set_collide(2, &xcoord, &ycoord, 1);

            /* West wall */
            commands.spawn((SpriteBundle {
                texture: west_wall_handle.clone(),
                transform: Transform::from_xyz(-MAX_X + ((TILE_SIZE / 2) as f32), y_offset, -3.),
                ..default()
            }, Wall));

            xcoord = (0) as usize;
            ycoord = (y_offset - ((TILE_SIZE / 2) as f32) + MAX_Y - 1.) as usize;
            set_collide(2, &xcoord, &ycoord, 1);

            /* Floor tiles */
            commands.spawn(SpriteBundle {
                texture: bg_texture_handle.clone(),
                transform: Transform::from_xyz(x_offset, y_offset, -2.),
                ..default()
            }).insert(Background);

            if (x_offset == MAX_X - (3 * TILE_SIZE/2) as f32) && (y_offset == (TILE_SIZE / 2) as f32)
            {
                commands.spawn((SpriteBundle {
                    texture: door_handle.clone(),
                    transform: Transform::from_xyz(x_offset, y_offset, -3.),
                    ..default()
                }, Door));
                xcoord = (MAX_X * 2. - (3 * TILE_SIZE/2) as f32) as usize;
                ycoord = (y_offset - ((TILE_SIZE / 2) as f32) + MAX_Y -1.) as usize;
                set_collide(2, &xcoord, &ycoord, 2);
            }

            y_offset += TILE_SIZE as f32;
        }
        y_offset = -MAX_Y + ((TILE_SIZE / 2) as f32);
        x_offset += TILE_SIZE as f32;
    }
    /*for a in 0..150
    {
        for b in 0..50
        {
            unsafe{print!("{}", GRID2[a][b])}
        }
        println!()
    }*/
}

fn aabb_collision(player_aabb: &Aabb, enemy_aabb: &Aabb) -> bool {
    player_aabb.intersects(&enemy_aabb)
}

fn move_player(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut Transform, &mut Velocity), (With<Player>, Without<Background>)>,
    mut enemies: Query<&mut Transform, (With<Enemy>, Without<Player>)>, 
    mut room: Query<&mut Transform, (Without<Player>, Without<Enemy>)>,
) {
    let (mut pt, mut pv) = player.single_mut();
    let mut deltav = Vec2::splat(0.);

    // INPUT HANDLING
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


    let mut dor: bool = false;

    // take care of horizontal and vertical movement + enemy collision check
    handle_movement_and_enemy_collisions(&mut pt, change, &mut dor, &mut enemies);

    // if we hit a door
    if dor {
        transition_map(&mut room, &mut pt);
    }
}

fn handle_movement_and_enemy_collisions(
    pt: &mut Transform,
    change: Vec2,
    dor: &mut bool,
    enemies: &mut Query<&mut Transform, (With<Enemy>, Without<Player>)>, 
) {
    // handle horizontal movement & collision
    handle_horizontal_movement(pt, change, dor, enemies);

    // handle vertical movement & collision
    handle_vertical_movement(pt, change, dor, enemies);
}

fn handle_horizontal_movement(
    pt: &mut Transform,
    change: Vec2,
    dor: &mut bool,
    enemies: &mut Query<&mut Transform, (With<Enemy>, Without<Player>)>, 
)
{
    let new_pos = pt.translation + Vec3::new(change.x, 0., 0.);
    let player_aabb = Aabb::new(new_pos, Vec2::splat(TILE_SIZE as f32));

    let (topleft, topright, bottomleft, bottomright) = translate_coords_to_grid(&player_aabb);

    // Check enemy collision
    for enemy_transform in enemies.iter() {
        let enemy_aabb = Aabb::new(enemy_transform.translation, Vec2::splat(TILE_SIZE as f32));
        if aabb_collision(&player_aabb, &enemy_aabb) {
            // Handle enemy collision logic
            return;
        }
    }

    // Movement within bounds and collision check
    if new_pos.x >= -MAX_X + (TILE_SIZE as f32) / 2.
        && new_pos.x <= MAX_X - (TILE_SIZE as f32) / 2.
        && topleft != 1 && topright != 1 && bottomleft != 1 && bottomright != 1
    {
        pt.translation = new_pos;
    }

    // Check for door transition
    if topleft == 2 || topright == 2 || bottomleft == 2 || bottomright == 2 {
        *dor = true;
    }
}

fn handle_vertical_movement (
    pt: &mut Transform,
    change: Vec2,
    dor: &mut bool,
    enemies: &mut Query<&mut Transform, (With<Enemy>, Without<Player>)>, 
)
{
    let new_pos = pt.translation + Vec3::new(0., change.y, 0.);
    let player_aabb = Aabb::new(new_pos, Vec2::splat(TILE_SIZE as f32));

    let (topleft, topright, bottomleft, bottomright) = translate_coords_to_grid(&player_aabb);

    // Check enemy collision
    for enemy_transform in enemies.iter() {
        let enemy_aabb = Aabb::new(enemy_transform.translation, Vec2::splat(TILE_SIZE as f32));
        if aabb_collision(&player_aabb, &enemy_aabb) {
            // Handle enemy collision logic
            return;
        }
    }

    // Movement within bounds and collision check
    if new_pos.y >= -MAX_Y + (TILE_SIZE as f32) / 2.
        && new_pos.y <= MAX_Y - (TILE_SIZE as f32) / 2.
        && topleft != 1 && topright != 1 && bottomleft != 1 && bottomright != 1
    {
        pt.translation = new_pos;
    }

    // Check for door transition
    if topleft == 2 || topright == 2 || bottomleft == 2 || bottomright == 2 {
        *dor = true;
    }
}

fn translate_coords_to_grid(aabb: &Aabb) -> (u32, u32, u32, u32){
    let arrymax: f32 = aabb.max.y / 32.0 + (ARR_H as f32 / 2.);
    let arrymin: f32 = aabb.min.y / 32.0 + (ARR_H as f32 / 2.);
    let arrxmax: f32 = aabb.max.x / 32.0 + (ARR_W as f32 / 2.);
    let arrxmin: f32 = aabb.min.x / 32.0 + (ARR_W as f32 / 2.);

    let topleft;
    let topright;
    let bottomleft;
    let bottomright;

    unsafe {
        topleft = GRID1[arrxmin as usize][arrymax as usize];
        topright = GRID1[arrxmax as usize][arrymax as usize];
        bottomleft = GRID1[arrxmin as usize][arrymin as usize];
        bottomright = GRID1[arrxmax as usize][arrymin as usize];
    }

    (topleft, topright, bottomleft, bottomright)
}

fn transition_map(room: &mut Query<&mut Transform, (Without<Player>, Without<Enemy>)>, pt: &mut Transform) {
    for mut wt in room.iter_mut() {
        wt.translation.z *= -1.;
    }
    let new_pos: Vec3 = pt.translation + Vec3::new(-MAX_X * 1.9, 0., 0.);
    pt.translation = new_pos;
}

fn spawn_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut rng = rand::thread_rng();

    for _ in 0..NUMBER_OF_ENEMIES {
        let random_x: f32 = rng.gen_range(-MAX_X..MAX_X);
        let random_y: f32 = rng.gen_range(-MAX_Y..MAX_Y);

        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(random_x, random_y, 900.),
                texture: asset_server.load("Skelly.png"),
                ..default()
            },
            Enemy {
                direction: Vec2::new(rng.gen::<f32>(), rng.gen::<f32>()).normalize(),
            },
        ));
    }

}

pub fn enemy_movement(
    mut enemy_query: Query<(&mut Transform, &Enemy)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>
) {
    let player_transform = player_query.single(); 

    for (mut transform, _enemy) in enemy_query.iter_mut() {

        let direction_to_player = player_transform.translation - transform.translation;

         let normalized_direction = direction_to_player.normalize();
        transform.translation += normalized_direction * ENEMY_SPEED * time.delta_seconds();
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
    let (v, mut ratlas, mut timer, _frame_count) = player.single_mut();
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

fn recv_packet(
    socket: Res<UDP>
){
    let mut buf = [0;1024];
    let (_amt, _src) = socket.socket.recv_from(&mut buf).unwrap();
    println!("{}", String::from_utf8_lossy(&buf));
}

fn send_packet(
    socket: Res<UDP>,
) {
    socket.socket.send_to(b"boo!", "localhost:5001").unwrap();
}

fn send_movement_info(
    socket: Res<UDP>, // defined in setup
    player: Query<&Transform, With<Player>>, // player transform
    
) { // consencus algs    let pt = player.single(); // get player transform
    let x = pt.translation.x;
    let y = pt.translation.y;
    let x_int = unsafe {x.to_int_unchecked::<u8>()}; // f32 to u8 WARNING: idk how dangerous this is but it's for sure unsafe :0
    let y_int = unsafe {y.to_int_unchecked::<u8>()};
    let buf:[u8;2] = [x_int, y_int]; // put it in u8 buffer format
    print!("{:?}", &buf);

    socket.socket.send_to(&buf,"localhost:5001").unwrap();  // send to surver at lh 5001 unwrap is error handling

}


/*fn change_room(
    mut wall: Query<&mut Transform, (Without<Player>, Without<Background>, With<Wall>)>,
    mut background: Query<&mut Transform, (Without<Player>, With<Background>)>,
) {
   for mut wt in wall.iter_mut() {
    wt.translation.z *= -1.;
   }

   for mut bt in background.iter_mut() {
    bt.translation.z *= -1.;
   }

}*/