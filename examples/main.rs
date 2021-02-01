use druid::{WindowDesc, im::vector, theme, widget::LineBreaking};
use druid::im::Vector;
use druid::{widget::Label, AppLauncher, Widget};
use druid::widget::RawLabel;
use druid_simple_table::{Table, WidgetExt as _};

fn main() {
    let win = WindowDesc::new(view);
    AppLauncher::with_window(win)
        .use_simple_logger()
        .launch(vector!["Hello World This Is A Great Lecture Written By Sir Manmeet Maan".into(), "Bye".into()])
        .unwrap();
}

use druid::WidgetExt;
fn view() -> impl Widget<Vector<String>> {
    Table::new()
        .col(Label::new("Name").padding(5.), || {
            RawLabel::new()
                .with_line_break_mode(LineBreaking::WordWrap)
                .padding(5.)
                .constrain_size(..400., ..)
        })
        .col(Label::new("Length").padding(5.), || {
            Label::dynamic(|data: &String, _| data.len().to_string()).padding(5.)
        })
        .border(theme::BORDER_LIGHT, 1.0)
        .center()
}
