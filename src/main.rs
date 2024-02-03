use fluid_euler::state::State;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

#[tokio::main]
async fn main() {
    run().await;
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window).await;

    event_loop
        .run(|event, elwt| match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key: Key::Named(NamedKey::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => elwt.exit(),
                WindowEvent::Resized(size) => state.resize(size),
                WindowEvent::RedrawRequested => state.render().unwrap(),
                _ => {}
            },
            Event::AboutToWait => {
                window.request_redraw(); // 暇になったらリクエストをする
            }
            _ => {}
        })
        .unwrap();
}
