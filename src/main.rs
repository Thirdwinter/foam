mod gui;

fn main() {
    println!("Hello, world!");
    let app = gui::AppDate {};
    gui::run(app);
}
