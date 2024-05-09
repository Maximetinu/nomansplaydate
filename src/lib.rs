#![no_std]

extern crate bevy;

use bevy::{prelude::*, time::TimePlugin, utils::Instant};
use bevy_playdate::prelude::*;
use bevy_playdate::sprites::PdSpritePlugin;
use crankstart::graphics::LCDColor;
use crankstart::sprite::{Sprite, SpriteManager};
use crankstart_sys::LCDBitmapFlip;
use euclid::Size2D;

extern crate alloc;

use {alloc::boxed::Box, alloc::format, anyhow::Error, bevy_playdate::*};

#[derive(Component)]
pub struct Ball(ScreenVector);

fn update_balls(mut locations: Query<(&mut Ball, &mut Transform)>) {
    for (mut ball, mut transform) in locations.iter_mut() {
        let velocity = ball.0;
        transform.translation.x += velocity.x as f32;
        transform.translation.y += velocity.y as f32;
        if (transform.translation.x < 8.0 && ball.0.x < 0)
            || (transform.translation.x > LCD_COLUMNS as f32 - 8.0 && ball.0.x > 0)
        {
            ball.0.x *= -1;
        }
        if (transform.translation.y < 8.0 && ball.0.y < 0)
            || (transform.translation.y > LCD_ROWS as f32 - 8.0 && ball.0.y > 0)
        {
            ball.0.y *= -1;
        }
    }
}

// TODO: make the load sprite a wrapper
fn load_sprite() -> Result<Sprite, Error> {
    let sprite_manager = SpriteManager::get_mut();
    let mut sprite = sprite_manager.new_sprite()?;
    let image = Graphics::get().load_bitmap("assets/heart")?;
    sprite.set_image(image, LCDBitmapFlip::kBitmapUnflipped)?;
    sprite.move_to(200.0, 120.0)?;
    sprite.set_z_index(10)?;
    sprite.set_opaque(false)?;
    sprite_manager.add_sprite(&sprite)?;
    Ok(sprite)
}

fn spawn_ball(mut commands: Commands) {
    let sprite = PdSprite(load_sprite().unwrap());
    PdSystem::log_to_console("Here!");
    let sprite = PdSpriteBundle {
        sprite,
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        global_transform: GlobalTransform::default(),
    };
    commands.spawn((Ball(ScreenVector::new(5, 5)), sprite));
}

fn log_fps(time: Res<Time>) {
    PdSystem::log_to_console(&format!("{}", 1.0 / time.delta_seconds()));
    PdSystem::log_to_console(&format!("{:?}", Instant::now()));
    if let Err(err) = crankstart::system::System::get().draw_fps(0, 0) {
        PdSystem::log_to_console(&format!("{:?}", err));
    }
}

#[derive(Component)]
pub struct HelloWorld(ScreenPoint, ScreenVector);

fn update_hello_world(mut locations: Query<&mut HelloWorld>) {
    for mut location in locations.iter_mut() {
        let velocity = location.1.clone();
        location.0 += velocity;
        if location.0.x < 0 || location.0.x > LCD_COLUMNS as i32 - 120 {
            location.1.x *= -1;
        }
        if location.0.y < 0 || location.0.y > LCD_ROWS as i32 - 16 {
            location.1.y *= -1;
        }
    }
}

fn draw_hello_world(locations: Query<&HelloWorld>) {
    let graphics = Graphics::get();
    let _ = graphics.clear(LCDColor::Solid(crankstart_sys::LCDSolidColor::kColorWhite));
    for location in locations.iter() {
        let _ = graphics.draw_text("Hello World Bevy", location.0);
    }
    let _ = crankstart::system::System::get().draw_fps(0, 0);
}

fn spawn_hellos(mut commands: Commands) {
    commands.spawn(HelloWorld(ScreenPoint::new(0, 0), ScreenVector::new(5, 5)));
    commands.spawn(HelloWorld(
        ScreenPoint::new(0, 50),
        ScreenVector::new(3, -3),
    ));
}

struct SpriteGame;

impl PlaydateBevyGame for SpriteGame {
    fn new(app: &mut App, _playdate: &crankstart::Playdate) -> Result<Box<Self>, Error> {
        app.add_plugins((TransformPlugin, PdSpritePlugin, TimePlugin));
        // Sprites
        app.add_systems(Update, update_balls)
            .add_systems(PostUpdate, log_fps)
            .add_systems(Startup, spawn_ball);
        // Hello Worlds
        // app.add_systems(Update, update_hello_world)
            app.add_systems(PostUpdate, draw_hello_world)
            .add_systems(Startup, spawn_hellos);
        Ok(Box::new(Self))
    }
}

type MyGame = PlaydateBevyState<SpriteGame>;

crankstart_game!(MyGame);
