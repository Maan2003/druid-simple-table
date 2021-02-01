use std::ops::RangeBounds;

use druid::{widget::prelude::*, WidgetPod};

pub struct SizeConstraint<T> {
    widget: super::DynWidget<T>,
    constraint: BoxConstraints,
}

impl<T> SizeConstraint<T> {
    pub fn new(widget: impl Widget<T> + 'static, constraint: BoxConstraints) -> Self {
        Self {
            widget: WidgetPod::new(Box::new(widget)),
            constraint,
        }
    }
}

impl<T: Data> Widget<T> for SizeConstraint<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.widget.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.widget.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.widget.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let b1_min = bc.min();
        let b2_min = self.constraint.min();
        let min = Size::new(
            b1_min.width.max(b2_min.width),
            b1_min.height.max(b2_min.height),
        );

        let b1_max = bc.max();
        let b2_max = self.constraint.max();
        let max = Size::new(
            b1_max.width.min(b2_max.width),
            b1_max.height.min(b2_max.height),
        );

        let inner_bc = if min.width > max.width || min.height > min.height {
	    eprintln!("[WARN] constraints are not compatible, using parent constraints");
            *bc
        } else {
            BoxConstraints::new(min, max)
        };

        let size = self.widget.layout(ctx, &inner_bc, data, env);
        self.widget.set_origin(ctx, data, env, (0., 0.).into());
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.widget.paint(ctx, data, env);
    }
}

pub trait WidgetExt<T>: Widget<T> + Sized + 'static {
    fn constrain_size(
        self,
        x: impl RangeBounds<f64>,
        y: impl RangeBounds<f64>,
    ) -> SizeConstraint<T> {
        let x_min = match x.start_bound() {
            std::ops::Bound::Included(size) => *size,
            std::ops::Bound::Excluded(size) => *size,
            std::ops::Bound::Unbounded => 0.,
        };
        let x_max = match x.end_bound() {
            std::ops::Bound::Included(size) => *size,
            std::ops::Bound::Excluded(size) => *size,
            std::ops::Bound::Unbounded => f64::INFINITY,
        };

        let y_min = match y.start_bound() {
            std::ops::Bound::Included(size) => *size,
            std::ops::Bound::Excluded(size) => *size,
            std::ops::Bound::Unbounded => 0.,
        };
        let y_max = match y.end_bound() {
            std::ops::Bound::Included(size) => *size,
            std::ops::Bound::Excluded(size) => *size,
            std::ops::Bound::Unbounded => f64::INFINITY,
        };
        let bc = BoxConstraints::new(Size::new(x_min, y_min), Size::new(x_max, y_max));
        SizeConstraint::new(self, bc)
    }
}

impl<T, W: Widget<T> + 'static> WidgetExt<T> for W {}
