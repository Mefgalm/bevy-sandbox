use bevy::prelude::*;

use crate::app_state::AppState;

#[derive(Component)]
pub struct Animation {
    pub timer: Timer,
    pub frames: usize,
}

#[derive(Component, Clone)]
pub struct ChangeAnimation {
    pub handle: Handle<TextureAtlas>,
    pub timer: Timer,
    pub frames: usize,
}

fn change_animation_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Handle<TextureAtlas>,
            &mut TextureAtlasSprite,
            &mut Animation,
            &ChangeAnimation,
        ),
        Added<ChangeAnimation>,
    >,
) {
    for (entity, mut texture_atlas_handle, mut sprite, mut animation, change_animation) in
        query.iter_mut()
    {
        sprite.index = 0;
        *texture_atlas_handle = change_animation.handle.clone();

        animation.timer = change_animation.timer.clone();
        animation.frames = change_animation.frames;

        commands.entity(entity).remove::<ChangeAnimation>();
    }
}

fn animation_system(time: Res<Time>, mut query: Query<(&mut TextureAtlasSprite, &mut Animation)>) {
    for (mut sprite, mut animation) in query.iter_mut() {
        animation.timer.tick(time.delta());
        if animation.timer.finished() {
            sprite.index = (sprite.index + 1) % animation.frames;
        }
    }
}

pub struct AnimationSystemPlugin;

impl Plugin for AnimationSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::Active)
                .with_system(animation_system)
                .with_system(change_animation_system),
        );
    }
}
