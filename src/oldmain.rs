use bevy::prelude::*;       
use bevy::window::CursorGrabMode;
use bevy::input::mouse::MouseMotion;
use bevy_rapier3d::prelude::{RapierContext, RapierPhysicsPlugin, NoUserData, RigidBody, Collider, KinematicCharacterController, Velocity, Friction, CoefficientCombineRule, QueryFilter, KinematicCharacterControllerOutput};
//use bevy_rapier3d::plugin::{RapierPhysicsPlugin};
use bevy_renet::{RenetClientPlugin, RenetServerPlugin};
use renet::{ConnectionConfig, RenetClient, RenetServer};
use serde::{Deserialize, Serialize};
use rand::Rng;
use bincode::{config, Decode, Encode};
use std::net::UdpSocket;


const PLAYER_HEIGHT: f32 = 1.8;
const MOVE_SPEED: f32 = 5.0;

// === Components & Resources ===
#[derive(Component)]
struct LocalPlayer;

#[derive(Component)]
struct Target;

#[derive(Resource, Default)]
struct MouseState {
    yaw: f32,
    pitch: f32,
    locked: bool,
}

pub const CH_RELIABLE: u8 = 0;
pub const CH_UNRELIABLE: u8 = 1;

#[derive(Resource, Debug, Serialize, Deserialize, Encode, Decode, Clone)]
struct ClientInput {
    forward: f32,
    right: f32,
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode, Clone)]
struct ServerSnapshot {
    pos: [f32; 3],
}

#[derive(Component, Default)]
struct Grounded(bool);


// === Entry point ===
fn main() {
    let is_server: bool = std::env::args().any(|a: String| a == "--server");
    let is_client: bool = std::env::args().any(|a: String| a == "--client");

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb_u8(15, 15, 20)))
        .insert_resource(MouseState::default())
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Rust FPS Net".into(),
                    ..default()
                }),
                ..default()
            }),
            RapierPhysicsPlugin::<NoUserData>::default(),
        ));

    if is_server {
        app.add_plugins(RenetServerPlugin)
            .add_systems(Startup, (setup_scene, server_startup))
            .add_systems(Update, server_receive_inputs)
            .add_systems(Update, server_broadcast);
    } else if is_client {
        app.add_plugins(RenetClientPlugin)
            .add_systems(Startup, (setup_scene, spawn_player, client_startup))
            .add_systems(Update, toggle_mouse_lock)
            .add_systems(Update, mouse_look)
            .add_systems(Update, collect_input)
            .add_systems(Update, client_send)
            .add_systems(Update, client_receive);
    } else {
        app.add_systems(Startup, (setup_scene, spawn_player))
            .add_systems(Update, toggle_mouse_lock)
            .add_systems(Update, mouse_look)
            .add_systems(Update, player_movement_sp)
            .add_systems(PostUpdate, update_grounded_state)
            .add_systems(Update, shooting_system);
           }

    app.run();
}

// === Scene setup ===
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ljus
    commands.spawn((
    PointLight {
        intensity: 3500.0,
        range: 60.0,
        ..default()
    },
    Transform::from_xyz(6.0, 10.0, 6.0),
));

commands.spawn((
    Mesh3d(meshes.add(Mesh::from(bevy::math::primitives::Cuboid::new(
        60.0, 0.4, 60.0,
    )))),
    MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.13, 0.15),
        ..default()
    })),
    Transform::from_xyz(0.0, -0.2, 0.0),
    RigidBody::Fixed,
    Collider::cuboid(30.0, 0.2, 30.0),
));

// lådor
let mut rng = rand::rng();
for _ in 0..10 {
    let x = rng.random_range(-10.0..10.0);
    let z = rng.random_range(-10.0..10.0);
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(bevy::math::primitives::Cuboid::new(
            1.0, 1.0, 1.0,
        )))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.2, 0.2),
            ..default()
        })),
        Transform::from_xyz(x, 0.5, z),
        Collider::cuboid(0.5, 0.5, 0.5),
        Target,
    ));
}}

fn spawn_player(mut commands: Commands) {
    // Skapa spelarkropp
    let player = commands
        .spawn((
            RigidBody::Dynamic,
            Collider::capsule_y(PLAYER_HEIGHT / 2.0, 0.3),
            KinematicCharacterController::default(),
            Velocity::default(),
            Friction {
                coefficient: 5.0, // hög friktion så man inte glider
                combine_rule: CoefficientCombineRule::Multiply,
            },
            Grounded::default(),
            LocalPlayer,
            Transform::from_xyz(0.0, PLAYER_HEIGHT, 6.0),
        ))
        .id();

    // Kamera som barn (pitch separat från kropp)
    commands.entity(player).with_children(|parent| {
        parent.spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 0.6 * PLAYER_HEIGHT, 0.0),
        ));
    });
}

