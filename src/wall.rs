use bevy::prelude::*;

use crate::{
    BOUNCE,
    DAMAGE_MIN_SPEED,
    DAMAGE_PER_SPEED,
    Health,
    LevelEntity,
    Velocity
};
use crate::player::{Form, Player};


const WALL_HEIGHT: f32 = 1.0;

#[derive(Component)]
pub struct Wall {
    /// 壁の半分の大きさ
    half: Vec2,
}

const WALL_COLOR: Color = Color::srgb(0.4, 0.35, 0.3);

/// 壁を出す
pub fn spawm_wall(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    center: Vec2,
    size: Vec2,
) {
    commands.spawn((
        LevelEntity,
        Wall { half: size / 2.0 },
        Mesh3d(meshes.add(Cuboid::new(size.x, WALL_HEIGHT, size.y))),
        MeshMaterial3d(materials.add(WALL_COLOR)),
        Transform::from_xyz(center.x, WALL_HEIGHT / 2.0, center.y),
    ));
}

pub fn wall_collision(
    walls: Query<(&Transform, &Wall), Without<Player>>,
    player: Single<(&mut Transform, &mut Velocity, &mut Health, &Form), With<Player>>,
) {
    let (
        mut transform,
        mut velocity,
        mut health,
        form,
    ) = player.into_inner();
    let radius = form.radius();

    for (wall_transform, wall) in &walls {
        let p = Vec2::new(transform.translation.x, transform.translation.z);
        let c = Vec2::new(wall_transform.translation.x, wall_transform.translation.z);

        // 壁の矩形のうち、プレイヤーの中心に一番近い点
        let closest = p.clamp(c - wall.half, c + wall.half);
        let diff = p - closest;
        let dist = diff.length();
        if radius <= dist || dist == 0.0 {
            continue;
        }

        // 壁の外に押し出す
        let normal = diff / dist;
        let push = normal * (radius - dist);
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
            }
        }
    }
}
