pub mod player;
pub mod utils;
pub mod wall;
pub mod woman;

use bevy::prelude::*;

use player::{Form, Player, PLAYER_RADIUS};
use wall::{spawm_wall, Wall, wall_collision};
use woman::{spawn_woman, Woman, woman_collision};


const CAMERA_X: f32 = 0.0;
const CAMERA_Y: f32 = 14.0;
const CAMERA_Z: f32 = 6.0;
const CAMERA_FOLLOW_SPEED: f32 = 5.0;

const LIGHT_X: f32 = 4.0;
const LIGHT_Y: f32 = 10.0;
const LIGHT_Z: f32 = 4.0;

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

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameState {
    #[default]
    Playing,
    GameOver,
    Clear,
}

const GOAL_Z: f32 = -30.0;

#[derive(Component)]
pub struct Health(f32);

const MAX_HEALTH: f32 = 100.0;
pub const DAMAGE_PER_SPEED: f32 = 0.5;
pub const DAMAGE_MIN_SPEED: f32 = 2.0;
pub const BOUNCE: f32 = 0.5;
// スピードを0とみなす閾値
const SPEED_THRESHOLD_ZERO: f32 = 0.05;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.5, 0.7, 0.9)))
        .init_resource::<DragStart>()
        .init_state::<GameState>()
        .add_systems(Startup, (setup_scene, setup_ui))
        .add_systems(Update, update_hp_ui)
        .add_systems(
            OnEnter(GameState::Playing),
            (cleanup_level, setup_level).chain(),
        )
        .add_systems(
            Update,
            restart_input.run_if(not(in_state(GameState::Playing))),
        )
        .add_systems(
            Update,
            (
                switch_form,
                drag_input,
                movement,
                wall_collision,
                woman_collision,
                check_goal,
                check_health,
                camera_follow,
            )
            .chain()
            .run_if(in_state(GameState::Playing)),
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
            )
        ]
    ));
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
) {
    // 地面
    commands.spawn((
            LevelEntity,
            Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 200.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    // プレイヤー
    commands.spawn((
            LevelEntity,
            Player,
            Velocity::default(),
            Form::Human,
            Health(MAX_HEALTH),
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.9, 0.7, 0.6))),
            Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // 壁
    // 外周
    spawm_wall(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(-10.5, 0.0),
        Vec2::new(1.0, 200.0)
    );
    spawm_wall(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(10.5, 0.0),
        Vec2::new(1.0, 200.0)
    );
    spawm_wall(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(0.0, -100.5),
        Vec2::new(22.0, 1.0)
    );
    spawm_wall(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(0.0, 100.5),
        Vec2::new(22.0, 1.0)
    );
    // 練習用
    spawm_wall(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec2::new(0.0, -5.0),
        Vec2::new(8.0, 1.0)
    );

    // ゴール
    commands.spawn((
        LevelEntity,
        Mesh3d(meshes.add(Cuboid::new(20.0, 0.05, 0.5))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.9, 0.2))),
        Transform::from_xyz(0.0, 0.03, GOAL_Z),
    ));

    // 女性
    spawn_woman(&mut commands, &mut meshes, &mut materials, -4.0, -12.0);
    spawn_woman(&mut commands, &mut meshes, &mut materials, 5.0, -18.0);
    spawn_woman(&mut commands, &mut meshes, &mut materials, 0.0, -25.0);
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
    player: Single<&Transform, With<Player>>,
) {
    if player.translation.z < GOAL_Z {
        next_state.set(GameState::Clear);
        info!("ゴール!");
    }
}

fn check_health(
    mut next_state: ResMut<NextState<GameState>>,
    health: Single<&Health, With<Player>>,
) {
    if health.0 <= 0.0 {
        next_state.set(GameState::GameOver);
        info!("HP 0");
    }
}

/// 画面上のカーソル位置を地面上の座標に変換
fn cursor_to_ground(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform
) -> Option<Vec3> {
    let cursor = window.cursor_position()?;
    let ray = camera
        .viewport_to_world(camera_transform, cursor)
        .ok()?;
    let dist = ray
        .intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?;
    Some(ray.get_point(dist))
}

/// 引っ張って離す入力システム
fn drag_input(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut drag: ResMut<DragStart>,
    mut player: Single<(&mut Velocity, &Form), With<Player>>,
) {
    let (camera, camera_transform) = camera.into_inner();
    let (mut velocity, form) = player.into_inner();

    if buttons.just_pressed(MouseButton::Left) && velocity.0.length() < 0.01 {
        drag.0 = cursor_to_ground(&window, camera, camera_transform);
    }

    if buttons.just_released(MouseButton::Left) {
        if let (Some(start), Some(end)) = (
            drag.0.take(),
            cursor_to_ground(&window, camera, camera_transform),
        ) {
            // 引いた向きと逆に飛ぶ
            let pull = start - end;
            velocity.0 = (pull * form.power()).clamp_length_max(form.max_speed());
        }
    }
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

fn switch_form(
    keys: Res<ButtonInput<KeyCode>>,
    player: Single<(&mut Form, &MeshMaterial3d<StandardMaterial>, &Velocity), With<Player>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (
        mut form,
        material,
        velocity
    ) = player.into_inner();
    if keys.just_pressed(KeyCode::Space) && velocity.0 == Vec3::ZERO {
        *form = match *form {
            Form::Human => Form::Shark,
            Form::Shark => Form::Human,
        };
        if let Some(mut m) = materials.get_mut(&material.0) {
            m.base_color = form.color();
        }
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
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    drag.0 = None;
}

fn restart_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Playing);
    }
}
