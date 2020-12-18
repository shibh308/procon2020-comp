use druid::AppLauncher;
use druid::WindowDesc;

use procon31_comp::api::parse;
use procon31_comp::field;
use procon31_comp::simulator;
use procon31_comp::visualizer;

fn main() {
    let main_window = WindowDesc::new(visualizer::ui_builder);

    let data = visualizer::AppData {
        simulator: simulator::Simulator::new(field::Field::new(None, None, None)),
        config: parse::read_config_json("./data/config.json"),
    };

    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed")
}
