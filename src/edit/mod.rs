use std::{collections::HashMap};

use eframe::egui::{self, CollapsingHeader, Frame, ScrollArea, TextEdit, Ui};
use serde::{Deserialize, Serialize};
use ree_lib::{context::EngineContext, rsz::Value, types::StringU16};

use crate::save::{SaveFile, SaveFlags, remap::Remap, types::{Array, Class, EnumValue, Field, FieldValue, Struct}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathNode {
    Class(u32),
    Field(u32),
    Index(usize),
}

pub enum Action {
    CopyObject(Class),
    CopyArray(Array),
    PasteObject(Class),
    PasteArray(Array),
    ModifyReference
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ArrayConfig {
    pub right_margin: f32,
    pub target_row_height: f32,
    pub item_spacing: f32,
    pub max_inf_height: f32,
    pub available_height_delta: f32,
    pub min_max_height: f32,
    pub height_clamp: f32,
    pub max_rows: f32,
}


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct EditorConfig {
    pub array: ArrayConfig
}

impl Default for ArrayConfig {
    fn default() -> Self {
        Self {
            right_margin: 5.0,
            target_row_height: 22.0,
            item_spacing: 6.0,
            max_inf_height: 1000.0,
            available_height_delta: 5.0,
            min_max_height: 500.0,
            height_clamp: 40.0,
            max_rows: 50.0,
        }
    }
}

pub struct EditContext<'a> {
    pub engine_context: &'a EngineContext<'a>,
    pub remaps: &'a HashMap<String, Remap>,
    pub config: &'a EditorConfig,
    pub path: &'a mut Vec<PathNode>,
    pub query_cache: &'a mut HashMap<(String, String), Value>,
}

pub trait Editable {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut EditContext);
}

impl Editable for SaveFile {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut EditContext) {
        self.flags.ui(ui, ctx);
        for (unk, class) in &mut self.fields {
            let class_label = ctx.engine_context.rsz_map.get_by_hash(class.hash)
                .map(|t| t.name.clone()).unwrap_or(format!("{:08x}", class.hash));
            let label = format!("{:08x}: {}", unk, class_label);
            CollapsingHeader::new(label)
                .show(ui, |ui| {
                    class.ui(ui, ctx);
                });
        }
    }
}

impl Editable for SaveFlags {
    fn ui(&mut self, ui: &mut Ui, _ctx: &mut EditContext) {
        ui.collapsing("Save Flags", |ui| {
            ui.vertical(|ui| {
                let flag_checkbox = |ui: &mut Ui, flags: &mut SaveFlags, flag: SaveFlags, label: &str| {
                    let mut is_on = flags.contains(flag);
                    if ui.checkbox(&mut is_on, label).changed() {
                        flags.set(flag, is_on);
                    }
                };

                flag_checkbox(ui, self, SaveFlags::BLOWFISH, "Blowfish");
                flag_checkbox(ui, self, SaveFlags::HAS_ID, "HasID");
                flag_checkbox(ui, self, SaveFlags::CITRUS, "Citrus");
                flag_checkbox(ui, self, SaveFlags::DEFLATE, "Deflate");
                flag_checkbox(ui, self, SaveFlags::MANDARIN, "Mandarin");
            });
        });
    }
}

impl Editable for Class {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut EditContext) {
        ctx.path.push(PathNode::Class(self.hash));

        for  field in self.fields.iter_mut() {
            field.ui(ui, ctx);
        }

        ctx.path.pop();
    }
}

impl Editable for Array {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut EditContext) {
        let ArrayConfig { 
            right_margin, 
            target_row_height: target_height, 
            item_spacing: spacing, 
            max_inf_height, 
            available_height_delta, 
            min_max_height, 
            height_clamp, 
            max_rows 
        } = ctx.config.array;

        let mut add_array_value = |value: &mut FieldValue, i: usize, ui: &mut egui::Ui| {
            let index_label = format!("{}:", i);
            ui.push_id(i, |ui| {
                ctx.path.push(PathNode::Index(i));
                match value {
                    FieldValue::Class(_) | FieldValue::Array(_) => {
                        egui::CollapsingHeader::new(index_label)
                            .show(ui, |ui| {
                                value.ui(ui, ctx);
                            });
                    }
                    _ => {
                        ui.horizontal(|ui| {
                            ui.label(index_label);
                            value.ui(ui, ctx);
                        });
                    }
                }
                ctx.path.pop();
            });
        };

