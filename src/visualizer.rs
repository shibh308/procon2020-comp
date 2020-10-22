use druid::kurbo::Circle;
use druid::{
    BoxConstraints, Color, LayoutCtx, LifeCycle, LifeCycleCtx, MouseButton, MouseEvent, PaintCtx,
    Rect, Size, UpdateCtx, Widget, WidgetExt,
};
use druid::{Data, RenderContext};
use druid::{Env, Event, EventCtx};

use crate::algorithms;
use crate::field;
use crate::simulator;
use crate::simulator::Simulator;
use druid::widget::Flex;
use piet::{FontBuilder, Text, TextLayoutBuilder};

const MARGIN: f64 = 0.15;
const LINE_WIDTH: f64 = 0.01;
const ACTIVE_LINE_WIDTH: f64 = 0.03;
const ACT_LINE_WIDTH: f64 = 0.03;
const FONT_SIZE: f64 = 0.5;

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
const FONT_COLOR: [druid::Color; 2] = [
    druid::Color::rgba8(160, 160, 140, 180),
    druid::Color::rgba8(160, 160, 140, 220),
];

enum ColorData {
    Bg,
    Grid,
    Field,
    Point,
    ScoreText,
    Agent(bool),
    Tile(field::Tile),
}

fn get_color(color_data: ColorData) -> &'static Color {
    match color_data {
        ColorData::Bg => &BG_COLOR,
        ColorData::Grid => &GRID_COLOR,
        ColorData::Field => &FIELD_COLOR,
        ColorData::Point => &FONT_COLOR[0],
        ColorData::ScoreText => &FONT_COLOR[1],
        ColorData::Tile(tile) => match tile.state() {
            field::State::Neutral => &TILE_COLOR[0],
            field::State::Wall(fl) => &TILE_COLOR[1 + fl as usize],
            field::State::Position(fl) => &TILE_COLOR[3 + fl as usize],
        },
        ColorData::Agent(side) => &AGENT_COLOR[side as usize],
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

pub fn make_button<T: algorithms::Solver>(flex: &mut Flex<AppData>, side: bool) {
    let typename = std::any::type_name::<T>();
    let pos = typename.to_string().rfind("::");
    let text = &typename[(if pos.is_none() { 0 } else { pos.unwrap() + 2 })..];
    flex.add_flex_child(
        druid::widget::Button::new(text).on_click(move |_ctx, data: &mut AppData, _env| {
            let field = data.simulator.get_field();
            let res = T::solve(side, field);
            for id in 0..field.agent_count() {
                data.simulator.set_act(side.clone(), id, res[id].clone());
            }
        }),
        1.0,
    );
}

pub fn make_side_ui(side: bool) -> impl Widget<AppData> {
    let mut flex = Flex::column().must_fill_main_axis(true);
    make_button::<algorithms::GreedySelect>(&mut flex, side);
    flex.padding(10.).center()
}

pub fn ui_builder() -> impl Widget<AppData> {
    let mut flex = Flex::row();
    flex.add_flex_child(make_side_ui(false), 0.2);
    flex.add_spacer(10.);
    flex.add_flex_child(GameWidget::new(), 3.0);
    flex.add_spacer(10.);
    flex.add_flex_child(make_side_ui(true), 0.2);
    flex.background(get_color(ColorData::Bg).clone())
}

pub struct GameWidget {
    size: Size,
    grid_size: f64,
    corner_x: f64,
    corner_y: f64,
    selected: Option<(bool, usize)>,
}

impl GameWidget {
    pub fn new() -> GameWidget {
        GameWidget {
            size: Default::default(),
            grid_size: 0.0,
            corner_x: 0.0,
            corner_y: 0.0,
            selected: None,
        }
    }
    fn update(&mut self, widget_size: Size, field: &field::Field) {
        self.size = widget_size;
        self.grid_size = ((self.size.width * (1.0 - MARGIN)) / field.width() as f64)
            .min((self.size.height * (1.0 - MARGIN)) / field.height() as f64);
        self.corner_x = (self.size.width - self.grid_size * field.width() as f64) / 2.0;
        self.corner_y = (self.size.height - self.grid_size * field.height() as f64) / 2.0;
    }
    fn tile_center(&self, i: usize, j: usize, _field: &field::Field) -> druid::Point {
        druid::Point {
            x: self.corner_x + (i as f64 + 0.5) * self.grid_size,
            y: self.corner_y + (j as f64 + 0.5) * self.grid_size,
        }
    }
    fn agent_center(&self, side: bool, id: usize, field: &field::Field) -> druid::Point {
        match field.agent(side, id) {
            Some(pos) => self.tile_center(pos.x as usize, pos.y as usize, field),
            None => {
                let center = self.size.width / 2.0;
                let circle_center_x = center
                    + self.grid_size
                        * ((field.width() as f64 / 2.0)
                            + 0.5
                            + (if id % 2 == 1 { 1.0 } else { 0.0 }))
                        * (if side { 1.0 } else { -1.0 });
                let circle_center_y = self.corner_y + self.grid_size * ((id / 2) as f64 + 0.5);
                druid::Point::new(circle_center_x, circle_center_y)
            }
        }
    }
    fn tile_to_vis(&self, i: usize, j: usize, _field: &field::Field) -> Rect {
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
        Circle::new(self.agent_center(side, id, field), self.grid_size * 0.4)
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

    fn event_set_act(
        &mut self,
        e: &MouseEvent,
        simulator: &mut Simulator,
        side: bool,
        id: usize,
        tile_pos: field::Point,
    ) {
        let op_agent_pos = simulator.get_field().agent(side, id);
        let op_state = match op_agent_pos {
            Some(agent_pos) => match e.button {
                MouseButton::Left | MouseButton::Right => {
                    if !tile_pos.neighbor(agent_pos) {
                        None
                    } else {
                        match e.button {
                            MouseButton::Left => Some(simulator::Act::MoveAct(tile_pos)),
                            MouseButton::Right => Some(simulator::Act::RemoveAct(tile_pos)),
                            _ => None,
                        }
                    }
                }
                _ => None,
            },
            None => Some(simulator::Act::PutAct(tile_pos)),
        };

        if op_state.is_some() {
            simulator.set_act(side, id, op_state.unwrap());
        }
        self.selected = None;
    }
}

impl Widget<AppData> for GameWidget {
    fn event(&mut self, event_ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {
        match event {
            Event::MouseDown(e) => match self.clicked_element(e.pos, &data.simulator.get_field()) {
                Some(ClickedElement::Tile(tile_pos)) => {
                    if let Some((side, id)) = self.selected {
                        self.event_set_act(e, &mut data.simulator, side, id, tile_pos);
                    }
                }
                Some(ClickedElement::Agent((side, id))) => match self.selected {
                    Some((selected_side, selected_id)) => {
                        let tile_pos = data.simulator.get_field().agent(side, id);
                        self.event_set_act(
                            e,
                            &mut data.simulator,
                            selected_side,
                            selected_id,
                            tile_pos.unwrap(),
                        );
                    }
                    None => {
                        self.selected = Some((side, id));
                        event_ctx.request_paint();
                    }
                },
                _ => {}
            },
            Event::Wheel(_) => data.simulator.change_turn(),
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
                paint_ctx.paint_with_z_index(3, move |paint_ctx| {
                    paint_ctx.fill(rect, get_color(ColorData::Tile(tile)))
                });
                let width = self.grid_size * LINE_WIDTH;
                paint_ctx.paint_with_z_index(4, move |paint_ctx| {
                    paint_ctx.stroke(rect, get_color(ColorData::Grid), width)
                });
                let point_str = &tile.point().to_string();
                let text = paint_ctx.render_ctx.text();
                let font = text
                    .new_font_by_name("Segoe UI", self.grid_size * FONT_SIZE)
                    .build()
                    .expect("font not found");
                let layout = text
                    .new_text_layout(&font, point_str, self.grid_size * FONT_SIZE)
                    .build()
                    .expect("layout build failed");
                let mut pos = self.tile_center(i, j, field);
                pos.x -= self.grid_size * 0.4;
                pos.y += self.grid_size * 0.2;
                paint_ctx.paint_with_z_index(2, move |paint_ctx| {
                    paint_ctx.draw_text(&layout, pos, get_color(ColorData::Point));
                });
            }
        }

        for side in vec![false, true] {
            for id in 0..field.agent_count() {
                let circle = self.agent_to_vis(side, id, data.simulator.get_field());
                paint_ctx.paint_with_z_index(5, move |paint_ctx| {
                    paint_ctx.fill(circle, get_color(ColorData::Agent(side)));
                });
                let width = self.grid_size
                    * if self.selected == Some((side, id)) {
                        ACTIVE_LINE_WIDTH
                    } else {
                        LINE_WIDTH
                    };
                paint_ctx.paint_with_z_index(6, move |paint_ctx| {
                    paint_ctx.stroke(circle, get_color(ColorData::Grid), width);
                });
            }
            for id in 0..field.agent_count() {
                match data.simulator.get_act(side, id) {
                    simulator::Act::PutAct(act_pos)
                    | simulator::Act::MoveAct(act_pos)
                    | simulator::Act::RemoveAct(act_pos) => {
                        let line = druid::kurbo::Line::new(
                            self.agent_center(side, id, field),
                            self.tile_center(act_pos.x as usize, act_pos.y as usize, field),
                        );
                        let width = self.grid_size * ACT_LINE_WIDTH;
                        paint_ctx.paint_with_z_index(7, move |paint_ctx| {
                            paint_ctx.stroke(line, get_color(ColorData::Agent(side)), width);
                        });
                    }
                    _ => {}
                }
            }
        }
        let x_pos = [
            self.corner_x - self.grid_size * 2.0,
            self.corner_x + self.grid_size * (field.width() as f64 + 0.2),
        ];
        for side in vec![false, true] {
            let score = field.score(side);
            let score_str = &format!("{}+{}={}", score.tile(), score.region(), score.sum());
            let text = paint_ctx.render_ctx.text();
            let font = text
                .new_font_by_name("Segoe UI", self.grid_size * FONT_SIZE)
                .build()
                .expect("font not found");
            let layout = text
                .new_text_layout(&font, score_str, self.grid_size * FONT_SIZE)
                .build()
                .expect("layout build failed");
            let pos = druid::Point::new(
                x_pos[side as usize],
                self.corner_y + self.grid_size * (field.height() as f64 + 0.6),
            );
            paint_ctx.paint_with_z_index(2, move |paint_ctx| {
                paint_ctx.draw_text(&layout, pos, get_color(ColorData::Agent(side)));
            });
        }
        let turn_str = &format!("{}/{}", field.now_turn(), field.final_turn());
        let text = paint_ctx.render_ctx.text();
        let font = text
            .new_font_by_name("Segoe UI", self.grid_size * FONT_SIZE)
            .build()
            .expect("font not found");
        let layout = text
            .new_text_layout(&font, turn_str, self.grid_size * FONT_SIZE)
            .build()
            .expect("layout build failed");
        let pos = druid::Point::new(
            self.corner_x - self.grid_size * 2.0,
            self.corner_y + self.grid_size * (field.height() as f64 - 1.4),
        );
        paint_ctx.paint_with_z_index(2, move |paint_ctx| {
            paint_ctx.draw_text(&layout, pos, get_color(ColorData::ScoreText));
        });
    }
}
