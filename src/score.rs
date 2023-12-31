use bevy::prelude::*;

use crate::{AppState, GameConfig};

// pub const BRICK_MAX_SCORE: i32 = 10;
// pub const WIN_SCORE_BONUS: i32 = 500;
// const SCORE_LOSS_PER_INTERVAL: i32 = 1;
// const SCORE_LOSS_INTERVAL_SECONDS: f32 = 1.0;

#[derive(Resource, Debug)]
pub struct Score {
    pub score: i32,
}

#[derive(Component, Debug)]
pub struct ScoreText;

#[derive(Resource, Debug)]
struct ScoreLossTimer(Timer);

#[derive(Resource, Debug)]
pub struct GameNumber {
    pub number: usize,
}

#[derive(Component, Debug)]
pub struct FinalScoreDisplay;

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Score { score: 0 })
            .insert_resource(GameNumber { number: 1 })
            .add_systems(OnEnter(AppState::InGame), spawn_score_display)
            .add_systems(
                Update,
                update_score_display.run_if(in_state(AppState::InGame)),
            )
            .add_systems(Update, score_loss.run_if(in_state(AppState::InGame)))
            .add_systems(OnEnter(AppState::GaveOver), spawn_final_score_display)
            .add_systems(OnExit(AppState::GaveOver), spawn_previous_score_display)
            .add_systems(Update, start_next_game);
    }
}

fn spawn_score_display(
    mut commands: Commands,
    score_text_query: Query<With<ScoreText>>,
    game_config: Res<GameConfig>,
) {
    for _ in score_text_query.iter() {
        return;
    }

    commands.spawn((
        TextBundle::from_section(
            format!("Score: 0"),
            TextStyle {
                font_size: 32.0,
                ..default()
            },
        )
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(5.0),
            top: Val::Px(5.0),
            ..default()
        }),
        ScoreText,
    ));

    commands.insert_resource(ScoreLossTimer(Timer::from_seconds(
        game_config.score_loss_interval,
        TimerMode::Repeating,
    )));
}

fn update_score_display(mut query: Query<&mut Text, With<ScoreText>>, score: Res<Score>) {
    for mut text in query.iter_mut() {
        // Update the text based on the score resource
        text.sections[0].value = format!("Score: {}", score.score);
    }
}

fn score_loss(
    time: Res<Time>,
    mut score: ResMut<Score>,
    mut timer: ResMut<ScoreLossTimer>,
    game_config: Res<GameConfig>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        score.score -= game_config.score_loss;
    }
}

fn spawn_final_score_display(mut commands: Commands, score: Res<Score>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section(
                    format!("Final Score: {}\nPress space to play again", score.score),
                    TextStyle {
                        font_size: 64.0,
                        color: Color::BLACK,
                        ..default()
                    },
                )
                .with_text_alignment(TextAlignment::Center),
            );
        })
        .insert(FinalScoreDisplay);

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(191.0 / 255.0, 148.0 / 255.0, 228.0 / 255.0),
                ..default()
            },
            transform: Transform {
                scale: Vec3::new(900.0, 200.0, 1.0),
                translation: Vec3::new(0.0, 0.0, 2.0),
                ..default()
            },
            ..default()
        },
        FinalScoreDisplay,
    ));
}

fn spawn_previous_score_display(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut game_number: ResMut<GameNumber>,
    final_score_display: Query<Entity, With<FinalScoreDisplay>>,
) {
    for entity in final_score_display.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.spawn((TextBundle::from_section(
        format!("Game {}: {}", game_number.number, score.score),
        TextStyle {
            font_size: 24.0,
            ..default()
        },
    )
    .with_text_alignment(TextAlignment::Center)
    .with_style(Style {
        position_type: PositionType::Absolute,
        right: Val::Px(5.0),
        top: Val::Px(5.0 + (game_number.number - 1) as f32 * 36.0),
        ..default()
    }),));
    game_number.number += 1;
    score.score = 0;
}

fn start_next_game(
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.any_just_pressed([KeyCode::Space, KeyCode::Return, KeyCode::NumpadEnter]) {
        next_state.set(match state.get() {
            AppState::InGame => AppState::GaveOver,
            _ => AppState::InGame,
        });
    }
}