// === Input & Camera ===
fn toggle_mouse_lock(
    mut windows: Query<&mut Window>,
    mut mouse: ResMut<MouseState>,
    kb: Res<ButtonInput<KeyCode>>,
) {
    if kb.just_pressed(KeyCode::Escape) {
        mouse.locked = !mouse.locked;
        let mut w = windows.single_mut().unwrap();
        w.cursor_options.visible = !mouse.locked;
        w.cursor_options.grab_mode = if mouse.locked {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
    }
}

fn mouse_look(
    mut ev: EventReader<MouseMotion>,
    mut q_player: Query<&mut Transform, (With<LocalPlayer>, Without<Camera3d>)>,
    mut q_cam: Query<&mut Transform, With<Camera3d>>,
    mut mouse: ResMut<MouseState>,
) {
    if !mouse.locked {
        return;
    }

    for e in ev.read() {
        mouse.yaw -= e.delta.x * 0.002;
        mouse.pitch -= e.delta.y * 0.002;
        mouse.pitch = mouse.pitch.clamp(-1.5, 1.5);
    }

    // Spelaren får bara yaw
    if let Ok(mut player_t) = q_player.single_mut() {
        player_t.rotation = Quat::from_axis_angle(Vec3::Y, mouse.yaw);
    }
    // Kameran får bara pitch
    if let Ok(mut cam_t) = q_cam.single_mut() {
        cam_t.rotation = Quat::from_axis_angle(Vec3::X, mouse.pitch);
    }
}


fn player_movement_sp(
    kb: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut q: Query<(&mut KinematicCharacterController, &mut Velocity, &Transform, &Grounded), With<LocalPlayer>>,
) {
    if let Ok((mut controller, mut velocity, player_t, grounded)) = q.single_mut() {
        let forward = (kb.pressed(KeyCode::KeyW) as i8 - kb.pressed(KeyCode::KeyS) as i8) as f32;
        let right = (kb.pressed(KeyCode::KeyD) as i8 - kb.pressed(KeyCode::KeyA) as i8) as f32;

        let rot = player_t.rotation;
        let dir = (rot * Vec3::new(right, 0.0, -forward)).normalize_or_zero();

        // Horisontell rörelse
        let movement = dir * MOVE_SPEED * time.delta_secs();
        controller.translation = Some(movement);

        // Hoppa (ger direkt velocity uppåt)
        if kb.just_pressed(KeyCode::Space) && grounded.0 {
            velocity.linvel.y = 5.0;
        }

        // Om ingen input → stoppa glid
        if forward == 0.0 && right == 0.0 {
            velocity.linvel.x = 0.0;
            velocity.linvel.z = 0.0;
        }
    }
}

fn update_grounded_state(
    mut q: Query<(&mut Grounded, &KinematicCharacterControllerOutput), With<LocalPlayer>>,
) {
    for (mut grounded, output) in q.iter_mut() {
        grounded.0 = output.grounded;
    }
}

fn shooting_system(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    q_cam: Query<&GlobalTransform, With<Camera3d>>,
    rapier_context: Res<RapierContext>,
    targets: Query<Entity, With<Target>>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    if let Ok(cam_t) = q_cam.single() {
        let origin: Vec3 = cam_t.translation();
        let dir: Vec3 = cam_t.forward().into();

        if let Some((entity, toi)) =
            rapier_context.cast_ray(origin, dir, 100.0, true, QueryFilter::default())
        {
            if targets.get(entity).is_ok() {
                println!("Träffade target på avstånd {:?}", toi);
                commands.entity(entity).despawn();
            }
        }
    }
}

// === Server ===
fn server_startup(mut commands: Commands) {
    let socket = UdpSocket::bind(("0.0.0.0", 5000)).unwrap();
    socket.set_nonblocking(true).unwrap();
    let server = RenetServer::new(ConnectionConfig::default());
    commands.insert_resource(server);
}

fn server_receive_inputs(mut server: ResMut<RenetServer>) {
    for client_id in server.clients_id() {
        while let Some(bytes) = server.receive_message(client_id, CH_UNRELIABLE) {
            let (inp, _): (ClientInput, usize) =
                bincode::decode_from_slice(&bytes, config::standard()).unwrap();
            println!("Server input from {:?}: {:?}", client_id, inp);
        }
    }
}

fn server_broadcast(mut server: ResMut<RenetServer>, q: Query<&Transform, With<LocalPlayer>>) {
    if let Ok(t) = q.single() {
        let snap = ServerSnapshot {
            pos: [t.translation.x, t.translation.y, t.translation.z],
        };
        let bytes = bincode::encode_to_vec(&snap, config::standard()).unwrap();
        for id in server.clients_id() {
            server.send_message(id, CH_RELIABLE, bytes.clone());
        }
    }
}

// === Client ===
fn client_startup(mut commands: Commands) {
    let socket = UdpSocket::bind(("0.0.0.0", 0)).unwrap();
    socket.set_nonblocking(true).unwrap();
    let client = RenetClient::new(ConnectionConfig::default());
    commands.insert_resource(client);
}

fn collect_input(kb: Res<ButtonInput<KeyCode>>, mut input_res: ResMut<ClientInput>) {
    let f = (kb.pressed(KeyCode::KeyW) as i8 - kb.pressed(KeyCode::KeyS) as i8) as f32;
    let r = (kb.pressed(KeyCode::KeyD) as i8 - kb.pressed(KeyCode::KeyA) as i8) as f32;
    input_res.forward = f;
    input_res.right = r;
}

fn client_send(mut client: ResMut<RenetClient>, input: Res<ClientInput>) {
    let bytes = bincode::encode_to_vec(&*input, config::standard()).unwrap();
    client.send_message(CH_UNRELIABLE, bytes);
}

fn client_receive(
    mut client: ResMut<RenetClient>,
    mut q: Query<&mut Transform, With<LocalPlayer>>,
) {
    while let Some(bytes) = client.receive_message(CH_RELIABLE) {
        let (snap, _): (ServerSnapshot, usize) =
            bincode::decode_from_slice(&bytes, config::standard()).unwrap();
        if let Ok(mut t) = q.single_mut() {
            t.translation = Vec3::new(snap.pos[0], snap.pos[1], snap.pos[2]);
        }
    }
}