        ui.scope(|ui| {
            // wtf are these variable names dawg

            ui.style_mut().spacing.item_spacing.y = spacing;
            ui.style_mut().spacing.interact_size.y = target_height;

            let state_id = ui.make_persistent_id("row_heights");
            let mut row_heights = ui.data_mut(|d| d.get_temp::<Vec<f32>>(state_id).unwrap_or_default());

            if row_heights.len() != self.values.len() {
                row_heights.resize(self.values.len(), target_height);
            }

            let (visible_sum, visible_count): (_, u32) = row_heights.iter().enumerate()
                //.filter(|(i, _)| ctx.search_range.contains(i))
                .fold((0.0, 0), |(acc_h, acc_c), (_, h)| (acc_h + h, acc_c + 1));

            let total_content_height = visible_sum + (visible_count.saturating_sub(1) as f32 * spacing);

            let h = ui.available_height();
            let max_height = if h.is_infinite() { max_inf_height } else { (h - available_height_delta).max(min_max_height) };

            let explicit_max_height = (target_height + spacing) * max_rows;
            let view_height = total_content_height.clamp(height_clamp, max_height.min(explicit_max_height));

            let max_width = (ui.available_width() - right_margin).max(1.0);

            Frame::new()
                .fill(ui.visuals().faint_bg_color)
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .inner_margin(4.0)
                .show(ui, |ui| {
                    ScrollArea::vertical()
                        .auto_shrink([false, true])
                        .min_scrolled_height(view_height)
                        .max_width(max_width)
                        .show(ui, |ui| {
                            let clip_rect = ui.clip_rect();
                            let mut current_y = ui.cursor().min.y;
                            let last_visible_index = self.values.len().saturating_sub(1);
                            let mut rendered_count = 0;
                            for (i, value) in self.values.iter_mut().enumerate() {
                                let cached_h = row_heights[i];
                                let add_spacing = if i < last_visible_index { spacing } else { 0.0 };
                                if current_y + cached_h < clip_rect.min.y {
                                    ui.add_space(cached_h + add_spacing);
                                    current_y += cached_h + add_spacing;
                                    continue;
                                }
                                if current_y > clip_rect.max.y + 200.0 {
                                    let (rem_h, rem_c): (_, u32) = row_heights[i..].iter().enumerate()
                                        //.filter(|(offset, _)| ctx.search_range.contains(&(i + offset)))
                                        .fold((0.0, 0), |(acc_h, acc_c), (_, h)| (acc_h + h, acc_c + 1));

                                    let rem_spacing = rem_c.saturating_sub(1) as f32 * spacing;
                                    ui.add_space(rem_h + rem_spacing);
                                    break;
                                }

                                let start_pos = ui.cursor().min;
                                add_array_value(value, i, ui);

                                // update cache
                                let actual_height = ui.cursor().min.y - start_pos.y;
                                if (row_heights[i] - actual_height).abs() > 0.1 {
                                    row_heights[i] = actual_height;
                                }
                                current_y += actual_height + add_spacing;
                                rendered_count += 1;
                            }
                            log::debug!("rendered {rendered_count} elements in the array");
                        });
                });
                ui.data_mut(|d| d.insert_temp(state_id, row_heights));
        });
    }
}

impl Editable for Field {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut EditContext) {
        ctx.path.push(PathNode::Field(self.hash));

        let parent_class_hash = {
            let path = &ctx.path;
            path.iter().rev().find_map(|node| {
                if let PathNode::Class(hash) = node { Some(*hash) } else { None }
            })
        };

        //parent type info
        let type_info = parent_class_hash.and_then(|h| ctx.engine_context.rsz_map.get_by_hash(h));

        // parent class
        let class_name = type_info.map(|t| t.name.clone())
            .unwrap_or_default();

        let field_info = type_info.and_then(|ti| ti.get_field_by_hash(self.hash));

        let field_name = field_info.map(|f| f.name.clone())
            .unwrap_or(format!("{:08x}", self.hash));
        // field type label
        let mut type_label = field_info.map(|f| f.original_type.clone())
            .unwrap_or(format!("{:?}", self.field_type));


        // parent class -> field -> remapped field type
        if let Some(remap) = ctx.remaps.get(&class_name)
            && let Some(remapped_type) = remap.fields.get(&field_name) {
                if &type_label != "ace.Bitset" {
                    log::info!("Remapped {field_name}: {type_label} -> {remapped_type}");
                }
                type_label = remapped_type.clone();
        }

