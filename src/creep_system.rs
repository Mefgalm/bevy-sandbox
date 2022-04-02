use bevy::{prelude::*};

#[derive(Hash, PartialEq, Eq)]
pub enum CreepState {
    Idle,
    Running,
}

#[derive(Component)]
pub struct Creep {
    pub speed: f32,
    pub angle: f32,
    pub state: CreepState
}
