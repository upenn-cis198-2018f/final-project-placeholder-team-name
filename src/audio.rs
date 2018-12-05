extern crate hound;
extern crate num;
extern crate rustfft;
extern crate find_folder;
extern crate portaudio;
extern crate sample;

use std::i16;
use hound::*;
use num::complex::Complex;
use rustfft::FFTplanner;
use std::sync::mpsc::Sender;
use std::sync::mpsc;

// FFT function
pub fn find_spectral_peak(filename: &str) -> Option<f32> {
	let mut reader = hound::WavReader::open(filename).expect("Failed to open WAV file");
	let num_samples = reader.len() as usize;
	let mut planner = FFTplanner::new(false);
	let fft = planner.plan_fft(num_samples);
	let mut signal = reader.samples::<i16>().map(|x| Complex::new(x.unwrap() as f32, 0f32)).collect::<Vec<_>>();
	let mut spectrum = signal.clone();
	fft.process(&mut signal[..], &mut spectrum[..]);
	let max_peak = spectrum.iter().take(num_samples / 2).enumerate().max_by_key(|&(_, freq)| freq.norm() as u32);
	if let Some((i, _)) = max_peak {
		let bin = 44100f32 / num_samples as f32;
		Some(i as f32 * bin)
	} else {
		None
	}
}

// Splits a .wav file into 10ms slices and precomputes the peak frequency for each
// slice using a FFT. Returns a vector containing the computed frequencies.
pub fn get_peaks(filename: &str) -> Vec<f32> {
    let mut peaks : Vec<f32> = Vec::new();
	let mut reader = hound::WavReader::open(filename).expect("Failed to open WAV file");
    let spec = reader.spec();

    let num_samples = (spec.sample_rate / 50) as usize;
	let mut planner = FFTplanner::new(false);
	let fft = planner.plan_fft(num_samples);

	let mut signal = reader.samples::<i16>().map(|x| Complex::new(x.unwrap() as f32, 0f32)).collect::<Vec<_>>();
	let mut spectrum = signal.clone();

    println!("Audio samples loaded");

    let mut start_idx = 0;
    while start_idx + num_samples < signal.len() {
        let end_idx = start_idx + num_samples;
        fft.process(&mut signal[start_idx..end_idx], &mut spectrum[start_idx..end_idx]);
        let max_peak = spectrum[start_idx..end_idx].iter().take(num_samples / 2).enumerate().max_by_key(|&(_, freq)| freq.norm() as u32);
        if let Some((i, _)) = max_peak {
            let bin = 44100f32 / num_samples as f32;
            peaks.push(i as f32 * bin);
        } else {
            peaks.push(0f32);
        }
        start_idx = end_idx;
    }

    peaks
}

// Possibly useful for analysis, could also be called in buffer
pub fn return_rms(filename: &str) {
	let mut reader = hound::WavReader::open(filename).unwrap();
	let sum = reader.samples::<i16>().fold(0.0, |sum, s| {
		let sample = s.unwrap() as f64;
		sum + sample * sample
	});
	println!("RMS is {}", (sum / reader.len() as f64).sqrt());		
}

// Playback function
pub fn playback(filename: &str, tevent_tx: Sender<f64>, pdone_tx: Sender<bool>) {
	let reader = hound::WavReader::open(filename).unwrap();
	let spec = reader.spec();
	let sample_vec : Vec<i16> = reader.into_samples::<i16>()
									  .filter_map(Result::ok)
									  .collect();
	let mut samples = sample_vec.into_iter();

	let pa = portaudio::PortAudio::new().unwrap();
	let ch = spec.channels as i32;
	let sr = spec.sample_rate as f64;
	let buffer_len = 64;
	let settings = pa.default_output_stream_settings::<i16>(ch, sr, buffer_len).unwrap();

	let (complete_tx, complete_rx) = mpsc::channel();

	let callback = move |portaudio::OutputStreamCallbackArgs { buffer, time, .. }| {
		// Every time the callback is executed, pass time data along the channel
        // to allow sychronization with the frequency data
        tevent_tx.send(time.current).ok();

        // Pass the sample data through to the output buffer
		for out_sample in buffer {
			match samples.next() {
				Some(sample) => *out_sample = sample,
				None => {
					complete_tx.send(()).unwrap();
					return portaudio::Complete;
				},
			}
		}
		portaudio::Continue
	};

	let mut stream = pa.open_non_blocking_stream(settings, callback).unwrap();
	stream.start().unwrap();

	// Wait on a Complete message from the callback
	complete_rx.recv().unwrap();
	
    // We recieved a Complete message in the loop so stop/close the stream
    stream.stop().unwrap();
	stream.close().unwrap();

    // Notify main that playback has ended
    pdone_tx.send(true).ok();
}
