use std::{mem::transmute, net::UdpSocket};
use bevy::{ecs::query::QueryIter, log::tracing_subscriber::fmt::format, prelude::*, render::extract_component::ExtractComponent, window::PresentMode};

use rand::Rng;

const TITLE: &str = "Cuscuta Demo";// window title
const WIN_W: f32 = 1280.;// window width
const WIN_H: f32 = 720.;// window height

const PLAYER_SPEED: f32 = 480.; 
const ACCEL_RATE: f32 = 4800.; 
const SPRINT_MULTIPLIER: f32 = 2.0;

const ENEMY_SPEED: f32 = 0.;
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

#[derive(Resource)]
struct RoomManager {
    grids: Vec<Vec<Vec<u32>>>,
    current_room: usize,
    room_sizes: Vec<(f32, f32)>,
    max_sizes: Vec<(f32, f32)>,  
}

impl RoomManager {
    fn new() -> Self {
        Self {
            grids: Vec::new(),
            current_room: 0,
            room_sizes: Vec::new(),
            max_sizes: Vec::new(), 

        }
    }

    // add new grid for new room 
    fn add_room(&mut self, width: usize, height: usize, room_width: f32, room_height: f32) {
        let new_grid = vec![vec![0; height]; width];
        self.grids.push(new_grid);
        self.room_sizes.push((room_width, room_height));
        
        // Calculate and store the max_x and max_y based on room size
        let max_x = room_width / 2.0;
        let max_y = room_height / 2.0;
        self.max_sizes.push((max_x, max_y));

        self.current_room = self.grids.len() - 1;
    }

    // Get mutable reference to the current grid
    fn current_grid(&mut self) -> &mut Vec<Vec<u32>> {
    &mut self.grids[self.current_room]
    }
    
    // Get the size of the current room (width, height)
    fn current_room_size(&self) -> (f32, f32) {
        self.room_sizes[self.current_room]
    }

    fn switch_room(&mut self, room_index: usize) {
        if room_index < self.grids.len() {
            self.current_room = room_index;
        }
    }

