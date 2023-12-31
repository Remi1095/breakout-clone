mod ball;
mod bricks;
mod camera;
mod debug;
mod paddle;
mod schedule;
mod score;
mod walls;

use bevy::{
    app::AppExit,
    prelude::*,
    window::{Cursor, PresentMode, WindowMode, WindowResolution},
};
use bevy_framepace::{FramepaceSettings, Limiter};
use bevy_rapier2d::prelude::*;
use notify_rust::Notification;
use serde::Deserialize;
use std::f32::consts::PI;
use std::fs;

use ball::BallPlugin;
use bricks::BrickPlugin;
use camera::CameraPlugin;
use paddle::PaddlePlugin;
use score::ScorePlugin;
use walls::WallPlugin;

const DEGREE_TO_RADIAN_FACTOR: f32 = PI / 180.0;
const BACKGROUND_COLOR: Color = Color::rgb(0.2, 0.2, 0.2);
const SCREEN_WIDTH: i32 = 900;
const SCREEN_HEIGHT: i32 = 800;
// const PHYSICS_UPDATES_PER_SECOND: i32 = 60;
// const PHYSICS_SUBSTEPS: usize = 2;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    SelectConfig,
    InGame,
    GaveOver,
}

#[derive(Resource, Deserialize, TypePath)]
struct GameConfig {
    window_width: i32,
    window_height: i32,
    window_fullscreen: bool,
    ball_diameter: f32,
    ball_initial_speed: f32,
    ball_speed: f32,
    score_to_ball_speed_factor: f32,
    ball_anti_gravity_time: f32,
    ball_gravity_scale: f32,
    ball_restitution: f32,
    wall_thickness: f32,
    area_width: f32,
    area_height: f32,
    brick_min_width: f32,
    brick_max_width: f32,
    brick_width_step: f32,
    brick_margin: f32,
    brick_bottom_margin_ratio: f32,
    brick_top_margin_ratio: f32,
    paddle_bottom_margin: f32,
    paddle_width: f32,
    paddle_height: f32,
    paddle_mesh_segments: i32,
    paddle_collider_segments: i32,
    paddle_max_speed: f32,
    paddle_acceleration: f32,
    brick_max_score: i32,
    brick_min_score: i32,
    score_loss_interval: f32,
    score_loss: i32,
    win_score_bonus: i32,
}
impl GameConfig {
    fn load(file_path: &str) -> Self {
        let contents =
            std::fs::read_to_string(file_path).expect(&format!("Failed to read {}", file_path));

        match serde_yaml::from_str(&contents) {
            Ok(game_config) => game_config,
            Err(e) => {
                show_notification(
                    "Config File Parsing Error",
                    &format!("Failed to parse {file_path}: {e}"),
                );
                panic!("Failed to parse {file_path}: {e}");
            }
        }
    }

    fn get_ball_starting_position(&self) -> Vec3 {
        let distance_to_bottom =
            self.brick_bottom_margin_ratio * self.area_height - self.ball_diameter / 2.0;
        let y = -self.area_height / 2.0 + distance_to_bottom;
        let x = -((self.area_width / 2.0)
            .min(distance_to_bottom - self.paddle_bottom_margin - self.paddle_height));
        Vec3::new(x, y, 1.0)
    }

    fn get_ball_initial_linvel(&self) -> Vec2 {
        Vec2::from_angle(-45.0 * DEGREE_TO_RADIAN_FACTOR) * self.ball_initial_speed
    }

    fn get_brick_bounding_box(&self) -> Transform {
        let height_ratio = 1.0 - self.brick_bottom_margin_ratio - self.brick_top_margin_ratio;
        let mid_y = (height_ratio / 2.0 + self.brick_bottom_margin_ratio) * self.area_height
            - self.area_height / 2.0;

        Transform {
            translation: Vec3::new(0.0, mid_y, 0.0),
            scale: Vec3::new(self.area_width, height_ratio * self.area_height, 1.0),
            ..default()
        }
    }

    fn get_brick_score(&self, brick_width: f32) -> i32 {
        let score_diff = (self.brick_max_score - self.brick_min_score) as f32;
        let brick_diff = (self.brick_max_width - self.brick_min_width).max(1.0);
        (((brick_width - self.brick_min_width) / brick_diff) * score_diff
            + self.brick_min_score as f32)
            .round() as i32
    }

    fn get_paddle_segment_point(&self, i: i32, segments: i32) -> Vec2 {
        let radius =
            self.paddle_height / 2.0 + self.paddle_width.powi(2) / (8.0 * self.paddle_height);
        let x = -self.paddle_width / 2.0 + (self.paddle_width / segments as f32) * i as f32;
        let y = (radius.powi(2) - (x).powi(2)).powf(0.5) - radius + self.paddle_height;
        Vec2::new(x, y)
    }
}

// #[derive(Resource, Debug)]
// struct GameConfigHandle(Handle<GameConfig>);

#[derive(Component)]
struct ConfigFilesUI;

#[derive(Component)]
struct ConfigFileOption(String);

