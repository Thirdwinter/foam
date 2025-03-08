use std::{error::Error, time::Duration};

use cairo::{Context, Format, ImageSurface};
use log::debug;
use smithay_client_toolkit::{
    output::OutputState,
    reexports::{
        calloop::{EventLoop, LoopHandle},
        calloop_wayland_source::WaylandSource,
        client::{
            globals::registry_queue_init,
            protocol::{wl_keyboard, wl_pointer, wl_shm},
            Connection, QueueHandle,
        },
    },
    registry::RegistryState,
    seat::SeatState,
    shell::{
        wlr_layer::{Anchor, KeyboardInteractivity, Layer, LayerShell, LayerSurface},
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm},
};

use crate::button::*;

#[allow(unused)]
pub enum Action {
    EXIT,
    COLOR,
}

#[allow(unused)]
pub enum Status {
    RUNNING,
    CHANGE,
}
pub struct AppDate {}

#[allow(unused)]
pub struct Foam {
    pub registry_state: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,
    pub shm: Shm,

    pub layer: LayerSurface,
    pub pointer: Option<wl_pointer::WlPointer>,
    pub keyboard: Option<wl_keyboard::WlKeyboard>,
    pub scale_factor: i32,
    pub pool: SlotPool,

    pub exit: bool,
    pub first_configure: bool,
    pub next_action: Option<Action>,
    pub app: AppDate,
    pub status: Option<Status>,

    pub sc_width: u32,
    pub sc_height: u32,

    pub width: u32,
    pub height: u32,

    pub loop_handle: LoopHandle<'static, Foam>,
    pub position: (f64, f64),
    pub buttons: Vec<Button>,
}

impl Foam {
    pub fn draw(&mut self, qh: &QueueHandle<Self>) -> Result<(), Box<dyn Error>> {
        // 计算实际显示尺寸（考虑缩放因子）
        let width = self.sc_width * self.scale_factor as u32;
        let height = self.sc_height * self.scale_factor as u32;
        let stride = width as i32 * 4;

        // 设置图层大小
        self.layer.set_size(width, height);

        // 创建缓冲区
        let (buffer, canvas) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .map_err(|e| format!("Failed to create buffer: {}", e))?;

        // 初始化缓冲区
        canvas.fill(0);

        // 创建 Cairo surface
        let surface = unsafe {
            ImageSurface::create_for_data(
                std::slice::from_raw_parts_mut(canvas.as_mut_ptr(), canvas.len()),
                Format::ARgb32,
                width as i32,
                height as i32,
                stride,
            )
            .map_err(|e| format!("Failed to create Cairo surface: {}", e))?
        };

        // 创建 Cairo 上下文
        let ctx =
            Context::new(&surface).map_err(|e| format!("Failed to create Cairo context: {}", e))?;

        // 清空背景为透明
        ctx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        ctx.set_operator(cairo::Operator::Source);
        ctx.paint()
            .map_err(|e| format!("Failed to paint background: {}", e))?;

        // 加载本地图片
        // 修改图片加载部分的代码
        let image_path = "asstes/Anime-Girl4.png"; // 替换为实际的图片路径
        let image_surface = match std::fs::File::open(image_path) {
            Ok(mut file) => ImageSurface::create_from_png(&mut file)
                .map_err(|e| format!("Failed to load image: {}", e))?,
            Err(e) => {
                eprintln!("Warning: Failed to open image file: {}", e);
                ctx.set_source_rgb(1.0, 0.0, 0.0);
                ctx.rectangle(0.0, 0.0, 200.0, 200.0);
                ctx.fill()
                    .map_err(|e| format!("Failed to draw fallback rectangle: {}", e))?;
                return Ok(());
            }
        };

        // 获取图片尺寸
        let img_width = image_surface.width() as f64;
        let img_height = image_surface.height() as f64;

        // 计算缩放比例（保持宽高比）
        let target_width = 200.0;
        let scale = target_width / img_width;
        let scaled_height = img_height * scale;

        // 保存当前绘图状态
        ctx.save()
            .map_err(|e| format!("Failed to save context state: {}", e))?;

        // 设置缩放和位置
        ctx.scale(scale, scale);
        ctx.set_source_surface(&image_surface, 0.0, 0.0)
            .map_err(|e| format!("Failed to set image as source: {}", e))?;

        // 设置图像混合模式
        ctx.set_operator(cairo::Operator::Over);

        // 绘制图像
        ctx.paint()
            .map_err(|e| format!("Failed to paint image: {}", e))?;

        // 恢复绘图状态
        ctx.restore()
            .map_err(|e| format!("Failed to restore context state: {}", e))?;

        // 设置文本样式
        ctx.set_source_rgb(1.0, 1.0, 1.0); // 白色文字
        ctx.select_font_face(
            "Maple Mono NF CN",
            cairo::FontSlant::Italic,
            cairo::FontWeight::Bold,
        );
        ctx.set_font_size(14.0);

        // 在图片下方绘制文字
        let text = "Hello Wayland!";
        let text_extents = ctx
            .text_extents(text)
            .map_err(|e| format!("Failed to get text extents: {}", e))?;

        // 计算文字位置（居中）
        let text_x = (width as f64 - text_extents.width()) / 2.0;
        let text_y = scaled_height + 20.0; // 图片下方20像素处

        ctx.move_to(text_x, text_y);
        ctx.show_text(text)
            .map_err(|e| format!("Failed to draw text: {}", e))?;

        // 完成绘制
        surface.flush();

        // 更新 surface
        self.layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);

