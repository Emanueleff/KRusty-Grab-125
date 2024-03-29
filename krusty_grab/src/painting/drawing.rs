use std::collections::VecDeque;

use egui::{Context, Pos2, Stroke, Rect, Vec2, Rgba, Color32, Layout, Align, Button, Id, color_picker::{color_edit_button_rgba, Alpha}, DragValue, Ui, LayerId, Order, pos2, Align2, FontId, Widget, Window, Painter, CursorIcon, RichText, TextStyle};
use egui_extras::RetainedImage;
use native_dialog::FileDialog;
use serde::{Serialize, Deserialize};
use crate::krustygrab::{self, KrustyGrab };
use crate::painting::icons::{icon_img, ICON_SIZE};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum DrawingMode {
    Brush,
    Highlighter, 
    Rectangle,
    FilledRectangle,
    Circle,
    FilledCircle, 
    Arrow,
    Text, // BUGGED
}

#[derive(Clone, Debug)]
pub enum DrawingType {
    Brush {points: Vec<Pos2>, s: Stroke, end: bool},
    Rectangle {r: Rect, s: Stroke},
    FilledRectangle {r: Rect, s: Stroke},
    Highlighter {r: Rect, s: Stroke},
    Circle {c: Pos2, r: f32, s: Stroke},
    FilledCircle {c: Pos2, r: f32, s: Stroke},
    Arrow {p: Pos2, v: Vec2, s: Stroke},
    Text {p: Pos2, t: String, s: Stroke}, // BUGGED
}

#[derive(Clone)]
pub struct RedoList {
    drawings: VecDeque<DrawingType>,
    capacity: usize,
}

#[allow(unused)]
impl RedoList {
    pub fn new(capacity: usize) -> Self {
        RedoList { drawings: VecDeque::<DrawingType>::with_capacity(capacity), capacity }
    }

    pub fn push(&mut self, d: DrawingType) {
        if self.drawings.len() >= self.capacity {
           self.drawings.pop_front(); 
        }
        self.drawings.push_back(d);
    }

    pub fn pop(&mut self) -> Option<DrawingType> {
        self.drawings.pop_back()
    }

    pub fn capacity(&self) -> usize { 
        self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.drawings.is_empty()
    }
}


impl KrustyGrab {
    pub const REDO_LIST_SIZE: usize = 10;
    pub const BASE_TEXT_SIZE: f32 = 30.0;
    pub const HIGHLIGTHER_FACTOR: u8 = 70;

