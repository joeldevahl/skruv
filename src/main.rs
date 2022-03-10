mod renderer;
mod graph;

use renderer::*;
use graph::*;

use rusty_d3d12;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::windows::WindowExtWindows,
    window::WindowBuilder,
};


fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("Cannot create window");
    window.set_inner_size(winit::dpi::LogicalSize::new(
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
    ));

    let mut renderer = Renderer::new(window.hwnd());

    let mut graph = Graph::new();

    let node1 = Box::new(DrawNode {});
    let node2 = Box::new(PresentNode {});

    let node1_id = graph.add(node1, &[]);
    let _node2_id = graph.add(node2, &[node1_id]);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                renderer.execute();
            }
            _ => (),
        }
    });
}
