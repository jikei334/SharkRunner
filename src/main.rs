pub mod maze;
pub mod player;
pub mod police;
pub mod sfx;
pub mod utils;
pub mod wall;
pub mod woman;

use bevy::{picking::events::Pointer, prelude::*};
use bevy::asset::AssetMetaCheck;
use bevy::audio::Volume;

use maze::Maze;
use player::{
    Form, ModelRoot, Player, PlayerAnim, PLAYER_MODEL_PATH, Transforming,
};
use police::{
    PoliceAnimations,
    Trap,
};
use sfx::Sfx;
use utils::{Rng, Seed};
use wall::{spawm_wall, Wall, wall_collision};
use woman::{Woman, WomanAnimations};


const CAMERA_X: f32 = 0.0;
const CAMERA_Y: f32 = 14.0;
const CAMERA_Z: f32 = 6.0;
const CAMERA_FOLLOW_SPEED: f32 = 5.0;

const LIGHT_X: f32 = 4.0;
const LIGHT_Y: f32 = 10.0;
const LIGHT_Z: f32 = 4.0;

// 横のセルの数
const MAZE_W: usize = 5;
// 縦のセルの数
const MAZE_H: usize = 12;
// 壁を壊す試行回数
const EXTRA_OPENINGS: usize = 24;
const WOMEN_COUNT: usize = 8;

#[derive(Component)]
pub struct LevelEntity;

#[derive(Component)]
struct HpText;

#[derive(Component)]
struct HpBarFill;

#[derive(Component, Default)]
pub struct Velocity(Vec3);

#[derive(Resource, Default)]
struct DragStart(Option<Vec3>);

// 1秒あたりに減る速度
const FRICTION: f32 = 6.0;

struct PointerInput {
    pos: Vec2,
    just_pressed: bool,
    just_released: bool,
}

#[derive(Resource, Default)]
struct Aim(Option<Vec3>);

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameState {
    #[default]
    Title,
    Playing,
    Caught,
    Ambush,
    Defeated,
    GameOver,
    Clear,
}

#[derive(Component)]
struct FormButtonText;

#[derive(Component)]
pub struct Health(f32);

const MAX_HEALTH: f32 = 100.0;
pub const DAMAGE_PER_SPEED: f32 = 0.5;
pub const DAMAGE_MIN_SPEED: f32 = 2.0;
pub const BOUNCE: f32 = 0.5;
// スピードを0とみなす閾値
const SPEED_THRESHOLD_ZERO: f32 = 0.05;
const PULL_THRESHOLD: f32 = 0.5;

#[derive(Resource, Default)]
struct Strokes(u32);

#[derive(Component)]
struct StrokesText;