    // Render the part of head toolbar for the drawing 
    pub fn render_drawing_toolbar(&mut self, ctx: &Context, ui: &mut Ui, frame: &mut eframe::Frame) {
        //setting color and thickness
        let mut color = match ctx.memory(|mem| mem.data.get_temp::<Rgba>(Id::from("Color"))){
            Some(c) => c,
            None => Rgba::from(Color32::GREEN)
        };

        let mut thickness: f32 = match ctx.memory(|mem| mem.data.get_temp(Id::from("Thickness"))) {
            Some(t) => t,
            None => 1.0,
        };
        
        //setting drawing mode
        let drawing_mode = match ctx.memory_mut(|mem| mem.data.get_temp(Id::from("DrawingMode"))) {
            Some(dm) => dm,
            None => DrawingMode::Brush,    
        };

        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            //Brush button
            let mut brush_button = Button::image_and_text(icon_img("pencil", ctx), ICON_SIZE, "")
                .stroke(Stroke::new(1.0,
                Color32::from_rgb(128, 106, 0)))
                .ui(ui)
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text_at_pointer("Brush");

            if drawing_mode == DrawingMode::Brush {
                brush_button = brush_button.highlight();
            }

            if brush_button.clicked() {
                ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("DrawingMode"), DrawingMode::Brush));
                tracing::info!("Pencil selected");
            }
            
            // Highlighter button
            let mut highlighter_button = Button::image_and_text(icon_img("highlighter", ctx), ICON_SIZE, "")
            .stroke(Stroke::new(1.0,
            Color32::from_rgb(128, 106, 0)))
            .ui(ui)
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text_at_pointer("Highlighter");

            if drawing_mode == DrawingMode::Highlighter {
                highlighter_button = highlighter_button.highlight();
            }

            if highlighter_button.clicked() {
                ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("DrawingMode"), DrawingMode::Highlighter));
                tracing::info!("Highlighter selected");
            }
                        
            //[Circle|Filled Circle] button 
            let mut name_icon =  "circle";
            if drawing_mode == DrawingMode::FilledCircle {
                name_icon = "circle_full";
            }
            let circle_button = ui.menu_image_button(icon_img(name_icon, ctx), ICON_SIZE, |ui| {
                if ui
                    .button(RichText::new("Circle").text_style(TextStyle::Body))
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("DrawingMode"), DrawingMode::Circle));
                    tracing::info!("Circle selected");
                    ui.close_menu();
                }
                if ui
                    .button(RichText::new("Circle Filled").text_style(TextStyle::Body))
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("DrawingMode"), DrawingMode::FilledCircle));
                    tracing::info!("Circle Filled selected");
                    ui.close_menu();
                }
            
            }).response
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text_at_pointer("Circles");


            if drawing_mode == DrawingMode::Circle || drawing_mode == DrawingMode::FilledCircle {
                circle_button.highlight();
            }
            
            //[Rectangle|Filled Rectangle] button
            let mut name_icon =  "rect";
            if drawing_mode == DrawingMode::FilledRectangle {
                name_icon = "rect_full";
            }

            let rectangle_button = ui.menu_image_button(icon_img(name_icon, ctx), ICON_SIZE, |ui| {
                if ui
                    .button(RichText::new("Rectangle").text_style(TextStyle::Body))
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("DrawingMode"), DrawingMode::Rectangle));
                    tracing::info!("Rectangle selected");
                    ui.close_menu();
                }
                if ui
                    .button(RichText::new("Rectangle Filled").text_style(TextStyle::Body))
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("DrawingMode"), DrawingMode::FilledRectangle));
                    tracing::info!("Rectangle Filled selected");
                    ui.close_menu();
                }

            }).response.on_hover_cursor(CursorIcon::PointingHand).on_hover_text_at_pointer("Rectangles");

            if drawing_mode == DrawingMode::Rectangle || drawing_mode == DrawingMode::FilledRectangle {
                rectangle_button.highlight();
            }

            //Arrow button
            let mut arrow_button = Button::image_and_text(icon_img("arrow", ctx), ICON_SIZE, "")
                .stroke(Stroke::new(1.0,
                Color32::from_rgb(128, 106, 0)))
                .ui(ui)
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text_at_pointer("Arrow");

            if drawing_mode == DrawingMode::Arrow {
                arrow_button = arrow_button.highlight();
            }

            if arrow_button.clicked() {
                ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("DrawingMode"), DrawingMode::Arrow));
                tracing::info!("Arrow selected");
            }

            //Text button - TODO bugged ATM
            // let mut text_button = Button::image_and_text(icon_img("text", ctx), ICON_SIZE, "")
            //     .stroke(Stroke::new(1.0,
            //     Color32::from_rgb(128, 106, 0)))
            //     .ui(ui)
            //     .on_hover_cursor(CursorIcon::PointingHand)
            //     .on_hover_text_at_pointer("Text");

            // if drawing_mode == DrawingMode::Text {
            //     text_button = text_button.highlight();
            // }

            // if text_button.clicked() {
            //     ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("DrawingMode"), DrawingMode::Text));
            //     tracing::info!("Text selected");
            // }

            //Color picker rendering
            let color_picker = color_edit_button_rgba(ui, &mut color, Alpha::BlendOrAdditive)
                                            .on_hover_cursor(CursorIcon::PointingHand)
                                            .on_hover_text_at_pointer("Change color");
                        
            if ctx.memory(|mem| mem.any_popup_open()) {
                ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("CP_open"), true));
            }
            else {
                ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("CP_open"), false));
            }
            if color_picker.changed() {
                ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("Color"), color));
                tracing::info!("Color changed to {:?}", color);
            }

            //Thickness of the tools
            if DragValue::new(&mut thickness)
                .prefix("Thickness: ")
                .speed(0.1)
                .clamp_range(1.0..=10.0)
                .ui(ui)
                .on_hover_text_at_pointer("Change thickness")
                .changed() {
                ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("Thickness"), thickness));
                tracing::info!("Thickness changed to {:?}", thickness);
            }

            //Undo button
            let render_undo = ctx.memory(|mem| {
                match mem.data.get_temp::<Vec<DrawingType>>(Id::from("Drawing")) {
                    Some(d) => !d.is_empty(),
                    None => false,
                }
            });

            if ui.add_enabled(render_undo, 
                Button::image_and_text(icon_img("undo", ctx), ICON_SIZE, "")
                    .stroke(Stroke::new(1.0, Color32::from_rgb(128, 106, 0)))
            )
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text_at_pointer("Undo")
                .on_disabled_hover_text("No more drawings to undo")
                .clicked() {
                    tracing::info!("Undo selected");
                    ctx.memory_mut(|mem| {
                        match mem.data.get_temp::<Vec<DrawingType>>(Id::from("Drawing")) {
                            Some(mut drawings) => {
                                let last = drawings.pop().expect("Drawings list should contains at least one element at this point");

                                //Retrieve and update Redo list
                                let redo_list = match mem.data.get_temp::<RedoList>(Id::from("Redo_list")){
                                    Some(mut redo) => {
                                        redo.push(last);
                                        redo
                                    },
                                    None => {
                                        let mut redo = RedoList::new(KrustyGrab::REDO_LIST_SIZE);
                                        redo.push(last);
                                        redo
                                    },
                                };
                                
                                mem.data.insert_temp(Id::from("Redo_list"), redo_list);
                                mem.data.insert_temp(Id::from("Drawing"), drawings);
                            },
                            None => {},
                        };
                    });
                }

            //Redo button
            let render_redo = ctx.memory(|mem| {
                match mem.data.get_temp::<RedoList>(Id::from("Redo_list")) {
                    Some(d) => !d.is_empty(),
                    None => false,
                }
            });

            if ui.add_enabled(render_redo, 
                Button::image_and_text(icon_img("redo", ctx), ICON_SIZE, "")
                .stroke(Stroke::new(1.0, Color32::from_rgb(128, 106, 0)))
            )
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text_at_pointer("Redo")
                .on_disabled_hover_text("No more drawings to redo")
                .clicked() {
                    tracing::info!("Redo selected");
                    ctx.memory_mut(|mem| {
                        match mem.data.get_temp::<RedoList>(Id::from("Redo_list")) {
                            Some(mut redo) => {            
                                let last = redo.pop().expect("Redo list should contains at least one element at this point");

                                //Retrieve and update drawings list
                                match mem.data.get_temp::<Vec<DrawingType>>(Id::from("Drawing")) {
                                    Some(mut d) => {
                                        d.push(last);
                                        mem.data.insert_temp(Id::from("Drawing"), d);
                                    },
                                    None => panic!("Drawings list should exists in memory at this point"),
                                }
                                
                                mem.data.insert_temp(Id::from("Redo_list"), redo);
                            },
                            None => {},
                        };
                    });
                }

            //Cut button
            if Button::image_and_text(icon_img("cut", ctx), ICON_SIZE, "")
                .stroke(Stroke::new(1.0,Color32::from_rgb(128, 106, 0)))
                .ui(ui)
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text_at_pointer("Cut screenshot")
                .clicked() {
                    self.set_window_status(krustygrab::WindowStatus::Crop);
                    tracing::info!("Cut screenshot button selected");
            }

            //Save button
            if Button::image_and_text(icon_img("save", ctx), ICON_SIZE, "")
                .stroke(Stroke::new(1.0, Color32::from_rgb(128, 106, 0)))
                .ui(ui)
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text_at_pointer("Save")
                .clicked() {
                    let mut save_path = self.config.save_folder.clone();
                    save_path.push(format!("{}", chrono::Utc::now().format("%Y_%m_%d-%H_%M_%S")));
                    save_path.set_extension(self.config.save_format.to_string());

                    self.save_path_request = Some(save_path);
                    
                    frame.set_visible(false);
                    frame.set_fullscreen(true);
                    self.set_window_status(krustygrab::WindowStatus::Save); 

                    tracing::info!("Save button selected");
                }

            //Save as button
            if Button::image_and_text(icon_img("save_as", ctx), ICON_SIZE, "")
                .stroke(Stroke::new(1.0, Color32::from_rgb(128, 106, 0)))
                .ui(ui)
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text_at_pointer("Save as")
                .clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("PNG", &["png"])
                        .add_filter("JPG", &["jpg"])
                        .add_filter("GIF", &["gif"])
                        .show_save_single_file()
                        .expect("Unable to visualize the file selection window") {
                            self.save_path_request = Some(path);

                            frame.set_visible(false);
                            frame.set_fullscreen(true);
                            self.set_window_status(krustygrab::WindowStatus::Save); 
                        }
                    tracing::info!("Save as button selected");
                }
        });
    }

    /// Manage the canva
    pub fn render_canva(&mut self, ctx: &Context, ui: &mut Ui) {
        let screen = RetainedImage::from_color_image("Screenshot", self.screen.clone().unwrap());

        let mut painter = ctx.layer_painter(LayerId::new(Order::Background, Id::from("Painter")));

        let aspect_ratio = screen.width() as f32 / screen.height() as f32;
        let mut w = ui.available_width();
        let mut h = w / aspect_ratio;
        if h > ui.available_height() {
            h = ui.available_height();
            w = h * aspect_ratio;
        }

        let mut area = ui.available_rect_before_wrap();
        if area.width() > w {
            area.min.x += (area.width() - w) / 2.0;
            area.max.x = area.min.x + w;
        }  
        if area.height() > h {
            area.min.y += (area.height() - h) / 2.0;
            area.max.y = area.min.y + h;
        }
        area.set_width(w);
        area.set_height(h);
        
        let visualization_ratio = screen.width() as f32 / w;

        ctx.memory_mut(|mem| {
            mem.data.insert_temp(Id::from("Visualization_ratio"), visualization_ratio);
            mem.data.insert_temp(Id::from("Visualization_pos"), area.min);
        });
    
        painter.set_clip_rect(area);
        painter.image(screen.texture_id(ctx), area, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);

        let mut drawings = match ctx.memory(|mem| mem.data.get_temp::<Vec<DrawingType>>(Id::from("Drawing"))) {
            Some(v) => v,
            None => Vec::<DrawingType>::new(),
        };

        let color = match ctx.memory(|mem| mem.data.get_temp::<Rgba>(Id::from("Color"))){
            Some(c) => c,
            None => Rgba::from(Color32::GREEN)
        };

        let thickness = match ctx.memory(|mem| mem.data.get_temp::<f32>(Id::from("Thickness"))){
            Some(t) => t,
            None => 1.0
        };

        let stroke = Stroke::new(thickness, color);

        self.show_drawings(ctx, &painter, visualization_ratio);

        let color_picker_open = match ctx.memory(|mem| mem.data.get_temp::<bool>(Id::from("CP_open"))){
            Some(c) => c,
            None => false
        };
        
        let settings_menu_open = match ctx.memory(|mem| mem.data.get_temp::<bool>(Id::from("SM_open"))){
            Some(c) => {
                ctx.memory_mut(|mem| mem.data.remove::<bool>(Id::from("SM_open")));
                c
            },
            None => false
        };

        let drawing_mode = match ctx.memory_mut(|mem| mem.data.get_temp(Id::from("DrawingMode"))) {
            Some(m) => m,
            None => DrawingMode::Brush,    
        };
        
        //TEXT
        let te_window = match ctx.memory_mut(|mem| mem.data.get_temp(Id::from("TE_open"))) {
            Some(te) => te,
            None => false,
        };

        if drawing_mode != DrawingMode::Text {
            let last_was_text = match drawings.last() {
                Some(last) => match last {
                    DrawingType::Text { p: _p, t: _t, s: _s } => true,
                    _ => false,
                },
                None => false,
            };
            if last_was_text {
                ctx.memory_mut(|mem| mem.data.remove::<bool>(Id::from("TE_open")));
                ctx.memory_mut(|mem| mem.data.remove::<bool>(Id::from("TE_continue")));
                ctx.memory_mut(|mem| mem.data.remove::<String>(Id::from("TE_text")));
            }
        }
        else if te_window {
            let mut text: String = match ctx.memory_mut(|mem| mem.data.get_temp(Id::from("TE_text"))) {
                Some(text) => text,
                None => Default::default(),
            };
            let text_pos = match ctx.memory_mut(|mem| mem.data.get_temp(Id::from("Text_pos"))) {
                Some(pos) => pos,
                None => Pos2::ZERO,
            };
            let te_continue = match ctx.memory_mut(|mem| mem.data.get_temp(Id::from("TE_continue"))) {
                Some(c) => c,
                None => false,
            };

            //Computation of text editor position in order to maintain it inside the screen (values are obtained experimentally)
            //TODO add orizzontal check for text box
            let mut te_pos = self.adjust_drawing_pos(ctx, text_pos, true);

            if te_pos.y + 97.0 > area.size()[1] + area.min.y {
                te_pos = te_pos - Vec2::new(0.0, 95.0);
            }
            else {
                te_pos = te_pos + Vec2::new(0.0, 20.0);
            }
            
            Window::new("")
            .fixed_pos(te_pos)
            .show(ctx, |ui| {
                let text_box = ui.text_edit_singleline(&mut text);

                if text_box.lost_focus() {
                    ctx.memory_mut(|mem| mem.data.remove::<bool>(Id::from("TE_open")));
                    ctx.memory_mut(|mem| mem.data.remove::<bool>(Id::from("TE_continue")));
                    ctx.memory_mut(|mem| mem.data.remove::<String>(Id::from("TE_text")));
                }
                else if text_box.changed() {
                    ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("TE_text"), text.clone()));
                    if te_continue {
                        drawings.pop().unwrap_or(DrawingType::Text { p: text_pos, t: text.clone(), s: stroke });
                    }
                    else {
                        ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("TE_continue"), true));
                    }

                    drawings.push(DrawingType::Text { p: text_pos, t: text, s: stroke });
                    ctx.memory_mut(|mem| {
                        mem.data.insert_temp(Id::from("Drawing"), drawings.clone());
                        mem.data.remove::<RedoList>(Id::from("Redo_list"));
                    });
                }
            });
        }

        match ctx.pointer_hover_pos() {
            Some(mut mouse) => {
                let hover_rect = match ctx.memory(|mem| mem.data.get_temp(Id::from("hover_rect"))){
                    Some(r) => r,
                    None => area,
                };

                //The interaction with the canvas is only sensed when color picker and settings menu are closed and when not interacting with the configuration window (when it is open)
                if hover_rect.contains(mouse) && !color_picker_open && !settings_menu_open 
                    && !(self.config_window && ctx.layer_id_at(mouse).unwrap_or(LayerId { order: Order::Background, id: Id::from("Configuration_check") }).order == Order::Middle) {
                    //Rescaling mouse position on the size of the screen to keep the position fixed on the canva
                    mouse = self.adjust_drawing_pos(ctx, mouse, false);

                    if ctx.input(|i| i.pointer.primary_clicked()) && !te_window{
                        match drawing_mode {
                            DrawingMode::Text => {
                                ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("TE_open"), true));
                                ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("TE_continue"), false));
                                ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("Text_pos"), mouse));
                            },
                            _ => {},
                        }
                    }

                    // sense clicking for drawing 
                    if ctx.input(|i| i.pointer.primary_down()) {
                        //If the interaction is no longer with the configuration window it is closed without saving the changed values
                        if self.config_window {
                            self.config_window = false;
                        }

                        let mut p0 = match ctx.memory(|mem| mem.data.get_temp(Id::from("initial_pos"))) {
                            Some(p) => p,
                            None => {
                                let mut starting_pos = ctx.input(|i| i.pointer.press_origin()).unwrap();

                                //Resize initial position (also previous position)
                                starting_pos = self.adjust_drawing_pos(ctx, starting_pos, false);
                                ctx.memory_mut(|mem| mem.data.insert_temp(Id::from("initial_pos"), starting_pos));
                                
                                starting_pos
                            }
                        };
                        //print of the drawings while dragging on the screen
                        match drawing_mode {
                            DrawingMode::Brush => {
                                match drawings.last_mut() {
                                    Some(d) => {
                                        match d {
                                            DrawingType::Brush { points, s: _s, end } => {
                                                if !*end {
                                                    points.push(mouse);
                                                }
                                                else {
                                                    let mut ps = Vec::new();
                                                    ps.push(mouse);
                                                    drawings.push(DrawingType::Brush { points: ps, s: stroke, end: false });
                                                }
                                            },
                                            _ => {
                                                let mut ps = Vec::new();
                                                ps.push(mouse);
                                                drawings.push(DrawingType::Brush { points: ps, s: stroke, end: false });
                                            },
                                        }
                                    },
                                    None => {
                                        let mut ps = Vec::new();
                                        ps.push(mouse);
                                        drawings.push(DrawingType::Brush { points: ps, s: stroke, end: false });
                                    },
                                };
                                ctx.memory_mut(|mem| {
                                    mem.data.insert_temp(Id::from("previous_pos"), mouse);
                                    mem.data.insert_temp(Id::from("Drawing"), drawings.clone());
                                });
                            },
                            DrawingMode::Rectangle => {
                                if mouse.x < p0.x {
                                    (mouse.x, p0.x) = (p0.x, mouse.x);
                                }
                                if mouse.y < p0.y {
                                    (mouse.y, p0.y) = (p0.y, mouse.y);
                                }
                                let to_paint_border = Rect::from_min_max(self.adjust_drawing_pos(ctx, p0, true), self.adjust_drawing_pos(ctx, mouse, true));
                                painter.rect_stroke(to_paint_border, 0.0, stroke);
                                // tracing::info!("Painted rect with p0 {:?}, mouse {:?}, stroke {:?}", p0, mouse, stroke);
                            },
                            DrawingMode::Highlighter => {
                                let mut minx = p0.x;
                                let mut maxx = mouse.x;
                                if mouse.x < p0.x {
                                    (minx, maxx) = (maxx, minx);
                                }
                                let from_here = Pos2::new(minx , p0.y - stroke.width*5.);
                                let to_there = Pos2::new(maxx , p0.y + stroke.width*5.);
                                let mut color = stroke.color.clone();
                                color[3] = color.a() / Self::HIGHLIGTHER_FACTOR ;
                                let to_paint_border = Rect::from_min_max(self.adjust_drawing_pos(ctx, from_here, true), self.adjust_drawing_pos(ctx, to_there, true));
                                painter.rect_filled(to_paint_border, 0.0, color );
                                // tracing::info!("Painted highlight with p0 {:?}, mouse {:?}, stroke {:?}", p0, mouse, stroke);
                            },
                            DrawingMode::FilledRectangle => {
                                if mouse.x < p0.x {
                                    (mouse.x, p0.x) = (p0.x, mouse.x);
                                }
                                if mouse.y < p0.y {
                                    (mouse.y, p0.y) = (p0.y, mouse.y);
                                }
                                let to_paint_border = Rect::from_min_max(self.adjust_drawing_pos(ctx, p0, true), self.adjust_drawing_pos(ctx, mouse, true));
                                painter.rect_filled(to_paint_border, 0.0, stroke.color);
                                // tracing::info!("Painted fillrect with p0 {:?}, mouse {:?}, stroke {:?}", p0, mouse, stroke);
                            },
                            DrawingMode::Circle => {
                                // Constructed fixing the center with the starting position and the radius is considered between the start and the cursor current position
                                // let radius = mouse.distance(p0) / visualization_ratio;
                                // let mut center =  p0;
                                // Constructed with one side on the starting point and the opposite on the cursor
                                let radius = mouse.distance(p0) / (visualization_ratio*2.0);
                                let mut center =  p0 + Vec2::new((mouse.x-p0.x)/2.0, (mouse.y-p0.y)/2.0);
                                center = self.adjust_drawing_pos(ctx, center, true);

                                painter.circle_stroke(center, radius, stroke);
                                // tracing::info!("Painted circle with center {:?}, radius {:?}, stroke {:?}", center, radius, stroke);
                            },
                            DrawingMode::FilledCircle => {
                                let radius = mouse.distance(p0) / (visualization_ratio*2.0);
                                let mut center =  p0 + Vec2::new((mouse.x-p0.x)/2.0, (mouse.y-p0.y)/2.0);
                                center = self.adjust_drawing_pos(ctx, center, true);

                                painter.circle_filled(center, radius, stroke.color);
                                // tracing::info!("Painted circle with center {:?}, radius {:?}, stroke {:?}", center, radius, stroke);
                            },
                            DrawingMode::Arrow => {
                                let origin = self.adjust_drawing_pos(ctx, p0, true);
                                let direction = Vec2::new(mouse.x - p0.x, mouse.y - p0.y) / visualization_ratio;
                                painter.arrow(origin, direction, stroke);
                                // tracing::info!("Painted arrow with origin {:?}, vector {:?}, stroke {:?}", p0, Vec2::new(mouse.x - p0.x, mouse.y - p0.y), stroke);
                            },
                            _ => {},
                        }
                        ctx.memory_mut(|mem| {
                            mem.data.insert_temp(Id::from("mouse_pos"), mouse);
                            mem.data.insert_temp(Id::from("hover_rect"), area);
                        });
                    }

                    // save the drawing after releasing 
                    if ctx.input(|i| i.pointer.primary_released()) {
                        // tracing::info!("Pointer primary released");
                        match ctx.memory(|mem| mem.data.get_temp::<Pos2>(Id::from("initial_pos"))) {
                            Some(mut p0) => {
                                match drawing_mode {
                                    DrawingMode::Brush => {
                                        match drawings.last_mut() {
                                            Some(d) => match d {
                                                DrawingType::Brush { points, s: _s, end } => {
                                                    points.push(mouse);
                                                    *end = true;
                                                },
                                                _ => {},
                                            }
                                            _ => {},
                                        }
                                        ctx.memory_mut(|mem| mem.data.remove::<Pos2>(Id::from("previous_pos")));
                                    },
                                    DrawingMode::Highlighter => {
                                        let mut minx = p0.x;
                                        let mut maxx = mouse.x;
                                        if mouse.x < p0.x {
                                            (minx, maxx) = (maxx, minx);
                                        }
                                        let from_here = Pos2::new(minx , p0.y - stroke.width*5.);
                                        let to_there = Pos2::new(maxx , p0.y + stroke.width*5.);
                                        drawings.push(DrawingType::Highlighter{ r: Rect::from_min_max(from_here, to_there), s: stroke });
                                        tracing::info!("Added highlight with p0 {:?}, mouse {:?}, stroke {:?}", p0, mouse, stroke);
                                    },
                                    DrawingMode::Rectangle => {
                                        if mouse.x < p0.x {
                                            (mouse.x, p0.x) = (p0.x, mouse.x);
                                        }
                                        if mouse.y < p0.y {
                                            (mouse.y, p0.y) = (p0.y, mouse.y);
                                        }
        
                                        drawings.push(DrawingType::Rectangle { r: Rect::from_min_max(p0, mouse), s: stroke });
                                        tracing::info!("Added rect with p0 {:?}, mouse {:?}, stroke {:?}", p0, mouse, stroke);
                                    },
                                    DrawingMode::FilledRectangle => {
                                        if mouse.x < p0.x {
                                            (mouse.x, p0.x) = (p0.x, mouse.x);
                                        }
                                        if mouse.y < p0.y {
                                            (mouse.y, p0.y) = (p0.y, mouse.y);
                                        }
        
                                        drawings.push(DrawingType::FilledRectangle { r: Rect::from_min_max(p0, mouse), s: stroke });
                                        tracing::info!("Added rect with p0 {:?}, mouse {:?}, stroke {:?}", p0, mouse, stroke);
                                    },
                                    DrawingMode::Circle => {
                                        // Constructed fixing the center with the starting position and the radius is considered between the start and the cursor current position
                                        // let radius = mouse.distance(p0);
                                        // let center = p0;
                                        // Constructed with one side on the starting point and the opposite on the cursor
                                        let radius = mouse.distance(p0) / 2.0;
                                        let center =  p0 + Vec2::new((mouse.x-p0.x)/2.0, (mouse.y-p0.y)/2.0);

                                        drawings.push(DrawingType::Circle { c: center, r: radius, s: stroke });
                                        tracing::info!("Added circle with center {:?}, radius {:?}, stroke {:?}", center, radius, stroke);
                                    },
                                    DrawingMode::FilledCircle => {
                                        // Constructed fixing the center with the starting position and the radius is considered between the start and the cursor current position
                                        // let radius = mouse.distance(p0);
                                        // let center = p0;
                                        // Constructed with one side on the starting point and the opposite on the cursor
                                        let radius = mouse.distance(p0) / 2.0;
                                        let center =  p0 + Vec2::new((mouse.x-p0.x)/2.0, (mouse.y-p0.y)/2.0);

                                        drawings.push(DrawingType::FilledCircle { c: center, r: radius, s: stroke });
                                        tracing::info!("Added circle with center {:?}, radius {:?}, stroke {:?}", center, radius, stroke);
                                    },
                                    DrawingMode::Arrow => {
                                        drawings.push(DrawingType::Arrow { p: p0, v: Vec2::new(mouse.x - p0.x, mouse.y - p0.y), s: stroke });
                                        tracing::info!("Added arrow with origin {:?}, vector {:?}, stroke {:?}", p0, Vec2::new(mouse.x - p0.x, mouse.y - p0.y), stroke);
                                    },
                                    _ => {},
                                }
        
                                ctx.memory_mut(|mem| {
                                    mem.data.insert_temp(Id::from("Drawing"), drawings.clone());
                                    mem.data.remove::<Pos2>(Id::from("initial_pos"));
                                    mem.data.remove::<Rect>(Id::from("hover_rect"));
                                    mem.data.remove::<RedoList>(Id::from("Redo_list"));
                                });
                            },
                            None => {},
                        };

                        
                    }
                }
                else {
                    let primary_up = !ctx.input(|i| i.pointer.primary_down());

                    if drawing_mode == DrawingMode::Brush || primary_up {
                        ctx.memory_mut(|mem| {
                            if drawing_mode == DrawingMode::Brush {
                                match drawings.last_mut() {
                                    Some(d) => match d {
                                        DrawingType::Brush { points: _points, s: _s, end } => {
                                            *end = true;
                                        },
                                        _ => {},
                                    }
                                    _ => {},
                                }
                                mem.data.insert_temp(Id::from("Drawing"), drawings.clone());

                                if !primary_up {
                                    mem.data.insert_temp(Id::from("previous_pos"), mouse);
                                }
                                else {
                                    mem.data.remove::<Pos2>(Id::from("previous_pos"));
                                }
                            }
                            
                            mem.data.remove::<Pos2>(Id::from("initial_pos"));
                            mem.data.remove::<Rect>(Id::from("hover_rect"));
                        });
                    }
                }
            },
            None => {},
        }
    }

    ///Scale the drawing position with the actual image reduction ratio in order to maintain the drawing in position after a rescale of the window that visualize it
    fn adjust_drawing_pos(&mut self, ctx: &Context, pos: Pos2, render: bool) -> Pos2{
        let adjusted_pos: Pos2;
        let v_ratio: f32 = ctx.memory(|mem| {
            match mem.data.get_temp::<f32>(Id::from("Visualization_ratio")){
                Some(ratio) => ratio,
                None => f32::default(),
            }
        });
        let v_pos: Pos2 = ctx.memory(|mem| {
            match mem.data.get_temp::<Pos2>(Id::from("Visualization_pos")){
                Some(ratio) => ratio,
                None => Pos2::default(),
            }
        });
        let area_min = match self.get_selected_area() {
            Some(area) => area.min,
            None => pos2(0., 0.),
        };
        
        if render {
            adjusted_pos = pos2(((pos.x - area_min.x) / v_ratio) + v_pos.x, ((pos.y - area_min.y) / v_ratio) + v_pos.y);
        } else {
            adjusted_pos = pos2((pos.x - v_pos.x) * v_ratio + area_min.x, (pos.y - v_pos.y) * v_ratio + area_min.y);
        }
        adjusted_pos
    }

    ///Shows the saved drawings
    pub fn show_drawings(&mut self, ctx: &Context, painter: &Painter, visualization_ratio: f32) {
        let drawings = match ctx.memory(|mem| mem.data.get_temp::<Vec<DrawingType>>(Id::from("Drawing"))) {
            Some(v) => v,
            None => Vec::<DrawingType>::new(),
        };

        //Visualization of saved drawings
        for d in &drawings {
            match d.clone() {
                DrawingType::Brush { points, mut s, .. } => {
                    //Change the thickness of the strok according to the window size in order to obtain a static dimension among the visualization 
                    s.width /= visualization_ratio;
                    for i in 1..points.len() {
                        let to_paint = [self.adjust_drawing_pos(ctx, points[i], true), self.adjust_drawing_pos(ctx, points[i-1], true)];
                        painter.line_segment(to_paint, s);
                    }
                },
                DrawingType::Highlighter { r, s } => {
                    let to_paint = Rect::from_min_max(self.adjust_drawing_pos(ctx, r.min, true), self.adjust_drawing_pos(ctx, r.max, true));
                    let mut color = s.color.clone();
                    color[3] = color.a() / Self::HIGHLIGTHER_FACTOR ;
                    painter.rect_filled(to_paint, 0.0, color);
                },
                DrawingType::Rectangle { r, s } => {
                    let to_paint = Rect::from_min_max(self.adjust_drawing_pos(ctx, r.min, true), self.adjust_drawing_pos(ctx, r.max, true));
                    painter.rect_stroke(to_paint, 0.0, s);
                },
                DrawingType::FilledRectangle { r, s } => {
                    let to_paint = Rect::from_min_max(self.adjust_drawing_pos(ctx, r.min, true), self.adjust_drawing_pos(ctx, r.max, true));
                    painter.rect_filled(to_paint, 0.0, s.color);
                },
                DrawingType::Circle { mut c, mut r, s } => {
                    c = self.adjust_drawing_pos(ctx, c, true);
                    r /= visualization_ratio;
                    painter.circle_stroke(c, r, s);
                },
                DrawingType::FilledCircle { mut c, mut r, s } => {
                    c = self.adjust_drawing_pos(ctx, c, true);
                    r /= visualization_ratio;
                    painter.circle_filled(c, r, s.color);
                },
                DrawingType::Arrow { p, v, s } => {
                    let origin = self.adjust_drawing_pos(ctx, p, true);
                    let direction = v / visualization_ratio;
                    painter.arrow(origin, direction, s);
                },
                DrawingType::Text { mut p , t , s} => {
                    p = self.adjust_drawing_pos(ctx, p, true);
                    //Regolazione del font in base alla dimensione della finestra di render
                    let font_size = (KrustyGrab::BASE_TEXT_SIZE * s.width) / visualization_ratio; //TODO Giggling text, UNFIXABLE! Library bug! 
                    // println!("Size: {font_size:?}");
                    painter.text(p, Align2::LEFT_CENTER, t, FontId::new(font_size, egui::FontFamily::Proportional), s.color);
                },
            }
        }
    }

    ///Shows the saved drawings in the select mode (fullscreen)
    pub fn show_drawings_in_select(&mut self, ctx: &Context, painter: &Painter) {
        let drawings = match ctx.memory(|mem| mem.data.get_temp::<Vec<DrawingType>>(Id::from("Drawing"))) {
            Some(v) => v,
            None => Vec::<DrawingType>::new(),
        };

        //Visualization of saved drawings
        for d in &drawings {
            match d.clone() {
                DrawingType::Brush { points, s, .. } => {
                    for i in 1..points.len() {
                        let to_paint = [points[i], points[i-1]];
                        painter.line_segment(to_paint, s);
                    }
                },
                DrawingType::Highlighter { r, s } => {
                    let mut color = s.color.clone();
                    color[3] = color.a() / Self::HIGHLIGTHER_FACTOR ;
                    painter.rect_filled(r, 0.0, color);
                },
                DrawingType::Rectangle { r, s } => {
                    painter.rect_stroke(r, 0.0, s);
                },
                DrawingType::FilledRectangle { r, s } => {
                    painter.rect_filled(r, 0.0, s.color);
                },
                DrawingType::Circle { c, r, s } => {
                    painter.circle_stroke(c, r, s);
                },
                DrawingType::FilledCircle { c, r, s } => {
                    painter.circle_filled(c, r, s.color);
                },
                DrawingType::Arrow { p, v, s } => {
                    painter.arrow(p, v, s);
                },
                DrawingType::Text { p , t , s} => {
                    //Regolazione del font in base alla dimensione della finestra di render
                    let font_size = KrustyGrab::BASE_TEXT_SIZE * s.width;
                    painter.text(p, Align2::LEFT_CENTER, t, FontId::new(font_size, egui::FontFamily::Proportional), s.color);
                },
            }
        }
    }
}
