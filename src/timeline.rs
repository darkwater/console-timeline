use egui::{Rect, pos2, vec2};

use crate::{
    data::{END_YEAR, LINEAGES, START_YEAR},
    model::{MilestoneKind, PartialDate},
};

const LEFT_MARGIN: f32 = 200.;
const PIXELS_PER_YEAR: f32 = 100.;
const WIDTH: f32 = LEFT_MARGIN + PIXELS_PER_YEAR * (END_YEAR - START_YEAR) as f32;

pub struct Timeline {
    pub viewport: Rect,
}

impl Timeline {
    pub fn show(&self, ui: &mut egui::Ui) {
        let left = x_to_date(self.viewport.left());
        let right = x_to_date(self.viewport.right());

        for year in left.year().max(START_YEAR)..=right.year().min(END_YEAR - 1) {
            let x = date_to_x(PartialDate::Year { y: year }) - self.viewport.left();

            let thickness = 1.;
            ui.painter().line_segment(
                [
                    pos2(x + thickness / 2., ui.clip_rect().top()),
                    pos2(x + thickness / 2., ui.clip_rect().bottom()),
                ],
                egui::Stroke::new(thickness, ui.style().visuals.weak_text_color()),
            );

            ui.painter().text(
                egui::Pos2::new(x + 2., ui.clip_rect().bottom()),
                egui::Align2::LEFT_BOTTOM,
                year,
                egui::FontId::proportional(14.),
                ui.style().visuals.weak_text_color(),
            );
        }

        for lineage in LINEAGES {
            let (_, rect) = ui.allocate_space(vec2(WIDTH, lineage.consoles.len() as f32 * 24.));

            let (header, rest) = rect.split_left_right_at_x(rect.left() + LEFT_MARGIN);

            ui.painter()
                .rect_filled(header, 0., lineage.color.gamma_multiply(0.6));

            ui.painter().text(
                header.center(),
                egui::Align2::CENTER_CENTER,
                lineage.name,
                egui::FontId::proportional(16.),
                egui::Color32::WHITE,
            );

            ui.painter().rect_filled(
                rest.split_left_right_at_x(ui.clip_rect().right()).0,
                0.,
                lineage.color.gamma_multiply(0.2),
            );

            let row_height = 24.;
            for (idx, console) in lineage.consoles.iter().enumerate() {
                let y = rect.top() + idx as f32 * row_height;

                let start = date_to_x(
                    console
                        .releases
                        .iter()
                        .map(|r| r.date)
                        .min_by_key(|d| d.cmp_key())
                        .expect("Console must have at least one release date"),
                );

                let end = console
                    .milestones
                    .iter()
                    .find(|m| m.kind == MilestoneKind::EndOfProduction)
                    .map(|m| m.date)
                    .map(date_to_x);

                let rect = Rect::from_x_y_ranges(
                    start..=end.unwrap_or(ui.available_width()),
                    y..=(y + row_height),
                )
                .translate(-self.viewport.min.to_vec2());

                if !ui.clip_rect().intersects(rect) {
                    continue;
                }

                ui.painter().rect_filled(
                    rect.with_min_y(rect.bottom() - 2.),
                    0.,
                    lineage.color.gamma_multiply(0.8),
                );

                ui.painter().text(
                    rect.left_bottom(),
                    egui::Align2::LEFT_BOTTOM,
                    console.name,
                    egui::FontId::proportional(14.),
                    egui::Color32::WHITE,
                );
            }
        }
    }
}

fn x_to_date(mut x: f32) -> PartialDate {
    x -= LEFT_MARGIN;

    let years_from_start = x / PIXELS_PER_YEAR;
    let year = START_YEAR + years_from_start as i32;
    let month = ((years_from_start.fract() * 12.).floor() as u8).clamp(1, 12);
    PartialDate::YearMonth { y: year, m: month }
}

fn date_to_x(date: PartialDate) -> f32 {
    let (year, month, _day) = match date {
        PartialDate::Year { y } => (y, 1, 1),
        PartialDate::YearMonth { y, m } | PartialDate::YearMonthDay { y, m, d: _ } => (y, m, 1),
    };

    let years_from_start = (year - START_YEAR) as f32 + (month as f32 - 1.) / 12.0;
    LEFT_MARGIN + years_from_start * PIXELS_PER_YEAR
}