        // 请求新一帧
        self.layer
            .wl_surface()
            .frame(qh, self.layer.wl_surface().clone());

        // 附加缓冲区到 surface
        buffer
            .attach_to(self.layer.wl_surface())
            .map_err(|e| format!("Failed to attach buffer: {}", e))?;

        // 提交更改
        self.layer.commit();

        Ok(())
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
    let output = OutputState::new(&globals, &qh);

    let surface = compositor.create_surface(&qh);

    let layer = layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("Foam"), None);
    layer.set_anchor(Anchor::all());
    // layer.set_margin(0, 0, 0, 0);
    layer.set_layer(Layer::Bottom);
    // layer.set_size(1366, 768);
    layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);

    layer.commit();
    let pool = SlotPool::new(256 * 256 * 4, &shm).expect("Failed to create pool");

    let sc_width = 200;
    let sc_height = 200;
    let width = 200;
    let height = 200;
    let buttons = vec![
        Button::new(
            "left_top".to_string(),
            (sc_width - width) / 2,
            (sc_height - height) / 2,
            width / 2,
            height / 2,
            0xFF0078D7u32.to_ne_bytes(),
            0xFF000000u32.to_ne_bytes(),
        ),
        Button::new(
            "right_top".to_string(),
            (sc_width - width) / 2 + width / 2,
            (sc_height - height) / 2,
            width / 2,
            height / 2,
            0xFF00BCF2u32.to_ne_bytes(),
            0xFF000000u32.to_ne_bytes(),
        ),
        Button::new(
            "left_bottom".to_string(),
            (sc_width - width) / 2,
            (sc_height - height) / 2 + height / 2,
            width / 2,
            height / 2,
            0xFF7FBA00u32.to_ne_bytes(),
            0xFF000000u32.to_ne_bytes(),
        ),
        Button::new(
            "right_bottom".to_string(),
            (sc_width - width) / 2 + width / 2,
            (sc_height - height) / 2 + height / 2,
            width / 2,
            height / 2,
            0xFFF25022u32.to_ne_bytes(),
            0xFF000000u32.to_ne_bytes(),
        ),
    ];

    let mut foam = Foam {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: output,
        shm,

        exit: false,
        first_configure: true,
        pool,
        sc_width: (sc_width as u32),
        sc_height: (sc_height as u32),
        height: (height as u32),
        width: (width as u32),
        status: Some(Status::RUNNING),
        layer,
        pointer: None,
        keyboard: None,
        scale_factor: 1,
        next_action: None,
        loop_handle: event_loop.handle(),
        position: (0.0, 0.0),
        app,
        buttons,
    };

    loop {
        event_loop
            .dispatch(Duration::from_millis(50), &mut foam)
            .unwrap();
        match &foam.next_action.take() {
            Some(Action::EXIT) => foam.exit = true,
            // Some(Action::COLOR) => foam.draw(&qh),
            _ => {}
        }

        if foam.exit {
            debug!("exit!");
            break;
        }
    }
}

// 修改cairo_draw函数以接受动态尺寸
fn cairo_draw(width: i32, height: i32) -> ImageSurface {
    let surface =
        ImageSurface::create(Format::ARgb32, width, height).expect("Failed to create surface");
    let ctx = Context::new(&surface).unwrap();

    // 绘制示例内容
    ctx.set_source_rgb(1.0, 1.0, 1.0); // 白色背景
    ctx.paint().unwrap();

    ctx.set_source_rgb(1.0, 0.0, 0.0); // 红色矩形
    ctx.rectangle(50.0, 50.0, (width - 100) as f64, (height - 100) as f64);
    ctx.fill().unwrap();

    ctx.set_source_rgb(0.0, 0.0, 0.0); // 黑色文字
    ctx.select_font_face("Arial", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    ctx.set_font_size(40.0);
    ctx.move_to(60.0, height as f64 / 2.0);
    ctx.show_text("Hello Wayland!").unwrap();

    surface
}
