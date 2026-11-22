use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;

use std::time::Duration;

use crate::{Aim, AIM_MIN, Velocity};


pub const PLAYER_MODEL_PATH: &str = "naked_man_back_shark.glb";
// モデルの大きさ
const SHARK_OFFSET: Vec3 = Vec3::new(0.0, -0.5, 0.0);

const REVEAL_SPEED: f32 = 4.0;
const DOWN_SPEED: f32 = 6.0;
const MODEL_SMOOTHING: f32 = 4.0;

const ANIM_SPEED_MIN: f32 = 0.5;
const ANIM_SPEED_MAX: f32 = 6.0;

#[derive(Component)]
pub struct Transforming {
    pub target: Form,
}

const ANIM_DOWN: usize = 2;
const ANIM_IDLE: usize = 4;
const ANIM_REVEAL: usize = 5;
const ANIM_SWIM: usize = 6;
const ANIM_WALK: usize = 7;

#[derive(Component)]
pub struct ModelRoot;

#[derive(Resource)]
pub struct ModelAnimations {
    graph: Handle<AnimationGraph>,
    idle: AnimationNodeIndex,
    walk: AnimationNodeIndex,
    swim: AnimationNodeIndex,
    reveal: AnimationNodeIndex,
    down: AnimationNodeIndex,
}

#[derive(Component)]
pub struct PlayerAnim;

pub fn setup_player_animations(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let clip = |i: usize| -> Handle<AnimationClip> {
        asset_server.load(GltfAssetLabel::Animation(i).from_asset(PLAYER_MODEL_PATH))
    };
    let (graph, nodes) = AnimationGraph::from_clips([
        clip(ANIM_IDLE),
        clip(ANIM_WALK),
        clip(ANIM_SWIM),
        clip(ANIM_REVEAL),
        clip(ANIM_DOWN),
    ]);
    commands.insert_resource(ModelAnimations {
        graph: graphs.add(graph),
        idle: nodes[0],
        walk: nodes[1],
        swim: nodes[2],
        reveal: nodes[3],
        down: nodes[4],
    });
}

