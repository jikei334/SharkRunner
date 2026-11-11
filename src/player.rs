use bevy::prelude::*;


pub const PLAYER_RADIUS: f32 = 0.5;

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

    const HUMAN_COLOR: Color = Color::srgb(0.9, 0.7, 0.6);
    const SHARK_COLOR: Color = Color::srgb(0.3, 0.4, 0.6);

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

    pub fn color(self) -> Color {
        match self {
            Form::Human => Self::HUMAN_COLOR,
            Form::Shark => Self::SHARK_COLOR,
        }
    }
}

#[derive(Component)]
pub struct Player;