const TRAP_COUNT: usize = 6;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            }),
        )
        .insert_resource(ClearColor(Color::srgb(0.5, 0.7, 0.9)))
        .insert_resource(Seed::new_with_time())
        .init_resource::<DragStart>()
        .init_resource::<Aim>()
        .init_resource::<Strokes>()
        .init_state::<GameState>()
        .add_systems(Startup, (
                setup_scene,
                setup_ui,
                PlayerAnim::setup,
                PoliceAnimations::setup,
                WomanAnimations::setup,
                Sfx::setup,
        ))
        .add_systems(Update, (
                update_hp_ui,
                update_form_button_label,
                update_strokes_ui,
                PoliceAnimations::attack,
                PoliceAnimations::caught.run_if(in_state(GameState::Caught)),
                WomanAnimations::react,
                WomanAnimations::tick_eaten,
                Sfx::play_scream,
                Sfx::play_transform,
        ))
        .add_systems(
            OnEnter(GameState::Title),
            show_title,
        )
        .add_systems(
            OnEnter(GameState::Playing),
            (cleanup_level, setup_level).chain(),
        )
        .add_systems(
            OnEnter(GameState::Caught),
            Player::stop,
        )
        .add_systems(
            Update,
            PlayerAnim::update
                .run_if(
                    in_state(GameState::Caught)
                    .or(in_state(GameState::Defeated))
                    .or(in_state(GameState::Ambush)),
            ),
        )
        .add_systems(OnEnter(GameState::Defeated), Player::start_defeat)
        .add_systems(Update, Player::update_defeated.run_if(in_state(GameState::Defeated)))
        .add_systems(
            Update,
            (
                switch_form,
                drag_input,
                update_facing,
                movement,
                PlayerAnim::update,
                PlayerAnim::finish_transform,
                wall_collision,
                Sfx::play_wall_hit,
                Woman::collision,
                Trap::collision,
                check_goal,
                check_health,
                camera_follow,
                draw_guide,
            )
            .chain()
            .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            OnEnter(GameState::Ambush),
            (Player::stop, PoliceAnimations::start_ambush)
        )
        .add_systems(
            Update,
            PoliceAnimations::ambush.run_if(in_state(GameState::Ambush))
        )
        .add_systems(
            OnEnter(GameState::GameOver),
            (show_game_over, Sfx::play_game_over, Sfx::pause_bgm),
        )
        .add_systems(
            OnEnter(GameState::Clear),
            (show_clear, Sfx::play_clear, Sfx::pause_bgm),
        )
        .add_systems(
            OnEnter(GameState::Playing),
            Sfx::resume_bgm,
        )
        .run();
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        },
        children![
            // 文字
            (
                HpText,
                Text::new("HP: 100"),
                TextFont { font_size: FontSize::Px(24.0), ..default() },
                TextColor(Color::WHITE),
            ),
            // バーの背景
            (
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                // バーの中身
                children![(
                    HpBarFill,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.8, 0.3)),
                )]
            ),
            // 打数
            (
                StrokesText,
                Text::new("STROKES: 0"),
                TextFont { font_size: FontSize::Px(24.0), ..default() },
                TextColor(Color::WHITE),
            ),
        ]
    ));
    commands
        .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(24.0),
                    right: Val::Px(24.0),
                    padding: UiRect::axes(Val::Px(28.0), Val::Px(18.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.35, 0.6)),
                children![(
                    FormButtonText,
                    Text::new("TO SHARK"),
                    TextFont { font_size: FontSize::Px(28.0), ..default() },
                    TextColor(Color::WHITE),
                )],
        ))
        .observe(on_form_button_click);
}

fn setup_scene(mut commands: Commands) {
    // カメラ
    commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(
                CAMERA_X,
                CAMERA_Y,
                CAMERA_Z,
            )
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 光
    commands.spawn((
            DirectionalLight::default(),
            Transform::from_xyz(
                LIGHT_X,
                LIGHT_Y,
                LIGHT_Z,
            )
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    seed: Res<Seed>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = Rng::new(seed.into_inner());
    let mut maze = Maze::new(MAZE_H, MAZE_W, EXTRA_OPENINGS, &mut rng);
    maze.place_traps(TRAP_COUNT, &mut rng);
    // 地面
    let ground_z = (maze.tile_to_world(0, 0).y + maze.tile_to_world(0, maze.tile_rows() - 1).y) / 2.0;
    commands.spawn((
            LevelEntity,
            Mesh3d(meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(
                        maze.tile_cols() as f32 * maze.tile(),
                        maze.tile_rows() as f32 * maze.tile(),
                    ),
            )),
            MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
            Transform::from_xyz(0.0, 0.0, ground_z),
    ));

    // プレイヤー
    commands.spawn((
            LevelEntity,
            Player,
            Velocity::default(),
            Form::Human,
            Health(MAX_HEALTH),
            Transform::from_xyz(0.0, 0.5, 0.0),
            Visibility::default(),
    ))
        .with_children(|parent| {
            parent
                .spawn((
                    ModelRoot,
                    WorldAssetRoot(
                        asset_server.load(GltfAssetLabel::Scene(0).from_asset(PLAYER_MODEL_PATH)),
                    ),
                    Transform::from_xyz(0.0, -Form::HUMAN_RADIUS, 0.0),
                ))
                .observe(PlayerAnim::on_ready);
        });

    // 壁
    for row in 0..maze.tile_rows() {
        let mut col = 0;
        while col < maze.tile_cols() {
            if !maze.is_wall(col, row) {
                col += 1;
                continue;
            }
            let start = col;
            while col < maze.tile_cols() && maze.is_wall(col, row) {
                col += 1;
            }
            let len = col - start;
            let center = (maze.tile_to_world(start, row) + maze.tile_to_world(col - 1, row)) / 2.0;
            spawm_wall(
                &mut commands,
                &mut meshes,
                &mut materials,
                center,
                Vec2::new(len as f32 * maze.tile(), maze.tile()),
            );
        }
    }

    // ゴール
    commands.spawn((
        LevelEntity,
        Mesh3d(meshes.add(Cuboid::new(
            (maze.tile_cols() - 2) as f32 * maze.tile(),
            0.05,
            0.5
        ))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.9, 0.2))),
        Transform::from_xyz(0.0, 0.03, maze.goal_z()),
    ));

    // トラップ
    for pos in maze.trap_positions() {
        PoliceAnimations::spawn(&mut commands, &asset_server, pos, 0.0);
    }

    // 女性
    for spot in maze.pick_women_spots(WOMEN_COUNT, &mut rng) {
        Woman::spawn(&mut commands, &asset_server, spot.x, spot.y);
    }

    commands.insert_resource(maze);
}