pub fn update_player_visual(
    time: Res<Time>,
    animations: Res<ModelAnimations>,
    aim: Res<Aim>,
    player: Single<(&Form, &Velocity, Option<&Transforming>), With<Player>>,
    mut model_root: Single<&mut Transform, With<ModelRoot>>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions), With<PlayerAnim>>,
    mut parts: Query<(&Name, &mut Visibility)>,
    mut current: Local<Option<AnimationNodeIndex>>,
) {
    let (form, velocity, transforming) = player.into_inner();

    let pull = aim.0.map(|v| v.length()).filter(|l| *l > AIM_MIN);
    let aiming = pull.is_some();

    let intensity = match pull {
        Some(len) => (len * form.power()).min(form.max_speed()) / form.max_speed(),
        None => (velocity.0.length() / form.max_speed()).min(1.0),
    };
    let anim_speed = ANIM_SPEED_MIN + (ANIM_SPEED_MAX - ANIM_SPEED_MIN) * intensity;

    let placement = if transforming.is_some() {
        Form::Human
    } else {
        *form
    };
    let scale = form.model_scale();
    let feet = Vec3::new(0.0, -Form::HUMAN_RADIUS, 0.0);
    let target_translation = match placement {
        Form::Human => feet,
        Form::Shark => feet + SHARK_OFFSET * scale,
    };
    let t = 1.0 - (-MODEL_SMOOTHING * time.delta_secs()).exp();
    model_root.translation = model_root.translation.lerp(target_translation, t);
    model_root.scale = model_root.scale.lerp(Vec3::splat(scale), t);

    for (name, mut visibility) in &mut parts {
        let show = match (name.as_str(), *form, transforming.is_some()) {
            ("Man" | "Shark" | "Crack", _, true) => true,
            ("Man", Form::Human, _) => true,
            ("Shark", Form::Shark, _) => true,
            ("Man" | "Shark" | "Crack", _, _) => false,
            _ => continue,
        };
        *visibility = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    let wanted = match transforming {
        Some(t) if t.target == Form::Shark => animations.reveal,
        Some(t) if t.target == Form::Human => animations.down,
        Some(_) => panic!(),
        None => match *form {
            Form::Human if velocity.0 == Vec3::ZERO && !aiming => animations.idle,
            Form::Human => animations.walk,
            Form::Shark => animations.swim,
        },
    };
    for (mut player, mut transitions) in &mut animation_players {
        if *current != Some(wanted) {
            let active = transitions
                .play(&mut player, wanted, Duration::from_millis(150));
            match transforming {
                Some(t) => match t.target {
                    Form::Shark => {
                        active.set_speed(REVEAL_SPEED);
                    },
                    Form::Human => {
                        active.set_speed(DOWN_SPEED);
                    },
                },
                None => {
                    active.repeat();
                },
            }
            *current = Some(wanted);
        }

        if transforming.is_none() && wanted != animations.idle {
            if let Some(active) = player.animation_mut(wanted) {
                active.set_speed(anim_speed);
            }
        }
    }
}

pub fn finish_transform(
    mut commands: Commands,
    animations: Res<ModelAnimations>,
    player: Single<(Entity, &mut Form, &Transforming), With<Player>>,
    animation_players: Query<&AnimationPlayer, With<PlayerAnim>>,
) {
    let (entity, mut form, transforming) = player.into_inner();
    let node = match transforming.target {
        Form::Shark => animations.reveal,
        Form::Human => animations.down,
    };
    let finished = animation_players.iter().next().map_or(true, |p| {
        p.animation(node).is_some_and(|a| a.is_finished())
    });
    if finished {
        *form = transforming.target;
        commands.entity(entity).remove::<Transforming>();
    }
}

pub fn on_player_model_ready(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    animations: Res<ModelAnimations>,
    children: Query<&Children>,
    mut animation_players: Query<&mut AnimationPlayer>,
) {
    for e in children.iter_descendants(ready.entity) {
        let Ok(mut player) = animation_players.get_mut(e) else { continue };
        let mut translations = AnimationTransitions::new();
        translations
            .play(&mut player, animations.idle, Duration::ZERO)
            .repeat();
        commands.entity(e).insert((
            AnimationGraphHandle(animations.graph.clone()),
            translations,
            PlayerAnim,
        ));
        break;
    }
}

#[derive(Component, Clone, Copy, PartialEq)]
pub enum Form {
    Human,
    Shark,
}

impl Form {
    const HUMAN_POWER: f32 = 3.0;
    const SHARK_POWER: f32 = 6.0;

    const HUMAN_MAX_SPEED: f32 = 10.0;
    const SHARK_MAX_SPEED: f32 = 30.0;

    pub const HUMAN_RADIUS: f32 = 0.5;
    const SHARK_RADIUS: f32 = 1.6;

    const HUMAN_SCALE: f32 = 1.0;
    const SHARK_SCALE: f32 = 0.8;

    /// 引っ張りの長さから速度への倍率
    pub fn power(self) -> f32 {
        match self {
            Form::Human => Self::HUMAN_POWER,
            Form::Shark => Self::SHARK_POWER,
        }
    }

    /// 速度の上限
    pub fn max_speed(self) -> f32 {
        match self {
            Form::Human => Self::HUMAN_MAX_SPEED,
            Form::Shark => Self::SHARK_MAX_SPEED,
        }
    }

    /// 当たり判定の半径
    pub fn radius(self) -> f32 {
        match self {
            Form::Human => Self::HUMAN_RADIUS,
            Form::Shark => Self::SHARK_RADIUS,
        }
    }

    /// モデルの倍率
    pub fn model_scale(self) -> f32 {
        match self {
            Form::Human => Self::HUMAN_SCALE,
            Form::Shark => Self::SHARK_SCALE,
        }
    }
}

#[derive(Component)]
pub struct Player;

impl Player {
    pub fn stop(
        mut velocity: Single<&mut Velocity, With<Self>>,
        mut aim: ResMut<Aim>,
    ) {
        velocity.0 = Vec3::ZERO;
        aim.0 = None;
    }
}
