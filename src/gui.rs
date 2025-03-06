use std::time::Duration;

// use cairo::glib::bitflags::Flags;
use log::debug;
use smithay_client_toolkit::{
    compositor::CompositorHandler,
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::{EventLoop, LoopHandle},
        calloop_wayland_source::WaylandSource,
        client::{
            globals::registry_queue_init,
            protocol::{wl_keyboard, wl_pointer, wl_shm},
            Connection, QueueHandle,
        },
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::KeyboardHandler,
        pointer::{PointerEventKind, PointerHandler},
        SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};

enum Action {
    EXIT,
    MAX,
    MIN,
}

enum Status {
    RUNNING,
    CHANGE,
}
pub struct AppDate {}

struct Foam {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,

    layer: LayerSurface,
    pointer: Option<wl_pointer::WlPointer>,
    scale_factor: i32,
    pool: SlotPool,

    exit: bool,
    first_configure: bool,
    next_action: Option<Action>,

    app: AppDate,
    status: Option<Status>,

    width: u32,
    height: u32,

    loop_handle: LoopHandle<'static, Foam>,
    cursor: (f64, f64),
}

impl Foam {
    pub fn draw(&mut self, qh: &QueueHandle<Self>) {
        let width = self.width * self.scale_factor as u32;
        let height = self.height * self.scale_factor as u32;
        let stride = width as i32 * 4;
        self.layer.set_size(width, height);

        let (buffer, canvas) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .expect("Failed to create buffer");
        canvas.fill(0);

        // Define Windows standard colors for each section and border
        let border_color = 0xFF000000u32.to_ne_bytes(); // Black border
        let color_top_left = 0xFF0078D7u32.to_ne_bytes(); // Blue
        let color_top_right = 0xFF00BCF2u32.to_ne_bytes(); // Light Blue
        let color_bottom_left = 0xFF7FBA00u32.to_ne_bytes(); // Green
        let color_bottom_right = 0xFFF25022u32.to_ne_bytes(); // Orange

        let rect_width = 200 as i32;
        let rect_height = 200 as i32;
        println!("{}", self.width);
        println!("{}", self.height);
        let rect_x = (width as i32 - rect_width) / 2;
        let rect_y = (height as i32 - rect_height) / 2;
        let border_thickness = 10;

        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let offset = (y * stride + x * 4) as usize;

                // Check if pixel is within the border
                if x >= rect_x - border_thickness
                    && x < rect_x + rect_width + border_thickness
                    && y >= rect_y - border_thickness
                    && y < rect_y + rect_height + border_thickness
                    && (x < rect_x
                        || x >= rect_x + rect_width
                        || y < rect_y
                        || y >= rect_y + rect_height)
                {
                    canvas[offset..offset + 4].copy_from_slice(&border_color);
                }
                // Check if pixel is within the rectangle
                else if x >= rect_x
                    && x < rect_x + rect_width
                    && y >= rect_y
                    && y < rect_y + rect_height
                {
                    // Determine which section the pixel is in
                    if y < rect_y + rect_height / 2 {
                        if x < rect_x + rect_width / 2 {
                            canvas[offset..offset + 4].copy_from_slice(&color_top_left);
                        } else {
                            canvas[offset..offset + 4].copy_from_slice(&color_top_right);
                        }
                    } else {
                        if x < rect_x + rect_width / 2 {
                            canvas[offset..offset + 4].copy_from_slice(&color_bottom_left);
                        } else {
                            canvas[offset..offset + 4].copy_from_slice(&color_bottom_right);
                        }
                    }
                }
            }
        }

        self.layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);
        self.layer
            .wl_surface()
            .frame(qh, self.layer.wl_surface().clone());

        buffer
            .attach_to(self.layer.wl_surface())
            .expect("buffer attach err");
        self.layer.commit();
    }
}

