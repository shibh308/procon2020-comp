use druid::kurbo::Circle;
use druid::widget::Flex;
use druid::{
    BoxConstraints, Color, LayoutCtx, LifeCycle, LifeCycleCtx, MouseButton, PaintCtx, Rect, Size,
    UpdateCtx, Widget, WidgetExt,
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
const AGENT_COLOR: [druid::Color; 2] = [
    druid::Color::rgba8(255, 0, 0, 100),
    druid::Color::rgba8(0, 0, 255, 100),
];

enum ColorData {
    Bg,
    Grid,
    Field,
    Agent(bool),
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
        ColorData::Agent(side) => {
            if !side {
                &AGENT_COLOR[0]
            } else {
                &AGENT_COLOR[1]
            }
        }
    }
}

enum ClickedElement {
    Tile(field::Point),
    Agent((bool, usize)),
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
    selected: Option<(bool, usize)>,
}

impl GameWidget {
    fn update(&mut self, widget_size: Size, field: &field::Field) {
        self.size = widget_size;
        self.grid_size = ((self.size.width * (1.0 - MARGIN)) / field.width() as f64)
            .min((self.size.height * (1.0 - MARGIN)) / field.height() as f64);
        self.corner_x = (self.size.width - self.grid_size * field.width() as f64) / 2.0;
        self.corner_y = (self.size.height - self.grid_size * field.height() as f64) / 2.0;
    }
    fn tile_to_vis(&self, i: usize, j: usize, field: &field::Field) -> Rect {
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
    fn agent_to_vis(&self, side: bool, id: usize, field: &field::Field) -> Circle {
        match field.agent(side, id) {
            Some(pos) => {
                let x = self.corner_x + (pos.x as f64 + 0.5) * self.grid_size;
                let y = self.corner_y + (pos.y as f64 + 0.5) * self.grid_size;
                Circle::new((x, y), self.grid_size * 0.4)
            }
            None => {
                let center = self.size.width / 2.0;
                let circle_center_x = center
                    + self.grid_size
                        * ((field.width() / 2) as f64
                            + 0.5
                            + (if id % 2 == 1 { 1.0 } else { 0.0 }))
                        * (if side { 1.0 } else { -1.0 });
                let circle_center_y = self.corner_y + self.grid_size * ((id / 2) as f64 + 0.5);
                Circle::new((circle_center_x, circle_center_y), self.grid_size * 0.4)
            }
        }
    }
    fn tile_from_vis(&self, pos: druid::Point, field: &field::Field) -> Option<field::Point> {
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
    fn agent_from_vis(&self, pos: druid::Point, field: &field::Field) -> Option<(bool, usize)> {
        for side in vec![false, true] {
            for id in 0..field.agent_count() {
                let circle = self.agent_to_vis(side, id, field);
                if pos.distance(circle.center) < circle.radius {
                    return Some((side, id));
                }
            }
        }
        None
    }
    fn clicked_element(&self, pos: druid::Point, field: &field::Field) -> Option<ClickedElement> {
        if let Some(elm) = self.agent_from_vis(pos, field) {
            Some(ClickedElement::Agent(elm))
        } else if let Some(elm) = self.tile_from_vis(pos, field) {
            Some(ClickedElement::Tile(elm))
        } else {
            None
        }
    }
}

impl Widget<AppData> for GameWidget {
    fn event(&mut self, _event_ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {
        match event {
            Event::MouseDown(e) => match self.clicked_element(e.pos, &data.simulator.get_field()) {
                Some(ClickedElement::Tile(tile_pos)) => {
                    if let Some((side, id)) = self.selected {
                        println!("selected done: {}, {}", side, id);
                        let mut mut_field = data.simulator.get_mut_field();
                        mut_field.set_agent(side, id, Some(tile_pos));
                        mut_field.set_state(tile_pos.usize(), field::State::Wall(side));
                        self.selected = None;
                    } else {
                        println!("selected normal");
                        let state = match e.button {
                            MouseButton::Left => Some(field::State::Wall(false)),
                            MouseButton::Right => Some(field::State::Wall(true)),
                            MouseButton::Middle => Some(field::State::Neutral),
                            _ => None,
                        };
                        if let Some(raw_state) = state {
                            data.simulator
                                .get_mut_field()
                                .set_state(tile_pos.usize(), raw_state);
                            data.simulator.get_mut_field().update_region();
                        }
                    }
                }
                Some(ClickedElement::Agent((side, id))) => {
                    self.selected = Some((side, id));
                }
                _ => {}
            },
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
        let field = data.simulator.get_field();
        self.update(paint_ctx.size(), field);

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
                let rect = self.tile_to_vis(i, j, field);
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
        for side in vec![false, true] {
            for id in 0..field.agent_count() {
                let circle = self.agent_to_vis(side, id, data.simulator.get_field());
                paint_ctx.paint_with_z_index(4, move |paint_ctx| {
                    paint_ctx.fill(circle, get_color(ColorData::Agent(side)));
                });
                let width = self.grid_size * LINE_WIDTH;
                paint_ctx.paint_with_z_index(5, move |paint_ctx| {
                    paint_ctx.stroke(circle, get_color(ColorData::Grid), width);
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
                selected: None,
            },
            1.0,
        )
        .background(get_color(ColorData::Bg).clone())
}
