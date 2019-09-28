extern crate minifb;
extern crate rayon;

use std::time::Instant;
use minifb::{Key, WindowOptions, Window};
use rayon::prelude::*;

const RESOLUTION : usize	= 768;
const CHUNK_SIZE : usize	= (RESOLUTION / 16);
const FRES : f32 			= RESOLUTION as f32;
const HALF_RES : f32		= FRES * 0.5;
const MAX_ITER : u32 = 128;

// From http://www.programming-during-recess.net/2016/06/26/color-schemes-for-mandelbrot-sets/
fn get_color_grey(iteration : u32) -> u32 {
	if iteration > MAX_ITER {
		return 0;
	}
	
	let factor = (iteration as f32 / MAX_ITER as f32).sqrt();
	let intensity = (factor * 255.0) as u32;
	
	(0xFF << 24) | (intensity << 16) | (intensity << 8) | intensity
}

//More or less
// https://stackoverflow.com/questions/16500656/which-color-gradient-is-used-to-color-mandelbrot-in-wikipedia
fn get_color(iteration : u32) -> u32 {
	if iteration >= MAX_ITER {
		return 0;
	}
	
	let factor = (iteration as f32 / MAX_ITER as f32).sqrt();
	
	let colors = [
		66,  30,  15, // # brown 3
		25,   7,  26,// # dark violett
		9,  1,  47,// # darkest blue
		4,   4,  73,// # blue 5
		0,   7, 100,// # blue 4
		12,  44, 138,// # blue 3
		24,  82, 177,// # blue 2
		57, 125, 209,// # blue 1
		134, 181, 229,// # blue 0
		211, 236, 248,// # lightest blue
		241, 233, 191,// # lightest yellow
		248, 201,  95,// # light yellow
		255, 170,   0,// # dirty yellow
		204, 128,   0,// # brown 0
		153,  87,   0,// # brown 1
		106,  52,   3 ];// # brown 2
	let idx = ((factor * (colors.len() / 3 - 1) as f32) as usize) * 3;
	
	(0xFF << 24) | (colors[idx] << 16) | (colors[idx + 1] << 8) | colors[idx + 2]
}

/*fn get_color(iteration : u32) -> u32 {
	if iteration >= MAX_ITER {
		return 0;
	}
	if iteration < MAX_ITER/8 {
		return (0xFF << 24) | (7 << 8) | 100;
	}
	if iteration < MAX_ITER / 4 {
		return (0xFF << 24) | (32 << 16) | (107 << 8) | 203;
	}
	if iteration < MAX_ITER * 3 / 8 {
		return (0xFF << 24) | (237 << 16) | (255 << 8) | 255;
	}
	if iteration < MAX_ITER / 2 {
		return (0xFF << 24) | (107 << 16) | (15 << 8) | 167;
	}
	if iteration < MAX_ITER * 5 / 8 {
		return (0xFF << 24) | (57 << 16) | (158 << 8) | 16;
	}
	if iteration < MAX_ITER * 6 / 8 {
		return (0xFF << 24) | (57 << 16) | (158 << 8) | 16;
	}
	if iteration < MAX_ITER * 7 / 8 {
		return (0xFF << 24) | (157 << 16) | (37 << 8) | 200;
	}
	
	(0xFF << 24) | (2 << 16)
}*/

fn mandelbrot(x_start : f32, y_start : f32) -> u32 {
	let mut x = x_start;
	let mut y = y_start;
	
	let mut iteration = 0;
	while x*x+y*y <= 4.0 && iteration < MAX_ITER {
            let x_new = x*x - y*y + x_start;
            y = 2.0*x*y + y_start;
            x = x_new;
            iteration += 1;
	}
	iteration
}

fn mandelbrot_chunk(zoom : f32, x_off : f32, y_off : f32, color_fn : impl Fn(u32)->u32, buffer : &mut[u32], offset : usize) {
	let x_start = offset % RESOLUTION;
	let y_start = offset / RESOLUTION;
	
	let buf_size = buffer.len();
	let mut x = (x_start as f32) - HALF_RES;
	let mut y = (y_start as f32) - HALF_RES;
	let scale = zoom / FRES;
	for chunk_offset in 0..buf_size {
			let real = x * scale + x_off;
			let imag = y * scale + y_off;
			let c = color_fn(mandelbrot(real, imag));
			buffer[chunk_offset] = c;
			x += 1.0;
			if x >= HALF_RES {
				x = -HALF_RES;
				y += 1.0;
			}
	}
}

fn main() 
{
	let mut window = Window::new("rust_mand - ESC to exit",
                                RESOLUTION, RESOLUTION,
                                WindowOptions::default()).unwrap_or_else(|e| {
        panic!("{}", e);
    });
	
	let mut framebuffer:Vec<u32> = vec![0; RESOLUTION*RESOLUTION];

	let mut zoom = 8.0;
	let mut x_off = 0.0;
	let mut y_off = 0.0;
	
	let color_functions = [ get_color, get_color_grey];
	let mut color_function_index = 0;
	
	while window.is_open() && !window.is_key_down(Key::Escape)
	{
		let start_time = Instant::now();
	
		framebuffer.par_chunks_mut(CHUNK_SIZE*CHUNK_SIZE)
			.enumerate()
			.map(|mut x| mandelbrot_chunk(zoom, x_off, y_off, &color_functions[color_function_index], &mut x.1, x.0*CHUNK_SIZE*CHUNK_SIZE))
			.collect::<Vec<_>>();
		
		// Single thread
		//mandelbrot_chunk(zoom, x_off, y_off, framebuffer.as_mut_slice(), 0);
	
		let time_taken = Instant::now().duration_since(start_time);
		let time_taken_dbl = time_taken.as_secs() as f64 + time_taken.subsec_nanos() as f64 * 1e-9;
		let fps = (1.0 / time_taken_dbl) as u32;
		window.set_title(&format!("FPS {}", fps));
		
        window.update_with_buffer(framebuffer.as_slice()).unwrap();
		
		if window.is_key_down(Key::Q) {
			zoom *= 0.995;
		}
		if window.is_key_down(Key::W) {
			zoom /= 0.995;
		}
		if window.is_key_down(Key::Left) {
			x_off -= 0.01*zoom;
		}
		if window.is_key_down(Key::Right) {
			x_off += 0.01*zoom;
		}
		if window.is_key_down(Key::Up) {
			y_off -= 0.01*zoom;
		}
		if window.is_key_down(Key::Down) {
			y_off += 0.01*zoom;
		}
		if window.is_key_down(Key::C) {
			color_function_index ^= 1;
		}
	}
}