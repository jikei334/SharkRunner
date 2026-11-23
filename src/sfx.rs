use bevy::prelude::*;
use bevy::audio::Volume;

use crate::Health;
use crate::player::{Form, Player, Transforming};
use crate::woman::{Eaten, Startled};
use crate::{Bgm, LevelEntity};

const SCREAM_VOLUME: f32 = 0.6;
const GAME_OVER_VOLUME: f32 = 0.7;
const CLEAR_VOLUME: f32 = 0.7;
const TRANSFORM_VOLUME: f32 = 0.6;

const SCREAM_SE_PATH: &str = "scream_female_high.ogg";
const GAME_OVER_SE_PATH: &str = "game_over.ogg";
const CLEAR_SE_PATH: &str = "game_clear_goal.ogg";
const TO_SHARK_SE_PATH: &str = "transform_to_shark.ogg";
const TO_HUMAN_SE_PATH: &str = "transform_to_human.ogg";
const WALL_HUMAN_SE_PATH: &str = "wall_hit_human.ogg";
const WALL_SHARK_SE_PATH: &str = "wall_hit_shark.ogg";

const WALL_HIT_VOLUME_MIN: f32 = 0.4;
const WALL_HIT_VOLUME_MAX: f32 = 0.9;

#[derive(Resource)]
pub struct Sfx {
    scream: Handle<AudioSource>,
    game_over: Handle<AudioSource>,
    clear: Handle<AudioSource>,
    to_shark: Handle<AudioSource>,
    to_human: Handle<AudioSource>,
    wall_human: Handle<AudioSource>,
    wall_shark: Handle<AudioSource>,
}

impl Sfx {
    pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        commands.insert_resource(Sfx {
            scream: asset_server.load(SCREAM_SE_PATH),
            game_over: asset_server.load(GAME_OVER_SE_PATH),
            clear: asset_server.load(CLEAR_SE_PATH),
            to_shark: asset_server.load(TO_SHARK_SE_PATH),
            to_human: asset_server.load(TO_HUMAN_SE_PATH),
            wall_human: asset_server.load(WALL_HUMAN_SE_PATH),
            wall_shark: asset_server.load(WALL_SHARK_SE_PATH),
        });
    }

    fn play(commands: &mut Commands, handle: &Handle<AudioSource>, volume: f32) {
        commands.spawn((
                LevelEntity,
                AudioPlayer::new(handle.clone()),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume)),
        ));
    }

    /// 女性が驚いたとき
    pub fn play_scream(
        mut commands: Commands,
        sfx: Res<Self>,
        startled: Query<(), Added<Startled>>,
        eaten: Query<(), Added<Eaten>>,
    ) {
        if !startled.is_empty() || !eaten.is_empty() {
            Self::play(&mut commands, &sfx.scream, SCREAM_VOLUME);
        }
    }

    /// 変身が始まったとき
    pub fn play_transform(
        mut commands: Commands,
        sfx: Res<Self>,
        transforming: Query<&Transforming, Added<Transforming>>,
    ) {
        for t in &transforming {
            let handle = match t.target {
                Form::Shark => &sfx.to_shark,
                Form::Human => &sfx.to_human,
            };
            Self::play(&mut commands, handle, TRANSFORM_VOLUME);
        }
    }

    pub fn play_game_over(mut commands: Commands, sfx: Res<Self>) {
        Self::play(&mut commands, &sfx.game_over, GAME_OVER_VOLUME);
    }

    pub fn play_clear(mut commands: Commands, sfx: Res<Self>) {
        Self::play(&mut commands, &sfx.clear, CLEAR_VOLUME);
    }

    pub fn pause_bgm(bgm: Single<&AudioSink, With<Bgm>>) {
        bgm.pause();
    }

    pub fn resume_bgm(bgm: Single<&AudioSink, With<Bgm>>) {
        bgm.play();
    }

    pub fn play_wall_hit(
        mut commands: Commands,
        sfx: Res<Self>,
        player: Single<(&Health, &Form), With<Player>>,
        mut last: Local<Option<f32>>,
    ) {
        let (health, form) = player.into_inner();
        if let Some(prev) = *last {
            let damage = prev - health.0;
            if damage > 0.0 {
                let handle = match *form {
                    Form::Human => &sfx.wall_human,
                    Form::Shark => &sfx.wall_shark,
                };
                let volume = (WALL_HIT_VOLUME_MIN + damage / 25.0).min(WALL_HIT_VOLUME_MAX);
                Self::play(&mut commands, handle, volume);
            }
        }
        *last = Some(health.0);
    }
}
