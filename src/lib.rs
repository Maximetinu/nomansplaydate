#![no_std]

extern crate alloc;

use alloc::string::{String, ToString};
use bevy::app::{App, PreUpdate, Startup, Update};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, Query, Res, ResMut, Resource};
use bevy::prelude::{Deref, DerefMut};
use crankstart::log_to_console;
use crankstart::sprite::{Sprite, SpriteManager};
use crankstart_sys::{LCDBitmapFlip, PDButtons};
use {
    alloc::boxed::Box,
    anyhow::Error,
    crankstart::{
        crankstart_game,
        geometry::{ScreenPoint, ScreenVector},
        graphics::{Graphics, LCDColor, LCDSolidColor},
        system::System,
        Game, Playdate,
    },
    crankstart_sys::{LCD_COLUMNS, LCD_ROWS},
    euclid::vec2,
};

struct State {
    // location: ScreenPoint,
    // velocity: ScreenVector,
    // sprite: Sprite,
    app: App,
}

// XXX: This is not actually safe, but it's the best I can do for now.
unsafe impl Send for PdSprite {}
unsafe impl Sync for PdSprite {}
#[derive(Component, Deref, DerefMut)]
pub struct PdSprite(pub Sprite);

// XXX: This is not actually safe, but it's the best I can do for now.
unsafe impl Send for Text {}
unsafe impl Sync for Text {}
#[derive(Component, Deref, DerefMut)]
pub struct Text(pub String);

#[derive(Component)]
pub struct Extents {
    width: i32,
    height: i32,
}

#[derive(Component, Deref, DerefMut)]
pub struct Location(pub ScreenPoint);

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub ScreenVector);

#[derive(Component, Deref, DerefMut)]
pub struct Visibility(pub bool);

fn load_sprite() -> Result<Sprite, Error> {
    let sprite_manager = SpriteManager::get_mut();
    let mut sprite = sprite_manager.new_sprite()?;
    let image = Graphics::get().load_bitmap("assets/heart")?;
    sprite.set_image(image, LCDBitmapFlip::kBitmapUnflipped)?;

    // TODO: support z index
    // sprite.set_z_index(10)?;
    // TODO: support opacity
    // sprite.set_opaque(false)?;

    sprite_manager.add_sprite(&sprite)?;
    Ok(sprite)
}

fn print_tick() {
    // log_to_console!("TICK!");
}

// TODO: put some for loops and setup a benchmark
fn setup_example(mut commands: Commands) {
    const TEXT_WIDTH: i32 = 86;
    const TEXT_HEIGHT: i32 = 16;

    let random_pos_x = generate_random_number_in_range(150, 250) as i32;
    let random_pos_y = generate_random_number_in_range(100, 140) as i32;
    let random_vel_x = generate_random_number_in_range(0, 10) as i32 - 5i32;
    let random_vel_y = generate_random_number_in_range(0, 10) as i32 - 5i32;
    commands.spawn((
        Text("Hello".to_string()),
        Location(ScreenPoint::new(random_pos_x, random_pos_y)),
        Velocity(vec2(random_vel_x, random_vel_y)),
        Extents {
            width: TEXT_WIDTH,
            height: TEXT_HEIGHT,
        },
    ));

    let random_pos_x = generate_random_number_in_range(150, 250) as i32;
    let random_pos_y = generate_random_number_in_range(100, 140) as i32;
    let random_vel_x = generate_random_number_in_range(0, 10) as i32 - 5i32;
    let random_vel_y = generate_random_number_in_range(0, 10) as i32 - 5i32;
    commands.spawn((
        Text("World".to_string()),
        Location(ScreenPoint::new(random_pos_x, random_pos_y)),
        Velocity(vec2(random_vel_x, random_vel_y)),
        Extents {
            width: TEXT_WIDTH,
            height: TEXT_HEIGHT,
        },
    ));

    let random_pos_x = generate_random_number_in_range(150, 250) as i32;
    let random_pos_y = generate_random_number_in_range(100, 140) as i32;
    let random_vel_x = generate_random_number_in_range(0, 10) as i32 - 5i32;
    let random_vel_y = generate_random_number_in_range(0, 10) as i32 - 5i32;
    commands.spawn((
        PdSprite(load_sprite().unwrap()),
        Location(ScreenPoint::new(random_pos_x, random_pos_y)),
        Velocity(vec2(random_vel_x, random_vel_y)),
        // TODO: extents don't bounce as I expected
        Extents {
            width: 0,
            height: 0,
        },
        Visibility(true),
    ));

    let random_pos_x = generate_random_number_in_range(150, 250) as i32;
    let random_pos_y = generate_random_number_in_range(100, 140) as i32;
    let random_vel_x = generate_random_number_in_range(0, 10) as i32 - 5i32;
    let random_vel_y = generate_random_number_in_range(0, 10) as i32 - 5i32;
    commands.spawn((
        PdSprite(load_sprite().unwrap()),
        Location(ScreenPoint::new(random_pos_x, random_pos_y)),
        Velocity(vec2(random_vel_x, random_vel_y)),
        // TODO: extents don't bounce as I expected
        Extents {
            width: 0,
            height: 0,
        },
        Visibility(false),
    ));
}

