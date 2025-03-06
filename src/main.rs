mod gui;

fn main() {
    env_logger::init();
    let app = gui::AppDate {};
    gui::run(app);
}
