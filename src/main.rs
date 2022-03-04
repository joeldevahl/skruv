use rusty_d3d12;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::windows::WindowExtWindows,
    window::WindowBuilder,
};

#[no_mangle]
pub static D3D12SDKVersion: u32 = 600;

#[no_mangle]
pub static D3D12SDKPath: &[u8; 9] = b".\\D3D12\\\0";

const WINDOW_WIDTH: u32 = 640;
const WINDOW_HEIGHT: u32 = 480;
const ASPECT_RATIO: f32 = WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32;

const FRAMES_IN_FLIGHT: u32 = 3;

struct Renderer {
    device: rusty_d3d12::Device,
    command_queue: rusty_d3d12::CommandQueue,
    fence: rusty_d3d12::Fence,
    fence_event: rusty_d3d12::Win32Event,
    fence_values: [u64; FRAMES_IN_FLIGHT as usize],
    swapchain: rusty_d3d12::Swapchain,
    frame_index: usize,
    frame_count: u32,
    viewport_desc: rusty_d3d12::Viewport,
    scissor_desc: rusty_d3d12::Rect,
    render_targets: Vec<rusty_d3d12::Resource>,
    rtv_heap: rusty_d3d12::DescriptorHeap,
    rtv_descriptor_handle_size: rusty_d3d12::ByteCount,
    dsv_heap: rusty_d3d12::DescriptorHeap,
    command_allocators: Vec<rusty_d3d12::CommandAllocator>,
    command_list: rusty_d3d12::CommandList,
    depth_stencil: Option<rusty_d3d12::Resource>,
}

impl Renderer {
    fn new(hwnd: *mut std::ffi::c_void) -> Self {
        let mut factory_flags = rusty_d3d12::CreateFactoryFlags::None;
        let factory = rusty_d3d12::Factory::new(factory_flags).expect("Cannot create factory");

        let hw_adapter = factory
            .enum_adapters()
            .expect("Cannot enumerate adapters")
            .remove(0);
        let device = rusty_d3d12::Device::new(&hw_adapter).expect("Cannot create device");

        let command_queue = device
            .create_command_queue(&rusty_d3d12::CommandQueueDesc::default())
            .expect("Cannot create command queue");

        let fence = device
            .create_fence(0, rusty_d3d12::FenceFlags::None)
            .expect("Cannot create fence");

        let fence_event = rusty_d3d12::Win32Event::default();
        let fence_values = [0; FRAMES_IN_FLIGHT as usize];
        let frame_index = 0;

        let swapchain_desc = rusty_d3d12::SwapchainDesc::default()
            .set_width(WINDOW_WIDTH)
            .set_height(WINDOW_HEIGHT)
            .set_buffer_count(u32::from(FRAMES_IN_FLIGHT));
        let swapchain = factory
            .create_swapchain(&command_queue, hwnd as *mut rusty_d3d12::HWND__, &swapchain_desc)
            .expect("Cannot create swapchain");
        factory
            .make_window_association(hwnd, rusty_d3d12::MakeWindowAssociationFlags::NoAltEnter)
            .expect("Cannot make window association");

        let viewport_desc = rusty_d3d12::Viewport::default()
            .set_width(WINDOW_WIDTH as f32)
            .set_height(WINDOW_HEIGHT as f32);

        let scissor_desc = rusty_d3d12::Rect::default()
            .set_right(WINDOW_WIDTH as i32)
            .set_bottom(WINDOW_HEIGHT as i32);

        let rtv_descriptor_handle_size = device
            .get_descriptor_handle_increment_size(rusty_d3d12::DescriptorHeapType::Rtv);

        let rtv_heap = device
            .create_descriptor_heap(
                &rusty_d3d12::DescriptorHeapDesc::default()
                    .set_heap_type(rusty_d3d12::DescriptorHeapType::Rtv)
                    .set_num_descriptors(u32::from(FRAMES_IN_FLIGHT)),
            )
            .expect("Cannot create RTV heap");
        rtv_heap
            .set_name("RTV heap")
            .expect("Cannot set RTV heap name");

        let dsv_heap = device
            .create_descriptor_heap(
                &rusty_d3d12::DescriptorHeapDesc::default()
                    .set_heap_type(rusty_d3d12::DescriptorHeapType::Dsv)
                    .set_num_descriptors(1),
            )
            .expect("Cannot create RTV heap");
        dsv_heap
            .set_name("DSV heap")
            .expect("Cannot set DSV heap name");

        let mut rtv_handle = rtv_heap.get_cpu_descriptor_handle_for_heap_start();

        let mut render_targets = vec![];
        for frame_idx in 0..FRAMES_IN_FLIGHT {
            let render_target = swapchain
                .get_buffer(u32::from(frame_idx))
                .expect("cannot get buffer from swapchain");

            device.create_render_target_view(&render_target, rtv_handle);
            render_targets.push(render_target);

            rtv_handle = rtv_handle.advance(1, rtv_descriptor_handle_size);
        }

        let mut command_allocators = vec![];
        for _ in 0..FRAMES_IN_FLIGHT {
            command_allocators.push(
                device
                    .create_command_allocator(rusty_d3d12::CommandListType::Direct)
                    .expect("Cannot create command allocator"),
            );
        }

        let command_list = device
            .create_command_list(
                rusty_d3d12::CommandListType::Direct,
                &command_allocators[0],
                None,
            )
            .expect("Cannot create command list");
        command_list.close().expect("Cannot close command list");

        let mut skruv_main = Self {
            device,
            command_queue,
            fence,
            fence_event,
            fence_values,
            swapchain,
            frame_index,
            frame_count: 0,
            viewport_desc,
            scissor_desc,
            render_targets,
            rtv_heap,
            rtv_descriptor_handle_size,
            dsv_heap,
            command_allocators,
            command_list,
            depth_stencil: None,
        };

        skruv_main
    }

