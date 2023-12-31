use crate::{ball::*, paddle::HorizontalVelocity};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

const DEBUG_INTERVAL_SECONDS: f32 = 1.0;

#[derive(Resource, Debug)]
pub struct DebugTimer {
    timer: Timer,
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugTimer {
            timer: Timer::from_seconds(DEBUG_INTERVAL_SECONDS, TimerMode::Repeating),
        })
        .add_systems(Update, print_ball_position);
    }
}

fn print_ball_position(
    ball_query: Query<(&Transform, &Velocity), With<Ball>>,
    paddle_query: Query<(&Transform, &HorizontalVelocity)>,
    mut debug_timer: ResMut<DebugTimer>,
    time: Res<Time>,
) {
    // Log the entity ID and translation of each entity with a `Position` component.
    debug_timer.timer.tick(time.delta());
    if !debug_timer.timer.just_finished() {
        return;
    }
    let (ball_transform, ball_velocity) =
        ball_query.get_single().expect("only one ball should exist");
    info!(
        "Ball: position {}, velocity: {}, norm: {:?}",
        ball_transform.translation,
        ball_velocity.linvel,
        ball_velocity.linvel.length(),
    );
    let (transform, paddle) = paddle_query
        .get_single()
        .expect("only one paddle should exist");
    info!(
        "Paddle: position {}, velocity: {}",
        transform.translation, paddle.value,
    );
}