pub fn run(app: AppDate) {
    let conn = Connection::connect_to_env().unwrap();

    let (globals, event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();
    let mut event_loop: EventLoop<Foam> =
        EventLoop::try_new().expect("Failed to initialize event loop");
    let loop_handle = event_loop.handle();
    WaylandSource::new(conn, event_queue)
        .insert(loop_handle)
        .unwrap();
    let compositor = smithay_client_toolkit::compositor::CompositorState::bind(&globals, &qh)
        .expect("wl_compositor is not available");
    let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");

    let surface = compositor.create_surface(&qh);

    let layer = layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("Foam"), None);
    // layer.set_anchor(Anchor::all());
    // layer.set_margin(0, 0, 0, 0);
    layer.set_layer(Layer::Bottom);
    layer.set_size(1366, 768);
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);

    layer.commit();
    let pool = SlotPool::new(256 * 256 * 4, &shm).expect("Failed to create pool");

    let mut foam = Foam {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,

        exit: false,
        first_configure: true,
        pool,
        width: 256,
        height: 256,
        status: Some(Status::RUNNING),
        layer,
        pointer: None,
        scale_factor: 1,
        next_action: None,
        loop_handle: event_loop.handle(),
        cursor: (0.0, 0.0),
        app,
    };

    loop {
        event_loop
            .dispatch(Duration::from_millis(50), &mut foam)
            .unwrap();
        match &foam.next_action.take() {
            Some(Action::EXIT) => foam.exit = true,
            // Some(Action::MAX) => {
            //     foam.resize(256, 512, &qh);
            //     println!("max")
            // }
            // Some(Action::MIN) => {
            //     foam.resize(256, 256, &qh);
            //     println!("MIN")
            // }
            _ => {}
        }

        if foam.exit {
            debug!("exit!");
            break;
        }
    }
}

impl CompositorHandler for Foam {
    fn scale_factor_changed(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        new_factor: i32,
    ) {
        self.scale_factor = new_factor;
        self.layer.set_buffer_scale(new_factor as u32).unwrap();
    }

    fn transform_changed(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        _new_transform: smithay_client_toolkit::reexports::client::protocol::wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        _time: u32,
    ) {
        match self.status.take() {
            Some(Status::RUNNING) => {
                return;
            }
            Some(Status::CHANGE) => {
                println!("change!");

                self.draw(qh);
                self.status = Some(Status::RUNNING);
            }

            _ => {}
        }
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        _output: &smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
        println!("enter surface_enter");
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        _output: &smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
        println!("leave surface_enter");
    }
}

impl OutputHandler for Foam {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _output: smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _output: smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _output: smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for Foam {
    fn closed(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _layer: &LayerSurface,
    ) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        _configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
        _serial: u32,
    ) {
        // if configure.new_size.0 == 0 || configure.new_size.1 == 0 {
        //     self.width = 256;
        //     self.height = 256;
        // } else {
        //     println!("{}", configure.new_size.1);
        //     println!("{}", configure.new_size.0);
        //     self.width = configure.new_size.0;
        //     self.height = configure.new_size.1;
        // }
        //
        if self.first_configure {
            self.first_configure = false;

            self.draw(qh);
        }
    }
}

impl KeyboardHandler for Foam {
    fn enter(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        _serial: u32,
        _raw: &[u32],
        _keysyms: &[smithay_client_toolkit::seat::keyboard::Keysym],
    ) {
    }

    fn leave(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        _serial: u32,
    ) {
    }

    fn press_key(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        _serial: u32,
        event: smithay_client_toolkit::seat::keyboard::KeyEvent,
    ) {
        debug!("Key press: {event:?}");
    }

    fn release_key(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        _serial: u32,
        event: smithay_client_toolkit::seat::keyboard::KeyEvent,
    ) {
        debug!("Key release: {event:?}");
    }

    fn update_modifiers(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _keyboard: &wl_keyboard::WlKeyboard,
        _serial: u32,
        modifiers: smithay_client_toolkit::seat::keyboard::Modifiers,
        _layout: u32,
    ) {
        debug!("Update modifiers: {modifiers:?}");
    }
}

