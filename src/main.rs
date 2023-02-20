use interpolation::*;
use std::time::{Duration, Instant};

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Animation>()
        .add_system(transform_track_system)
        .add_startup_system(setup_system)
        .run();
}

fn setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("icon.png"),
            ..default()
        })
        .insert(TransformTrack {
            position_x: Track::new(vec![
                Key {
                    value: 0.0,
                    duration: 0.0,
                    ease: EaseFunction::CubicIn,
                },
                Key {
                    value: 500.0,
                    duration: 10.0,
                    ease: EaseFunction::CubicIn,
                },
            ]),
            ..Default::default()
        });
}

type Scalar = f32;

#[derive(Component, Default)]
struct TransformTrack {
    position_x: Track,
    position_y: Track,
    position_z: Track,
    rotation_x: Track,
    rotation_y: Track,
    rotation_z: Track,
    scale_x: Track,
    scale_y: Track,
    scale_z: Track,
}

struct Key {
    value: Scalar,
    duration: Scalar,
    ease: EaseFunction,
}

impl Key {
    fn interpolate(&self, previous_key: &Self, duration: Scalar) -> Scalar {
        let lerp = (duration / self.duration).calc(self.ease);
        let d = duration / self.duration;
        println!("lerp: {d} -> {lerp}");
        previous_key.value.lerp(&self.value, &lerp)
    }
}

impl Default for Track {
    fn default() -> Self {
        Self { keys: Vec::new() }
    }
}

struct Track {
    keys: Vec<Key>,
}

impl Track {
    fn new(keys: Vec<Key>) -> Self {
        Self { keys }
    }

    fn value(&self, mut duration: Scalar) -> Option<Scalar> {
        let mut value: Option<Scalar> = None;
        let mut keys = self.keys.iter();

        if let Some(mut previous_key) = keys.next() {
            for key in keys {
                if duration > key.duration {
                    duration -= key.duration;
                    value = Some(key.value);
                    previous_key = key;
                    continue;
                }
                return Some(key.interpolate(previous_key, duration));
            }
        }

        value
    }
}

#[derive(Resource)]
struct Animation {
    start: Instant,
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

fn transform_track_system(
    mut query: Query<(&mut Transform, &TransformTrack)>,
    animation: Res<Animation>,
) {
    let duration = Instant::now().duration_since(animation.start).as_secs_f32();
    for (mut transform, track) in query.iter_mut() {
        let translation = transform.translation;

        transform.translation = Vec3::new(
            track.position_x.value(duration).unwrap_or(translation.x),
            track.position_y.value(duration).unwrap_or(translation.y),
            track.position_z.value(duration).unwrap_or(translation.z),
        );
    }
}
