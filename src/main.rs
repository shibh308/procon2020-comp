use druid::AppLauncher;
use druid::WindowDesc;

use procon31_comp::api::parse;
use procon31_comp::field;
use procon31_comp::manager;
use procon31_comp::simulator;
use procon31_comp::visualizer;

fn main() {
    let data = visualizer::AppData {
        simulator: simulator::Simulator::new(field::Field::new(None, None, None)),
        config: parse::read_config_json("./data/config.json"),
        match_data: None,
        team_data: None,
        team_data_idx: 0,
    };

    if data.config.visualizer {
        let main_window = WindowDesc::new(visualizer::ui_builder);

        AppLauncher::with_window(main_window)
            .launch(data)
            .expect("launch failed")
    } else {
        let params = parse::read_params("./data/params.json");
        manager::simulate(params);
    }
}
