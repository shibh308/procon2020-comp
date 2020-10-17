use druid::AppLauncher;
use druid::WindowDesc;

use procon31_comp::field;
use procon31_comp::visualizer;

fn main() {
    let main_window = WindowDesc::new(visualizer::ui_builder);

    let data = visualizer::AppData {
        field: field::Field::make(16, 12),
    };

    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed")
}
