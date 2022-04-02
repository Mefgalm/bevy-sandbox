mod animation_system;
mod app_state;
mod creep_system;
mod plugins;
mod projectile_system;
mod skill_system;

use std::collections::HashMap;
use std::time::Duration;

use crate::plugins::texture_plugin::TextureResourcePlugin;
use animation_system::{Animation, AnimationSystemPlugin, ChangeAnimation};
use app_state::AppState;
use bevy::asset::LoadState;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::render::camera::{DepthCalculation, ScalingMode};
use bevy::render::primitives::Frustum;
use bevy::render::view::VisibleEntities;
use bevy::window::{WindowMode, WindowResized};
use bevy::{prelude::*, sprite::collide_aabb::collide};
use creep_system::*;
use plugins::texture_plugin::{TextureDescription, TextureStore};
use projectile_system::projectile_system;
use skill_system::{skill_system, SkillSystem};

const WINDOW_WIDTH: f32 = 1280.;
const WINDOW_HEIGHT: f32 = 720.;

#[derive(Component)]
struct Player;

fn to2(v3: Vec3) -> Vec2 {
    Vec2::new(v3.x, v3.y)
}

#[derive(Component)]
pub struct ChangeAnimationHub(HashMap<CreepState, ChangeAnimation>);

fn run_animation(
    entity: Entity,
    commands: &mut Commands,
    creep_state: &CreepState,
    animation_hub: &ChangeAnimationHub,
) {
    let change_animation = animation_hub.0.get(creep_state).unwrap();
    commands.entity(entity).insert(change_animation.clone());
}

fn player_movable_system(
    time: Res<Time>,
    mouse_input: Res<Input<MouseButton>>,
    //window_info: Res<WindowInfo>,
    mut commands: Commands,
    windows: Res<Windows>,
    mut query: Query<(
        Entity,
        &mut Creep,
        &mut TextureAtlasSprite,
        &mut Transform,
        &ChangeAnimationHub,
        &Player,
    )>,
) {
    let (entity, mut creep, mut sprite, mut transform, animation_hub, _) = query.single_mut();
    let left_mouse_pressed = mouse_input.pressed(MouseButton::Left);
    let window = windows.get_primary().unwrap();
    if let Some(cursor_position) = window.cursor_position() {
        let scale = WINDOW_WIDTH / window.physical_width() as f32;
        let center = Vec2::new(
            window.physical_width() as f32 / 2.0,
            window.physical_height() as f32 / 2.0,
        );
        let cursor_position_centered =
            (cursor_position + to2(transform.translation) - center) * scale;
        let angle = f32::atan2(
            cursor_position_centered.y - transform.translation.y,
            cursor_position_centered.x - transform.translation.x,
        );
        creep.angle = angle;
        sprite.flip_x = cursor_position_centered.x < transform.translation.x;
    }
    if left_mouse_pressed {
        if creep.state != CreepState::Running {
            creep.state = CreepState::Running;
            run_animation(entity, &mut commands, &creep.state, animation_hub);
        }
        transform.translation += Quat::from_rotation_z(creep.angle)
            .mul_vec3(Vec3::X * creep.speed * time.delta_seconds());
    } else if creep.state != CreepState::Idle {
        creep.state = CreepState::Idle;
        run_animation(entity, &mut commands, &creep.state, animation_hub);
    }
}

// fn animation_change_system(
//     mut query: Query<
//         (
//             &Animation,
//             &mut TextureAtlasSprite,
//             &mut Handle<TextureAtlas>,
//         ),
//         Changed<Animation>,
//     >,
// ) {
//     for (animation, mut sprite, mut texure_atlas_handle) in query.iter_mut() {
//         match animation.state {
//             AnimationType::Idle => {
//                 *texure_atlas_handle = animation.idle_texture_atlast_handle.clone()
//             }
//             AnimationType::Move => {
//                 *texure_atlas_handle = animation.move_texture_atlast_handle.clone()
//             }
//         }
//         sprite.index = 0;
//     }
// }

// fn animate_sprite_system(
//     time: Res<Time>,
//     texture_atlases: Res<Assets<TextureAtlas>>,
//     mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
// ) {
//     for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
//         timer.tick(time.delta());
//         if timer.finished() {
//             let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
//             sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
//         }
//     }
// }

fn camera_follow(
    query_player: Query<&Transform, (With<Player>, Without<Camera>)>,
    mut query_camera: Query<
        &mut Transform,
        (
            With<Camera>,
            With<Frustum>,
            With<VisibleEntities>,
            Without<Player>,
        ),
    >,
) {
    let player_transform = query_player.single();
    let mut camera_tranform = query_camera.single_mut();
    camera_tranform.translation = player_transform.translation;
}

