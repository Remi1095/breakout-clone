use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::{
    distributions::{Uniform, WeightedIndex},
    prelude::*,
};

use crate::{AppState, GameConfig};

// const BRICK_MIN_WIDTH: f32 = 30.0;
// const BRICK_MAX_WIDTH: f32 = 80.0;
// const BRICK_WIDTH_STEP: f32 = 10.0;
// const BRICK_MARGIN: f32 = 3.0;
const BRICK_COLORS: [Color; 7] = [
    Color::hsl(0.0, 0.5, 0.5),   // Red
    Color::hsl(30.0, 0.5, 0.5),  // Orange
    Color::hsl(60.0, 0.5, 0.5),  // Yellow
    Color::hsl(120.0, 0.5, 0.5), // Green
    Color::hsl(240.0, 0.5, 0.5), // Blue
    Color::hsl(270.0, 0.5, 0.5), // Indigo
    Color::hsl(290.0, 0.5, 0.5), // Violet
];

const BRICK_BORDER_COLORS: [Color; 7] = [
    Color::hsl(0.0, 0.2, 0.3),   // Red
    Color::hsl(30.0, 0.2, 0.3),  // Orange
    Color::hsl(60.0, 0.2, 0.3),  // Yellow
    Color::hsl(120.0, 0.2, 0.3), // Green
    Color::hsl(240.0, 0.2, 0.3), // Blue
    Color::hsl(270.0, 0.2, 0.3), // Indigo
    Color::hsl(290.0, 0.2, 0.3), // Violet
];
const BRICK_BORDER_WIDTH: f32 = 5.0;
// const BOUNDING_BOX: Transform = Transform {
//     translation: Vec3::new(0.0, 50.0, 0.0),
//     scale: Vec3::new(990.0, 400.0, 1.0),
//     rotation: Quat::IDENTITY,
// };

#[derive(Component, Debug)]
pub struct Brick {
    pub score: i32,
}

pub struct BrickPlugin;

impl Plugin for BrickPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_bricks);
    }
}

fn spawn_bricks(
    mut commands: Commands,
    bricks_query: Query<Entity, With<Brick>>,
    game_config: Res<GameConfig>,
) {
    for brick_entity in bricks_query.iter() {
        commands.entity(brick_entity).despawn_recursive();
    }

    let get_brick_color = |side_length: f32, colors: &[Color]| {
        colors[((side_length - game_config.brick_max_width) / game_config.brick_width_step)
            .abs()
            .round() as usize
            % BRICK_COLORS.len()]
    };

    // commands.spawn(SpriteBundle {
    //     sprite: Sprite {
    //         color: Color::rgba(1.0, 0.0, 0.0, 0.03),
    //         ..default()
    //     },
    //     transform: game_config.get_brick_bounding_box(),
    //     ..default()
    // });

    for transform in compute_brick_layout(
        game_config.get_brick_bounding_box(),
        &mut commands,
        &game_config,
    ) {
        commands
            .spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: get_brick_color(transform.scale.x, &BRICK_BORDER_COLORS),
                        ..default()
                    },
                    transform: Transform {
                        scale: Vec3 {
                            x: transform.scale.x - game_config.brick_margin,
                            y: transform.scale.y - game_config.brick_margin,
                            z: 1.0,
                        },
                        translation: Vec3 {
                            z: 0.0,
                            ..transform.translation
                        },
                        ..transform
                    },
                    ..default()
                },
                Brick {
                    score: game_config.get_brick_score(transform.scale.x),
                },
                RigidBody::Fixed,
                Collider::cuboid(0.5, 0.5),
                Friction::coefficient(0.0),
                Restitution::coefficient(1.0),
            ))
            .with_children(|parent| {
                parent.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: get_brick_color(transform.scale.x, &BRICK_COLORS),
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3::new(0.0, 0.0, 1.0),
                        scale: Vec3 {
                            x: (transform.scale.x - BRICK_BORDER_WIDTH) / transform.scale.x,
                            y: (transform.scale.y - BRICK_BORDER_WIDTH) / transform.scale.y,
                            z: 1.0,
                        },
                        ..default()
                    },
                    ..default()
                });
            });
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Axis {
    X,
    Y,
}

#[derive(Debug, Clone, PartialEq)]
enum Side {
    NEGATIVE,
    POSITIVE,
}

