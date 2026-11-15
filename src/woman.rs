use bevy::prelude::*;

use crate::{GameState, LevelEntity};
use crate::player::{Form, Player};


const WOMAN_RADIUS: f32 = 0.5;

#[derive(Component)]
pub struct Woman;

/// 女性を出す
pub fn spawn_woman(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    x: f32,
    z: f32,
) {
    commands.spawn((
        LevelEntity,
        Woman,
        Mesh3d(meshes.add(Capsule3d::new(0.35, 0.8))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.3, 0.5))),
        Transform::from_xyz(x, 0.75, z),
    ));
}

pub fn woman_collision(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    women: Query<(Entity, &Transform), (With<Woman>, Without<Player>)>,
    player: Single<(&Transform, &Form), With<Player>>,
) {
    let (player_transform, form) = player.into_inner();
    for (entity, woman_transform) in &women {
        let distance = player_transform.translation.distance(woman_transform.translation);
        if distance < form.radius() + WOMAN_RADIUS {
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
