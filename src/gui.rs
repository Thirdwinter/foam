use std::time::Duration;

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

    is_show: bool,
    has_init: bool,
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

        let cx = width as i32 / 2;
        let cy = height as i32 / 2;
        let radius = (width.min(height) as f32 / 2.0).round() as i32;
        let color = 0x801E1E2Eu32.to_ne_bytes();

        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy <= radius * radius {
                    let offset = (y * stride + x * 4) as usize;
                    canvas[offset..offset + 4].copy_from_slice(&color);
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

    pub fn resize(&mut self, width: u32, height: u32, qh: &QueueHandle<Self>) {
        self.width = width;
        self.height = height;
        self.status = Some(Status::CHANGE);
        self.draw(qh)
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
        self.is_show = false;
    }

    fn configure(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if configure.new_size.0 == 0 || configure.new_size.1 == 0 {
            self.width = 256;
            self.height = 256;
        } else {
            self.width = configure.new_size.0;
            self.height = configure.new_size.1;
        }

        if !self.has_init {
            self.has_init = true;

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
            //NOTE: byd 是layer内部的相对坐标

            match event.kind {
                Enter { .. } => {
                    println!("enter!");
                    self.next_action = Some(Action::MAX); // 鼠标进入时扩大
                }
                Leave { .. } => {
                    println!("Leave!");
                    self.next_action = Some(Action::MIN); // 鼠标离开时缩小
                }
                Press { .. } => {
                    println!("click!");
                    self.next_action = Some(Action::EXIT);
                }
                Motion { .. } => {
                    println!("{} : {}", self.cursor.0, self.cursor.1);
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
    layer.set_anchor(Anchor::LEFT | Anchor::RIGHT | Anchor::TOP | Anchor::BOTTOM);
    layer.set_margin(200, 0, 0, 400);
    layer.set_size(256, 256);
    layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);

    layer.commit();
    let pool = SlotPool::new(256 * 256 * 4, &shm).expect("Failed to create pool");

    let mut foam = Foam {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,

        is_show: true,
        has_init: false,
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
            Some(Action::EXIT) => foam.is_show = false,
            Some(Action::MAX) => {
                foam.resize(256, 512, &qh);
                println!("max")
            }
            Some(Action::MIN) => {
                foam.resize(256, 256, &qh);
                println!("MIN")
            }
            _ => {}
        }

        if !foam.is_show {
            debug!("exit!");
            break;
        }
    }
}

delegate_compositor!(Foam);
delegate_output!(Foam);
delegate_shm!(Foam);
delegate_seat!(Foam);
delegate_keyboard!(Foam);
delegate_pointer!(Foam);
delegate_layer!(Foam);
delegate_registry!(Foam);
