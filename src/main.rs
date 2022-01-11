use glow::*;
use nalgebra_glm::*;
use sdl2::{
    self,
    event::Event,
};
use std::{
    mem::size_of,
    time::Instant,
};
const ANGLE_STEP: f32 = 3.14159256;


fn main() {
    let mut time: Instant = Instant::now();
    unsafe {
        let (gl, window, mut event_loop, _context) = create_sdl2_context();
        let program = create_program(&gl, VERTEX_SOURCE, FRAG_SOURCE);
        gl.use_program(Some(program));

        let (vbo, vao, n) = init_vertex_buffer(&gl, program);

        let u_model_matrix = gl.get_uniform_location(program, "uModelMatrix").unwrap();
        let model_matrix: TMat4<f32> = TMat4::identity();
        let mut current_angle = 0.0;
        let mut tick = || {
            current_angle = animate(&mut time, current_angle);
            draw(&gl, n, current_angle, &model_matrix, &u_model_matrix);
        };

        gl.clear(glow::COLOR_BUFFER_BIT);
        'render: loop {
            for event in event_loop.poll_iter() {
                if let sdl2::event::Event::Quit { .. } = event {
                    break 'render;
                }
                match event {
                    Event::KeyDown { keycode, .. } => match keycode {
                        Some(x) => match x {
                            sdl2::keyboard::Keycode::Q => break 'render,
                            _ => (),
                        },
                        None => (),
                    },
                    _ => (),
                }
            }

            tick();
            window.gl_swap_window();
        }

        gl.delete_program(program);
        gl.delete_vertex_array(vao);
        gl.delete_buffer(vbo);
    }
}

unsafe fn create_sdl2_context() -> (
    glow::Context,
    sdl2::video::Window,
    sdl2::EventPump,
    sdl2::video::GLContext,
) {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 0);
    let window = video
        .window("Hello Triangle!", 800, 800)
        .opengl()
        .build()
        .unwrap();
    let gl_context = window.gl_create_context().unwrap();
    let gl = glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _);
    let event_loop = sdl.event_pump().unwrap();
    (gl, window, event_loop, gl_context)
}

unsafe fn create_program(
    gl: &glow::Context,
    vertex_source: &str,
    fragment_source: &str,
) -> NativeProgram {
    let program = gl.create_program().expect("Cannot create program");
    let shader_sources = [
        (glow::VERTEX_SHADER, vertex_source),
        (glow::FRAGMENT_SHADER, fragment_source),
    ];
    let mut shaders = Vec::with_capacity(shader_sources.len());
    for (shader_type, shader_source) in shader_sources.iter() {
        let shader = gl
            .create_shader(*shader_type)
            .expect("Cannot create shader");
        gl.shader_source(shader, shader_source);
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            panic!("{}", gl.get_shader_info_log(shader));
        }
        gl.attach_shader(program, shader);
        shaders.push(shader);
    }

    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        panic!("{}", gl.get_program_info_log(program));
    }
    for shader in shaders {
        gl.detach_shader(program, shader);
        gl.delete_shader(shader);
    }
    program
}

unsafe fn draw(
    gl: &Context,
    n: i32,
    current_angle: f32,
    model_matrix: &TMat4<f32>,
    u_model_matrix: &NativeUniformLocation,
) {
    let mut result = nalgebra_glm::rotate(model_matrix, current_angle, &vec3(0.0, 0.0, 1.0));
    let translation:TVec3<f32> = vec3(0.35, 0.0,0.0);
    result.append_translation_mut(&translation);
    let matrix_slice = result.as_slice();
    gl.uniform_matrix_4_f32_slice(Some(&u_model_matrix), false, matrix_slice);
    gl.clear(glow::COLOR_BUFFER_BIT);
    gl.draw_arrays(glow::TRIANGLES, 0, n);
}
fn animate(start: &mut Instant, angle: f32) -> f32 {
    let now = Instant::now();
    let elapsed = now.duration_since(start.clone());
    *start = now;
    let new_angle = angle + (ANGLE_STEP * elapsed.as_millis() as f32) / 1000.0;
    new_angle % 360.0
}
const VERTEX_SOURCE: &'static str = include_str!("vertex.glsl");
const FRAG_SOURCE: &'static str = include_str!("frag.glsl");

unsafe fn init_vertex_buffer(
    gl: &glow::Context,
    program: glow::NativeProgram,
) -> (NativeBuffer, NativeVertexArray, i32) {
    let vertices: &[f32] = &[0.0, 0.5, -0.5, -0.5, 0.5, -0.5];

    let trigs = core::slice::from_raw_parts(
        vertices.as_ptr() as *const u8,
        vertices.len() * size_of::<f32>(),
    );

    let vbo = gl.create_buffer().unwrap();
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, trigs, glow::STATIC_DRAW);
    let vao = gl.create_vertex_array().unwrap();
    gl.bind_vertex_array(Some(vao));

    let a_pos = gl.get_attrib_location(program, "aPos").unwrap();
    gl.vertex_attrib_pointer_f32(a_pos, 2, glow::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(a_pos);

    (vbo, vao, vertices.len() as i32)
}

unsafe fn set_uniform(gl: &glow::Context, program: NativeProgram, name: &str, value: f32) {
    let uniform_location = gl.get_uniform_location(program, name);

    gl.uniform_1_f32(uniform_location.as_ref(), value);
}
