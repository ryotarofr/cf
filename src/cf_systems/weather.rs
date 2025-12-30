use bevy::prelude::*;
use rand::Rng;

use crate::components::{RainDrop, SunLight};
use crate::constants::*;
use crate::resources::WeatherState;

/// 天候状態を更新するシステム
pub fn update_weather(
    mut weather: ResMut<WeatherState>,
    time: Res<Time>,
    mut sun_query: Query<&mut DirectionalLight, With<SunLight>>,
) {
    let mut rng = rand::rng();

    weather.time_until_change -= time.delta_secs();

    if weather.time_until_change <= 0.0 {
        weather.is_raining = !weather.is_raining;

        weather.time_until_change = if weather.is_raining {
            rng.random_range(WEATHER_RAIN_DURATION_MIN..WEATHER_RAIN_DURATION_MAX)
        } else {
            rng.random_range(WEATHER_CLEAR_DURATION_MIN..WEATHER_CLEAR_DURATION_MAX)
        };

        if let Ok(mut sun_light) = sun_query.single_mut() {
            sun_light.illuminance = if weather.is_raining {
                SUN_ILLUMINANCE_RAIN
            } else {
                SUN_ILLUMINANCE_CLEAR
            };
        }

        println!(
            "天候変化: {} (次の変化まで: {:.1}秒)",
            if weather.is_raining { "雨" } else { "晴れ" },
            weather.time_until_change
        );
    }
}

/// 雨粒を生成するシステム
pub fn spawn_rain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    weather: Res<WeatherState>,
) {
    if !weather.is_raining {
        return;
    }

    let mut rng = rand::rng();
    let drops_to_spawn = (RAIN_SPAWN_RATE * time.delta_secs()) as i32;
    let field_size = FIELD_SIZE as f32 * BLOCK_SIZE;

    for _ in 0..drops_to_spawn {
        let x = rng.random_range(-field_size / 2.0..field_size / 2.0);
        let z = rng.random_range(-field_size / 2.0..field_size / 2.0);

        let rain_material = materials.add(StandardMaterial {
            base_color: Color::srgba(RAIN_COLOR.0, RAIN_COLOR.1, RAIN_COLOR.2, RAIN_COLOR.3),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        });

        let rain_mesh = meshes.add(Capsule3d::new(RAIN_CAPSULE_RADIUS, RAIN_CAPSULE_HEIGHT));

        commands.spawn((
            Mesh3d(rain_mesh),
            MeshMaterial3d(rain_material),
            Transform::from_xyz(x, RAIN_SPAWN_HEIGHT, z),
            RainDrop {
                velocity: Vec3::new(0.0, RAIN_FALL_VELOCITY, 0.0),
                lifetime: RAIN_LIFETIME,
            },
        ));
    }
}

/// 雨粒を更新するシステム
pub fn update_rain(
    mut commands: Commands,
    mut rain_query: Query<(Entity, &mut Transform, &mut RainDrop)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut raindrop) in rain_query.iter_mut() {
        transform.translation += raindrop.velocity * time.delta_secs();
        raindrop.lifetime -= time.delta_secs();

        if transform.translation.y < 0.0 || raindrop.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
