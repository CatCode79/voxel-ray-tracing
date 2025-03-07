//= MODS =====================================================================

pub mod game;
pub mod open_simplex;
pub mod player;
pub mod world;

//= IMPORTS ==================================================================

use crate::game::GameState;
use crate::world::World;

use voxel_config::Config;
use voxel_render::Renderer;
use voxel_winput::{
    input::{InputSource, KeyCode},
    mapping::{InputKind, InputMapping},
    window::{Event, Window},
};

use log::LevelFilter;

use std::process;

//= CONSTS ===================================================================

const CONFIG_FILEPATH: &str = "config.json";

//= MAIN =====================================================================

fn main() -> process::ExitCode {
    // Possible values for filter are, in order:
    // Off, Error, Warn, Info, Debug, Trace.
    env_logger::builder()
        .filter(None, LevelFilter::Info)
        .filter_module("naga", LevelFilter::Warn)
        .filter_module("wgpu", LevelFilter::Warn)
        .init();

    let config = Config::load_or_default(CONFIG_FILEPATH);

    let mut window = create_window(&config);

    let mut world = World::new(Renderer::max_buffer_sizes());

    // Create the renderer with some bundles to draw.
    let mut renderer = create_renderer(&config, &window, &world);

    let mut game_state = GameState::new(&mut world, &renderer);

    let _num_cpus = num_cpus::get() as u16;

    loop {
        //- Window Inputs and Events Acquisition -----------------------------

        {
            profiling::scope!("Window Process Events");
            let events = window.process_events();
            for event in events {
                match event {
                    Event::Resize { width, height } => {
                        renderer.resize(width, height);
                    }
                }
            }
        }

        //- Game Logic -------------------------------------------------------

        let update_rs = {
            profiling::scope!("Game State Update");
            game_state.update(&window, &mut world, &mut renderer)
        };

        //- Renderer Logic ---------------------------------------------------

        if !window.is_minimized() {
            profiling::scope!("Renderer Update");
            if update_rs.world_changed || update_rs.player_moved {
                renderer.reset_frame_counter()
            }
            renderer
                .update(game_state.player.create_camera(renderer.surface_size()))
                .unwrap_or_else(|e| handle_error_and_panic(e));
        }

        //- Frame Present ----------------------------------------------------

        {
            profiling::scope!("Present");
            renderer.present();
        }

        //- Frame Sync -------------------------------------------------------

        {
            profiling::scope!("Wait For Frame Sync");
            window.wait_for_frame_sync();
        }

        profiling::finish_frame!();
    }
}

fn create_window(config: &Config) -> Window {
    let title = format!(
        "{} v{}",
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_VERSION")
    );
    let window = Window::new(
        title,
        config.surface_width,
        config.surface_height,
        configure_input_mapping(),
    )
    .unwrap_or_else(|e| handle_error_and_panic(e));

    let monitor = window.current_monitor();
    log::info!(
        "Monitor device name {:?} refresh rate {:?}",
        monitor.device_name(),
        monitor.refresh_rate()
    );

    window
}

// Create the input map binding.
#[rustfmt::skip]
fn configure_input_mapping() -> InputMapping {
    let mut input_mapping = InputMapping::new();

    input_mapping.set_primary(InputKind::GetVoxel, InputSource::Key { source: KeyCode::KeyQ });
    input_mapping.set_primary(InputKind::PutVoxel, InputSource::Key { source: KeyCode::KeyE });
    input_mapping.set_primary(InputKind::WalkForward, InputSource::Key { source: KeyCode::KeyW });
    input_mapping.set_primary(InputKind::WalkBackward, InputSource::Key { source: KeyCode::KeyS });
    input_mapping.set_primary(InputKind::WalkLeft, InputSource::Key { source: KeyCode::KeyA });
    input_mapping.set_primary(InputKind::WalkRight, InputSource::Key { source: KeyCode::KeyD });
    input_mapping.set_primary(InputKind::Jump, InputSource::Key { source: KeyCode::Space });
    input_mapping.set_primary(InputKind::SlowPace, InputSource::Key { source: KeyCode::ShiftLeft });
    input_mapping.set_secondary(InputKind::SlowPace, InputSource::Key { source: KeyCode::ShiftRight });
    input_mapping.set_primary(InputKind::Flying, InputSource::Key { source: KeyCode::KeyZ });

    input_mapping
}

fn create_renderer(_config: &Config, window: &Window, world: &World) -> Renderer {
    let surface_size = window.inner_size();
    let result = Renderer::new(
        window.raw_display_handle(),
        window.raw_window_handle(),
        surface_size.width,
        surface_size.height,
        world.max_nodes,
        1,
    );

    result.unwrap_or_else(|e| handle_error_and_panic(e))
}

#[track_caller]
fn handle_error_and_panic(error: String) -> ! {
    panic!("{}", error);
}
