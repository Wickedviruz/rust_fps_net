use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::{camera_controller::CameraController, input::*, player::Player};

pub fn update_movement_input(
    keys : Res<ButtonInput<KeyCode>>,
    mut input : ResMut<PlayerInput>,
){
    input.movement = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyW){
        input.movement.x += 1.;
    }
    if keys.pressed(KeyCode::KeyA){
        input.movement.y -= 1.;
    }
    if keys.pressed(KeyCode::KeyS){
        input.movement.x -= 1.;
    }
    if keys.pressed(KeyCode::KeyD){
        input.movement.y += 1.;
    }

    input.jump = keys.pressed(KeyCode::Space);
    input.crouch = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
}

pub fn update_movement(
    time : Res<Time<Fixed>>,
    input : Res<PlayerInput>,
    camera_query : Query<&CameraController>,
    mut player_query : Query<(
        &mut Player, 
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>
    )>,
    mut cam_transforms: Query<&mut Transform, With<Camera>>,
){
    let camera = camera_query.get_single().unwrap();

    for(mut player,mut controller,controller_output) in player_query.iter_mut(){
        if let Some(output) = controller_output{
            if output.grounded{
                player.velocity = Vec3::ZERO;
            }
            if input.jump && output.grounded {
                player.velocity.y = 8.0;
            } 
        }
        let camera_rotation_converted = -camera.rotation.y.to_radians() - 90.0_f32.to_radians();

        let forward = Vec2::new(
            f32::cos(camera_rotation_converted),
            f32::sin(camera_rotation_converted)
        );

        let right = Vec2::new(-forward.y,forward.x);

        if let Some(movement_direction) = (forward*input.movement.x + right*input.movement.y).try_normalize(){
            let speed = player.current_speed(input.crouch);
            player.velocity.x = movement_direction.x*speed;
            player.velocity.z = movement_direction.y*speed;
        }

        // Gravitation
        player.velocity.y -= player.gravity*time.timestep().as_secs_f32();
        
        let target_height = if input.crouch {
            player.crouch_height
        } else {
            player.stand_height
        };

        // smooth transition (lerp)
        player.eye_height = player.eye_height + (target_height - player.eye_height) * 10.0 * time.timestep().as_secs_f32();

        // flytta kameran till rätt höjd
        if let Ok(mut cam_transform) = cam_transforms.get_single_mut() {
            cam_transform.translation.y = player.eye_height;
        }

        // collider offset (för att kännas lägre)
        controller.offset = if input.crouch {
            CharacterLength::Absolute(0.5)
        } else {
            CharacterLength::Absolute(0.01)
        };

        // Flytta spelaren
        controller.translation = Some(player.velocity*time.timestep().as_secs_f32());
    }
}