fn clear_framebuffer() {
    let graphics = Graphics::get();
    graphics.clear_context().unwrap();
    graphics
        .clear(LCDColor::Solid(LCDSolidColor::kColorWhite))
        .unwrap();
}

fn draw_text(mut text_q: Query<(&Text, &mut Location)>) {
    let graphics = Graphics::get();
    for (text_str, text_location) in text_q.iter_mut() {
        graphics.draw_text(text_str, **text_location).unwrap();
    }
}

fn move_and_bounce(mut movable_q: Query<(&mut Location, &mut Velocity, &Extents)>) {
    for (mut location, mut velocity, extents) in movable_q.iter_mut() {
        **location += **velocity;

        if location.x < 0 || location.x > LCD_COLUMNS as i32 - extents.width {
            velocity.x = -velocity.x;
        }

        if location.y < 0 || location.y > LCD_ROWS as i32 - extents.height {
            velocity.y = -velocity.y;
        }
    }
}

fn show_hide_sprites(mut sprite_visibility_q: Query<&mut Visibility, With<PdSprite>>) {
    let button_pushed = {
        let (_, pushed, _) = System::get().get_button_state().unwrap();
        (pushed & PDButtons::kButtonA).0 != 0
    };

    if !button_pushed {
        return;
    }

    for mut sprite_visibility in sprite_visibility_q.iter_mut() {
        **sprite_visibility = !**sprite_visibility;
    }
}

fn apply_visibility(mut sprite_q: Query<(&mut PdSprite, &Visibility)>) {
    for (mut sprite, visibility) in sprite_q.iter_mut() {
        sprite.set_visible(**visibility).unwrap();
    }
}

fn draw_sprites(mut sprite_q: Query<(&mut PdSprite, &Location)>) {
    for (mut sprite, location) in sprite_q.iter_mut() {
        sprite
            .move_to(location.x as f32, location.y as f32)
            .unwrap();
        sprite.set_z_index(10).unwrap();
        sprite.set_opaque(false).unwrap();
    }
}

fn draw_fps() {
    System::get().draw_fps(0, 0).unwrap();
}

#[derive(Resource, Default, Clone, Copy, Deref, DerefMut, PartialEq, Eq, PartialOrd, Ord)]
struct TargetInstanceCount(pub i64);

fn spawn_despawn_sprites_to_match_instance_count(
    mut commands: Commands,
    target_instance_count: Res<TargetInstanceCount>,
    sprites_q: Query<Entity, With<PdSprite>>,
) {
    let current_instance_count = sprites_q.iter().len();

    let difference = current_instance_count as i32 - **target_instance_count as i32;

    if difference > 0 {
        // there are more than should be, let's despawn them
        let mut i = 0;
        for e in sprites_q.iter() {
            commands.entity(e).despawn();
            i += 1;
            if i == difference {
                break;
            }
        }
    } else if difference < 0 {
        // there are less than should be, let's spawn new ones
        let mut i = difference;
        while i != 0 {
            let random_pos_x = generate_random_number_in_range(150, 250) as i32;
            let random_pos_y = generate_random_number_in_range(100, 140) as i32;
            let random_vel_x = generate_random_number_in_range(0, 10) as i32 - 5i32;
            let random_vel_y = generate_random_number_in_range(0, 10) as i32 - 5i32;
            commands.spawn((
                PdSprite(load_sprite().unwrap()),
                Location(ScreenPoint::new(random_pos_x, random_pos_y)),
                Velocity(vec2(random_vel_x, random_vel_y)),
                // TODO: extents don't bounce as I expected
                Extents {
                    width: 0,
                    height: 0,
                },
                Visibility(true),
            ));
            i += 1;
        }
    }
}

// fn increment_target_instance_count(
//     mut target_instance_count: ResMut<TargetInstanceCount>,
//     mut frame_number: Local<u32>,
// ) {
//     *frame_number += 1;
//     *frame_number %= 1; // increment this to slow down spawn rate

//     if *frame_number == 0 {
//         **target_instance_count += 1;
//     }
// }

