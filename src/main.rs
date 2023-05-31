use glow::{TEXTURE_2D, TEXTURE_MIN_FILTER, *};
use image::GenericImageView;
use nalgebra_glm::*;
use sdl2::{self, event::Event};
use std::{
    mem::{size_of, transmute},
    time::Instant,
};
const ANGLE_STEP: f32 = 3.14159256;

const REM_BYTES: &[u8] = include_bytes!("rem.png");

fn main() {
    let mut time: Instant = Instant::now();
    unsafe {
        let (gl, window, mut event_loop, _context) = create_sdl2_context();
        let program = create_program(&gl, VERTEX_SOURCE, FRAG_SOURCE);
        gl.use_program(Some(program));

        let (vbo, vao, ebo, n) = init_vertex_buffer(&gl, program);

        let rem_img = image::load_from_memory(REM_BYTES).expect("Failed to load image");
        let dim = rem_img.dimensions();
        let rem_bytes = rem_img.into_rgba8().into_raw();
        let rem_texture = gl.create_texture().expect("failed to create texture");
        gl.bind_texture(TEXTURE_2D, Some(rem_texture));

        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            TEXTURE_MIN_FILTER,
            transmute(glow::LINEAR_MIPMAP_NEAREST),
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            TEXTURE_MAG_FILTER,
            transmute(glow::LINEAR),
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            transmute(glow::CLAMP_TO_EDGE),
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            transmute(glow::CLAMP_TO_EDGE),
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            transmute(glow::REPEAT),
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            transmute(glow::REPEAT),
        );

        gl.tex_image_2d(
            TEXTURE_2D,
            0,
            std::mem::transmute(glow::RGBA8),
            dim.0 as i32,
            dim.1 as i32,
            0,
            glow::RGBA8,
            glow::UNSIGNED_BYTE,
            Some(rem_bytes.as_slice()),
        );

        gl.generate_mipmap(glow::TEXTURE_2D);

        // let rem_texture_uniform = gl
        //     .get_uniform_location(program, "ourTexture00")
        //     .expect("Failed to get texture uniform location");

        let u_model_matrix = gl.get_uniform_location(program, "uModelMatrix").unwrap();
        let model_matrix: TMat4<f32> = TMat4::identity();
        let mut current_angle = 0.0;
        let mut tick = || {
            current_angle = animate(&mut time, current_angle);
            draw(
                &gl,
                n,
                current_angle,
                //(&rem_texture_uniform, &rem_texture),
                &model_matrix,
                &u_model_matrix,
                &program
            );
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
        gl.delete_buffer(ebo);
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
    //texture: (&NativeUniformLocation, &NativeTexture),
    model_matrix: &TMat4<f32>,
    u_model_matrix: &NativeUniformLocation,
    program: &NativeProgram,
) {
    gl.use_program(Some(*program));

    let mut result = nalgebra_glm::rotate(model_matrix, current_angle, &vec3(0.0, 0.0, 1.0));
    let translation: TVec3<f32> = vec3(0.35, 0.0, 0.0);
    result.append_translation_mut(&translation);
    let matrix_slice = result.as_slice();
    gl.uniform_matrix_4_f32_slice(Some(&u_model_matrix), false, matrix_slice);

    // gl.uniform_1_i32(Some(&texture.0), 0);
    // gl.active_texture(glow::TEXTURE0);
    // gl.bind_texture(glow::TEXTURE_2D, Some(*texture.1));

    gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
    gl.draw_elements(glow::TRIANGLES, n, glow::UNSIGNED_INT, 1);
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
) -> (NativeBuffer, NativeVertexArray, NativeBuffer, i32) {
    let vertices: &[f32] = &[
        // positions          // colors           // texture coords
        0.5, 0.5, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, // top right
        0.5, -0.5, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, // bottom right
        -0.5, -0.5, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, // bottom left
        -0.5, 0.5, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, // top left
    ];

    let indices: &[u32] = &[0, 1, 3, 1, 2, 3];

    let trigs = core::slice::from_raw_parts(
        vertices.as_ptr() as *const u8,
        vertices.len() * size_of::<f32>(),
    );

    let idx = core::slice::from_raw_parts(
        indices.as_ptr() as *const u8,
        indices.len() * size_of::<u32>(),
    );

    let vao = gl.create_vertex_array().unwrap();
    gl.bind_vertex_array(Some(vao));

    let vbo = gl.create_buffer().unwrap();
    let ebo = gl.create_buffer().unwrap();

    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
    gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, idx, glow::STATIC_DRAW);

    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, trigs, glow::STATIC_DRAW);

    // let a_pos = gl
    //     .get_attrib_location(program, "aPos")
    //     .expect("Failed to get aPos location");
    // let a_color = gl
    //     .get_attrib_location(program, "aColor")
    //     .expect("failed to get aColor location");
    // let a_texc = gl
    //     .get_attrib_location(program, "aTexCoord")
    //     .expect("failed to get aTexCoord location");

    let stride = size_of::<[f32; 8]>() as i32;
    gl.vertex_attrib_pointer_f32(
        0,
        3,
        glow::FLOAT,
        false,
        stride,
        size_of::<[f32; 3]>() as i32,
    );
    gl.enable_vertex_attrib_array(1);

    gl.vertex_attrib_pointer_f32(
        1,
        3,
        glow::FLOAT,
        false,
        stride,
        size_of::<[f32; 3]>() as i32,
    );
    gl.enable_vertex_attrib_array(1);

    gl.vertex_attrib_pointer_f32(
        2,
        2,
        glow::FLOAT,
        false,
        stride,
        size_of::<[f32; 2]>() as i32,
    );

    gl.enable_vertex_attrib_array(2);

    (vbo, vao, ebo, indices.len() as i32)
}

unsafe fn set_uniform_f32(gl: &glow::Context, program: NativeProgram, name: &str, value: f32) {
    let uniform_location = gl.get_uniform_location(program, name);

    gl.uniform_1_f32(uniform_location.as_ref(), value);
}
