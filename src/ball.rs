use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_rapier2d::prelude::*;

use crate::{bricks::Brick, paddle::Paddle, score::Score, walls::Death, AppState, GameConfig};

// const BALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
// const BALL_BORDER_COLOR: Color = Color::rgb(0.05, 0.05, 0.05);
const BALL_COLOR: Color = Color::hsl(0.0, 1.0, 0.75);
const BALL_BORDER_COLOR: Color = Color::hsl(0.0, 1.0, 0.1);
const BALL_BORDER_WIDTH: f32 = 2.0;
// const BALL_STARTING_POSITION: Vec3 = Vec3::new(-200.0, -170.0, 1.0);
// const BALL_RADIUS: f32 = 30.0;
// const BALL_INITIAL_SPEED: f32 = 50.0;
// const BALL_SPEED: f32 = 400.0;
// const BALL_INITIAL_ANGLE: f32 = -45.0 * DEGREE_TO_RADIAN_FACTOR;
// const SCORE_TO_BALL_SPEED_FACTOR: f32 = 0.4;
// const BALL_MAX_AIR_TIME: f32 = 1.0;

#[derive(Component, Debug)]
pub struct Ball;

#[derive(Component, Debug)]
pub struct BallTopSpeed {
    speed: f32,
}
#[derive(Resource, Debug)]
struct BallAirTime(f32);

pub struct BallPlugin;

impl Plugin for BallPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BallAirTime(0.0))
            .add_systems(OnEnter(AppState::InGame), spawn_ball)
            .add_systems(OnExit(AppState::InGame), despawn_ball)
            .add_systems(Update, ball_collision.run_if(in_state(AppState::InGame)))
            .add_systems(
                Update,
                update_ball_air_time.run_if(in_state(AppState::InGame)),
            );
    }
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game_config: Res<GameConfig>,
) {
    commands
        .spawn((
            Ball,
            BallTopSpeed {
                speed: game_config.ball_speed,
            },
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::default().into()).into(),
                material: materials.add(ColorMaterial::from(BALL_BORDER_COLOR)),
                transform: Transform::from_translation(game_config.get_ball_starting_position())
                    .with_scale(Vec3 {
                        x: game_config.ball_diameter,
                        y: game_config.ball_diameter,
                        z: 2.0,
                    }),
                ..default()
            },
            RigidBody::Dynamic,
            Velocity {
                linvel: game_config.get_ball_initial_linvel(),
                ..default()
            },
            Collider::ball(0.5),
            Friction::coefficient(0.0),
            Restitution::coefficient(1.0),
            GravityScale(0.0),
            Ccd::enabled(),
            ActiveEvents::COLLISION_EVENTS,
        ))
        .with_children(|parent| {
            parent.spawn(MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::default().into()).into(),
                material: materials.add(ColorMaterial::from(BALL_COLOR)),
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 3.0),
                    scale: Vec3 {
                        x: (game_config.ball_diameter - BALL_BORDER_WIDTH * 2.0)
                            / game_config.ball_diameter,
                        y: (game_config.ball_diameter - BALL_BORDER_WIDTH * 2.0)
                            / game_config.ball_diameter,
                        z: 1.0,
                    },
                    ..default()
                },
                ..default()
            });
        });
}

fn despawn_ball(
    mut commands: Commands,
    ball_query: Query<Entity, With<Ball>>,
    mut ball_air_time: ResMut<BallAirTime>,
) {
    let ball_entity = ball_query.single();
    commands.entity(ball_entity).despawn_recursive();
    ball_air_time.0 = 0.0;
}

fn update_ball_air_time(
    time: Res<Time>,
    mut ball_air_time: ResMut<BallAirTime>,
    mut ball_gravity_query: Query<(&mut GravityScale, &mut Restitution), With<Ball>>,
    game_config: Res<GameConfig>,
) {
    ball_air_time.0 += time.delta_seconds();
    let (mut ball_gravity, mut ball_restitution) = ball_gravity_query.single_mut();
    (ball_gravity.0, ball_restitution.coefficient) =
        if ball_air_time.0 > game_config.ball_anti_gravity_time {
            (game_config.ball_gravity_scale, game_config.ball_restitution)
        } else {
            (0.0, 1.0)
        };
}

fn ball_collision(
    mut commands: Commands,
    mut ball_query: Query<(Entity, &mut Velocity, &mut BallTopSpeed)>,
    paddle_query: Query<With<Paddle>>,
    brick_query: Query<&Brick>,
    death_wall_query: Query<With<Death>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut score: ResMut<Score>,
    mut collision_events: EventReader<CollisionEvent>,
    mut ball_air_time: ResMut<BallAirTime>,
    game_config: Res<GameConfig>,
) {
    let Ok((ball_entity, mut ball_velocity, mut ball_top_speed)) = ball_query.get_single_mut()
    else {
        return;
    };

    for collision_event in collision_events.read() {
        if let CollisionEvent::Stopped(entity1, entity2, _) = collision_event {
            let entity_pair = if ball_entity == *entity1 {
                Some((entity1, entity2))
            } else if ball_entity == *entity2 {
                Some((entity2, entity1))
            } else {
                None
            };
            if let Some((_, other_entity)) = entity_pair {
                // println!(
                //     "ball speed: {}, top speed: {}",
                //     ball_velocity.linvel.length(),
                //     ball_top_speed.speed
                // );
                if let Ok(()) = paddle_query.get(*other_entity) {
                    ball_velocity.linvel =
                        ball_velocity.linvel.normalize_or_zero() * ball_top_speed.speed;
                    ball_air_time.0 = 0.0;
                    continue;
                }
                if let Ok(brick) = brick_query.get(*other_entity) {
                    score.score += brick.score;
                    ball_top_speed.speed +=
                        brick.score as f32 * game_config.score_to_ball_speed_factor;
                    commands.entity(*other_entity).despawn_recursive();
                    if brick_query.iter().take(2).count() == 1 {
                        score.score += game_config.win_score_bonus;
                        next_state.set(AppState::GaveOver);
                    }
                    continue;
                }
                if let Ok(()) = death_wall_query.get(*other_entity) {
                    next_state.set(AppState::GaveOver);
                    continue;
                }
            }
        }
    }
}
