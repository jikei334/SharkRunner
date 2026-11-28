use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;

use std::time::Duration;

use crate::{GameState, LevelEntity};
use crate::player::{Biting, BITE_DURATION, Form, Player};


const WOMAN_MODEL_PATH: &str = "beach_girl.glb";
const WOMAN_RADIUS: f32 = 0.5;

const ANIM_EATEN: usize = 0;
const ANIM_IDLE: usize = 1;
const ANIM_STARTLE: usize = 3;

#[derive(Component)]
pub struct Startled;

const EATEN_DURATION: f32 = 2.0;

#[derive(Component)]
pub struct Eaten(f32);

#[derive(Component)]
pub struct WomanAnim {
    root: Entity,
}

#[derive(Resource)]
pub struct WomanAnimations {
    graph: Handle<AnimationGraph>,
    idle: AnimationNodeIndex,
    startle: AnimationNodeIndex,
    eaten: AnimationNodeIndex,
}

impl WomanAnimations {
    pub fn setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut graphs: ResMut<Assets<AnimationGraph>>,
    ) {
        let clip = |i: usize| -> Handle<AnimationClip> {
            asset_server.load(GltfAssetLabel::Animation(i).from_asset(WOMAN_MODEL_PATH))
        };
        let (graph, nodes) = AnimationGraph::from_clips([
            clip(ANIM_IDLE),
            clip(ANIM_STARTLE),
            clip(ANIM_EATEN),
        ]);
        commands.insert_resource(WomanAnimations {
            graph: graphs.add(graph),
            idle: nodes[0],
            startle: nodes[1],
            eaten: nodes[2],
        });
    }

    fn on_ready(
        ready: On<WorldInstanceReady>,
        mut commands: Commands,
        animations: Res<WomanAnimations>,
        children: Query<&Children>,
        mut anim_players: Query<&mut AnimationPlayer>,
    ) {
        for e in children.iter_descendants(ready.entity) {
            let Ok(mut player) = anim_players.get_mut(e) else { continue };
            let mut transitions = AnimationTransitions::new();
            transitions
                .play(&mut player, animations.idle, Duration::ZERO)
                .repeat();
            commands.entity(e).insert((
                    AnimationGraphHandle(animations.graph.clone()),
                    transitions,
                    WomanAnim { root: ready.entity },
            ));
            break;
        }
    }

    pub fn react(
        animations: Res<WomanAnimations>,
        startled: Query<Entity, Added<Startled>>,
        eaten: Query<Entity, Added<Eaten>>,
        mut anims: Query<(&WomanAnim, &mut AnimationPlayer, &mut AnimationTransitions)>,
    ) {
        for root in &startled {
            for (
                wa,
                mut player,
                mut transitions,
            ) in &mut anims {
                if wa.root == root {
                    transitions.play(
                        &mut player,
                        animations.startle,
                        Duration::from_millis(100),
                    );
                }
            }
        }

        for root in &eaten {
            for (
                wa,
                mut player,
                mut transitions,
            ) in &mut anims {
                if wa.root == root {
                    transitions.play(
                        &mut player,
                        animations.eaten,
                        Duration::from_millis(100),
                    );
                }
            }
        }
    }

    pub fn tick_eaten(
        time: Res<Time>,
        mut commands: Commands,
        mut query: Query<(Entity, &mut Eaten)>,
    ) {
        for (entity, mut eaten) in &mut query {
            eaten.0 += time.delta_secs();
            if eaten.0 >= EATEN_DURATION {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[derive(Component)]
pub struct Woman;

impl Woman {
    /// 女性を出す
    pub fn spawn(
        commands: &mut Commands,
        asset_server: &AssetServer,
        x: f32,
        z: f32,
    ) {
        commands.spawn((
            LevelEntity,
            Woman,
            WorldAssetRoot(
                asset_server.load(GltfAssetLabel::Scene(0).from_asset(WOMAN_MODEL_PATH)),
            ),
            Transform::from_xyz(x, 0.0, z),
            Visibility::default(),
        ))
        .observe(WomanAnimations::on_ready);
    }

    pub fn collision(
        mut commands: Commands,
        mut next_state: ResMut<NextState<GameState>>,
        women: Query<
        (Entity, &Transform),
        (With<Woman>, Without<Player>, Without<Eaten>, Without<Startled>),
        >,
        player: Single<(Entity, &Transform, &Form), (With<Player>, Without<Woman>)>,
    ) {
        let (player_entity, player_transform, form) = player.into_inner();
        let p = Vec2::new(
            player_transform.translation.x,
            player_transform.translation.z
        );

        for (woman, woman_transform) in &women {
            let q = Vec2::new(woman_transform.translation.x, woman_transform.translation.z);
            if p.distance(q) >= form.radius() + WOMAN_RADIUS {
                continue;
            }
            match *form {
                Form::Shark => {
                    commands.entity(woman).insert(Eaten(0.0));
                    commands.entity(player_entity).insert(Biting(BITE_DURATION));
                },
                Form::Human => {
                    commands.entity(woman).insert(Startled);
                    next_state.set(GameState::Ambush);
                    return;
                },
            }
        }
    }
}
