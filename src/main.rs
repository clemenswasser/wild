#![deny(clippy::all)]
#![deny(warnings)]

mod renderer;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    let mut renderer = renderer::Renderer::new(&window);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = winit::event_loop::ControlFlow::Poll;
        match event {
            winit::event::Event::WindowEvent {
                window_id: _,
                event,
            } => {
                if let winit::event::WindowEvent::CloseRequested = event {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
            }
            winit::event::Event::RedrawRequested(_) => {
                renderer.render();
            }
            _ => {}
        }
    })
}