    fn populate_command_list(&mut self, frame_index: usize) {
        self.command_allocators[frame_index]
            .reset()
            .expect("Cannot reset command allocator");

        self.command_list
            .reset(&self.command_allocators[frame_index], None)
            .expect("Cannot reset command list");

        self.command_list.set_viewports(&[self.viewport_desc]);
        self.command_list.set_scissor_rects(&[self.scissor_desc]);

        self.command_list
            .resource_barrier(&[rusty_d3d12::ResourceBarrier::new_transition(
                &rusty_d3d12::ResourceTransitionBarrier::default()
                    .set_resource(
                        &self.render_targets[self.frame_index as usize],
                    )
                    .set_state_before(rusty_d3d12::ResourceStates::Common)
                    .set_state_after(rusty_d3d12::ResourceStates::RenderTarget),
            )]);

        let rtv_handle = self
            .rtv_heap
            .get_cpu_descriptor_handle_for_heap_start()
            .advance(
                self.swapchain.get_current_back_buffer_index(),
                self.rtv_descriptor_handle_size,
            );

        self.command_list.set_render_targets(
            &mut [rtv_handle],
            false,
            Some(self.dsv_heap.get_cpu_descriptor_handle_for_heap_start()),
        );

        let clear_color: [f32; 4] = [0.9, 0.2, 0.4, 1.0];
        self.command_list.clear_render_target_view(
            rtv_handle,
            clear_color,
            &[],
        );

        //self.command_list.dispatch_mesh(self.meshlet_count, 1, 1);

        self.command_list
            .resource_barrier(&[rusty_d3d12::ResourceBarrier::new_transition(
                &rusty_d3d12::ResourceTransitionBarrier::default()
                    .set_resource(
                        &self.render_targets[self.frame_index as usize],
                    )
                    .set_state_before(rusty_d3d12::ResourceStates::RenderTarget)
                    .set_state_after(rusty_d3d12::ResourceStates::Common),
            )]);

        self.command_list
            .close()
            .expect("Cannot close command list");
    }

    fn draw(&mut self) {
        let last_fence_value = self.fence_values[self.frame_index];
        let fence_completed_value = self.fence.get_completed_value();

        if fence_completed_value < last_fence_value {
            self.fence
                .set_event_on_completion(last_fence_value, &self.fence_event)
                .expect("Cannot set event on fence");

            self.fence_event.wait(None);
        }

        self.populate_command_list(self.frame_index);

        self.command_queue.execute_command_lists(std::slice::from_ref(&self.command_list));

        self.swapchain
            .present(1, rusty_d3d12::PresentFlags::None)
            .expect("Cannot present");

        self.fence_values[self.frame_index] = last_fence_value + 1;

        self.command_queue
            .signal(&self.fence, self.fence_values[self.frame_index])
            .expect("Cannot signal fence");

        self.frame_index = (self.frame_index + 1) % FRAMES_IN_FLIGHT as usize;

        self.fence_values[self.frame_index as usize] = last_fence_value + 1;

        self.frame_count += 1;
    }

    fn sort(&self, graph: &Graph) -> Vec<usize> {
        let mut local_edges = graph.edges.clone();
        let mut exec_order = Vec::<usize>::new();
        let mut entry_nodes = Vec::<usize>::new();

        for (id, _node) in graph.nodes.iter().enumerate() {
            let num_deps = local_edges.iter().filter(|e| e.1 == id).count();
            if num_deps == 0 {
                entry_nodes.push(id);
            }
        }

        while !entry_nodes.is_empty() {
            let id = entry_nodes.pop().unwrap();
            exec_order.push(id);

            let depending_nodes = local_edges
                .iter()
                .filter(|e| e.0 == id)
                .map(|e| e.1).collect::<Vec<usize>>();
            for dep_id in depending_nodes {
                local_edges.retain(|e| e.1 != dep_id);
                let num_deps = local_edges.iter().filter(|e| e.1 == dep_id).count();
                if num_deps == 0 {
                    entry_nodes.push(dep_id);
                }
            }
        }

        exec_order
    }

    pub fn execute(&mut self, graph: &Graph) {
        let exec_order = self.sort(graph);

        for id in exec_order {
            let node = &graph.nodes[id];
            node.execute();
        }

        self.draw();
    }
}

trait Node {
    fn execute(&self);
}

struct DrawNode {}

impl Node for DrawNode {
    fn execute(&self) {
        println!("executing draw");
    }
}

struct PresentNode {}

impl Node for PresentNode {
    fn execute(&self) {
        println!("executing present");
    }
}

struct Graph {
    nodes: Vec<Box<dyn Node>>,
    edges: Vec<(usize, usize)>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add(&mut self, node: Box<dyn Node>, deps: &[usize]) -> usize {
        let id = self.nodes.len();
        self.nodes.push(node);
        for dep in deps {
            // TODO: check for loops
            self.edges.push((*dep, id));
        }
        id
    }
}

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
                renderer.execute(&graph);
            }
            _ => (),
        }
    });
}