#[derive(Debug, Clone, PartialEq)]
struct Edge {
    start: f32,
    end: f32,
    pos: f32,
    side: Side,
    axis: Axis,
}
impl Edge {
    fn new(a: Vec2, b: Vec2, side: Side) -> Self {
        if a.y == b.y {
            if a.x <= b.x {
                Edge {
                    start: a.x,
                    end: b.x,
                    pos: a.y,
                    side,
                    axis: Axis::X,
                }
            } else {
                Edge {
                    start: b.x,
                    end: a.x,
                    pos: a.y,
                    side,
                    axis: Axis::X,
                }
            }
        } else if a.x == b.x {
            if a.y <= b.y {
                Edge {
                    start: a.y,
                    end: b.y,
                    pos: a.x,
                    side,
                    axis: Axis::Y,
                }
            } else {
                Edge {
                    start: b.y,
                    end: a.y,
                    pos: a.x,
                    side,
                    axis: Axis::Y,
                }
            }
        } else {
            panic!("Edges must be horizontal or vertical");
        }
    }

    fn get_square(&self, side_length: f32) -> (f32, f32, f32, f32) {
        (
            self.start - side_length / 2.0,
            self.end + side_length / 2.0,
            self.pos - side_length / 2.0,
            self.pos + side_length / 2.0,
        )
    }
}

fn transform_left(rect: &Transform) -> f32 {
    rect.translation.x - rect.scale.x / 2.0
}
fn transform_right(rect: &Transform) -> f32 {
    rect.translation.x + rect.scale.x / 2.0
}
fn transform_bottom(rect: &Transform) -> f32 {
    rect.translation.y - rect.scale.y / 2.0
}
fn transform_top(rect: &Transform) -> f32 {
    rect.translation.y + rect.scale.y / 2.0
}

