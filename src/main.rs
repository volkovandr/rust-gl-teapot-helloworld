use glium::{glutin, Surface};
use std::time::Instant;

#[macro_use]
extern crate glium;

mod teapot;

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let wb = glutin::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        //backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        ..Default::default()
    };

    let mut closed = false;
    let mut color = [0.0, 0.0, 0.0f32];
    let mut delta = [0.001, 0.002, 0.003];

    let mut now = Instant::now();
    let mut frame_count = 0;

    let positions = glium::VertexBuffer::new(&display, &teapot::VERTICES).unwrap();
    let normals = glium::VertexBuffer::new(&display, &teapot::NORMALS).unwrap();
    let indices = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &teapot::INDICES,
    )
    .unwrap();

    let vertex_shader_src = r#"
        #version 140
        in vec3 position;
        in vec3 normal;

        out vec3 v_normal;
        out vec3 v_position;

        uniform mat4 perspective;
        uniform mat4 view;
        uniform mat4 model;

        void main() {
            mat4 modelview = view * model;
            v_normal = transpose(inverse(mat3(modelview))) * normal;
            gl_Position = perspective * modelview * vec4(position, 1.0);
            v_position = gl_Position.xyz / gl_Position.w;
        }
        "#;

    let fragment_shader_src = r#"
        #version 140
        
        in vec3 v_normal;
        in vec3 v_position;

        out vec4 color;
        
        uniform vec3 u_light;
        uniform vec3 light_col;

        const vec3 specular_color = vec3(1.0, 1.0, 1.0);

        void main() {
            vec3 ambient_color = light_col * 0.2;
            vec3 diffuse_color = light_col;
        
            float diffuse = max(dot(normalize(v_normal), normalize(u_light)), 0.0);

            vec3 camera_dir = normalize(-v_position);
            vec3 half_direction = normalize(normalize(u_light) + camera_dir);
            float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 16.0);

            color = vec4(ambient_color + diffuse * diffuse_color + specular * specular_color, 1.0);
        }
        "#;

    let program =
        glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
            .unwrap();

    let light = [-0.9, 0.0, -0.9f32];

    while !closed {
        if now.elapsed().as_millis() >= 1000 {
            println!("FPS: {}", frame_count);
            now = Instant::now();
            frame_count = 0;
        };
        frame_count += 1;

        let view = view_matrix(&[0.0, 0.0, -2.0], &[0.0, 0.0, 2.0], &[0.0, 1.0, 0.0]);

        let mut target = display.draw();
        // target.clear_color_and_depth((color[0], color[1], color[2], 1.0), 1.0);
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

        let perspective = perspective_matrix(&target);

        for x in 0..10 {
            for y in 0..10 {
                for z in 0..10 {
                    let move_x = -1.8 + x as f32 * 0.4;
                    let move_y = -1.8 + y as f32 * 0.4;
                    let move_z = 1.0 + z as f32 * 0.4;
                    let scale = 0.0025;

                    let model = [
                        [scale, 0.0, 0.0, 0.0],
                        [0.0, scale, 0.0, 0.0],
                        [0.0, 0.0, scale, 0.0],
                        [move_x, move_y, move_z, 1.0f32],
                    ];

                    target
                        .draw(
                            (&positions, &normals),
                            &indices,
                            &program,
                            &uniform! { model: model, u_light: light, light_col: color, perspective: perspective, view: view },
                            &params,
                        )
                        .unwrap();
                }
            }
        }

        target.finish().unwrap();

        for i in 0..3 {
            color[i] += delta[i];
            if color[i] < 0.0 {
                color[i] = 0.0;
                delta[i] = -delta[i];
            };
            if color[i] > 1.0 {
                color[i] = 1.0;
                delta[i] = -delta[i];
            };
        }

        events_loop.poll_events(|ev| match ev {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => closed = true,
                _ => (),
            },
            _ => (),
        });
    }
}

fn perspective_matrix(target: &glium::Frame) -> [[f32; 4]; 4] {
    let (width, height) = target.get_dimensions();
    let aspect_ratio = height as f32 / width as f32;

    let fov: f32 = 3.141592 / 3.0;
    let zfar = 1024.0;
    let znear = 0.1;

    let f = 1.0 / (fov / 2.0).tan();

    [
        [f * aspect_ratio, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, (zfar + znear) / (zfar - znear), 1.0],
        [0.0, 0.0, -(2.0 * zfar * znear) / (zfar - znear), 0.0],
    ]
}

fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [
        up[1] * f[2] - up[2] * f[1],
        up[2] * f[0] - up[0] * f[2],
        up[0] * f[1] - up[1] * f[0],
    ];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [
        f[1] * s_norm[2] - f[2] * s_norm[1],
        f[2] * s_norm[0] - f[0] * s_norm[2],
        f[0] * s_norm[1] - f[1] * s_norm[0],
    ];

    let p = [
        -position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
        -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
        -position[0] * f[0] - position[1] * f[1] - position[2] * f[2],
    ];

    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}
