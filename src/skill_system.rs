use std::time::Duration;

use bevy::prelude::*;

use crate::{
    creep_system::Creep, plugins::texture_plugin::TextureStore, projectile_system::Projectile,
};


#[derive(Component)]
pub enum SkillType {
    Fireball,
}

#[derive(Component)]
pub struct Skill {
    pub skill_type: SkillType,
    pub cd: f32,
}

#[derive(Component)]
pub struct SkillSystem {
    pub skills: Vec<Skill>,
    pub cast_atlas_opt: Option<Handle<TextureAtlas>>,
    pub on_cast_index: Option<usize>
}


#[derive(Component)]
pub struct CastState {
    pub index: usize,
    pub duration: Duration,
}

fn fireball_spawn(
    creep: &Creep,
    transform: &Transform,
    texture_store: &Res<TextureStore>,
    commands: &mut Commands,
    texture_atlases: &Res<Assets<TextureAtlas>>,
) {
    let move_texture_atlas = texture_store.get_atlas_handle("textures/skills/fireball.png");
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: move_texture_atlas.clone(),
            transform: Transform {
                translation: transform.translation,
                rotation: Quat::from_rotation_z(creep.angle),
                scale: Vec3::splat(2.0),
            },
            ..Default::default()
        })
        .insert(Projectile {
            speed: 190.0,
            angle: creep.angle,
            timer: Timer::from_seconds(3.0, false),
        })
        .insert(Timer::from_seconds(0.1, true));
}

pub fn skill_system(
    time: Res<Time>,
    texture_store: Res<TextureStore>,
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        Entity,
        &Creep,
        &SkillSystem,
        &Transform,
    )>,
) {
    let delta = time.delta();
    for (
        entity,
        creep,
        skill_system,
        transform,
    ) in query.iter_mut()
    {
        if keyboard_input.pressed(KeyCode::Q) {
            fireball_spawn(
                creep,
                &transform,
                &texture_store,
                &mut commands,
                &texture_atlases,
            );
        }
    }
}
