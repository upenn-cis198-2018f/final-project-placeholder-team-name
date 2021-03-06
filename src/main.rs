extern crate glutin;
extern crate gl;
extern crate cgmath;
extern crate hound;
extern crate num;
extern crate rustfft;
extern crate portaudio;

mod graphics;
mod visualizer;
mod audio;

use visualizer::*;
use graphics::*;
use glutin::*;
use std::time;
use std::process;
use audio::*;
use std::env;
use std::sync::mpsc::{Sender, Receiver, TryRecvError};
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
	return_rms(filename);

	// Preload a vector of frequencies from the track to sync with the audio
	let peak_data : Vec<f32> = get_peaks(filename);

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

	// Channel for sending time data
	let (tevent_tx, tevent_rx) : (Sender<f64>, Receiver<f64>) = mpsc::channel();

	// The playback thread will pass a message on this channel to signify it has closed the stream
	// and the main thread can cleanup the audio and time threads
	let (pdone_tx, pdone_rx) : (Sender<bool>, Receiver<bool>) = mpsc::channel();
    
	// Spawn a separate thread to stream the audio
	let song_arg = filename.clone();
	let audio_thread = thread::spawn(move || {
		playback(&song_arg, tevent_tx, pdone_tx);
	});

	// Spawn a separate thread to sync the time and freq data
	let time_thread = thread::spawn(move || {
		print_time(tevent_rx, peak_data);
	});

    let program_start = time::Instant::now();
    let mut keep_running = true;
    let mut previous_tick = program_start;
    let frame_period: f64 = 1.0 / 60.0; // in secs
    let frame_duration = time::Duration::from_millis(
        (frame_period * 1000.0) as u64);
	
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

		// Check if audio playback has ended
		match pdone_rx.try_recv() {
			Err(TryRecvError::Empty) => {}, // Do nothing
			// If the channel is not empty, it either contains an end message or
			// has disconnected. Either way, we are done with the graphics.
			_ => {
				keep_running = false;
			}
		}
    }
	
	// Cleanup the threads before exiting
	audio_thread.join().unwrap();
	time_thread.join().unwrap();
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

// We run this in a separate thread, synchronizing the time data from the playback
// and the precomputed frequency data. We sychronize the data by mapping each
// frequency to a 10ms time slice. Theoretically, we would then pass the frequency
// back in a message to the main function to forward to the graphics.
fn print_time(tevent_rx: Receiver<f64>, peaks: Vec<f32>) {
	let mut peaks_iter = peaks.into_iter();
	// Wait until the playback thread begins streaming time data
	tevent_rx.recv().unwrap();

	let start_time = time::Instant::now();
	let inc = time::Duration::from_millis(10 as u64);
	let mut diff_time = time::Duration::from_millis(0 as u64);

	loop {
		let elapsed = start_time.elapsed();
		if elapsed > diff_time {
			let curr_peak = match peaks_iter.next() {
				Some(p) => p,
				None => break
			};
			diff_time = diff_time + inc;
			println!("Time: {:?}, Peak: {:?}", elapsed, curr_peak);
		}
		while let Ok(_) = tevent_rx.try_recv() {
			println!("Time: {:?}", start_time.elapsed());
		}

		thread::sleep(time::Duration::from_millis(1));
	}
}
