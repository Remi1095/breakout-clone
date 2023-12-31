use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{AppState, GameConfig};

// const WALL_THICKNESS: f32 = 10.0;
// // x coordinates
// const LEFT_WALL: f32 = -500.0;
// const RIGHT_WALL: f32 = 500.0;
// // y coordinates
// const BOTTOM_WALL: f32 = -400.0;
// const TOP_WALL: f32 = 400.0;
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const DEATH_WALL_COLOR: Color = Color::rgb(0.5, 0.2, 0.2);

#[derive(Component, Debug, Clone)]
pub enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

#[derive(Component, Debug)]
pub struct Death;

impl WallLocation {
    fn position(&self, game_config: &Res<GameConfig>) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(
                -(game_config.area_width + game_config.wall_thickness) / 2.0,
                0.,
            ),
            WallLocation::Right => Vec2::new(
                (game_config.area_width + game_config.wall_thickness) / 2.0,
                0.,
            ),
            WallLocation::Bottom => Vec2::new(
                0.,
                -(game_config.area_height + game_config.wall_thickness) / 2.0,
            ),
            WallLocation::Top => Vec2::new(
                0.,
                (game_config.area_height + game_config.wall_thickness) / 2.0,
            ),
        }
    }
}

pub struct WallPlugin;

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_walls);
    }
}

fn spawn_walls(mut commands: Commands, game_config: Res<GameConfig>) {
    for wall_location in [
        WallLocation::Left,
        WallLocation::Right,
        WallLocation::Bottom,
        WallLocation::Top,
    ] {
        let position = wall_location.position(&game_config);
        let scale = match wall_location {
            WallLocation::Left | WallLocation::Right => Vec2::new(
                game_config.wall_thickness,
                game_config.area_height + 2.0 * game_config.wall_thickness,
            ),
            WallLocation::Bottom | WallLocation::Top => Vec2::new(
                game_config.area_width + 2.0 * game_config.wall_thickness,
                game_config.wall_thickness,
            ),
        };

        let mut entity_commands = commands.spawn((
            wall_location.clone(),
            SpriteBundle {
                transform: Transform {
                    translation: match wall_location {
                        WallLocation::Bottom => position.extend(0.0),
                        _ => position.extend(1.0),
                    },
                    scale: scale.extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: match wall_location {
                        WallLocation::Bottom => DEATH_WALL_COLOR,
                        _ => WALL_COLOR,
                    },
                    ..default()
                },
                ..default()
            },
            RigidBody::Fixed,
            Collider::cuboid(0.5, 0.5),
            Friction::coefficient(0.0),
            Restitution::coefficient(1.0),
        ));
        if let WallLocation::Bottom = wall_location {
            entity_commands.insert(Death);
        }
    }
}