fn compute_brick_layout(
    bounding_box: Transform,
    commands: &mut Commands,
    game_config: &Res<GameConfig>,
) -> Vec<Transform> {
    let seed = thread_rng().gen_range(u64::MIN..=u64::MAX);
    info!("seed: {seed}");
    let mut rng = StdRng::seed_from_u64(seed);

    let mut max_side_length = game_config.brick_max_width;
    let side_length_dist_builder = |max_side_length| {
        Uniform::new_inclusive(
            (game_config.brick_min_width / game_config.brick_width_step) as i32,
            (max_side_length / game_config.brick_width_step) as i32,
        )
    };
    let mut side_length_dist = side_length_dist_builder(max_side_length);

    let first_width = side_length_dist.sample(&mut rng) as f32 * game_config.brick_width_step;
    let first_square = Transform {
        translation: Vec3 {
            x: rng.gen_range(
                transform_left(&bounding_box) + first_width / 2.0
                    ..=transform_right(&bounding_box) - first_width / 2.0,
            ),
            y: rng.gen_range(
                transform_bottom(&bounding_box) + first_width / 2.0
                    ..=transform_top(&bounding_box) - first_width / 2.0,
            ),
            z: 0.0,
        },
        scale: Vec3 {
            x: first_width,
            y: first_width,
            z: 1.0,
        },
        ..default()
    };

    let mut squares = vec![first_square];

    // debug
    // let debug_brick_y = |width, offset_x: f32, offset_y: f32| Transform {
    //     translation: Vec3 {
    //         x: first_square.translation.x + offset_x,
    //         y: first_square.translation.y + (first_square.scale.y + width) / 2.0 + offset_y,
    //         z: 0.0,
    //     },
    //     scale: Vec3 {
    //         x: width,
    //         y: width,
    //         z: 1.0,
    //     },
    //     ..default()
    // };

    // //debug
    // let debug_brick_x = |width, offset_x: f32, offset_y: f32| Transform {
    //     translation: Vec3 {
    //         x: first_square.translation.x + (first_square.scale.y + width) / 2.0 + offset_x,
    //         y: first_square.translation.y + offset_y,
    //         z: 0.0,
    //     },
    //     scale: Vec3 {
    //         x: width,
    //         y: width,
    //         z: 1.0,
    //     },
    //     ..default()
    // };

    //debug
    let mut debug_draw_edges = |edges: &Vec<Edge>, side_length: f32, alpha: f32| {
        for edge in edges {
            match edge.axis {
                Axis::X => {
                    commands.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: match edge.side {
                                Side::POSITIVE => Color::rgba(1.0, 0.0, 0.0, alpha),
                                Side::NEGATIVE => Color::rgba(0.0, 0.0, 1.0, alpha),
                            },
                            ..default()
                        },
                        transform: Transform {
                            translation: Vec3::new((edge.start + edge.end) / 2.0, edge.pos, 1.0),
                            scale: Vec3::new(edge.end - edge.start + side_length, side_length, 1.0),
                            ..default()
                        },
                        ..default()
                    });
                }
                Axis::Y => {
                    commands.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: match edge.side {
                                Side::POSITIVE => Color::rgba(1.0, 1.0, 0.0, alpha),
                                Side::NEGATIVE => Color::rgba(0.0, 1.0, 0.0, alpha),
                            },
                            ..default()
                        },
                        transform: Transform {
                            translation: Vec3::new(edge.pos, (edge.start + edge.end) / 2.0, 1.0),
                            scale: Vec3::new(side_length, edge.end - edge.start + side_length, 1.0),
                            ..default()
                        },
                        ..default()
                    });
                }
            }
        }
    };

    let corners = |rect: &Transform| {
        vec![
            Vec2::new(transform_left(rect), transform_top(rect)),
            Vec2::new(transform_right(rect), transform_top(rect)),
            Vec2::new(transform_right(rect), transform_bottom(rect)),
            Vec2::new(transform_left(rect), transform_bottom(rect)),
        ]
    };

    let mut horizontal_edges: Vec<Edge> = Vec::new();
    let mut vertical_edges: Vec<Edge> = Vec::new();
    let limit: Option<i32> = None; //122
    let mut idx = 0;
    loop {
        let square_corners = corners(&squares[squares.len() - 1]);

        let mut edges = square_corners
            .iter()
            .zip(square_corners.iter().cycle().skip(1));

        let (a, b) = edges.next().unwrap();
        horizontal_edges.push(Edge::new(*a, *b, Side::POSITIVE));
        let (a, b) = edges.next().unwrap();
        vertical_edges.push(Edge::new(*a, *b, Side::POSITIVE));
        let (a, b) = edges.next().unwrap();
        horizontal_edges.push(Edge::new(*a, *b, Side::NEGATIVE));
        let (a, b) = edges.next().unwrap();
        vertical_edges.push(Edge::new(*a, *b, Side::NEGATIVE));

        let mut trunc_horizontal_edges = truncate_overlapping_edges(&horizontal_edges);
        let mut trunc_vertical_edges = truncate_overlapping_edges(&vertical_edges);

        let square_side_length =
            side_length_dist.sample(&mut rng) as f32 * game_config.brick_width_step;

        let horizontal_positions = get_square_positions(
            square_side_length,
            &trunc_horizontal_edges,
            &mut trunc_vertical_edges,
        );
        let vertical_positions = get_square_positions(
            square_side_length,
            &trunc_vertical_edges,
            &mut trunc_horizontal_edges,
        );
        let all_positions = truncate_out_of_bounds(
            square_side_length,
            &[horizontal_positions, vertical_positions].concat(),
            &bounding_box,
        );

        if all_positions.len() == 0 {
            max_side_length = square_side_length - game_config.brick_width_step;
            if max_side_length < game_config.brick_min_width {
                break;
            }
            side_length_dist = side_length_dist_builder(max_side_length);
            continue;
        }

        if let Some(limit) = limit {
            if idx == limit {
                debug_draw_edges(&all_positions, square_side_length, 0.08);
                debug_draw_edges(&all_positions, 1.0, 1.0);
                debug_draw_edges(&horizontal_edges, 1.0, 1.0);
                debug_draw_edges(&vertical_edges, 1.0, 1.0);
                debug_draw_edges(&trunc_horizontal_edges, 4.0, 1.0);
                debug_draw_edges(&trunc_vertical_edges, 4.0, 1.0);
                break;
            }
        }

        let weights = all_positions.iter().map(|edge| edge.end - edge.start);

        let weighted_index = match WeightedIndex::new(weights.clone()) {
            Ok(weighted_index) => weighted_index,
            Err(e) => panic!("{}\n weights: {:?}", e, weights.collect::<Vec<f32>>()),
        };

        let chosen_edge = &all_positions[weighted_index.sample(&mut rng)];
        let square_translation = match chosen_edge.axis {
            Axis::X => Vec3 {
                x: rng.gen_range(chosen_edge.start..=chosen_edge.end),
                y: chosen_edge.pos,
                z: 0.0,
            },
            Axis::Y => Vec3 {
                x: chosen_edge.pos,
                y: rng.gen_range(chosen_edge.start..=chosen_edge.end),
                z: 0.0,
            },
        };
        // squares.push(test_squares[idx]);

        squares.push(Transform {
            translation: square_translation,
            scale: Vec3 {
                x: square_side_length,
                y: square_side_length,
                z: 1.0,
            },
            ..default()
        });

        idx += 1;
    }

    squares
}

fn truncate_overlapping_edges(edges: &Vec<Edge>) -> Vec<Edge> {
    let mut trunc_edges: Vec<Edge> = Vec::new();

    for edge1 in edges.iter() {
        let mut line_masks: Vec<(f32, f32)> = Vec::new();

        for edge2 in edges.iter() {
            if edge1 != edge2
                && edge1.side != edge2.side
                && (edge1.pos - edge2.pos).abs() < 0.01
                && edge1.end > edge2.start
                && edge2.end > edge1.start
            {
                line_masks.push((edge1.start.max(edge2.start), edge1.end.min(edge2.end)));
            }
        }
        line_masks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let mut temp_end = edge1.end.clone();
        for (mask_start, mask_end) in line_masks {
            if mask_end < temp_end {
                trunc_edges.push(Edge {
                    start: mask_end,
                    end: temp_end,
                    ..edge1.clone()
                });
            }
            temp_end = mask_start;
        }
        if temp_end > edge1.start {
            trunc_edges.push(Edge {
                end: temp_end,
                ..edge1.clone()
            });
        }
    }

    trunc_edges
}