fn main() {
    App::new()
        // Bevy plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Breakout".to_string(),
                position: WindowPosition::Centered(MonitorSelection::Primary),
                resolution: WindowResolution::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        // .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        // .add_plugins(FrameTimeDiagnosticsPlugin::default())
        // .add_plugins(LogDiagnosticsPlugin::default())
        // .insert_resource(RapierConfiguration {
        //     timestep_mode: TimestepMode::Fixed {
        //         dt: 1.0 / PHYSICS_UPDATES_PER_SECOND as f32,
        //         substeps: PHYSICS_SUBSTEPS,
        //     },
        //     ..default()
        // })
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        // .add_plugins(RapierDebugRenderPlugin {
        //     style: DebugRenderStyle {
        //         rigid_body_axes_length: 0.0,
        //         ..default()
        //     },
        //     ..default()
        // })
        .add_plugins(bevy_framepace::FramepacePlugin)
        .insert_resource(FramepaceSettings {
            limiter: Limiter::from_framerate(300.0),
        })
        // .add_plugins(YamlAssetPlugin::<GameConfig>::new(&["config.yaml"]))
        // User
        .add_state::<AppState>()
        .add_plugins(CameraPlugin)
        .add_plugins(BallPlugin)
        .add_plugins(WallPlugin)
        .add_plugins(BrickPlugin)
        .add_plugins(PaddlePlugin)
        .add_plugins(ScorePlugin)
        .add_systems(OnEnter(AppState::SelectConfig), spawn_game_config_ui)
        .add_systems(
            Update,
            handle_config_click.run_if(in_state(AppState::SelectConfig)),
        )
        .add_systems(OnExit(AppState::SelectConfig), exit_select_config)
        .add_systems(Update, leave_game)
        // .add_plugins(DebugPlugin)
        .run();
}

fn spawn_game_config_ui(mut commands: Commands) {
    let asset_entries = match fs::read_dir("./assets") {
        Ok(entries) => entries,
        Err(e) => {
            show_notification("Config File Error", &format!("{e:?}"));
            panic!("{e:?}");
        }
    };

    let config_files: Vec<_> = asset_entries
        .filter_map(|entry| {
            // Unwrap each entry
            let entry = entry.ok()?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Check if the file name ends with ".config.yaml"
            if file_name_str.ends_with(".yaml") {
                Some(file_name_str.into_owned())
            } else {
                None
            }
        })
        .collect();

    if config_files.len() == 0 {
        show_notification("Config File Error", "no '.yaml' files in 'assets'");
        panic!("no '.yaml' files in 'assets'");
    }

    commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
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
                    "Select game config file",
                    TextStyle {
                        font_size: 36.0,
                        color: Color::WHITE,
                        ..Default::default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(24.0)),
                    ..Default::default()
                })
                .with_text_alignment(TextAlignment::Center),
            );
            for config_file in config_files {
                parent
                    .spawn(ButtonBundle {
                        style: Style {
                            padding: UiRect::all(Val::Px(10.0)),
                            margin: UiRect::all(Val::Px(10.0)),
                            ..Default::default()
                        },
                        background_color: BackgroundColor::from(Color::rgb(0.1, 0.1, 0.1)),
                        ..Default::default()
                    })
                    .with_children(|button_parent| {
                        button_parent.spawn(
                            TextBundle::from_section(
                                config_file.to_string(),
                                TextStyle {
                                    font_size: 24.0,
                                    color: Color::WHITE,
                                    ..Default::default()
                                },
                            )
                            .with_style(Style {
                                margin: UiRect::all(Val::Px(24.0)),
                                ..Default::default()
                            })
                            .with_text_alignment(TextAlignment::Center),
                        );
                    })
                    .insert(ConfigFileOption(config_file.clone()));
            }
        })
        .insert(ConfigFilesUI);
}

fn handle_config_click(
    mut commands: Commands,
    mut config_query: Query<
        (&mut BackgroundColor, &ConfigFileOption, &Interaction),
        Changed<Interaction>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (mut background_color, config_file, interaction) in config_query.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                let game_config = GameConfig::load(&format!("./assets/{}", config_file.0));
                commands.insert_resource(game_config);
                next_state.set(AppState::InGame);
            }
            Interaction::Hovered => {
                background_color.0 = Color::rgb(0.4, 0.4, 0.4);
            }
            Interaction::None => {
                background_color.0 = Color::rgb(0.1, 0.1, 0.1);
            }
        }
    }
}

fn exit_select_config(
    mut commands: Commands,
    ui_query: Query<Entity, With<ConfigFilesUI>>,
    mut window_query: Query<&mut Window>,

    game_config: Res<GameConfig>,
) {
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    let mut window = window_query.single_mut();

    if game_config.window_fullscreen {
        window.mode = WindowMode::BorderlessFullscreen
    } else {
        window.resolution = WindowResolution::new(
            game_config.window_width as f32,
            game_config.window_height as f32,
        );
        window.position = WindowPosition::Centered(MonitorSelection::Current)
    }
    window.cursor = Cursor {
        visible: false,
        ..default()
    };
}

fn leave_game(keyboard_input: Res<Input<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}

fn show_notification(title: &str, message: &str) {
    Notification::new()
        .summary(title)
        .body(message)
        .show()
        .expect("Failed to show notification");
}