    fn current_room_max(&self) -> (f32, f32) {
        self.max_sizes[self.current_room]
    }
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

static mut attacking: bool = false;

fn main() {
    App::new()
        .insert_resource(RoomManager::new())
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
         .add_systems(Update, send_packet)
         .add_systems(Update, recv_packet)
         .add_systems(Update, send_movement_info.after(move_player))
         .add_systems(Update, enemy_movement.after(move_player))
         .add_systems(Update, animate_player.after(move_player)) // animates player
         .add_systems(Update, player_attack.after(animate_player)) // animates attack swing
         .add_systems(Update, move_camera.after(animate_player))// follow character
         .run();
}

fn setup(
    mut commands: Commands, // to spawn in entities
    asset_server: Res<AssetServer>, // to access images
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>, // used in animation
    mut room_manager: ResMut<RoomManager>,
) {
    // spawn the starting room & next room
    spawn_start_room(&mut commands, &asset_server, &mut room_manager);

    /* initializes our networking socket */
    let socket = UdpSocket::bind("localhost:5000").unwrap();

    commands.insert_resource(UDP {socket: socket});


    // spawn camera
    commands.spawn(Camera2dBundle::default());

    // spawn player
    spawn_player(&mut commands, &asset_server, &mut texture_atlases);
}

fn set_collide(room_manager: &mut RoomManager, x: usize, y: usize, val: u32) {

    // convert world coordinates to grid indices
    let grid_x = (x / TILE_SIZE as usize);
    let grid_y = (y / TILE_SIZE as usize);


    let current_grid = room_manager.current_grid();

    let arr_w_limit = current_grid.len();
    let arr_h_limit = current_grid[0].len();


    if grid_x < arr_w_limit && grid_y < arr_h_limit {
        current_grid[grid_x][grid_y] = val;
    } else {
       println!("Error: index out of bounds for collision at ({}, {})", grid_x, grid_y);
    }

}

fn spawn_player(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>
) {
    let player_sheet_handle = asset_server.load("4x8_player.png");
    let player_layout = TextureAtlasLayout::from_grid(UVec2::splat(TILE_SIZE), 4, 8, None, None);
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

fn spawn_start_room(
    commands: &mut Commands, 
    asset_server: &Res<AssetServer>,
    room_manager: &mut RoomManager,
) {
    let mut rng = rand::thread_rng();

    // generate random integers between 50 and 250, * 32
    let random_width = rng.gen_range(50..=250);
    let random_height = rng.gen_range(50..=250);

    // Room width & height as a multiple of 32
    let room_width = random_width as f32 * TILE_SIZE as f32;  
    let room_height = random_height as f32 * TILE_SIZE as f32;

    let arr_w = (room_width / TILE_SIZE as f32) as usize;
    let arr_h = (room_height / TILE_SIZE as f32) as usize;

    // Add the room and switch to it
    room_manager.add_room(arr_w, arr_h, room_width, room_height);
    room_manager.switch_room(room_manager.grids.len() - 1); 

    let mut max_x = room_width / 2.;
    let mut max_y = room_height / 2.;

    // add collision grid for start room
    room_manager.add_room((room_width / TILE_SIZE as f32) as usize, (room_height / TILE_SIZE as f32) as usize, room_width, room_height);

    let bg_texture_handle = asset_server.load("tiles/cobblestone_floor/cobblestone_floor.png");
    let north_wall_texture_handle = asset_server.load("tiles/walls/north_wall.png");
    let south_wall_handle = asset_server.load("tiles/walls/bottom_wall.png");
    let east_wall_handle = asset_server.load("tiles/walls/right_wall.png");
    let west_wall_handle = asset_server.load("tiles/walls/left_wall.png");
    let door_handle = asset_server.load("tiles/walls/black_void.png");

    let mut x_offset = -max_x + ((TILE_SIZE / 2) as f32);
    let mut y_offset = -max_y + ((TILE_SIZE / 2) as f32);


    while x_offset < max_x {


        let mut xcoord: usize;
        let mut ycoord: usize;

        /* Spawn in north wall */
        commands.spawn((SpriteBundle {
            texture: north_wall_texture_handle.clone(),
            transform: Transform::from_xyz(x_offset, max_y - ((TILE_SIZE / 2) as f32), 1.),
            ..default()
        }, Wall));

        xcoord = (x_offset - ((TILE_SIZE / 2) as f32) + max_x) as usize;
        ycoord = (max_y * 2. - ((TILE_SIZE / 2) as f32)) as usize;
        //set_collide(room_manager, xcoord, ycoord, 1);

        /* Spawn in south wall */
        commands.spawn((SpriteBundle {
            texture: south_wall_handle.clone(),
            transform: Transform::from_xyz(x_offset, -max_y + ((TILE_SIZE / 2) as f32), 1.),
            ..default()
        }, Wall));

        xcoord = (x_offset - ((TILE_SIZE / 2) as f32) + MAX_X) as usize;
        ycoord = (0) as usize;
        //set_collide(room_manager, xcoord, ycoord, 1);

        while y_offset < max_y + (TILE_SIZE as f32) {


            /* East wall */
            commands.spawn((SpriteBundle {
                texture: east_wall_handle.clone(),
                transform: Transform::from_xyz(max_x - ((TILE_SIZE / 2) as f32), y_offset, 1.),
                ..default()
            }, Wall));

            xcoord = (MAX_X * 2. - ((TILE_SIZE / 2) as f32)) as usize;
            ycoord = (y_offset - ((TILE_SIZE / 2) as f32) + MAX_Y - 1.) as usize;
            //set_collide(room_manager, xcoord, ycoord, 1);

            /* West wall */
            commands.spawn((SpriteBundle {
                texture: west_wall_handle.clone(),
                transform: Transform::from_xyz(-max_x + ((TILE_SIZE / 2) as f32), y_offset, 1.),
                ..default()
            }, Wall));

            xcoord = (0) as usize;
            ycoord = (y_offset - ((TILE_SIZE / 2) as f32) + MAX_Y - 1.) as usize;
            //set_collide(room_manager, xcoord, ycoord, 1);

            /* Floor tiles */
            commands.spawn(SpriteBundle {
                texture: bg_texture_handle.clone(),
                transform: Transform::from_xyz(x_offset, y_offset, 0.),
                ..default()
            }).insert(Background);

            // door
            if (x_offset == max_x - (3.0 * (TILE_SIZE as f32) / 2.0)) && (y_offset == (TILE_SIZE as f32 / 2.0)) {
                commands.spawn((
                    SpriteBundle {
                        texture: door_handle.clone(),
                        transform: Transform::from_xyz(x_offset, y_offset, 1.),
                        ..default()
                    },
                    Door,
                ));

                xcoord = (MAX_X * 2. - (3 * TILE_SIZE / 2) as f32) as usize;
                ycoord = (y_offset - ((TILE_SIZE / 2) as f32) + MAX_Y) as usize;
                //set_collide(room_manager, xcoord, ycoord, 2);
            }
            y_offset += TILE_SIZE as f32;
        }

        y_offset = -max_y + ((TILE_SIZE / 2) as f32);
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
    mut enemies: Query<&mut Transform, (With<Enemy>, Without<Player>)>, 
    mut room: Query<&mut Transform, (Without<Player>, Without<Enemy>)>,
    mut room_manager: ResMut<RoomManager>,
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
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

    let mut hit_door: bool = false;

    let (room_width, room_height) = room_manager.current_room_size();

    // Calculate new player position and clamp within room boundaries
    let new_pos_x = (pt.translation.x + change.x).clamp(-room_width / 2.0 + TILE_SIZE as f32 / 2.0, room_width / 2.0 - TILE_SIZE as f32 / 2.0);
    let new_pos_y = (pt.translation.y + change.y).clamp(-room_height / 2.0 + TILE_SIZE as f32 / 2.0, room_height / 2.0 - TILE_SIZE as f32 / 2.0);

    pt.translation.x = new_pos_x;
    pt.translation.y = new_pos_y;


    // take care of horizontal and vertical movement + enemy collision check
    handle_movement_and_enemy_collisions(
        &mut pt, 
        change, 
        &mut hit_door, 
        &mut enemies,
        &mut room_manager, 
    );


    // if we hit a door
    if hit_door {
        println!("hit door!");
        transition_map(&mut room, &mut pt);
    }
}


fn handle_movement_and_enemy_collisions(
    pt: &mut Transform,
    change: Vec2,
    hit_door: &mut bool,
    enemies: &mut Query<&mut Transform, (With<Enemy>, Without<Player>)>, 
    room_manager: &mut RoomManager,
) {
    // Calculate new player position
    let new_pos = pt.translation + Vec3::new(change.x, change.y, 0.0);
    let player_aabb = Aabb::new(new_pos, Vec2::splat(TILE_SIZE as f32));

    // Translate player position to grid indices
    let (topleft, topright, bottomleft, bottomright) = translate_coords_to_grid(&player_aabb, room_manager);

     // Translate player position to grid indices
     let grid_x = (new_pos.x / TILE_SIZE as f32).floor();
     let grid_y = (new_pos.y / TILE_SIZE as f32).floor();
     println!("Player grid position: x = {}, y = {}", grid_x, grid_y);

    // Handle collisions and movement within the grid
    handle_movement(pt, Vec3::new(change.x, 0., 0.), room_manager, hit_door, enemies);
    handle_movement(pt, Vec3::new(0., change.y, 0.), room_manager, hit_door, enemies);
}


fn handle_movement(
    pt: &mut Transform,
    change: Vec3,
    room_manager: &mut RoomManager,
    hit_door: &mut bool,
    enemies: &mut Query<&mut Transform, (With<Enemy>, Without<Player>)>,
) {
    let new_pos = pt.translation + change;
    let player_aabb = Aabb::new(new_pos, Vec2::splat(TILE_SIZE as f32));

    // Get the current room's grid size (room width and height)
    let current_grid = room_manager.current_grid();
    let room_width = current_grid.len() as f32 * TILE_SIZE as f32;
    let room_height = current_grid[0].len() as f32 * TILE_SIZE as f32;

    let (topleft, topright, bottomleft, bottomright) = translate_coords_to_grid(&player_aabb, room_manager);

    // check for collisions with enemies
    for enemy_transform in enemies.iter() {
        let enemy_aabb = Aabb::new(enemy_transform.translation, Vec2::splat(TILE_SIZE as f32));
        if player_aabb.intersects(&enemy_aabb) {
            // handle enemy collision here (if necessary)
            return;
        }
    }

    // movement within bounds and wall/door collision check
    if new_pos.x >= -room_width / 2.0 + TILE_SIZE as f32 / 2. &&
        new_pos.x <= room_width / 2.0 - TILE_SIZE as f32 / 2. &&
        new_pos.y >= -room_height / 2.0 + TILE_SIZE as f32 / 2. &&
        new_pos.y <= room_height / 2.0 - TILE_SIZE as f32 / 2. &&
        topleft != 1 && topright != 1 && bottomleft != 1 && bottomright != 1
    {
        pt.translation = new_pos;
    }

    // check for door transition
    if topleft == 2 || topright == 2 || bottomleft == 2 || bottomright == 2 {
        *hit_door = true;
    }
}


fn translate_coords_to_grid(aabb: &Aabb, room_manager: &mut RoomManager) -> (u32, u32, u32, u32){
    // get the current room's grid size
    let current_grid = room_manager.current_grid();
    let room_width = current_grid.len() as f32 * TILE_SIZE as f32;
    let room_height = current_grid[0].len() as f32 * TILE_SIZE as f32;

    let max_x = room_width / 2.0;
    let max_y = room_height / 2.0;

    // Calculate the grid indices for the player's bounding box corners
    let arr_x_max = ((aabb.max.x + max_x) / TILE_SIZE as f32).floor().clamp(0., (current_grid.len() - 1) as f32);
    let arr_x_min = ((aabb.min.x + max_x) / TILE_SIZE as f32).floor().clamp(0., (current_grid.len() - 1) as f32);
    let arr_y_max = ((aabb.max.y + max_y) / TILE_SIZE as f32).floor().clamp(0., (current_grid[0].len() - 1) as f32);
    let arr_y_min = ((aabb.min.y + max_y) / TILE_SIZE as f32).floor().clamp(0., (current_grid[0].len() - 1) as f32);

    let topleft = current_grid[arr_x_min as usize][arr_y_max as usize];
    let topright = current_grid[arr_x_max as usize][arr_y_max as usize];
    let bottomleft = current_grid[arr_x_min as usize][arr_y_min as usize];
    let bottomright = current_grid[arr_x_max as usize][arr_y_min as usize];

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

fn enemy_movement(
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

fn player_attack(
    time: Res<Time>,
    input: Res<ButtonInput<MouseButton>>,
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
     * 0 - 3 = up
     * 4 - 7 = down
     * 8 - 11 = right
     * 12 - 15 = left
     * ratlas. heh. get it.*/
     let (v, mut ratlas, mut timer, _frame_count) = player.single_mut();

     let abx = v.velocity.x.abs();
     let aby = v.velocity.y.abs();

     if input.just_pressed(MouseButton::Left)
     {
        println!("SWINGING");
        unsafe{attacking = true;} //set attacking to true to override movement animations
        
        // deciding initial frame for swing (so not partial animation)
        if abx > aby {
            if v.velocity.x >= 0.{ratlas.index = 8;}
            else if v.velocity.x < 0. {ratlas.index = 12;}
        }
        else {
            if v.velocity.y >= 0.{ratlas.index = 0;}
            else if v.velocity.y < 0. {ratlas.index = 4;}
        }

        timer.reset();
     }
    unsafe{if attacking == true
    {
        timer.tick(time.delta());

        if abx > aby {
            if v.velocity.x >= 0.{
                if timer.finished(){ratlas.index = ((ratlas.index + 1) % 4) + 8;}
                if ratlas.index == 11{attacking = false; ratlas.index = 24} //allow for movement anims after last swing frame
            }
            else if v.velocity.x < 0. {
                if timer.finished(){ratlas.index = ((ratlas.index + 1) % 4) + 12;}
                if ratlas.index == 15{attacking = false; ratlas.index = 28} //allow for movement anims after last swing frame
            }
        }
        else {
            if v.velocity.y >= 0.{
                if timer.finished(){ratlas.index = (ratlas.index + 1) % 4;}
                if ratlas.index == 3{attacking = false; ratlas.index = 16} //allow for movement anims after last swing frame
            }
            else if v.velocity.y < 0. {
                if timer.finished(){ratlas.index = ((ratlas.index + 1) % 4) + 4;}
                if ratlas.index == 7{attacking = false; ratlas.index = 20} //allow for movement anims after last swing frame
            }
        }
    }}
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
     * 16 - 19 = up
     * 20 - 23 = down
     * 24 - 27 = right
     * 28 - 31 = left
     * ratlas. heh. get it.*/
    let (v, mut ratlas, mut timer, _frame_count) = player.single_mut();
    unsafe{if attacking == true{return;}} //checking if attack animations are running
    //if v.velocity.cmpne(Vec2::ZERO).any() {
        timer.tick(time.delta());

        let abx = v.velocity.x.abs();
        let aby = v.velocity.y.abs();

        if abx > aby {
            if v.velocity.x > 0.{
                if timer.finished(){ratlas.index = ((ratlas.index + 1) % 4) + 24;}
            }
            else if v.velocity.x < 0. {
                if timer.finished(){ratlas.index = ((ratlas.index + 1) % 4) + 28;}
            }
        }
        else {
            if v.velocity.y > 0.{
                if timer.finished(){ratlas.index = ((ratlas.index + 1) % 4) + 16;}
            }
            else if v.velocity.y < 0. {
                if timer.finished(){ratlas.index = ((ratlas.index + 1) % 4) + 20;}
            }
        }
    //}
}

fn move_camera(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (Without<Player>, With<Camera>)>,
    room_manager: Res<RoomManager>, // Access the RoomManager to get the room-specific max_x and max_y
) {
    let pt = player.single();
    let mut ct = camera.single_mut();

    // Retrieve the dynamically calculated max_x and max_y from RoomManager
    let (max_x, max_y) = room_manager.current_room_max();

    ct.translation.x = pt.translation.x.clamp(-max_x + (WIN_W / 2.), max_x - (WIN_W / 2.));
    ct.translation.y = pt.translation.y.clamp(-max_y + (WIN_H / 2.), max_y - (WIN_H / 2.));
}

fn recv_packet(
    socket: Res<UDP>
){
    let mut buf = [0;1024];
    let (_amt, _src) = socket.socket.recv_from(&mut buf).unwrap();
    //println!("{}", String::from_utf8_lossy(&buf));
}

fn send_packet(
    socket: Res<UDP>,
) {
    socket.socket.send_to(b"boo!", "localhost:5001").unwrap();
}

fn send_movement_info(
    socket: Res<UDP>, // defined in setup
    player: Query<&Transform, With<Player>>, // player transform
    
) { // consencus algs    
    let pt = player.single(); // get player transform
    let x = pt.translation.x;
    let y = pt.translation.y;
    let xInt = unsafe {x.to_int_unchecked::<u8>()};
    let yInt = unsafe {y.to_int_unchecked::<u8>()};
    let buf:[u8;2] = [xInt, yInt];
    //print!("{:?}", &buf);

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