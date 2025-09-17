use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::{camera_controller, input::*, player_movement::*, player_shooting::{update_player, TracerSpawnSpot}};
use crate::game::{math::coordinates::blender_to_world, shooting};
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(shooting::tracer::TracerPlugin)
            .init_resource::<PlayerInput>()
            .add_systems(
                Update,
                (
                    update_movement_input,
                    update_player, 
                    camera_controller::update_camera_controller
                ),
            )
            //physics timestep
            .add_systems(FixedUpdate, update_movement)
            .add_systems(Startup, init_player);
    }
}

#[derive(Component)]
pub struct Player {
    pub velocity : Vec3,
    pub gravity : f32,
    pub eye_height: f32,        
    pub stand_height: f32,      
    pub crouch_height: f32,
    pub base_speed: f32,
    pub weapon: WeaponType, 
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub armor: bool,
    pub helmet: bool,
}

#[derive(Component)]
pub struct Economy {
    pub money: i32,
}

#[derive(Component)]
pub struct Inventory {
    pub primary: Option<WeaponType>,
    pub secondary: Option<WeaponType>,
    pub knife: WeaponType,
    pub grenades: Vec<WeaponType>,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponType {
    Knife,
    Pistol,
    Rifle,
    Sniper,
}

#[derive(Component)]
pub struct Hitbox {
    pub part: HitboxPart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitboxPart {
    Head,
    Body,
    Legs,
}

// helper
impl Player {
    pub fn current_speed(&self, crouching: bool) -> f32 {
        let weapon_speed = match self.weapon {
            WeaponType::Knife => 5.0,   // ~250 u/s
            WeaponType::Pistol => 4.8, // ~240 u/s
            WeaponType::Rifle => 4.1,  // ~215 u/s
            WeaponType::Sniper => 3.9, // ~200 u/s
        };

        if crouching {
            weapon_speed * 0.55 // crouch sänker farten
        } else {
            weapon_speed
        }
    }
}

fn init_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let fov = 103.0_f32.to_radians();
    let camera_entity = commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 1.7, 0.0),
            projection: Projection::Perspective(PerspectiveProjection {
                fov: fov,
                ..default()
            }),
            ..default()
        },
        camera_controller::CameraController {
            sensitivity: 0.035,
            rotation: Vec2::ZERO,
            rotation_lock: 88.0,
        },
    )).id();
    let gun_model = asset_server.load("models/ak.glb#Scene0");
    let gun_entity = commands.spawn(
        SceneBundle{
            scene : gun_model,
            transform : Transform::IDENTITY,
            ..Default::default()
        }
    ).id();
    let spawn_spot = blender_to_world(Vec3::new(0.530462,2.10557,-0.466568));
    let tracer_spawn_entity = commands.spawn(
        (
            TransformBundle{
                local : Transform::from_translation(spawn_spot),
                ..Default::default()
            },
            TracerSpawnSpot
        )
    ).id();
    let player_entity = commands.spawn((
        Player {
            velocity : Vec3::ZERO,
            gravity : 20.0,
            base_speed : 5.0,
            eye_height: 1.7,
            stand_height: 1.7,
            crouch_height: 1.2,
            weapon: WeaponType::Rifle 
        },
        SpatialBundle{
            transform : Transform::from_translation(Vec3::new(0., 30., 0.)),
            ..Default::default()
        },
        Collider::capsule_y(0.8, 0.3),
        RigidBody::KinematicPositionBased,
        KinematicCharacterController{
            up : Vec3::Y,
            offset : CharacterLength::Absolute(0.01),
            ..default()
        },
    ))
.with_children(|parent| {
    // Head hitbox (boll)
    parent.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Sphere { radius: 0.25 })),
            material: materials.add(Color::srgb(1.0, 0.0, 0.0)), // röd
            transform: Transform::from_xyz(0.0, 1.8, 0.0),
            ..default()
        },
        Collider::ball(0.25),
        Hitbox { part: HitboxPart::Head },
    ));

    // Body hitbox (capsule)
    parent.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Capsule3d::new(0.3, 0.9))),
            material: materials.add(Color::srgb(0.0, 1.0, 0.0)), // grön
            transform: Transform::from_xyz(0.0, 0.9, 0.0),
            ..default()
        },
        Collider::capsule_y(0.9, 0.3),
        Hitbox { part: HitboxPart::Body },
    ));

    // Legs hitbox (kub)
    parent.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Cuboid::new(0.3, 0.5, 0.3))),
            material: materials.add(Color::srgb(0.0, 0.0, 1.0)), // blå
            transform: Transform::from_xyz(0.0, 0.3, 0.0),
            ..default()
        },
        Collider::cuboid(0.3, 0.5, 0.3),
        Hitbox { part: HitboxPart::Legs },
    ));
})
    .id();
    commands.entity(camera_entity).push_children(&[tracer_spawn_entity,gun_entity]);
    commands.entity(player_entity).add_child(camera_entity);
}

//fn apply_damage(commands: &mut Commands, entity: Entity, base_damage: f32) {
    // här kan du hämta & mutera Health
    // t.ex. om armor är aktiv: halvera skadan på body
    // om helmet + headshot: reducera damage
//}