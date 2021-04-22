mod contraint_size;

use std::cmp::Ordering;

pub use contraint_size::{SizeConstraint, WidgetExt};

use druid::widget::SizedBox;
use druid::{im::Vector, kurbo::Line, widget::prelude::*, WidgetPod};

type DynWidget<T> = WidgetPod<T, Box<dyn Widget<T>>>;
type WidgetMake<T> = Box<dyn Fn() -> DynWidget<T>>;

pub struct Table<T> {
    cols: Vec<(WidgetMake<T>, f64, DynWidget<()>)>,
    rows: Vec<(Vec<DynWidget<T>>, f64)>,
    seperator: (f64, f64),
}

impl<T> Table<T> {
    pub fn new() -> Self {
        Self {
            cols: Vec::new(),
            rows: Vec::new(),
            seperator: (1., 1.),
        }
    }

    pub fn seperator(mut self, rows: f64, cols: f64) -> Self {
        self.seperator = (rows, cols);
        self
    }

    pub fn col<W: Widget<T> + 'static>(
        mut self,
        // header: impl Widget<()> + 'static,
        w: impl Fn() -> W + 'static,
    ) -> Self {
        self.cols.push((
            Box::new(move || WidgetPod::new(Box::new(w()))),
            0.0,
            WidgetPod::new(Box::new(SizedBox::empty())),
        ));
        self
    }
}

impl<T: Data + std::fmt::Debug> Widget<Vector<T>> for Table<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Vector<T>, env: &Env) {
        for (_, _, header) in &mut self.cols {
            header.event(ctx, event, &mut (), env);
        }
        for ((row_w, _), mut item) in self.rows.iter_mut().zip(data.iter().cloned()) {
            for cell_w in row_w {
                cell_w.event(ctx, event, &mut item, env);
            }
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &Vector<T>,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            self.rows = data
                .iter()
                .map(|_| {
                    (
                        self.cols.iter().map(|(x, _, _)| x()).collect::<Vec<_>>(),
                        0.0,
                    )
                })
                .collect::<Vec<_>>();

            ctx.children_changed();
        }
        for (_, _, header) in &mut self.cols {
            header.lifecycle(ctx, event, &(), env);
        }
        for ((row_w, _), item) in self.rows.iter_mut().zip(data) {
            for cell_w in row_w {
                cell_w.lifecycle(ctx, event, item, env);
            }
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &Vector<T>, data: &Vector<T>, env: &Env) {
        for (_, _, header) in &mut self.cols {
            header.update(ctx, &(), env);
        }

        for ((row_w, _), item) in self.rows.iter_mut().zip(data) {
            for cell_w in row_w {
                cell_w.update(ctx, item, env);
            }
        }

        match self.rows.len().cmp(&data.len()) {
            Ordering::Greater => {
                self.rows.truncate(data.len());
                ctx.children_changed();
            }
            Ordering::Less => {
                for _ in 0..(data.len() - self.rows.len()) {
                    let child = (
                        self.cols.iter().map(|(x, _, _)| x()).collect::<Vec<_>>(),
                        0.0,
                    );
                    self.rows.push(child);
                    ctx.children_changed();
                }
            }
            Ordering::Equal => (),
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &Vector<T>,
        env: &Env,
    ) -> Size {
        // reset size to 0
        let mut header_h = 0.0f64;
        for (_, col_s, header) in &mut self.cols {
            let header_size = header.layout(ctx, &bc.loosen(), &(), env);
            *col_s = header_size.width;
            header_h = header_h.max(header_size.height);
        }

        // Measure the widgets
        for ((row_w, row_size), item) in self.rows.iter_mut().zip(data) {
            *row_size = 0f64;
            for ((_, col_size, _), cell_w) in self.cols.iter_mut().zip(row_w) {
                let cell_size = cell_w.layout(ctx, &bc.loosen(), item, env);
                *col_size = col_size.max(cell_size.width);
                *row_size = row_size.max(cell_size.height);
            }
        }

        let mut x = 0.0;
        for (_, col_size, header) in &mut self.cols {
            header.layout(
                ctx,
                &BoxConstraints::tight((*col_size, header_h).into()),
                &(),
                env,
            );
            header.set_origin(ctx, &(), env, (x, 0.0).into());
            x += *col_size;
        }

        // set the origin
        let mut y = header_h;

        for ((row_w, row_size), item) in self.rows.iter_mut().zip(data) {
            let mut x = 0.0;
            for ((_, col_size, _), cell_w) in self.cols.iter_mut().zip(row_w) {
                cell_w.layout(
                    ctx,
                    &BoxConstraints::new(
                        (*col_size, *row_size).into(),
                        (*col_size, *row_size).into(),
                    ),
                    item,
                    env,
                );
                cell_w.set_origin(ctx, item, env, (x, y).into());
                x += *col_size + self.seperator.1;
            }
            y += *row_size + self.seperator.0;
        }

        let width = self
            .cols
            .iter()
            .map(|(_, s, _)| *s + self.seperator.1)
            .sum();
        bc.constrain((width, y))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Vector<T>, env: &Env) {
        let mut y = self
            .cols
            .first()
            .map(|(_, _, h)| h.layout_rect().height())
            .unwrap_or(0.0);

        let width = ctx.size().width;
        for ((row_w, row_size), item) in self.rows.iter_mut().zip(data) {
            for cell_w in row_w {
                cell_w.paint(ctx, item, env);
            }
            ctx.stroke(
                Line::new((0., y), (width, y)),
                &env.get(druid::theme::BORDER_LIGHT),
                self.seperator.0,
            );
            y += *row_size + self.seperator.0;
        }

        // Column seperators
        let mut is_first = true;
        let mut x = 0.0;
        let height = ctx.size().height;
        for (_, col_size, header) in &mut self.cols {
            header.paint(ctx, &(), env);
            if !is_first {
                ctx.stroke(
                    Line::new((x, 0.), (x, height)),
                    &env.get(druid::theme::BORDER_LIGHT),
                    self.seperator.1,
                );
            } else {
                is_first = false;
            }
            x += *col_size + self.seperator.1;
        }
    }
}
