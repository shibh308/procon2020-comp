use druid::widget::Flex;
use druid::{
    BoxConstraints, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size, UpdateCtx, Widget,
};
use druid::{Data, RenderContext};
use druid::{Env, Event, EventCtx};

use super::field;

const MARGIN: f64 = 0.15;

#[derive(Clone, Data)]
pub struct AppData {
    pub field: field::Field,
}

struct GameWidget {
    i: i32,
}

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
        let field = &data.field;
        let grid_size = ((size.width * (1.0 - MARGIN)) / data.field.width() as f64)
            .min((size.height * (1.0 - MARGIN)) / data.field.height() as f64);
        let corner_x = (size.width - grid_size * field.width() as f64) / 2.0;
        let corner_y = (size.height - grid_size * field.height() as f64) / 2.0;
        for i in 0..field.width() {
            for j in 0..field.height() {
                let point = druid::Point {
                    x: corner_x + i as f64 * grid_size,
                    y: corner_y + j as f64 * grid_size,
                };
                let rect = druid::Rect::from_origin_size(
                    point,
                    Size {
                        width: grid_size,
                        height: grid_size,
                    },
                );
                let col = druid::Color::from_rgba32_u32(0x33FF33);
                paint_ctx.fill(rect, &col);
            }
        }
    }
}

pub fn ui_builder() -> impl Widget<AppData> {
    Flex::column().with_flex_child(GameWidget { i: 0_i32 }, 1.0)
}
