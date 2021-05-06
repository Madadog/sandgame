use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use rand::Rng;
use std::fs::File;
use image::io::Reader as ImageReader;

const WIDTH: u32 = 240;//256;
const HEIGHT: u32 = 136;//224;
const BOX_SIZE: i32 = 32;

/// Representation of the application state.
struct World {
    box_x: i32,
    box_y: i32,
    velocity_x: i32,
    velocity_y: i32,
    particles: Vec<Element>,
    mouse_x: f32,
    mouse_y: f32,
    window_width: u32,
    window_height: u32,
    cursor_element: Element,
    cursor_scale: u32,
    pause: bool,
    lr_sandfall: bool,
}

#[derive(Clone, PartialEq, Copy)]
enum Element {
    None,
    Wall,
    Sand,
    Fire,
    Water,
    Nova,
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    //let img = ImageReader::open("korby.png")?.decode()?;
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Sandgame")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    //let mut particles = vec![Element::None; 32640];
    world.particles[100] = Element::Sand;
    world.particles[101] = Element::Sand;
    world.particles[102] = Element::Sand;
    for i in world.particles.len()-WIDTH as usize..world.particles.len() {
        world.particles[i] = Element::Wall;
    }

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                //*control_flow = ControlFlow::Exit;
                //return;
            }
        }

        // Handle input events
        if input.update(event) {
            // Resize the window
            if let Some(size) = input.window_resized() {
                world.window_width = size.width;
                world.window_height = size.height;
                pixels.resize(size.width, size.height);
            }
            // Update internal state and request a redraw
            if input.window_resized()==Option::None {
                if !world.pause {
                    world.update();
                }
                window.request_redraw();
            }

            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            world.velocity_x=0;
            world.velocity_y=0;
            if input.key_held(VirtualKeyCode::Right) {
                world.velocity_x = 1;
            }
            if input.key_held(VirtualKeyCode::Left) {
                world.velocity_x = -1;
            }
            if input.key_held(VirtualKeyCode::Up) {
                world.velocity_y = -1;
            }
            if input.key_held(VirtualKeyCode::Down) {
                world.velocity_y = 1;
            }
            if input.key_pressed(VirtualKeyCode::Key1) {
                world.cursor_element = Element::Sand;
            }
            if input.key_pressed(VirtualKeyCode::Key2) {
                world.cursor_element = Element::Wall;
            }
            if input.key_pressed(VirtualKeyCode::Key3) {
                world.cursor_element = Element::Fire;
            }
            if input.key_pressed(VirtualKeyCode::Key4) {
                world.cursor_element = Element::Water;
            }
            if input.key_pressed(VirtualKeyCode::Space) {
                world.pause = !world.pause;
            }
            if input.key_pressed(VirtualKeyCode::Equals) {
                world.cursor_scale+=1;
            }
            if input.key_pressed(VirtualKeyCode::Minus) && world.cursor_scale > 0 {
                world.cursor_scale-=1;
            }
            if let Some((mx, my)) = input.mouse() {
                world.mouse_x = mx;
                world.mouse_y = my;

                let (mx_i, my_i) = pixels
                    .window_pos_to_pixel((mx, my))
                    .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));
                //let mx = ((mx/world.window_width as f32)*WIDTH as f32) as u32;
                //let my = ((my/world.window_height as f32)*HEIGHT as f32) as u32;
                let mx = mx_i as u32;
                let my = my_i as u32;

                if mx < WIDTH && mx > 0 && my < HEIGHT && my > 0 {
                    if input.mouse_held(0) {
                        for i in 0..world.cursor_scale {
                            if (mx+my*WIDTH) as u32-i as u32>0 && (mx+my*WIDTH)+(i as u32) < (WIDTH*HEIGHT) {
                                world.particles[(mx+my*WIDTH) as usize+i as usize] = world.cursor_element;
                                world.particles[(mx+my*WIDTH) as usize-i as usize] = world.cursor_element;
                            }
                        }
                    } else if input.mouse_held(1) {
                        for i in 0..world.cursor_scale {
                            if (mx+my*WIDTH) as u32-i as u32>0 && (mx+my*WIDTH)+(i as u32) < (WIDTH*HEIGHT) {
                                world.particles[(mx+my*WIDTH) as usize+i as usize] = Element::None;
                                world.particles[(mx+my*WIDTH) as usize-i as usize] = Element::None;
                            }
                        }
                    };
                }
            }

            //std::thread::sleep(std::time::Duration::from_millis(15));
        }
    });
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            box_x: 24,
            box_y: 16,
            velocity_x: 1,
            velocity_y: 1,
            particles: vec![Element::None; (WIDTH*HEIGHT) as usize],
            mouse_x: 0.,
            mouse_y: 0.,
            window_width: WIDTH,
            window_height: HEIGHT,
            cursor_element: Element::Sand,
            cursor_scale: 1,
            pause: false,
            lr_sandfall: false,
        }
    }

    fn update(&mut self) {
        if self.box_x + self.velocity_x < 0 || self.box_x + BOX_SIZE + self.velocity_x > WIDTH as i32 {
            self.velocity_x = 0;
        }
        if self.box_y + self.velocity_y < 0 || self.box_y + BOX_SIZE + self.velocity_y > HEIGHT as i32 {
            self.velocity_y = 0;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;

        if self.box_x < (WIDTH-2) as i32 && self.box_x > 0 && self.box_y < HEIGHT as i32 && self.box_y > 0 {
            self.particles[(self.box_x+self.box_y*WIDTH as i32) as usize] = Element::Sand;
            self.particles[(self.box_x+self.box_y*WIDTH as i32+1) as usize] = Element::Sand;
            self.particles[(self.box_x+self.box_y*WIDTH as i32+2) as usize] = Element::Sand;
        }
        let mut rng = rand::thread_rng();
        for i in 0..3 {
            self.particles[rng.gen_range(0..WIDTH as usize)] = Element::Sand;
        }

        // update particles starting from bottom moving up
        for i in (WIDTH as usize..self.particles.len()).rev() {
            let above = i-WIDTH as usize;
            /*self.particles[i] = match self.particles[above] {
                Element::Sand => Element::Sand,
                _ => Element::None,
            }*/
            if let Element::None = self.particles[i] {
                match self.particles[above] {
                Element::Sand => {
                    self.particles[i] = Element::Sand;
                    self.particles[above] = Element::None;
                }
                Element::Water => {
                    self.particles[i] = Element::Water;
                    self.particles[above] = Element::None;
                }
                _ => ()
                }
            } else if let Element::Sand = self.particles[i] {
                if let Element::Sand = self.particles[above]  {
                    let d = rng.gen_range(0..=1);
                    if d == 0 && i-1 > 0 {
                        if let Element::None = self.particles[i-1] {
                            self.particles[i-1] = Element::Sand;
                            self.particles[above] = Element::None;
                        }
                    } else if d == 1 && i+1 < (WIDTH*HEIGHT) as usize {
                        if let Element::None = self.particles[i+1] {
                            self.particles[i+1] = Element::Sand;
                            self.particles[above] = Element::None;
                        }
                    }
                }
                if rng.gen_range(0..=2)==1 && ((i-1>0 && self.particles[i-1]==Element::Fire) || (i+1< (WIDTH*HEIGHT) as usize && self.particles[i+1]==Element::Fire) || (above>0 && self.particles[above]==Element::Fire) || (i+WIDTH as usize) < (WIDTH*HEIGHT) as usize && self.particles[i+WIDTH as usize]==Element::Fire) {
                    self.particles[i] = Element::Fire;
                }
            } else if let Element::Fire = self.particles[i] {
                if rng.gen_range(0..=10) == 0 {
                    self.particles[i] = Element::None;
                }
            } else if let Element::Water = self.particles[i] {
                if let Element::Water = self.particles[above]  {
                    let d = rng.gen_range(0..=1);
                    if d == 0 && i-1 > 0 {
                        if let Element::None = self.particles[i-1] {
                            self.particles[i-1] = Element::Water;
                            self.particles[i] = Element::None;
                        }
                    } else if d == 1 && i+1 < (WIDTH*HEIGHT) as usize {
                        if let Element::None = self.particles[i+1] {
                            self.particles[i+1] = Element::Water;
                            self.particles[i] = Element::None;
                        }
                    }
                }
                let rand = rng.gen_range(-1..=1);
                if (i as i32+rand)>0 && (i as i32+rand)<(WIDTH*HEIGHT) as i32 {
                    if let Element::None = self.particles[i-1] {
                        self.particles[i-1] = Element::Water;
                        self.particles[i] = Element::None;
                    }
                }
            }
        }
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i32;
            let y = (i / WIDTH as usize) as i32;

            /*let rgba = if inside_the_box {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };*/


            let rgba = match self.particles[i] {
                Element::None => [0x05, 0x05, 0x05, 0xff],
                Element::Sand => [0xef, 0xcf, 0xae, 0xff],
                Element::Wall => [0x22, 0x22, 0x22, 0xff],
                Element::Fire => [0xff, 0x55, 0x55, 0xff],
                Element::Water => [0x00, 0xaa, 0xaa, 0xff],
                _ => [0xff, 0x00, 0xff, 0xff],
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