fn setup(mut commands: Commands, texture_store: Res<TextureStore>) {
    let half_width = WINDOW_WIDTH / 2.0;
    let helf_height = WINDOW_HEIGHT / 2.0;

    commands.spawn_bundle(OrthographicCameraBundle {
        orthographic_projection: OrthographicProjection {
            scaling_mode: ScalingMode::None,
            left: -half_width,
            right: half_width,
            top: helf_height,
            bottom: -helf_height,
            ..Default::default()
        },
        ..OrthographicCameraBundle::new_2d()
    });

    commands.spawn_bundle(UiCameraBundle::default());

    commands.spawn_bundle(SpriteBundle {
        texture: texture_store.get_image_handle("textures/map.png"),
        transform: Transform::from_scale(Vec2::splat(4.0).extend(0.0)),
        visibility: Visibility { is_visible: true },
        ..Default::default()
    });

    let run_atlas_handle = texture_store.get_atlas_handle("textures/chars/player/run.png");
    let idle_atlas_handle = texture_store.get_atlas_handle("textures/chars/player/idle.png");

    let changed_animations = HashMap::from_iter([
        (
            CreepState::Idle,
            ChangeAnimation {
                handle: idle_atlas_handle,
                timer: Timer::new(Duration::from_millis(100), true),
                frames: 15,
            },
        ),
        (
            CreepState::Running,
            ChangeAnimation {
                handle: run_atlas_handle.clone(),
                timer: Timer::new(Duration::from_millis(200), true),
                frames: 8,
            },
        ),
    ]);

    let default_change_animation = changed_animations.get(&CreepState::Idle).unwrap().clone();

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: run_atlas_handle,
            transform: Transform {
                scale: Vec3::new(3.0, 3.0, 0.0),
                translation: Vec2::ZERO.extend(5.0),
                ..Default::default()
            },
            visibility: Visibility { is_visible: true },
            ..Default::default()
        })
        .insert(Animation {
            timer: Timer::new(Duration::from_millis(100), true),
            frames: 15,
        })
        .insert(ChangeAnimationHub(changed_animations))
        .insert(default_change_animation)
        .insert(Creep {
            speed: 120.0,
            angle: 0.0,
            state: CreepState::Idle,
        })
        .insert(Player)
        .insert(SkillSystem {
            skills: vec![],
            cast_atlas_opt: Some(texture_store.get_atlas_handle("textures/chars/player/cast.png")),
            on_cast_index: Some(5),
        })
        .insert(Timer::from_seconds(0.1, true));
}

fn main() {
    let texture_resource_plugin = TextureResourcePlugin::new(vec![
        TextureDescription::new_atlas(
            "textures/chars/player/idle.png".to_owned(),
            64.0,
            64.0,
            15,
            1,
        ),
        TextureDescription::new_atlas("textures/chars/player/run.png".to_owned(), 96.0, 64.0, 8, 1),
        TextureDescription::new_atlas(
            "textures/chars/player/cast.png".to_owned(),
            144.0,
            64.0,
            22,
            1,
        ),
        TextureDescription::new_atlas("textures/skills/fireball.png".to_owned(), 64.0, 32.0, 5, 1),
        TextureDescription::new_image("textures/map.png".to_owned()),
    ]);

    App::new()
        .insert_resource(WindowDescriptor {
            title: "Game".to_owned(),
            resizable: true,
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            position: Some(Vec2::ZERO),
            scale_factor_override: Some(1.),
            vsync: false,
            decorations: true,
            cursor_visible: true,
            cursor_locked: false,
            mode: WindowMode::Windowed,
            transparent: false,
            ..Default::default()
        })
        // .init_resource::<WindowInfo>()
        .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
        .add_plugins(DefaultPlugins)
        .add_plugin(texture_resource_plugin)
        .add_plugin(AnimationSystemPlugin)
        .add_state(AppState::Setup)
        .add_system_set(SystemSet::on_enter(AppState::Active).with_system(setup))
        .add_system_set(
            SystemSet::on_update(AppState::Active)
                .with_system(player_movable_system)
                // .with_system(animation_change_system)
                .with_system(projectile_system)
                .with_system(skill_system)
                // .with_system(animate_sprite_system)
                .with_system(camera_follow),
        )
        .add_system(bevy::input::system::exit_on_esc_system)
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}
