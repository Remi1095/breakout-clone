
use bevy::{prelude::*, render::render_resource::PrimitiveTopology, sprite::MaterialMesh2dBundle};
use bevy_rapier2d::prelude::*;

use crate::{walls::WallLocation, AppState, GameConfig};

const PADDLE_COLOR: Color = Color::hsl(240.0, 1.0, 0.75);
const PADDLE_BORDER_COLOR: Color = Color::hsl(240.0, 1.0, 0.1);
// const PADDLE_BORDER_COLOR: Color = Color::rgb(0.05, 0.05, 0.05);
const PADDLE_BORDER_WIDTH: f32 = 2.0;
// pub const PADDLE_STARTING_POSITION: Vec3 = Vec3::new(0.0, -360.0, 1.0);
// const PADDLE_CHORD_LENGTH: f32 = 150.0;
// const PADDLE_HEIGHT: f32 = 30.0;
// const PADDLE_MESH_SEGMENTS: i32 = 32;
// const PADDLE_COLLIDER_SEGMENTS: i32 = 5;
// const PADDLE_MAX_SPEED: f32 = 750.0;
// const PADDLE_ACCELERATION: f32 = 6000.0;

#[derive(Component, Debug)]
pub struct Paddle;

#[derive(Component, Debug)]
pub struct HorizontalVelocity {
    pub value: f32,
}

impl Default for HorizontalVelocity {
    fn default() -> Self {
        HorizontalVelocity { value: 0.0 }
    }
}

pub struct PaddlePlugin;

impl Plugin for PaddlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_paddle)
            .add_systems(
                Update,
                paddle_movement_controls.run_if(in_state(AppState::InGame)),
            );
    }
}

fn spawn_paddle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    paddle_query: Query<Entity, With<Paddle>>,
    game_config: Res<GameConfig>,
) {
    for paddle_entity in paddle_query.iter() {
        commands.entity(paddle_entity).despawn_recursive();
    }

    // https://en.wikipedia.org/wiki/Circular_segment
    let paddle_mesh: Mesh = if game_config.paddle_mesh_segments > 1 {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let mut v_pos_mesh: Vec<Vec3> = Vec::new();
        for i in 0..game_config.paddle_mesh_segments {
            v_pos_mesh.push(
                game_config
                    .get_paddle_segment_point(i, game_config.paddle_mesh_segments)
                    .extend(0.0),
            );
            v_pos_mesh.push(
                game_config
                    .get_paddle_segment_point(i + 1, game_config.paddle_mesh_segments)
                    .extend(0.0),
            );
            v_pos_mesh.push(Vec3::ZERO);
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_pos_mesh);
        mesh
    } else {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                Vec3::new(-game_config.paddle_width / 2.0, 0.0, 0.0),
                Vec3::new(
                    -game_config.paddle_width / 2.0,
                    game_config.paddle_height,
                    0.0,
                ),
                Vec3::new(game_config.paddle_width / 2.0, 0.0, 0.0),
                Vec3::new(
                    game_config.paddle_width / 2.0,
                    game_config.paddle_height,
                    0.0,
                ),
            ],
        );
        mesh
    };

    let v_pos_collider = if game_config.paddle_collider_segments > 1 {
        let mut v_pos: Vec<Vec2> = Vec::new();
        for i in 0..=game_config.paddle_collider_segments {
            v_pos.push(
                game_config.get_paddle_segment_point(i, game_config.paddle_collider_segments),
            );
        }
        // v_pos_collider.push(get_segment_point(PADDLE_COLLIDER_SEGMENTS, PADDLE_COLLIDER_SEGMENTS) - Vec2::new(0.0, 50.0));
        // v_pos_collider.push(get_segment_point(0, PADDLE_COLLIDER_SEGMENTS) - Vec2::new(0.0, 50.0));
        v_pos.push(game_config.get_paddle_segment_point(0, game_config.paddle_collider_segments));
        v_pos
    } else {
        vec![
            Vec2::new(-game_config.paddle_width / 2.0, 0.0),
            Vec2::new(game_config.paddle_width / 2.0, 0.0),
            Vec2::new(game_config.paddle_width / 2.0, game_config.paddle_height),
            Vec2::new(-game_config.paddle_width / 2.0, game_config.paddle_height),
            Vec2::new(-game_config.paddle_width / 2.0, 0.0),
        ]
    };

    commands
        .spawn((
            Paddle,
            HorizontalVelocity::default(),
            MaterialMesh2dBundle {
                mesh: meshes.add(paddle_mesh.clone()).into(),
                material: materials.add(ColorMaterial::from(PADDLE_BORDER_COLOR)),
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    game_config.paddle_bottom_margin - game_config.area_height / 2.0,
                    2.0,
                )),
                ..default()
            },
            RigidBody::KinematicPositionBased,
            Collider::polyline(v_pos_collider, None),
            Friction::coefficient(0.0),
            Restitution::coefficient(1.0),
            Ccd::enabled(),
        ))
        .with_children(|parent| {
            parent.spawn(MaterialMesh2dBundle {
                mesh: meshes.add(paddle_mesh).into(),
                material: materials.add(ColorMaterial::from(PADDLE_COLOR)),
                transform: Transform {
                    translation: Vec3::new(0.0, PADDLE_BORDER_WIDTH, 3.0),
                    scale: Vec3 {
                        x: (game_config.paddle_width
                            - PADDLE_BORDER_WIDTH
                                * if game_config.paddle_mesh_segments > 1 {
                                    6.0
                                } else {
                                    2.0
                                })
                            / game_config.paddle_width,
                        y: (game_config.paddle_height - PADDLE_BORDER_WIDTH * 2.0)
                            / game_config.paddle_height,
                        z: 1.0,
                    },
                    ..default()
                },
                ..default()
            });
        });
}