fn update_hp_ui(
    health: Single<&Health, With<Player>>,
    mut text: Single<&mut Text, With<HpText>>,
    mut fill: Single<&mut Node, With<HpBarFill>>,
) {
    let ratio = (health.0 / MAX_HEALTH).clamp(0.0, 1.0);
    text.0 = format!("HP: {:.0}", health.0.max(0.0));
    fill.width = Val::Percent(ratio * 100.0);
}

fn check_goal(
    mut next_state: ResMut<NextState<GameState>>,
    maze: Res<Maze>,
    player: Single<&Transform, With<Player>>,
) {
    if player.translation.z < maze.goal_z() {
        next_state.set(GameState::Clear);
        info!("ゴール!");
    }
}

fn check_health(
    mut next_state: ResMut<NextState<GameState>>,
    health: Single<&Health, With<Player>>,
) {
    if health.0 <= 0.0 {
        next_state.set(GameState::Defeated);
        info!("HP 0");
    }
}

/// 画面上のカーソル位置を地面上の座標に変換
fn point_to_ground(
    point: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec3> {
    let ray = camera
        .viewport_to_world(camera_transform, point)
        .ok()?;
    let dist = ray
        .intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?;
    Some(ray.get_point(dist))
}

/// 引っ張って離す入力システム
fn drag_input(
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut drag: ResMut<DragStart>,
    mut aim: ResMut<Aim>,
    mut player: Single<(&mut Velocity, &Form), (With<Player>, Without<Transforming>)>,
    mut strokes: ResMut<Strokes>,
) {
    let (camera, camera_transform) = camera.into_inner();
    let (mut velocity, form) = player.into_inner();

    let Some(p) = read_pointer(&window, &mouse, &touches) else { return; };

    if p.just_pressed && velocity.0 == Vec3::ZERO {
        drag.0 = point_to_ground(p.pos, camera, camera_transform);
    }

    if p.just_released {
        if let (Some(start), Some(end)) = (
            drag.0.take(),
            point_to_ground(p.pos, camera, camera_transform),
        ) {
            // 引いた向きと逆に飛ぶ
            let pull = start - end;
            if pull.length() < PULL_THRESHOLD {
                return;
            }
            velocity.0 = (pull * form.power()).clamp_length_max(form.max_speed());
            strokes.0 += 1;
        }
    }

    aim.0 = match drag.0 {
        Some(start) => point_to_ground(p.pos, camera, camera_transform).map(|cur| start - cur),
        None => None,
    };
}

const BGM_PATH: &str = "bgm_loop.ogg";
const BGM_VOLUME: f32 = 0.5;

#[derive(Component)]
struct Bgm;

const FACING_SMOOTHING: f32 = 12.0;
const AIM_MIN: f32 = 0.3;
const FACE_MIN_SPEED: f32 = 0.3;
const FACING_YAW_OFFSET: f32 = 0.0;

fn update_facing(
    time: Res<Time>,
    aim: Res<Aim>,
    player: Single<(&mut Transform, &Velocity), With<Player>>,
) {
    let (mut transform, velocity) = player.into_inner();

    let dir = match aim.0 {
        Some(v) if v.length() > AIM_MIN => v,
        _ if velocity.0.length() > FACE_MIN_SPEED => velocity.0,
        _ => return,
    };

    let yaw = dir.x.atan2(dir.z) + FACING_YAW_OFFSET;
    let target = Quat::from_rotation_y(yaw);
    let t = 1.0 - (-FACING_SMOOTHING * time.delta_secs()).exp();
    transform.rotation = transform.rotation.slerp(target, t);
}

/// 移動システム
fn movement(time: Res<Time>, mut query: Query<(&mut Transform, &mut Velocity)>) {
    for (mut transform, mut velocity) in &mut query {
        transform.translation += velocity.0 * time.delta_secs();

        let speed = velocity.0.length();
        if 0.0 < speed {
            let new_speed = (speed - FRICTION * time.delta_secs()).max(0.0);
            velocity.0 = velocity.0 / speed * new_speed;
            if new_speed < SPEED_THRESHOLD_ZERO {
                velocity.0 = Vec3::ZERO;
            }
        }
    }
}

fn on_form_button_click(
    _click: On<Pointer<Click>>,
    mut commands: Commands,
    player: Single<(Entity, &Form, &Velocity), (With<Player>, Without<Transforming>)>,
) {
    let (entity, form, velocity) = player.into_inner();
    request_transform(&mut commands, entity, form, velocity);
}

fn update_form_button_label(
    form: Single<&Form, With<Player>>,
    mut text: Single<&mut Text, With<FormButtonText>>,
) {
    text.0 = match **form {
        Form::Human => "TO SHARK".to_string(),
        Form::Shark => "TO HUMAN".to_string(),
    };
}

fn request_transform(
    commands: &mut Commands,
    entity: Entity,
    form: &Form,
    velocity: &Velocity,
) {
    // 動いている間は変身できない
    if velocity.0 != Vec3::ZERO {
        return;
    }
    let target = match *form {
        Form::Human => Form::Shark,
        Form::Shark => Form::Human,
    };
    commands.entity(entity).insert(Transforming { target });
}

fn switch_form(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    player: Single<(Entity, &Form, &Velocity), (With<Player>, Without<Transforming>)>,
) {
    if keys.just_pressed(KeyCode::Space) {
        let (
            entity,
            mut form,
            velocity
        ) = player.into_inner();
        request_transform(&mut commands, entity, form, velocity);
    }
}

fn camera_follow(
    time: Res<Time>,
    player: Single<&Transform, (With<Player>, Without<Camera3d>)>,
    mut camera: Single<&mut Transform, (With<Camera3d>, Without<Player>)>,
) {
    let target = player.translation + Vec3::new(CAMERA_X, CAMERA_Y, CAMERA_Z);
    let t = 1.0 - (-CAMERA_FOLLOW_SPEED * time.delta_secs()).exp();
    camera.translation = camera.translation.lerp(target, t);
}

fn cleanup_level(
    mut commands: Commands,
    entities: Query<Entity, With<LevelEntity>>,
    mut drag: ResMut<DragStart>,
    mut strokes: ResMut<Strokes>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    drag.0 = None;
    strokes.0 = 0;
}

enum ResultKind {
    Retry,
    NextMaze,
    Start,
}

fn spawn_result_overlay(
    commands: &mut Commands,
    title: &str,
    color: Color,
    subtitle: &str,
    button_label: &str,
    kind: ResultKind,
) {
    commands.spawn((
        LevelEntity,
        Node {
            position_type:PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(24.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
    ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextFont { font_size: FontSize::Px(72.0), ..default() },
                TextColor(color),
            ));

            if !subtitle.is_empty() {
                parent.spawn((
                    Text::new(subtitle),
                    TextFont { font_size: FontSize::Px(36.0), ..default() },
                    TextColor(Color::WHITE),
                ));
            }

            let mut button = parent.spawn((
                Node {
                    padding: UiRect::axes(Val::Px(32.0), Val::Px(14.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
                children![(
                    Text::new(button_label),
                    TextFont { font_size: FontSize::Px(32.0), ..default() },
                    TextColor(Color::WHITE),
                )],
            ));
            match kind {
                ResultKind::Retry => { button.observe(on_retry_click); },
                ResultKind::NextMaze => { button.observe(on_next_maze_click); },
                ResultKind::Start => { button.observe(on_start_click); },
            }
        });
}

fn show_title(mut commands: Commands) {
    spawn_result_overlay(
        &mut commands,
        "SHARK RUNNER",
        Color::WHITE,
        "",
        "TAP TO START",
        ResultKind::Start,
    );
}

fn on_start_click(
    _click: On<Pointer<Click>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    commands.spawn((
            Bgm,
            AudioPlayer::new(asset_server.load(BGM_PATH)),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(BGM_VOLUME)),
    ));
    next_state.set(GameState::Playing);
}

fn show_game_over(mut commands: Commands) {
    spawn_result_overlay(
        &mut commands,
        "GAME OVER",
        Color::srgb(0.9, 0.2, 0.2),
        "",
        "RETRY",
        ResultKind::Retry,
    );
}

fn show_clear(mut commands: Commands, strokes: Res<Strokes>) {
    spawn_result_overlay(
        &mut commands,
        "CLEAR!",
        Color::srgb(1.0, 0.9, 0.2),
        &format!("STROKES: {}", strokes.0),
        "NEXT MAZE",
        ResultKind::NextMaze,
    );
}

fn on_retry_click(_click: On<Pointer<Click>>, mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Playing);
}

fn on_next_maze_click(
    _click: On<Pointer<Click>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut seed: ResMut<Seed>,
) {
    *seed = Seed::new_with_time();
    next_state.set(GameState::Playing);
}

fn read_pointer(
    window: &Window,
    mouse: &ButtonInput<MouseButton>,
    touches: &Touches,
) -> Option<PointerInput> {
    if let Some(t) = touches.iter_just_pressed().next() {
        return Some(PointerInput {
            pos: t.position(),
            just_pressed: true,
            just_released: false,
        });
    }

    if let Some(t) = touches.iter_just_released().next() {
        return Some(PointerInput {
            pos: t.position(),
            just_pressed: false,
            just_released: true,
        });
    }

    if let Some(t) = touches.iter().next() {
        return Some(PointerInput {
            pos: t.position(),
            just_pressed: false,
            just_released: false,
        })
    }

    let pos = window.cursor_position()?;
    Some(PointerInput {
        pos,
        just_pressed: mouse.just_pressed(MouseButton::Left),
        just_released: mouse.just_released(MouseButton::Left),
    })
}

const GUIDE_Y: f32 = 0.1;

fn draw_guide(
    mut gizmos: Gizmos,
    maze: Res<Maze>,
    player: Single<&Transform, With<Player>>,
) {
    let pos = player.translation;
    let Some(start) = maze.world_to_tile(Vec2::new(pos.x, pos.z)) else { return; };
    let Some(path) = maze.shortest_path(start) else { return; };

    let mut points = vec![Vec3::new(pos.x, GUIDE_Y, pos.z)];
    points.extend(path.iter().skip(1).map(|p| Vec3::new(p.x, GUIDE_Y, p.y)));
    if let Some(last) = path.last() {
        points.push(Vec3::new(last.x, GUIDE_Y, maze.goal_z()));
    }

    let color = Color::srgb(1.0, 0.9, 0.2);

    let points = simplify(points);
    for seg in points.windows(2) {
        gizmos.arrow(seg[0], seg[1], color).with_tip_length(0.6);
    }
}

/// 重複した点と、一直線上の中間点を取り除く
fn simplify(points: Vec<Vec3>) -> Vec<Vec3> {
    let mut out: Vec<Vec3> = Vec::new();
    for p in points {
        if let Some(last) = out.last() {
            if last.distance(p) < 0.01 {
                continue;
            }
        }

        if out.len() >= 2 {
            let a = out[out.len() - 2];
            let b = out[out.len() - 1];
            let d1 = (b - a).normalize_or_zero();
            let d2 = (p - b).normalize_or_zero();
            if d1.dot(d2) > 0.999 {
                out.pop();
            }
        }
        out.push(p);
    }
    out
}

fn update_strokes_ui(strokes: Res<Strokes>, mut text: Single<&mut Text, With<StrokesText>>) {
    if strokes.is_changed() {
        text.0 = format!("STROKES: {}", strokes.0);
    }
}
