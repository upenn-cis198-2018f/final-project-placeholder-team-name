extern crate hound;
extern crate num;
extern crate rustfft;
extern crate find_folder;
extern crate portaudio;
extern crate sample;

mod audio;

use audio::*;
use std::f32::consts::PI;
use std::i16;
use std::env;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::time;


// Writing Sine Wave for testing purposes
fn write_sin_wav(note: f32) {
	let sampling_freq = 44100;
	let sampling_bits = 16;
	let amplitude = i16::MAX as f32;
	let note_freq = note;
	let length = 2;

	let no_of_samples = sampling_freq * length;
	let normalized_sample_indices = (0 .. no_of_samples).
		map(|x| x as f32 / sampling_freq as f32);

	let spec = hound::WavSpec {
		channels: 1,
		sample_rate: sampling_freq,
		bits_per_sample: sampling_bits
	};

	let maybe_writer = hound::WavWriter::create("sine.wav", spec);
	let mut xs: Vec<f32> = Vec::with_capacity(no_of_samples as usize);
	let mut ys: Vec<f32> = Vec::with_capacity(no_of_samples as usize);

	match maybe_writer {
		Ok(writer_obj) => {
			let mut writer = writer_obj;
			for t in normalized_sample_indices {
				let sample = (t * note_freq * 2.0 * PI).sin();
				xs.push(t);
				ys.push(sample);
				writer.write_sample((sample * amplitude) as i16).unwrap();
			}
		},
		Err(e) => {
			println!("No");
			println!("{}", e);
		}
	}

	}


fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() == 2 {
		let filename = &args[1];
		println!("Song choice is: {}", filename);
		// if let Some(peak) = find_spectral_peak(filename) {
		// 	println!("Max frequency: {} Hz", peak);
		// }
		return_rms(filename);

		// Channel for sending time data
		let (tevent_tx, tevent_rx) : (Sender<f64>, Receiver<f64>) = mpsc::channel();

		// Channel for sending time requests
		let (tquery_tx, tquery_rx) : (Sender<bool>, Receiver<bool>) = mpsc::channel();

		// Spawn a separate thread to stream the audio
		let song_arg = filename.clone();
		let audio_thread = thread::spawn(move || {
			playback(&song_arg, tevent_tx);
		});

		let time_thread = thread::spawn(move || {
			print_time(tevent_rx);
		});

		time_thread.join().unwrap();
		audio_thread.join().unwrap();
	} else {
		println!("Please input one filename in quotation marks.");
	}
}

fn print_time(tevent_rx: Receiver<f64>) {
	tevent_rx.recv().unwrap();
	let start_time = time::Instant::now();

	loop {
		while let Ok(_) = tevent_rx.try_recv() {
			println!("count_down: {:?}", start_time.elapsed());
		}
		thread::sleep(time::Duration::from_millis(1));
	}
}
