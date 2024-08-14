use std::fmt::{self, Display, Formatter};
use egui::{self, Button, Ui};
use math_vector::Vector;

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
