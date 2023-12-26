use core::hash::Hash;

use crate::{EguiProbe, Style};

#[derive(Clone, Copy)]
struct ProbeHeaderState {
    open: bool,
    body_height: f32,
}

struct ProbeHeader {
    id: egui::Id,
    state: ProbeHeaderState,
    dirty: bool,
    openness: f32,
}

impl ProbeHeader {
    fn load(cx: &egui::Context, id: egui::Id) -> ProbeHeader {
        let state = cx.data_mut(|d| {
            *d.get_temp_mut_or(
                id,
                ProbeHeaderState {
                    open: true,
                    body_height: 0.0,
                },
            )
        });

        let openness = cx.animate_bool(id, state.open);

        ProbeHeader {
            id,
            state,
            dirty: false,
            openness,
        }
    }

    fn store(self, cx: &egui::Context) {
        if self.dirty {
            cx.data_mut(|d| d.insert_temp(self.id, self.state));
            cx.request_repaint();
        }
    }

    fn toggle(&mut self) {
        self.state.open = !self.state.open;
        self.dirty = true;
    }

    // fn is_open(&self) -> bool {
    //     self.state.open
    // }

    fn set_body_height(&mut self, height: f32) {
        // TODO: Better approximation
        if (self.state.body_height - height).abs() > 0.001 {
            self.state.body_height = height;
            self.dirty = true;
        }
    }

    fn body_shift(&self) -> f32 {
        (1.0 - self.openness) * self.state.body_height
    }

    fn collapse_button(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let desired_size = ui.spacing().icon_width_inner;
        let response =
            ui.allocate_response(egui::vec2(desired_size, desired_size), egui::Sense::click());

        if response.clicked() {
            self.toggle();
        }

        egui::collapsing_header::paint_default_icon(ui, self.openness, &response);
        response
    }
}

#[derive(Clone, Copy)]
struct ProbeLayoutState {
    labels_width: f32,
}

pub struct ProbeLayout {
    id: egui::Id,
    state: ProbeLayoutState,
    dirty: bool,
    min_labels_width: f32,
}

impl ProbeLayout {
    fn load(cx: &egui::Context, id: egui::Id) -> ProbeLayout {
        let state = cx.data_mut(|d| *d.get_temp_mut_or(id, ProbeLayoutState { labels_width: 0.0 }));
        ProbeLayout {
            id,
            state,
            dirty: false,
            min_labels_width: 0.0,
        }
    }

    fn store(mut self, cx: &egui::Context) {
        if self.dirty {
            self.state.labels_width = self.min_labels_width;
            cx.data_mut(|d| d.insert_temp(self.id, self.state));
            cx.request_repaint();
        }
    }

    fn bump_labels_width(&mut self, width: f32) {
        if self.min_labels_width < width {
            self.min_labels_width = width;
            self.dirty = true;
        }
    }

    pub fn inner_label_ui(&mut self, ui: &mut egui::Ui, add_content: impl FnOnce(&mut egui::Ui)) {
        let labels_width = self.state.labels_width;
        let cursor = ui.cursor();

        let max = egui::pos2(cursor.max.x.min(cursor.min.x + labels_width), cursor.max.y);
        let rect = egui::Rect::from_min_max(cursor.min, max);

        let mut label_ui = ui.child_ui(rect, *ui.layout());
        label_ui.set_clip_rect(
            ui.clip_rect()
                .intersect(egui::Rect::everything_left_of(max.x)),
        );

        add_content(&mut label_ui);
        let mut final_rect = label_ui.min_rect();

        self.bump_labels_width(final_rect.width());

        final_rect.max.x = final_rect.min.x + labels_width;

        ui.advance_cursor_after_rect(final_rect);
    }
}

/// Widget for editing a value via `EguiProbe` trait.
///
/// For simple values it will show a probe UI for it.
/// For complex values it will header with collapsible body.
#[must_use = "You should call .show()"]
pub struct Probe<'a, T> {
    id_source: egui::Id,
    label: egui::WidgetText,
    style: Style,
    value: &'a mut T,
}

impl<'a, T> Probe<'a, T>
where
    T: EguiProbe,
{
    /// Creates a new `Probe` widget.
    pub fn new(label: impl Into<egui::WidgetText>, value: &'a mut T) -> Self {
        let label = label.into();
        Probe {
            id_source: egui::Id::new(label.text()),
            label,
            style: Style::default(),
            value,
        }
    }

    /// Show probbing UI to edit the value.
    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        if !self.value.has_inner() {
            return self.value.probe(ui, &self.style);
        }

        let ref mut ui = ui.child_ui_with_id_source(
            ui.max_rect(),
            egui::Layout::top_down(egui::Align::Min),
            self.id_source,
        );

        let mut header = ProbeHeader::load(ui.ctx(), ui.id());

        egui::Frame::none()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(ui.spacing().item_spacing * 0.5)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    header.collapse_button(ui);
                    ui.label(self.label);
                });
            });

        if header.openness > 0.0 && self.value.has_inner() {
            let mut layout = ProbeLayout::load(ui.ctx(), ui.id());

            show_table(
                self.value,
                &mut header,
                &mut layout,
                ui,
                &self.style,
                "table",
            );

            layout.store(ui.ctx());
        }

        header.store(ui.ctx());

        let response = ui.interact(ui.min_rect(), ui.id(), egui::Sense::hover());
        response.widget_info(|| egui::WidgetInfo::new(egui::WidgetType::Other));
        response
    }
}

fn show_header(
    label: &str,
    value: &mut dyn EguiProbe,
    layout: &mut ProbeLayout,
    ui: &mut egui::Ui,
    style: &Style,
    id_source: impl Hash,
) -> Option<ProbeHeader> {
    let mut header = None;

    if value.has_inner() {
        header = Some(ProbeHeader::load(ui.ctx(), ui.id().with(id_source)));
    }

    ui.horizontal(|ui| {
        layout.inner_label_ui(ui, |ui| {
            if let Some(header) = &mut header {
                header.collapse_button(ui);
            }
            ui.label(label);
        });
        value.probe(ui, style);
    });

    header
}

fn show_table(
    value: &mut dyn EguiProbe,
    header: &mut ProbeHeader,
    layout: &mut ProbeLayout,
    ui: &mut egui::Ui,
    style: &Style,
    id_source: impl Hash,
) {
    let cursor = ui.cursor();

    let table_rect = egui::Rect::from_min_max(
        egui::pos2(cursor.min.x, cursor.min.y - header.body_shift()),
        ui.max_rect().max,
    );

    let mut table_ui = ui.child_ui_with_id_source(
        table_rect,
        egui::Layout::top_down(egui::Align::Min),
        id_source,
    );
    table_ui.set_clip_rect(
        ui.clip_rect()
            .intersect(egui::Rect::everything_below(ui.min_rect().max.y)),
    );

    let mut idx = 0;
    value.iterate_inner(&mut |label, value| {
        let header = show_header(label, value, layout, &mut table_ui, style, idx);

        if let Some(mut header) = header {
            if header.openness > 0.0 {
                show_table(value, &mut header, layout, &mut table_ui, style, idx);
            }
            header.store(table_ui.ctx());
        }

        idx += 1;
    });

    let final_table_rect = table_ui.min_rect();
    header.set_body_height(final_table_rect.height());

    ui.advance_cursor_after_rect(final_table_rect);
}