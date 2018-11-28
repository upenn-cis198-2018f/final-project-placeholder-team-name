use super::graphics::*;
use std::f32::consts::*;
use cgmath::*;

pub struct Visualizer {
    // TODO: add in anything that must be kept around between updates
}

impl Visualizer {
    
    pub fn new() -> Visualizer {
        return Visualizer {};
    }

    // TODO:
    // later, also pass in a struct containing the relevant audio data
    pub fn update(&mut self, delta_secs: f32, time_secs: f32) -> Canvas {
        let mut canvas = Canvas::new();
        
        // TODO: for debugging
        println!("time (s): {}", time_secs);

        // loops from 0 to 1, then back to 0, and so on
        let anim_factor = map((2f32 * PI * time_secs / 5.0f32).sin(), -1f32, 1f32);
        // loops from 0 to 1, with wraparound
        let a_p = 3f32;
        let anim_mod = (time_secs % a_p) / a_p;

        // move the camera in a loop around the center
        let angle = 2f32 * PI * anim_mod;
        let pos = 100f32 * (angle.cos() * vec3(1f32, 0f32, 0f32) +
                           angle.sin() * vec3(0f32, 0f32, 1f32));
        canvas.set_camera(pos + vec3(0f32, 40f32, 0f32),
            vec3(0f32, 0f32, 0f32), vec3(0f32, 1f32, 0f32));

        let l_pos = 500f32 * vec3(1f32, 1f32, 1f32);
        canvas.set_light_position(l_pos);

        // draw sample cube
        let len = 10f32;
        canvas.draw_ppiped(
            vec3(0f32, 0f32, 0f32),
            vec3(len, 0f32, 0f32),
            vec3(0f32, len, 0f32),
            vec3(0f32, 0f32, len),
            vec4(0.75f32, 0f32, 0f32, 1f32)
        );
        
        canvas
    }
}

/*
   Math utilities
*/

// Maps a value from the range [cur_min, cur_max] to [0, 1]
fn map(val: f32, cur_min: f32, cur_max: f32) -> f32 {
    (val - cur_min) / (cur_max - cur_min)
}

// Maps a value from the range [0, 1] to [min, max]
fn lerp(factor: f32, min: f32, max: f32) -> f32 {
    min + factor * (max - min)
}

