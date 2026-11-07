use bevy::prelude::*;


const CAMERA_X: f32 = 0.0;
const CAMERA_Y: f32 = 14.0;
const CAMERA_Z: f32 = 6.0;

const LIGHT_X: f32 = 4.0;
const LIGHT_Y: f32 = 10.0;
const LIGHT_Z: f32 = 4.0;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.5, 0.7, 0.9)))
        .add_systems(Startup, setup)
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
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.9, 0.7, 0.6))),
            Transform::from_xyz(0.0, 0.5, 0.0),
    ));
}