impl PointerHandler for Foam {
    fn pointer_frame(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[smithay_client_toolkit::seat::pointer::PointerEvent],
    ) {
        use PointerEventKind::{Enter, Leave, Motion, Press};
        for event in events {
            if &event.surface != self.layer.wl_surface() {
                continue;
            }
            self.cursor = event.position;

            let rect_width = 200;
            let rect_height = 200;
            let rect_x = (self.width as i32 - rect_width) / 2;
            let rect_y = (self.height as i32 - rect_height) / 2;

            let top_left_region = (
                rect_x,
                rect_y,
                rect_x + rect_width / 2,
                rect_y + rect_height / 2,
            );
            let top_right_region = (
                rect_x + rect_width / 2,
                rect_y,
                rect_x + rect_width,
                rect_y + rect_height / 2,
            );
            let bottom_left_region = (
                rect_x,
                rect_y + rect_height / 2,
                rect_x + rect_width / 2,
                rect_y + rect_height,
            );
            let bottom_right_region = (
                rect_x + rect_width / 2,
                rect_y + rect_height / 2,
                rect_x + rect_width,
                rect_y + rect_height,
            );

            match event.kind {
                Enter { .. } => {
                    println!("enter!");
                }
                Leave { .. } => {
                    println!("Leave!");
                }
                Press { .. } => {
                    println!("click!");
                    if (self.cursor.0 as i32 >= top_left_region.0)
                        && (self.cursor.0 as i32 <= top_left_region.2)
                        && (self.cursor.1 as i32 >= top_left_region.1)
                        && (self.cursor.1 as i32 <= top_left_region.3)
                    {
                        println!("Top-Left Region Clicked!");
                        // Add your top-left region click event logic here
                    } else if (self.cursor.0 as i32 >= top_right_region.0)
                        && (self.cursor.0 as i32 <= top_right_region.2)
                        && (self.cursor.1 as i32 >= top_right_region.1)
                        && (self.cursor.1 as i32 <= top_right_region.3)
                    {
                        println!("Top-Right Region Clicked!");
                        // Add your top-right region click event logic here
                    } else if (self.cursor.0 as i32 >= bottom_left_region.0)
                        && (self.cursor.0 as i32 <= bottom_left_region.2)
                        && (self.cursor.1 as i32 >= bottom_left_region.1)
                        && (self.cursor.1 as i32 <= bottom_left_region.3)
                    {
                        println!("Bottom-Left Region Clicked!");
                        // Add your bottom-left region click event logic here
                    } else if (self.cursor.0 as i32 >= bottom_right_region.0)
                        && (self.cursor.0 as i32 <= bottom_right_region.2)
                        && (self.cursor.1 as i32 >= bottom_right_region.1)
                        && (self.cursor.1 as i32 <= bottom_right_region.3)
                    {
                        println!("Bottom-Right Region Clicked!");
                        // Add your bottom-right region click event logic here
                    } else {
                        self.next_action = Some(Action::EXIT);
                    }
                }
                Motion { .. } => {
                    // println!("{} : {}", self.cursor.0, self.cursor.1);
                }
                _ => {}
            }
        }
    }
}

impl SeatHandler for Foam {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat,
    ) {
        println!("enter new seat");
    }

    fn new_capability(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        qh: &QueueHandle<Self>,
        seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat,
        capability: smithay_client_toolkit::seat::Capability,
    ) {
        if capability == smithay_client_toolkit::seat::Capability::Pointer && self.pointer.is_none()
        {
            debug!("Set pointer capability");
            let pointer = self
                .seat_state
                .get_pointer(qh, &seat)
                .expect("Failed to create pointer");
            self.pointer = Some(pointer);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat,
        _capability: smithay_client_toolkit::seat::Capability,
    ) {
    }

    fn remove_seat(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat,
    ) {
    }
}

impl ShmHandler for Foam {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl ProvidesRegistryState for Foam {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

delegate_compositor!(Foam);
delegate_output!(Foam);
delegate_shm!(Foam);
delegate_seat!(Foam);
delegate_keyboard!(Foam);
delegate_pointer!(Foam);
delegate_layer!(Foam);
delegate_registry!(Foam);
