use egui::{Align2, Color32, Mesh, Rect, RichText, Sense, Shape, pos2, vec2};

use crate::{
    data::{END_YEAR, LINEAGES, START_YEAR},
    model::{Measure, MeasureKind, Milestone, MilestoneKind, PartialDate, Region},
};

const LEFT_MARGIN: f32 = 200.;

pub struct Timeline<'a> {
    pub viewport: Rect,

    // we'll keep this scale large enough such that the timeline doesn't get smaller than the
    // viewport width
    pub pixels_per_year: &'a mut f32,
}

impl Timeline<'_> {
    pub fn width(&self) -> f32 {
        LEFT_MARGIN + *self.pixels_per_year * (END_YEAR - START_YEAR) as f32
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        *self.pixels_per_year = self
            .pixels_per_year
            .max((ui.available_width() - LEFT_MARGIN) / (END_YEAR - START_YEAR) as f32);

        let left = self.x_to_date(self.viewport.left());
        let right = self.x_to_date(self.viewport.right());

        for year in (left.year().max(START_YEAR)..=right.year().min(END_YEAR - 1)).step_by(
            match self.pixels_per_year {
                50.0.. => 1,
                25.0.. => 2,
                10.0.. => 5,
                _ => 10,
            },
        ) {
            let x = self.date_to_x(PartialDate::Year { y: year }) - self.viewport.left();

            let thickness = 1.;
            ui.painter().line_segment(
                [
                    pos2(x + thickness / 2., ui.clip_rect().top()),
                    pos2(x + thickness / 2., ui.clip_rect().bottom()),
                ],
                egui::Stroke::new(
                    thickness,
                    ui.style().visuals.weak_text_color().gamma_multiply(0.5),
                ),
            );

            ui.painter().text(
                egui::Pos2::new(x + 2., ui.clip_rect().bottom()),
                egui::Align2::LEFT_BOTTOM,
                year,
                egui::FontId::proportional(14.),
                ui.style().visuals.weak_text_color(),
            );
        }

        let row_height = 28.;

        for lineage in LINEAGES {
            let (_, rect) =
                ui.allocate_space(vec2(self.width(), lineage.consoles.len() as f32 * row_height));

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

            for (idx, console) in lineage.consoles.iter().enumerate() {
                let y = rect.top() + self.viewport.top() + idx as f32 * row_height;

                let start = self.date_to_x(
                    console
                        .releases
                        .iter()
                        .map(|r| r.date)
                        .min_by_key(|d| d.cmp_key())
                        .expect("Console must have at least one release date"),
                );

                let discontinued = console
                    .milestones
                    .iter()
                    .filter(|m| m.kind == MilestoneKind::Discontinued)
                    .max_by_key(|m| m.date.cmp_key());

                let end = discontinued.map(|m| m.date).map(|d| self.date_to_x(d));

                let console_rect = Rect::from_x_y_ranges(
                    start..=end.unwrap_or(ui.available_width()),
                    y..=(y + row_height),
                )
                .translate(-self.viewport.min.to_vec2());

                let id = ui.next_auto_id();
                ui.skip_ahead_auto_ids(1);
                let response = ui.interact(console_rect, id, Sense::click());

                if !ui.clip_rect().intersects(console_rect) {
                    continue;
                }

                let bottom_color = lineage.color;
                let bg_color =
                    bottom_color.gamma_multiply(if response.hovered() { 0.4 } else { 0.2 });

                ui.painter().rect_filled(console_rect, 0., bg_color);

                let bottom_rect = console_rect.with_min_y(console_rect.bottom() - 2.);
                ui.painter().rect_filled(bottom_rect, 0., bottom_color);

                let galley = ui.painter().layout_no_wrap(
                    console.name.to_string(),
                    egui::FontId::proportional(14.),
                    if response.hovered() {
                        ui.style().visuals.strong_text_color()
                    } else {
                        ui.style()
                            .visuals
                            .text_color()
                            .lerp_to_gamma(ui.style().visuals.strong_text_color(), 0.5)
                    },
                );

                let text_margin_x = 8.;
                let text_width = galley.size().x.ceil() + text_margin_x * 2.;

                let mut text_anchor = console_rect.left_center() + vec2(text_margin_x, 0.);

                if text_width < console_rect.width() {
                    // don't move the text off the left side of the screen...
                    text_anchor.x = text_anchor.x.max(ui.clip_rect().left() + text_margin_x);

                    // ...unless it's about to go off the right side of its timeline bar
                    text_anchor.x = text_anchor
                        .x
                        .min(console_rect.right() - text_width + text_margin_x);
                }

                let text_rect = Align2::LEFT_CENTER.anchor_size(text_anchor, galley.size());

                ui.painter().galley(text_rect.min, galley, Color32::GREEN);

                if let Some(Milestone { date: PartialDate::Year { .. }, .. }) = discontinued {
                    let fade_rect = Rect::from_min_size(
                        console_rect.right_top(),
                        vec2(*self.pixels_per_year, console_rect.height()),
                    );

                    ui.painter().add(Shape::mesh(gradient_mesh(
                        fade_rect,
                        bg_color,
                        Color32::TRANSPARENT,
                    )));

                    let bottom_fade_rect = fade_rect.with_min_y(bottom_rect.top());

                    ui.painter().add(Shape::mesh(gradient_mesh(
                        bottom_fade_rect,
                        bottom_color,
                        Color32::TRANSPARENT,
                    )));
                }

                response.on_hover_ui(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.;

                        ui.label("Sales:");
                        ui.end_row();

                        for Measure { region, value, source, .. } in console
                            .measures
                            .iter()
                            .filter(|m| m.kind == MeasureKind::UnitsSold)
                        {
                            let millions = value.point as f32 / 1_000_000.;
                            ui.label(match region {
                                Region::Global => format!("  Global: {millions:.2} M"),
                                Region::JP => format!("    JP: {millions:.2} M"),
                                Region::NA => format!("    NA: {millions:.2} M"),
                                Region::EU => format!("    EU: {millions:.2} M"),
                            });

                            if source.is_none() {
                                ui.label(
                                    RichText::new("⚠ Unverified").color(ui.visuals().warn_fg_color),
                                );
                            }

                            ui.end_row();
                        }
                    });
                });
            }
        }

        // padding for year labels
        ui.add_space(24.);
    }

    fn x_to_date(&self, mut x: f32) -> PartialDate {
        x -= LEFT_MARGIN;

        let years_from_start = x / *self.pixels_per_year;
        let year = START_YEAR + years_from_start as i32;
        let month = ((years_from_start.fract() * 12.).floor() as u8).clamp(1, 12);
        PartialDate::YearMonth { y: year, m: month }
    }

    fn date_to_x(&self, date: PartialDate) -> f32 {
        let (year, month, _day) = match date {
            PartialDate::Year { y } => (y, 1, 1),
            PartialDate::YearMonth { y, m } | PartialDate::YearMonthDay { y, m, d: _ } => (y, m, 1),
        };

        let years_from_start = (year - START_YEAR) as f32 + (month as f32 - 1.) / 12.0;
        LEFT_MARGIN + years_from_start * *self.pixels_per_year
    }
}

fn gradient_mesh(rect: Rect, color_start: Color32, color_end: Color32) -> Mesh {
    let mut mesh = Mesh::default();

    mesh.colored_vertex(rect.left_top(), color_start);
    mesh.colored_vertex(rect.right_top(), color_end);
    mesh.colored_vertex(rect.left_bottom(), color_start);
    mesh.colored_vertex(rect.right_bottom(), color_end);

    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(2, 1, 3);

    mesh
}
