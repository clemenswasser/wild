#![warn(clippy::all)]

mod renderer;

fn is_window_drawable(window: &winit::window::Window) -> bool {
    window.inner_size().width != 0 && window.inner_size().width != 0
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_min_inner_size(winit::dpi::LogicalSize::new(100, 100))
        .build(&event_loop)
        .unwrap();

    let mut renderer = renderer::Renderer::new(&window);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;
        match event {
            winit::event::Event::WindowEvent {
                window_id: _,
                event,
            } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                winit::event::WindowEvent::Resized(_size) if is_window_drawable(&window) => {
                    renderer.render()
                }
                _ => {}
            },
            winit::event::Event::RedrawRequested(_) if is_window_drawable(&window) => {
                renderer.render()
            }
            winit::event::Event::MainEventsCleared if is_window_drawable(&window) => {
                renderer.render()
            }
            _ => {}
        }
    });
}
