extern crate glutin;
extern crate gl;
extern crate cgmath;
extern crate hound;
extern crate num;
extern crate rustfft;
extern crate find_folder;
extern crate portaudio;
extern crate sample;

mod graphics;
mod visualizer;
mod audio;

use visualizer::*;
use graphics::*;
use glutin::*;
use std::time;
use std::process;
use audio::*;
use std::f32::consts::PI;
use std::i16;
use std::env;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() != 2 {
		println!("Please input one filename in quotation marks.");
		process::exit(1);
	}
	let filename = &args[1];
	println!("Song choice is: {}", filename);
	// if let Some(peak) = find_spectral_peak(filename) {
	// 	println!("Max frequency: {} Hz", peak);
	// }
	// return_rms(filename);

	let peak_data : Vec<f32> = get_peaks(filename);

	// Channel for sending time data
	let (tevent_tx, tevent_rx) : (Sender<f64>, Receiver<f64>) = mpsc::channel();

	// Channel for sending time requests
	// let (tquery_tx, tquery_rx) : (Sender<bool>, Receiver<bool>) = mpsc::channel();

    let mut events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_title("music visualizer")
        .with_dimensions(dpi::LogicalSize::new(1000.0, 700.0));
    let context = ContextBuilder::new()
        .with_vsync(true)
        .with_gl(GlRequest::Specific(glutin::Api::OpenGl, (4, 1)))
        .with_gl_profile(GlProfile::Core);

    // attempt to create a window
    let display_result = GlWindow::new(window, context, &events_loop);
    let mut display_opt = match display_result {
        Ok(display) => Some(display),
        Err(err) => {
            println!("{:?}", err);
            None
        }
    };

    // if we have a window, setup OpenGL
        if let Some(ref display) = display_opt {
        unsafe {
            display.make_current().unwrap_or_else(|err| {
                println!("Context creation error:\n{:?}", err);
            });
        }
        gl::load_with(
            |symbol| display.get_proc_address(symbol) as *const _);
    }

    let mut g_state = GraphicsState::new();
    if let Some(ref display) = display_opt {
        unsafe {
            display.make_current().unwrap_or_else(|err| {
                println!("Context creation error:\n{:?}", err);
            });
        }
        gl::load_with(
            |symbol| display.get_proc_address(symbol) as *const _);
        g_state.setup_opengl();
    }

    let program_start = time::Instant::now();
    let mut keep_running = true;
    let mut previous_tick = program_start;
    let frame_period: f64 = 1.0 / 60.0; // in secs
    let frame_duration = time::Duration::from_millis(
        (frame_period * 1000.0) as u64);
    
	// Spawn a separate thread to stream the audio
	let song_arg = filename.clone();
	let audio_thread = thread::spawn(move || {
		playback(&song_arg, tevent_tx);
	});

	let time_thread = thread::spawn(move || {
		print_time(tevent_rx, peak_data);
	});

	// TODO: Join threads
	
	let mut visualizer = Visualizer::new();
    while keep_running {
        // sleep until the start of the next frame
        let current_time = time::Instant::now();
        let sleep_duration_opt = frame_duration.checked_sub(
            current_time.duration_since(previous_tick));
        if let Some(sleep_duration) = sleep_duration_opt {
            std::thread::sleep(sleep_duration);
        };
        previous_tick = current_time;

        // if we have a window, poll for events and resize to fit the window
        if let Some(ref mut display) = display_opt {
            events_loop.poll_events(|event| {
                let keep_open = handle_event(display, event);
                if !keep_open && keep_running {
                    keep_running = false
                };
            });
            // This hack is required to fix a bug on OS Mojave
            // It resizes the window to its current size.
            // https://github.com/tomaka/glutin/issues/1069
            let dpi = display.get_hidpi_factor();
            if let Some(display_size) = display.get_inner_size() {
                let physical_size = display_size.to_physical(dpi);
                display.resize(physical_size);
                g_state.update_framebuffer_size(physical_size.width, physical_size.height);
            }
        }

        let program_duration = time::Instant::now().duration_since(
            program_start);
        let program_duration_secs = (program_duration.as_secs()  as f32) + 
            (program_duration.subsec_millis() as f32) / 1000.0;
        let canvas = visualizer.update(
            frame_period as f32, program_duration_secs);

        // if we have a window, render the canvas to it
        if let Some(ref display) = display_opt {
            g_state.draw_frame(&canvas);
            display.swap_buffers().unwrap_or_else(|err| {
                println!("Error on swap buffers:\n{:?}", err);
            });
        }
    }
}

fn handle_event(display: &mut GlWindow, event: Event) -> bool {
    match event {
        Event::WindowEvent{event: win_event, ..} => {
            match win_event {
                WindowEvent::CloseRequested => return false,
                WindowEvent::Resized(logical_size) => {
                    // when the window resizes, we must resize the context
                    let dpi = display.get_hidpi_factor();
                    display.resize(logical_size.to_physical(dpi));
                },
                _ => {
                    // TODO: forward mouse and key to ImGui backend
                }
            }
        },
        _ => ()
    };
    true
}

fn print_time(tevent_rx: Receiver<f64>, peaks: Vec<f32>) {
	let peaks_iter = peaks.iter();
	tevent_rx.recv().unwrap();
	let start_time = time::Instant::now();
	let inc = time::Duration::from_millis(10 as u64);
	let mut diff_time = time::Duration::from_millis(0 as u64);
	let mut curr_peak = 0.0;

	loop {
		if (start_time.elapsed() > diff_time) {
			match peaks_iter.next() {
				Some(p) => p,
				None => 0.0
			}
			diff_time = diff_time + inc;
			println!("Time: {:?}, Peak: {:?}", start_time.elapsed(), curr_peak);
		}
		while let Ok(_) = tevent_rx.try_recv() {
			println!("count_down: {:?}", start_time.elapsed());
		}
		thread::sleep(time::Duration::from_millis(1));
	}
}
