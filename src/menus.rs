use std::fmt::{self, Display, Formatter};
use egui::{self, Button, Color32, Ui};
use math_vector::Vector;
use egui_plot::{Line, Plot, PlotItem, PlotPoints};
use web_sys::console;

use crate::rasterizer::World;

#[derive(PartialEq)]
pub enum OpticalObject {
    LightSource,
    Polarizer,
    Mirror,
    BeamSplitter,
    Wall
}

// Needed for the drop down list
impl Display for OpticalObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::LightSource => write!(f, "Light source"),
            Self::Polarizer => write!(f, "Polarizer"),
            Self::Mirror => write!(f, "Mirror"),
            Self::BeamSplitter => write!(f, "Beam splitter"),
            Self::Wall => write!(f, "Wall"),
        }
    }
}

pub struct MenusState {
    selected_object: OpticalObject
}

impl MenusState {
    pub fn new() -> MenusState {
        return MenusState {
            selected_object: OpticalObject::LightSource
        };
    }

    pub fn inspect_object_menu(&mut self, ui: &mut Ui, world: &mut World) {
        let sin: PlotPoints = (-500..500).map(|i| {
            let x = i as f64 * 0.01;
            [x, x.sin()]
        }).collect();
        let line = Line::new(sin).color(Color32::from_rgb(255, 0, 0));

        let neg_sin: PlotPoints = (-500..500).map(|i| {
            let x = i as f64 * 0.01;
            [x, -x.sin()]
        }).collect();
        let neg_line = Line::new(neg_sin).color(Color32::from_rgb(255, 0, 0));

        Plot::new("my_plot").allow_drag(false).allow_boxed_zoom(false).allow_zoom(false).view_aspect(2.0).show(ui, |plot_ui| {
            console::log_1(&format!("{:?}", plot_ui.pointer_coordinate_drag_delta()).into());
            plot_ui.line(line);
            plot_ui.line(neg_line);
        });
    }

    pub fn select_object_menu(&mut self, ui: &mut Ui, world: &mut World, viewer_position: &Vector<f32>) {
        egui::ComboBox::from_label("Optical object type")
            .selected_text(format!("{}", self.selected_object))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.selected_object, OpticalObject::LightSource, "Light source");
                ui.selectable_value(&mut self.selected_object, OpticalObject::Polarizer, "Polarizer");
                ui.selectable_value(&mut self.selected_object, OpticalObject::Wall, "Wall");
            }
        );

        if ui.add(Button::new("Click me")).clicked() {
            world.insert_cube(viewer_position.as_i32s());
        }
    }
}