fn update_target_instance_count_from_input(mut target_instance_count: ResMut<TargetInstanceCount>) {
    let change = System::get().get_crank_change().unwrap() as i64;

    if change == 0 {
        return;
    }

    **target_instance_count += change;
    if **target_instance_count < 0 {
        **target_instance_count = 0;
    }

    log_to_console!("COUNT: {}", **target_instance_count);
}

impl State {
    pub fn new(_playdate: &Playdate) -> Result<Box<Self>, Error> {
        // crankstart::display::Display::get().set_refresh_rate(20.0)?; // FPS
        // let sprite = load_sprite()?;

        let mut app = App::new();

        app.init_resource::<TargetInstanceCount>();

        app.add_systems(Update, print_tick)
            // THIS PANICS
            // .add_systems(Update, update_example)
            .add_systems(PreUpdate, clear_framebuffer)
            .add_systems(Update, draw_text)
            .add_systems(Update, move_and_bounce)
            .add_systems(Update, show_hide_sprites)
            .add_systems(Update, apply_visibility)
            .add_systems(Update, draw_sprites)
            .add_systems(Update, draw_fps)
            .add_systems(Update, update_target_instance_count_from_input)
            // .add_systems(Update, increment_target_instance_count)
            .add_systems(Update, spawn_despawn_sprites_to_match_instance_count)
            .add_systems(Startup, setup_example);

        Ok(Box::new(Self {
            // location: point2(INITIAL_X, INITIAL_Y),
            // velocity: vec2(1, 2),
            // sprite,
            app,
        }))
    }
}

impl Game for State {
    fn update(&mut self, _playdate: &mut Playdate) -> Result<(), Error> {
        // let graphics = Graphics::get();
        // graphics.clear_context()?;
        // graphics.clear(LCDColor::Solid(LCDSolidColor::kColorWhite))?;
        // graphics.draw_text("Hello World Rust", self.location)?;

        // self.location += self.velocity;

        // if self.location.x < 0 || self.location.x > LCD_COLUMNS as i32 - TEXT_WIDTH {
        //     self.velocity.x = -self.velocity.x;
        // }

        // if self.location.y < 0 || self.location.y > LCD_ROWS as i32 - TEXT_HEIGHT {
        //     self.velocity.y = -self.velocity.y;
        // }

        // let (_, pushed, _) = System::get().get_button_state()?;
        // if (pushed & PDButtons::kButtonA).0 != 0 {
        //     log_to_console!("Button A pushed");
        //     self.sprite
        //         .set_visible(!self.sprite.is_visible().unwrap_or(false))
        //         .unwrap();
        // }

        // System::get().draw_fps(0, 0)?;

        self.app.update();

        Ok(())
    }

    fn update_sprite(
        &mut self,
        sprite: &mut Sprite,
        _playdate: &mut Playdate,
    ) -> Result<(), Error> {
        sprite.mark_dirty()?;
        Ok(())
    }
}

crankstart_game!(State);

mod bad_critical_section {
    use critical_section::{set_impl, Impl, RawRestoreState};

    struct SingleCoreCriticalSection;
    set_impl!(SingleCoreCriticalSection);

    unsafe impl Impl for SingleCoreCriticalSection {
        unsafe fn acquire() -> RawRestoreState {
            false
        }

        unsafe fn release(_was_active: RawRestoreState) {
            // We're really dumb.
        }
    }
}

extern crate getrandom;

#[allow(dead_code)]
fn generate_random_number() -> u32 {
    let mut buffer = [0u8; 4];
    getrandom::getrandom(&mut buffer).unwrap();
    u32::from_ne_bytes(buffer)
}

fn generate_random_number_in_range(min: u32, max: u32) -> u32 {
    use rand::Rng;
    use rand::SeedableRng;

    let mut buffer = [0u8; 8]; // Using 8 bytes for a larger seed space
    getrandom::getrandom(&mut buffer).unwrap();
    let seed = u64::from_ne_bytes(buffer);

    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    rng.gen_range(min..max)
}

const SEED_MASK: u64 = 0xdeadbeefbadc0ded;

fn getrandom_seeded(dest: &mut [u8]) -> Result<(), getrandom::Error> {
    use rand::Rng;
    use rand::SeedableRng;

    let seconds = crankstart::system::System::get()
        .get_seconds_since_epoch()
        .unwrap();
    let seed = seconds.1 as u64 + (seconds.0 as u64) << 32;
    let seed = SEED_MASK ^ seed;

    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    rng.fill(dest);
    Ok(())
}

getrandom::register_custom_getrandom!(getrandom_seeded);