fn paddle_movement_controls(
    mut paddle_query: Query<(&mut HorizontalVelocity, &mut Transform)>,
    walls_query: Query<(&Transform, &WallLocation), Without<HorizontalVelocity>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    game_config: Res<GameConfig>,
) {
    let (mut paddle_velocity, mut paddle_transform) = paddle_query.single_mut();
    let pressed_left = keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A);
    let pressed_right =
        keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D);

    if pressed_left && !pressed_right {
        if paddle_velocity.value > 0.0 {
            paddle_velocity.value = f32::max(
                -game_config.paddle_acceleration * time.delta_seconds(),
                -game_config.paddle_max_speed,
            )
        } else {
            paddle_velocity.value = f32::max(
                paddle_velocity.value - game_config.paddle_acceleration * time.delta_seconds(),
                -game_config.paddle_max_speed,
            )
        }
    } else if pressed_right && !pressed_left {
        if paddle_velocity.value < 0.0 {
            paddle_velocity.value = f32::min(
                game_config.paddle_acceleration * time.delta_seconds(),
                game_config.paddle_max_speed,
            )
        } else {
            paddle_velocity.value = f32::min(
                paddle_velocity.value + game_config.paddle_acceleration * time.delta_seconds(),
                game_config.paddle_max_speed,
            );
        }
    } else {
        paddle_velocity.value = 0.0;
        return;
    }

    paddle_transform.translation.x =
        paddle_transform.translation.x + paddle_velocity.value * time.delta_seconds();

    for (wall_transform, wall_location) in walls_query.iter() {
        match wall_location {
            WallLocation::Left => {
                if paddle_transform.translation.x - game_config.paddle_width / 2.0
                    < wall_transform.translation.x + wall_transform.scale.x / 2.0
                {
                    paddle_transform.translation.x = wall_transform.translation.x
                        + (wall_transform.scale.x + game_config.paddle_width) / 2.0;
                }
            }
            WallLocation::Right => {
                if paddle_transform.translation.x + game_config.paddle_width / 2.0
                    > wall_transform.translation.x - wall_transform.scale.x / 2.0
                {
                    paddle_transform.translation.x = wall_transform.translation.x
                        - (wall_transform.scale.x + game_config.paddle_width) / 2.0;
                }
            }
            _ => {}
        }
    }
}
