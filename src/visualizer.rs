use druid::widget::Flex;
use druid::{
    BoxConstraints, Color, KeyCode, LayoutCtx, LifeCycle, LifeCycleCtx, MouseButton, PaintCtx,
    Rect, Size, UpdateCtx, Widget, WidgetExt,
};
use druid::{Data, RenderContext};
use druid::{Env, Event, EventCtx};

use crate::field;
use crate::simulator;

const MARGIN: f64 = 0.15;
const LINE_WIDTH: f64 = 0.01;

const BG_COLOR: druid::Color = druid::Color::rgb8(245, 245, 220);
const GRID_COLOR: druid::Color = druid::Color::rgb8(0, 0, 0);
const FIELD_COLOR: druid::Color = druid::Color::rgb8(220, 220, 180);
const TILE_COLOR: [druid::Color; 5] = [
    druid::Color::rgba8(0, 0, 0, 0),
    druid::Color::rgba8(255, 0, 0, 80),
    druid::Color::rgba8(0, 0, 255, 80),
    druid::Color::rgba8(255, 0, 0, 30),
    druid::Color::rgba8(0, 0, 255, 30),
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

struct GameWidget {
    size: Size,
    grid_size: f64,
    corner_x: f64,
    corner_y: f64,
}

impl GameWidget {
    fn calc_rect(&self, i: usize, j: usize) -> Rect {
        druid::Rect::from_origin_size(
            druid::Point {
                x: self.corner_x + i as f64 * self.grid_size,
                y: self.corner_y + j as f64 * self.grid_size,
            },
            druid::Size {
                width: self.grid_size,
                height: self.grid_size,
            },
        )
    }
    fn calc_pos(&self, pos: druid::Point, field: &field::Field) -> Option<field::Point> {
        let x_pos = (pos.x - self.corner_x) / self.grid_size;
        let y_pos = (pos.y - self.corner_y) / self.grid_size;
        if x_pos < 0.0
            || field.width() as f64 <= x_pos
            || y_pos < 0.0
            || field.height() as f64 <= y_pos
        {
            None
        } else {
            Some(field::Point::new(x_pos as i8, y_pos as i8))
        }
    }
}

impl Widget<AppData> for GameWidget {
    fn event(&mut self, event_ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {
        match event {
            Event::MouseDown(e) => {
                println!("mousedown");
                if let Some(pos) = self.calc_pos(e.pos, data.simulator.get_field()) {
                    let state = match e.button {
                        MouseButton::Left => Some(field::State::Wall(false)),
                        MouseButton::Right => Some(field::State::Wall(true)),
                        MouseButton::Middle => Some(field::State::Neutral),
                        _ => None,
                    };
                    if let Some(raw_state) = state {
                        data.simulator
                            .get_mut_field()
                            .set_state(pos.usize(), raw_state);
                        data.simulator.calc_region();
                    }
                }
            }
            _ => {}
        }
    }
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
        self.size = paint_ctx.size();
        let field = data.simulator.get_field();
        self.grid_size = ((self.size.width * (1.0 - MARGIN)) / field.width() as f64)
            .min((self.size.height * (1.0 - MARGIN)) / field.height() as f64);
        self.corner_x = (self.size.width - self.grid_size * field.width() as f64) / 2.0;
        self.corner_y = (self.size.height - self.grid_size * field.height() as f64) / 2.0;

        let field_rect = Rect {
            x0: self.corner_x,
            y0: self.corner_y,
            x1: self.corner_x + self.grid_size * field.width() as f64,
            y1: self.corner_y + self.grid_size * field.height() as f64,
        };
        paint_ctx.paint_with_z_index(1, move |paint_ctx| {
            paint_ctx.fill(field_rect, get_color(ColorData::Field))
        });

        for i in 0..field.width() {
            for j in 0..field.height() {
                let rect = self.calc_rect(i, j);
                let tile = field.tile(field::PointUsize::new(i, j));
                paint_ctx.paint_with_z_index(2, move |paint_ctx| {
                    paint_ctx.fill(rect, get_color(ColorData::Tile(tile)))
                });
                let width = self.grid_size * LINE_WIDTH;
                paint_ctx.paint_with_z_index(3, move |paint_ctx| {
                    paint_ctx.stroke(rect, get_color(ColorData::Grid), width)
                });
            }
        }
    }
}

pub fn ui_builder() -> impl Widget<AppData> {
    Flex::column()
        .with_flex_child(
            GameWidget {
                size: Default::default(),
                grid_size: 0.0,
                corner_x: 0.0,
                corner_y: 0.0,
            },
            1.0,
        )
        .background(get_color(ColorData::Bg).clone())
}
