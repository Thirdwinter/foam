use log::debug;
use smithay_client_toolkit::{
    compositor::CompositorHandler,
    output::{OutputHandler, OutputState},
    reexports::client::{
        protocol::{wl_keyboard, wl_pointer},
        Connection, QueueHandle,
    },
    registry::{ProvidesRegistryState, RegistryState},
    seat::{
        keyboard::KeyboardHandler,
        pointer::{PointerEventKind, PointerHandler},
        SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{LayerShellHandler, LayerSurface},
        WaylandSurface,
    },
    shm::{Shm, ShmHandler},
};
use smithay_client_toolkit::{
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_shm, registry_handlers,
};

use crate::gui::{Action, Foam, Status};

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
        // println!("enter surface_enter");
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        _output: &smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
        // println!("leave surface_enter");
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
        //     self.sc_width = 256;
        //     self.sc_height = 256;
        // } else {
        //     println!("{}", configure.new_size.1);
        //     println!("{}", configure.new_size.0);
        //     self.sc_width = configure.new_size.0;
        //     self.sc_height = configure.new_size.1;
        // }

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
        if event.raw_code == 1 {
            println!("in KeyboardHandler, press_key");
            self.next_action = Some(Action::EXIT);
        }
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
                // println!("height");

                // 处理点击事件在surface之外
                // self.next_action = Some(Action::EXIT);
            }
            self.position = event.position;
            match event.kind {
                Enter { .. } => {}
                Leave { .. } => {
                    // println!("Leave!");
                }
                Press { .. } => {
                    // for button in &mut self.buttons {
                    //     if button.contains(self.position.0, self.position.1) {
                    //         button.click();
                    //         self.next_action = Some(Action::COLOR);
                    //         break;
                    //     }
                    // }
                }
                Motion { .. } => {
                    // println!("{} : {}", self.position.0, self.position.1);
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
        if capability == smithay_client_toolkit::seat::Capability::Keyboard
            && self.keyboard.is_none()
        {
            let keyboard = self
                .seat_state
                .get_keyboard(qh, &seat, None)
                .expect("Failed to create keyboard");
            self.keyboard = Some(keyboard);
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