fn get_square_positions(
    square_side_length: f32,
    parallel_edges: &Vec<Edge>,
    perpendicular_edges: &mut Vec<Edge>,
) -> Vec<Edge> {
    let mut position_edges: Vec<Edge> = Vec::new();
    let edge_rect = |edge: &Edge| {
        (
            // Axis::X/Axis::Y
            edge.start - square_side_length, // left/bottom
            edge.end + square_side_length,   // right/top
            edge.pos
                - (if let Side::NEGATIVE = edge.side {
                    square_side_length
                } else {
                    0.0
                }), // bottom/left
            edge.pos
                + (if let Side::POSITIVE = edge.side {
                    square_side_length
                } else {
                    0.0
                }), // top/right
        )
    };

    perpendicular_edges.sort_by(|a, b| a.pos.partial_cmp(&b.pos).unwrap());

    for par_edge in parallel_edges.iter() {
        let (par_start, par_end, perp_start, perp_end) = edge_rect(par_edge);
        let mut temp_par_start = par_start.clone();
        let mut temp_par_end = par_end.clone();
        let mut found_edges: Vec<Edge> = Vec::new();

        for perp_edge in perpendicular_edges.iter() {
            // perp_edge within parallel axis boundry
            if perp_edge.pos > par_start && perp_edge.pos < par_end &&
                // perp_edge within perpendicular axis boundry
                perp_edge.end > perp_start && perp_end > perp_edge.start
            {
                match perp_edge.side {
                    Side::POSITIVE => {
                        temp_par_start = perp_edge.pos;
                        temp_par_end = par_end;
                    }
                    Side::NEGATIVE => {
                        temp_par_end = perp_edge.pos;
                        let start = temp_par_start + square_side_length / 2.0;
                        let end = perp_edge.pos - square_side_length / 2.0;
                        if end > start {
                            found_edges.push(Edge {
                                start,
                                end,
                                pos: perp_start + square_side_length / 2.0,
                                ..par_edge.clone()
                            })
                        }
                        temp_par_start = perp_edge.pos;
                    }
                }
            }
        }
        let start = temp_par_start + square_side_length / 2.0;
        let end = temp_par_end - square_side_length / 2.0;
        if end > start {
            let tmp = Edge {
                start,
                end,
                pos: perp_start + square_side_length / 2.0,
                ..par_edge.clone()
            };
            found_edges.push(tmp);
        }
        for found_edge in found_edges {
            let (par_start, par_end, perp_start, perp_end) =
                found_edge.get_square(square_side_length);
            let mut overlap = false;
            for par_edge in parallel_edges.iter() {
                if par_edge.pos > perp_start
                    && par_edge.pos < perp_end
                    && par_edge.start - 0.5 < par_start
                    && par_edge.end + 0.5 > par_end
                {
                    overlap = true;
                    break;
                }
            }
            if !overlap {
                position_edges.push(found_edge);
            }
        }
    }

    position_edges
}

fn truncate_out_of_bounds(
    square_side_length: f32,
    square_positions: &Vec<Edge>,
    bounding_box: &Transform,
) -> Vec<Edge> {
    let mut trunc_positions: Vec<Edge> = Vec::new();
    for edge in square_positions {
        let (mut par_start, mut par_end, perp_start, perp_end) =
            edge.get_square(square_side_length);
        let (par_bound_start, par_bound_end, perp_bound_start, perp_bound_end) = match edge.axis {
            Axis::X => (
                transform_left(bounding_box),
                transform_right(bounding_box),
                transform_bottom(bounding_box),
                transform_top(bounding_box),
            ),
            Axis::Y => (
                transform_bottom(bounding_box),
                transform_top(bounding_box),
                transform_left(bounding_box),
                transform_right(bounding_box),
            ),
        };
        if perp_start < perp_bound_start || perp_end > perp_bound_end {
            continue;
        }
        if par_start < par_bound_start {
            par_start = par_bound_start;
        }
        if par_end > par_bound_end {
            par_end = par_bound_end;
        }
        let start = par_start + square_side_length / 2.0;
        let end = par_end - square_side_length / 2.0;
        if end > start {
            trunc_positions.push(Edge {
                start: par_start + square_side_length / 2.0,
                end: par_end - square_side_length / 2.0,
                ..edge.clone()
            })
        }
    }
    trunc_positions
}