        let rsz_val: Value = Value::from(&self.value);
        let remapped_text = ctx.try_remap(&class_name, &field_name, &rsz_val);

        let enum_string = ctx.try_enum_str(&type_label, &self.value);

        ui.push_id(self.hash, |ui| {
            match &mut self.value {
                FieldValue::Class(_) | FieldValue::Array(_) => {
                    let header_label = format!("{}: {}", field_name, type_label);
                    egui::CollapsingHeader::new(header_label)
                        .show(ui, |ui| {
                            self.value.ui(ui, ctx);
                        });
                }
                _ => {
                    ui.horizontal(|ui| {
                        if let Some(human_name) = remapped_text {
                            ui.label(format!("{}: {}", field_name, human_name));
                        } else {
                            ui.label(format!("{}:", field_name));
                        };
                        self.value.ui(ui, ctx);

                        if let Some(enum_string) = enum_string {
                            ui.label(format!("{}", enum_string));
                        }

                    });
                }
            }
        });

        ctx.path.pop();
    }
}

impl Editable for FieldValue {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut EditContext) {
        match self {
            FieldValue::Boolean(v) => v.ui(ui, ctx),
            FieldValue::Enum(v) => v.ui(ui, ctx),
            FieldValue::S8(v) => v.ui(ui, ctx),
            FieldValue::U8(v) => v.ui(ui, ctx),
            FieldValue::S16(v) => v.ui(ui, ctx),
            FieldValue::U16(v) => v.ui(ui, ctx),
            FieldValue::S32(v) => v.ui(ui, ctx),
            FieldValue::U32(v) => v.ui(ui, ctx),
            FieldValue::S64(v) => v.ui(ui, ctx),
            FieldValue::U64(v) => v.ui(ui, ctx),
            FieldValue::F32(v) => v.ui(ui, ctx),
            FieldValue::F64(v) => v.ui(ui, ctx),
            FieldValue::C8(v) => v.ui(ui, ctx),
            FieldValue::C16(v) => v.ui(ui, ctx),
            FieldValue::Class(c) => c.ui(ui, ctx),
            FieldValue::Array(a) => a.ui(ui, ctx),
            FieldValue::String(v) => v.ui(ui, ctx),
            FieldValue::Struct(v) => v.ui(ui, ctx),
            _ => {
                ui.label(format!("{:?}", self));
            }
        }
    }
}

macro_rules! derive_editable_num {
    ($( $ty:ty ),*) => {
        $(
            impl Editable for $ty {
                fn ui(&mut self, ui: &mut eframe::egui::Ui, _ctx: &mut EditContext) {
                    ui.add(
                        eframe::egui::DragValue::new(self)
                        .speed(1.0)
                        .range(<$ty>::MIN..=<$ty>::MAX)
                    );
                }
            }
        )*
    };
}

derive_editable_num!(i8, i16, i32, i64, u8, u16, u32, u64);
derive_editable_num!(f32, f64);

impl Editable for bool {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _ctx: &mut EditContext) {
        ui.checkbox(self, "");
    }
}

impl Editable for EnumValue {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut EditContext) {
        match self {
            EnumValue::E1(v) => v.ui(ui, ctx),
            EnumValue::E2(v) => v.ui(ui, ctx),
            EnumValue::E4(v) => v.ui(ui, ctx),
            EnumValue::E8(v) => v.ui(ui, ctx),
        }
    }
}

impl Editable for String {
    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &mut EditContext) {
        ui.add(TextEdit::singleline(self).clip_text(false));
    }
}

impl Editable for StringU16 {
    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &mut EditContext) {
        let mut s = String::from_utf16_lossy(&self.0);
        ui.add(TextEdit::singleline(&mut s).clip_text(false));
        let encoded: Vec<u16> = s.encode_utf16().collect();
        *self = Self(encoded);
    }
}

impl<T: Editable> Editable for Vec<T> {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut EditContext) {
        ui.horizontal(|ui| {
            for (i, v) in self.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    v.ui(ui, ctx);
                });
            }
        });
    }
}

impl<T: Editable, const N: usize> Editable for [T; N] {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut EditContext) {
        ui.horizontal(|ui| {
            for (i, v) in self.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    v.ui(ui, ctx);
                });
            }
        });
    }
}

impl Editable for Struct {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: &mut EditContext) {
        self.data.ui(ui, ctx);
    }
}
