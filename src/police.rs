use bevy::{asset, prelude::*};
use bevy::world_serialization::WorldInstanceReady;
use bevy::audio::Volume;

use std::path::Component;
use std::time::Duration;

use crate::{
    GameState,
    LevelEntity,
};
use crate::player::{
    Form,
    Player,
};


pub const POLICE_MODEL_PATH: &str = "police_chainsaw.glb";
const POLICE_SCALE: f32 = 1.0;

const ANIM_ATTACK: usize = 0;
const ANIM_IDLE: usize = 1;

pub const ATTACK_HIT_TIME: f32 = 0.8;

/// 攻撃中の印
#[derive(Component)]
pub struct Attacking;

const AMBUSH_COUNT: usize = 4;
const AMBUSH_RADIUS: f32 = 1.4;
const DROP_HEIGHT: f32 = 10.0;
const DROP_STAGGER: f32 = 4.0;
const DROP_SPEED: f32 = 20.0;

const SLASH_SE_PATH: &str = "chainsaw_slash.ogg";
const SLASH_VOLUME: f32 = 0.6;

/// 落下中の印
#[derive(Component)]
pub struct Dropping;

#[derive(Resource)]
pub struct AmbushTimer(f32);

/// アニメーション再生側に付ける印
#[derive(Component)]
pub struct PoliceAnim {
    root: Entity,
}

/// 捕まってからの経過秒数
#[derive(Resource)]
pub struct CaughtTimer(pub f32);

#[derive(Component)]
pub struct Trap;

impl Trap {
    pub const RADIUS: f32 = 0.6;

    pub fn collision(
        mut commands: Commands,
        mut next_state: ResMut<NextState<GameState>>,
        mut traps: Query<(Entity, &mut Transform), (With<Trap>, Without<Player>)>,
        player: Single<(&Transform, &Form), With<Player>>,
    ) {
        let (pt, form) = player.into_inner();
        let p = Vec2::new(pt.translation.x, pt.translation.z);
        for (entity, mut t) in &mut traps {
            let q = Vec2::new(t.translation.x, t.translation.z);
            if p.distance(q) < form.radius() + Self::RADIUS {
                let dir = p - q;
                t.rotation = Quat::from_rotation_y(dir.x.atan2(dir.y));
                commands.entity(entity).insert(Attacking);
                commands.insert_resource(CaughtTimer(0.0));
                next_state.set(GameState::Caught);
                info!("トラップ!");
                return;
            }
        }
    }
}

#[derive(Resource)]
pub struct PoliceAnimations {
    graph: Handle<AnimationGraph>,
    idle: AnimationNodeIndex,
    attack: AnimationNodeIndex,
    slash_se: Handle<AudioSource>,
}

impl PoliceAnimations {
    pub fn setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut graphs: ResMut<Assets<AnimationGraph>>,
    ) {
        let clip = |i: usize| -> Handle<AnimationClip> {
            asset_server.load(GltfAssetLabel::Animation(i).from_asset(POLICE_MODEL_PATH))
        };
        let (graph, nodes) = AnimationGraph::from_clips([
            clip(ANIM_IDLE),
            clip(ANIM_ATTACK),
        ]);
        let slash_se: Handle<AudioSource> = asset_server.load(SLASH_SE_PATH);
        commands.insert_resource(Self {
            graph: graphs.add(graph),
            idle: nodes[0],
            attack: nodes[1],
            slash_se,
        });
    }

    fn on_ready(
        ready: On<WorldInstanceReady>,
        mut commands: Commands,
        animations: Res<PoliceAnimations>,
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
                PoliceAnim { root: ready.entity },
            ));
            break;
        }
    }

    pub fn spawn(
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        pos: Vec2,
        y: f32
    ) -> Entity {
        commands
            .spawn((
                LevelEntity,
                Trap,
                WorldAssetRoot(
                    asset_server.load(GltfAssetLabel::Scene(0).from_asset(POLICE_MODEL_PATH)),
                ),
                Transform::from_xyz(pos.x, y, pos.y).with_scale(Vec3::splat(POLICE_SCALE)),
                Visibility::default(),
            ))
            .observe(Self::on_ready)
            .id()
    }

    pub fn attack(
        mut commands: Commands,
        animations: Res<Self>,
        attacking: Query<Entity, Added<Attacking>>,
        mut anims: Query<(&PoliceAnim, &mut AnimationPlayer, &mut AnimationTransitions)>,
    ) {
        for root in &attacking {
            commands.spawn((
                    LevelEntity,
                    AudioPlayer::new(animations.slash_se.clone()),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(SLASH_VOLUME)),
            ));

            for (pa, mut player, mut transitions) in &mut anims {
                if pa.root == root {
                    transitions
                        .play(
                            &mut player,
                            animations.attack,
                            Duration::from_millis(100)
                        );
                }
            }
        }
    }

    pub fn caught(
        time: Res<Time>,
        mut timer: ResMut<CaughtTimer>,
        mut next_state: ResMut<NextState<crate::GameState>>,
    ) {
        timer.0 += time.delta_secs();
        if timer.0 >= ATTACK_HIT_TIME {
            next_state.set(GameState::Defeated);
        }
    }

    pub fn start_ambush(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        player: Single<&Transform, With<Player>>,
    ) {
        let p = Vec2::new(player.translation.x, player.translation.z);
        for i in 0..AMBUSH_COUNT {
            let angle = i as f32 / AMBUSH_COUNT as f32 * std::f32::consts::TAU + 0.4;
            let pos = p + Vec2::new(angle.cos(), angle.sin()) * AMBUSH_RADIUS;
            let height = DROP_HEIGHT + i as f32 * DROP_STAGGER;
            let e = Self::spawn(&mut commands, &asset_server, pos, height);
            commands.entity(e).insert(Dropping);
        }
        commands.insert_resource(AmbushTimer(0.0));
    }

    pub fn ambush(
        time: Res<Time>,
        mut commands: Commands,
        mut timer: ResMut<AmbushTimer>,
        mut next_state: ResMut<NextState<GameState>>,
        mut falling: Query<(Entity, &mut Transform), (With<Dropping>, Without<Player>)>,
        player: Single<&Transform, (With<Player>, Without<Dropping>)>,
    ) {
        let target = player.translation;
        let dt = time.delta_secs();

        if falling.is_empty() {
            timer.0 += dt;
            if timer.0 >= ATTACK_HIT_TIME {
                next_state.set(GameState::Defeated);
            }
            return;
        }

        for (entity, mut t) in &mut falling {
            let dir = target - t.translation;
            t.rotation = Quat::from_rotation_y(dir.x.atan2(dir.z));

            t.translation.y -= DROP_SPEED * dt;
            if t.translation.y <= 0.0 {
                t.translation.y = 0.0;
                commands.entity(entity).remove::<Dropping>().insert(Attacking);
            }
        }
    }
}
