use druid::widget::Flex;
use druid::{
    BoxConstraints, Color, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Rect, Size, UpdateCtx,
    Widget, WidgetExt,
};
use druid::{Data, RenderContext};
use druid::{Env, Event, EventCtx};

use crate::field;
use crate::field::PointUsize;
use crate::simulator;

const MARGIN: f64 = 0.15;
const LINE_WIDTH: f64 = 0.01;

const BG_COLOR: druid::Color = druid::Color::rgb8(245, 245, 220);
const GRID_COLOR: druid::Color = druid::Color::rgb8(0, 0, 0);
const FIELD_COLOR: druid::Color = druid::Color::rgb8(220, 220, 180);
const TILE_COLOR: [druid::Color; 5] = [
    druid::Color::rgba8(0, 0, 0, 0),
    druid::Color::rgba8(255, 0, 0, 30),
    druid::Color::rgba8(0, 0, 255, 30),
    druid::Color::rgba8(255, 0, 0, 120),
    druid::Color::rgba8(0, 0, 255, 120),
];

enum ColorData {
    Bg,
    Grid,
    Field,
    Tile(field::Tile),
}

fn get_color(color_data: ColorData) -> &'static Color {
    match color_data {
        ColorData::Bg => &BG_COLOR,
        ColorData::Grid => &GRID_COLOR,
        ColorData::Field => &FIELD_COLOR,
        ColorData::Tile(tile) => match tile.state() {
            field::State::Neutral => &TILE_COLOR[0],
            field::State::Wall(fl) => &TILE_COLOR[1 + fl as usize],
            field::State::Position(fl) => &TILE_COLOR[3 + fl as usize],
        },
    }
}

#[derive(Clone, Data)]
pub struct AppData {
    pub simulator: simulator::Simulator,
}

struct GameWidget {}

impl Widget<AppData> for GameWidget {
    fn event(&mut self, event_ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {}
    fn lifecycle(
        &mut self,
        _lc_ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &AppData,
        _env: &Env,
    ) {
    }
    fn update(&mut self, update_ctx: &mut UpdateCtx, _old: &AppData, _data: &AppData, _env: &Env) {
        update_ctx.request_paint();
    }
    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AppData,
        _env: &Env,
    ) -> Size {
        bc.max()
    }
    fn paint(&mut self, paint_ctx: &mut PaintCtx, data: &AppData, _env: &Env) {
        let size = paint_ctx.size();
        let field = data.simulator.get_field();
        let grid_size = ((size.width * (1.0 - MARGIN)) / field.width() as f64)
            .min((size.height * (1.0 - MARGIN)) / field.height() as f64);
        let corner_x = (size.width - grid_size * field.width() as f64) / 2.0;
        let corner_y = (size.height - grid_size * field.height() as f64) / 2.0;

        let field_rect = Rect {
            x0: corner_x,
            y0: corner_y,
            x1: corner_x + grid_size * field.width() as f64,
            y1: corner_y + grid_size * field.height() as f64,
        };
        paint_ctx.paint_with_z_index(1, move |paint_ctx| {
            paint_ctx.fill(field_rect, get_color(ColorData::Field))
        });

        let calc_rect = |i, j| {
            druid::Rect::from_origin_size(
                druid::Point {
                    x: corner_x + i as f64 * grid_size,
                    y: corner_y + j as f64 * grid_size,
                },
                druid::Size {
                    width: grid_size,
                    height: grid_size,
                },
            )
        };

        for i in 0..field.width() {
            for j in 0..field.height() {
                let rect = calc_rect(i, j);
                let tile = field.tile(PointUsize::new(i, j));
                paint_ctx.paint_with_z_index(2, move |paint_ctx| {
                    paint_ctx.fill(rect, get_color(ColorData::Tile(tile)))
                });
                paint_ctx.paint_with_z_index(3, move |paint_ctx| {
                    paint_ctx.stroke(rect, get_color(ColorData::Grid), grid_size * LINE_WIDTH)
                });
            }
        }
    }
}

pub fn ui_builder() -> impl Widget<AppData> {
    Flex::column()
        .with_flex_child(GameWidget {}, 1.0)
        .background(get_color(ColorData::Bg).clone())
}
