use bevy::prelude::*;


const CAMERA_X: f32 = 0.0;
const CAMERA_Y: f32 = 14.0;
const CAMERA_Z: f32 = 6.0;

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
// スピードを0とみなす閾値
const SPEED_THRESHOLD_ZERO: f32 = 0.05;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.5, 0.7, 0.9)))
        .init_resource::<DragStart>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                switch_form,
                drag_input,
                movement,
            )
            .chain()
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
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.9, 0.7, 0.6))),
            Transform::from_xyz(0.0, 0.5, 0.0),
    ));
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
