use bevy::prelude::*;

#[derive(Component)]
pub struct Projectile {
    pub speed: f32,
    pub angle: f32,
    pub timer: Timer,
}

pub fn projectile_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Projectile)>,
    mut command: Commands,
) {
    for (entity, mut transform, mut projectile) in query.iter_mut() {
        projectile.timer.tick(time.delta());
        let quat = Quat::from_rotation_z(projectile.angle);
        transform.translation += quat.mul_vec3(Vec3::X * projectile.speed * time.delta_seconds());

        if projectile.timer.just_finished() {
            command.entity(entity).despawn();
        }
    }
}
