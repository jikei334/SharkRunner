use bevy::prelude::*;


const CAMERA_X: f32 = 0.0;
const CAMERA_Y: f32 = 14.0;
const CAMERA_Z: f32 = 6.0;
const CAMERA_FOLLOW_SPEED: f32 = 5.0;

const LIGHT_X: f32 = 4.0;
const LIGHT_Y: f32 = 10.0;
const LIGHT_Z: f32 = 4.0;

#[derive(Component, Clone, Copy, PartialEq)]
enum Form {
    Human,
    Shark,
}

impl Form {
    const HUMAN_POWER: f32 = 3.0;
    const SHARK_POWER: f32 = 6.0;

    const HUMAN_MAX_SPEED: f32 = 10.0;
    const SHARK_MAX_SPEED: f32 = 30.0;

    const HUMAN_COLOR: Color = Color::srgb(0.9, 0.7, 0.6);
    const SHARK_COLOR: Color = Color::srgb(0.3, 0.4, 0.6);

    /// 引っ張りの長さから速度への倍率
    fn power(self) -> f32 {
        match self {
            Form::Human => Self::HUMAN_POWER,
            Form::Shark => Self::SHARK_POWER,
        }
    }

    /// 速度の上限
    fn max_speed(self) -> f32 {
        match self {
            Form::Human => Self::HUMAN_MAX_SPEED,
            Form::Shark => Self::SHARK_MAX_SPEED,
        }
    }

    fn color(self) -> Color {
        match self {
            Form::Human => Self::HUMAN_COLOR,
            Form::Shark => Self::SHARK_COLOR,
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct Velocity(Vec3);

#[derive(Resource, Default)]
struct DragStart(Option<Vec3>);

// 1秒あたりに減る速度
const FRICTION: f32 = 6.0;

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum GameState {
    #[default]
    Playing,
    GameOver,
    Clear,
}

#[derive(Component)]
struct Woman;

const WOMAN_RADIUS: f32 = 0.5;
const GOAL_Z: f32 = -30.0;

#[derive(Component)]
struct Wall {
    /// 壁の半分の大きさ
    half: Vec2,
}

#[derive(Component)]
struct Health(f32);

const WALL_HEIGHT: f32 = 1.0;
const PLAYER_RADIUS: f32 = 0.5;
const MAX_HEALTH: f32 = 100.0;
const DAMAGE_PER_SPEED: f32 = 0.5;
const DAMAGE_MIN_SPEED: f32 = 2.0;
const BOUNCE: f32 = 0.5;

// スピードを0とみなす閾値
const SPEED_THRESHOLD_ZERO: f32 = 0.05;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.5, 0.7, 0.9)))
        .init_resource::<DragStart>()
        .init_state::<GameState>()
        .add_systems(Startup, setup)
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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

    // 地面
    commands.spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 200.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    // プレイヤー
    commands.spawn((
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
        Mesh3d(meshes.add(Cuboid::new(20.0, 0.05, 0.5))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.9, 0.2))),
        Transform::from_xyz(0.0, 0.03, GOAL_Z),
    ));

    // 女性
    spawn_woman(&mut commands, &mut meshes, &mut materials, -4.0, -12.0);
    spawn_woman(&mut commands, &mut meshes, &mut materials, 5.0, -18.0);
    spawn_woman(&mut commands, &mut meshes, &mut materials, 0.0, -25.0);
}

/// 女性を出す
fn spawn_woman(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    x: f32,
    z: f32,
) {
    commands.spawn((
        Woman,
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.8))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.3, 0.5))),
        Transform::from_xyz(x, 0.75, z),
    ));
}

fn woman_collision(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    women: Query<(Entity, &Transform), (With<Woman>, Without<Player>)>,
    player: Single<(&Transform, &Form), With<Player>>,
) {
    let (player_transform, form) = player.into_inner();
    for (entity, woman_transform) in &women {
        let distance = player_transform.translation.distance(woman_transform.translation);
        if distance < PLAYER_RADIUS + WOMAN_RADIUS {
            match *form {
                Form::Shark => {
                    commands.entity(entity).despawn();
                    info!("食べた!");
                },
                Form::Human => {
                    next_state.set(GameState::GameOver);
                    info!("捕まった!");
                },
            }
        }
    }
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

const WALL_COLOR: Color = Color::srgb(0.4, 0.35, 0.3);

/// 壁を出す
fn spawm_wall(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    center: Vec2,
    size: Vec2,
) {
    commands.spawn((
        Wall { half: size / 2.0 },
        Mesh3d(meshes.add(Cuboid::new(size.x, WALL_HEIGHT, size.y))),
        MeshMaterial3d(materials.add(WALL_COLOR)),
        Transform::from_xyz(center.x, WALL_HEIGHT / 2.0, center.y),
    ));
}

fn wall_collision(
    walls: Query<(&Transform, &Wall), Without<Player>>,
    player: Single<(&mut Transform, &mut Velocity, &mut Health), With<Player>>,
) {
    let (
        mut transform,
        mut velocity,
        mut health
    ) = player.into_inner();

    for (wall_transform, wall) in &walls {
        let p = Vec2::new(transform.translation.x, transform.translation.z);
        let c = Vec2::new(wall_transform.translation.x, wall_transform.translation.z);

        // 壁の矩形のうち、プレイヤーの中心に一番近い点
        let closest = p.clamp(c - wall.half, c + wall.half);
        let diff = p - closest;
        let dist = diff.length();
        if PLAYER_RADIUS <= dist || dist == 0.0 {
            continue;
        }

        // 壁の外に押し出す
        let normal = diff / dist;
        let push = normal * (PLAYER_RADIUS - dist);
        transform.translation.x += push.x;
        transform.translation.z += push.y;

        // 壁に向かう速さ成分があれば、跳ね返してダメージ
        let v = Vec2::new(velocity.0.x, velocity.0.z);
        let into_wall = -v.dot(normal);
        if into_wall > 0.0 {
            let bounced = v + normal * into_wall * (1.0 + BOUNCE);
            velocity.0.x = bounced.x;
            velocity.0.z = bounced.y;

            if into_wall > DAMAGE_MIN_SPEED {
                health.0 -= into_wall * DAMAGE_PER_SPEED;
                info!("HP: {:.1}", health.0);
            }
        }
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
