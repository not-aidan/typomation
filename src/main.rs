use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::RenderPlugin;
use bevy::sprite::Anchor;
use bevy::winit::WinitPlugin;
use interpolation::*;
use std::time::Instant;

use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.build().disable::<WinitPlugin>())
        .init_resource::<Animation>()
        .add_system(pre_animation_system)
        .add_system(transform_track_system.after(pre_animation_system))
        .add_system(sprite_track_system.after(pre_animation_system))
        .add_startup_system(setup_system);

    let frames = 100;
    let mut images = Vec::<Image>::new();

    for _ in 0..frames {
        app.update();
        let camera = app
            .world
            .query::<&Camera>()
            .iter(&app.world)
            .next()
            .expect("Can't find camera");

        if let RenderTarget::Image(image_handle) = &camera.target {
            let image = app
                .world
                .get_resource::<Assets<Image>>()
                .expect("Couldn't get image assets")
                .get(image_handle)
                .expect("No image found in camera");
            images.push(image.clone());
        }
    }
}

fn pre_animation_system(mut animation: ResMut<Animation>) {
    animation.duration = Instant::now().duration_since(animation.start).as_secs_f32();
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
                    ease: None,
                },
                Key {
                    value: 500.0,
                    duration: 10.0,
                    ease: None,
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

#[derive(Component)]
struct SpriteTrack {
    color_r: Track,
    color_g: Track,
    color_b: Track,
    color_a: Track,
    flip_x: BoolTrack,
    flip_y: BoolTrack,
    anchor_x: Track,
    anchor_y: Track,
}

struct BoolKey {
    value: bool,
    duration: Scalar,
}

struct Key {
    value: Scalar,
    duration: Scalar,
    ease: Option<EaseFunction>,
}

impl Key {
    fn interpolate(&self, previous_key: &Self, duration: Scalar) -> Scalar {
        let interpolation = duration / self.duration;
        if let Some(ease) = self.ease {
            return previous_key
                .value
                .lerp(&self.value, &interpolation.calc(ease));
        }
        previous_key.value.lerp(&self.value, &interpolation)
    }
}

#[derive(Default)]
struct BoolTrack {
    keys: Vec<BoolKey>,
}

#[derive(Default)]
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

impl BoolTrack {
    fn new(keys: Vec<BoolKey>) -> Self {
        Self { keys }
    }

    fn value(&self, mut duration: Scalar) -> Option<bool> {
        let mut value: Option<bool> = None;
        let mut keys = self.keys.iter();

        if let Some(mut previous_key) = keys.next() {
            for key in keys {
                if duration > key.duration {
                    duration -= key.duration;
                    value = Some(key.value);
                    previous_key = key;
                    continue;
                }
                return Some(previous_key.value);
            }
        }

        value
    }
}

#[derive(Resource)]
struct Animation {
    start: Instant,
    duration: f32,
}

impl Default for Animation {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            duration: 0.0,
        }
    }
}

fn transform_track_system(
    mut query: Query<(&mut Transform, &TransformTrack)>,
    animation: Res<Animation>,
) {
    let duration = animation.duration;
    for (mut transform, track) in query.iter_mut() {
        let translation = transform.translation;
        let rotation = transform.rotation;
        let scale = transform.scale;

        transform.translation = Vec3::new(
            track.position_x.value(duration).unwrap_or(translation.x),
            track.position_y.value(duration).unwrap_or(translation.y),
            track.position_z.value(duration).unwrap_or(translation.z),
        );

        transform.scale = Vec3::new(
            track.scale_x.value(duration).unwrap_or(scale.x),
            track.scale_y.value(duration).unwrap_or(scale.y),
            track.scale_z.value(duration).unwrap_or(scale.z),
        );

        transform.rotation = Quat::from_euler(
            EulerRot::XYZ,
            track.rotation_x.value(duration).unwrap_or(rotation.x),
            track.rotation_y.value(duration).unwrap_or(rotation.y),
            track.rotation_z.value(duration).unwrap_or(rotation.z),
        );
    }
}

fn sprite_track_system(mut query: Query<(&mut Sprite, &SpriteTrack)>, animation: Res<Animation>) {
    let duration = animation.duration;
    for (mut sprite, track) in query.iter_mut() {
        let color = sprite.color;

        sprite.color = Color::rgba_linear(
            track.color_r.value(duration).unwrap_or(color.r()),
            track.color_g.value(duration).unwrap_or(color.g()),
            track.color_b.value(duration).unwrap_or(color.b()),
            track.color_a.value(duration).unwrap_or(color.a()),
        );

        sprite.flip_x = track.flip_x.value(duration).unwrap_or(sprite.flip_x);
        sprite.flip_y = track.flip_y.value(duration).unwrap_or(sprite.flip_y);

        let anchor = sprite.anchor.as_vec();
        sprite.anchor = Anchor::Custom(Vec2::new(
            track.anchor_x.value(duration).unwrap_or(anchor.x),
            track.anchor_y.value(duration).unwrap_or(anchor.y),
        ));
    }
}

