use std::vec;
use futures::TryFutureExt;
use image::GenericImageView;
use eframe::egui::{ColorImage, TextureHandle, Color32, RichText, Vec2};
use eframe::egui;
mod server;
mod env1;
mod user;
use tokio::task::JoinHandle;
use futures::executor::block_on;

#[derive(Clone, Debug)]
pub struct CourseData {
    pub name: String,
    pub latest_assignment: Option<String>,
    pub grade: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ScheduleEntry {
    pub day_of_week: u8, // 0-6 (Monday-Sunday)
    pub hour: u8,        // 0-23
    pub minute: u8,      // 0-59
    pub course: String,
    pub assignment: String,
}
pub fn gradething(grade: i32) -> String {
    let result: String;
    if grade >= 90 {
        result = "A".to_string();
    } else if grade >= 80 {
        result = "B".to_string();
    } else if grade >= 60 {
        result = "C".to_string();
    } else if grade >= 50 {
        result = "D".to_string();
    } else if grade < 50 {
        result = "F".to_string();
    } else if grade == "N/A".parse::<i32>().unwrap_or(-1) {
        result = "N/A".to_string();
  
    } else {
        result = "N/A".to_string();
    };
    
    return result
}
#[tokio::main]
async fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        ..Default::default()
    };
  
    println!("{:?}", user::GetUser());

    eframe::run_native(
        "Login",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

struct MyApp {
    state: String,
    courses: Vec<CourseData>,
    course_tabs: Vec<usize>,
    pending: Option<JoinHandle<()>>,
    texture: Option<TextureHandle>,
    current_user_id: String,
    schedule_entries: Vec<ScheduleEntry>,

    schedule_form_day: usize,
    schedule_form_hour: String,
    schedule_form_minute: String,
    schedule_form_course: usize,
    schedule_form_assignment: String,

    rx: Option<tokio::sync::mpsc::Receiver<MyApp>>, 
    tx: Option<tokio::sync::mpsc::Sender<MyApp>>,   
}

impl MyApp {
    fn load_image(ctx: &egui::Context, path: &str) -> TextureHandle {
     
        let image = image::open(path).expect("Failed to load image");
        let size = [image.width() as usize, image.height() as usize];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
    
        let color_image = ColorImage::from_rgba_unmultiplied(
            size,
            pixels.as_slice(),
        );

        ctx.load_texture(
            "my-image",
            color_image,
            Default::default(),
        )
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            state: "login".to_string(),
            courses: Vec::new(),
            course_tabs: Vec::new(),
            pending: None,
            texture: None,
            current_user_id: String::new(),
            schedule_entries: Vec::new(),
            schedule_form_day: 0,
            schedule_form_hour: "09".to_string(),
            schedule_form_minute: "00".to_string(),
            schedule_form_course: 0,
            schedule_form_assignment: String::new(),
            rx: None,
            tx: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
         
        let panel_rect = ui.max_rect();
        let painter = ui.painter();
        let top_color = Color32::from_rgb(150, 255, 180);
        let bottom_color = Color32::from_rgb(180, 220, 255);
        let steps = 100;
        let step_height = panel_rect.height() / steps as f32;
        for i in 0..steps {
            let t = i as f32 / (steps - 1) as f32;
            let color = Color32::from_rgb(
                (top_color.r() as f32 * (1.0 - t) + bottom_color.r() as f32 * t) as u8,
                (top_color.g() as f32 * (1.0 - t) + bottom_color.g() as f32 * t) as u8,
                (top_color.b() as f32 * (1.0 - t) + bottom_color.b() as f32 * t) as u8,
            );
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(panel_rect.left(), panel_rect.top() + i as f32 * step_height),
                    egui::pos2(panel_rect.right(), panel_rect.top() + (i + 1) as f32 * step_height),
                ),
                0.0,
                color,
            );
        }

        if self.state == "login" {
         

            ui.vertical_centered(|ui| {
        ui.add_space(100.0);

   
        ui.heading(
            RichText::new("Welcome")
                .size(40.0)
                .strong()
                .color(Color32::WHITE),
        );
        ui.add_space(20.0);

      
        ui.label(
            RichText::new("Login to continue")
                .size(18.0)
                .color(Color32::WHITE),
        );
        ui.add_space(40.0);

     
        let login_button = egui::Button::new(
            RichText::new("Login")
                .size(20.0)
                .color(Color32::WHITE),
        )
        .fill(Color32::from_rgb(70, 100, 200))
        .min_size(Vec2::new(200.0, 50.0));

     if ui.add(login_button).clicked() {
   self.state = "loading".to_string();
   
 if self.tx.is_none() {
    let (tx, rx) = tokio::sync::mpsc::channel(1);
    self.tx = Some(tx.clone());
    self.rx = Some(rx);
} 
    let tx = self.tx.as_ref().unwrap().clone();
    self.pending = Some(tokio::spawn(async move {
       let data = server::func().await.unwrap();
     
       let _ = tx.send(data).await;
      
    }));
   
 
}



        ui.add_space(20.0);

      
      
    });
   

      
                
        }
     if self.state == "loading" {
   

   
        ui.heading(
            RichText::new("Authenticating (Must Have Google Classroom)...")
                .size(24.0)
                .strong()
                .color(Color32::BLACK),
        );
    



    
   
}
    if let Some(rx) = &mut self.rx {
     match rx.try_recv() {
        Ok(result) => {
            println!("Data received from async task!");
            self.courses = result.courses;
            self.course_tabs = vec![0; self.courses.len()];
            self.state = "Main".to_string();
            self.pending = None;
            ctx.request_repaint();
        }
        Err(_) => {
            // No data buddy
        }
    }
}
            if self.state == "Schedule" {
             
                ui.horizontal(|ui| {
                    let back_button = egui::Button::new(
                        RichText::new("← Back to Main")
                            .size(16.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(70, 100, 200))
                    .min_size(Vec2::new(150.0, 45.0));
                    
                    if ui.add(back_button).clicked() {
                        self.state = "Main".to_string();
                    }
                });
                ui.add_space(20.0);

                ui.heading(
                    RichText::new("📅 Schedule Builder")
                        .size(32.0)
                        .strong()
                        .color(Color32::from_rgb(40, 40, 40)),
                );
                ui.add_space(15.0);

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                     
                        ui.group(|ui| {
                            ui.set_width(ui.available_width());
                            ui.heading(
                                RichText::new("Create Schedule Entry")
                                    .size(18.0)
                                    .color(Color32::from_rgb(30, 80, 150)),
                            );
                            ui.add_space(10.0);

                            ui.label(RichText::new("Day of Week:").size(14.0).strong());
                            let days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
                            ui.horizontal(|ui| {
                                for (idx, day) in days.iter().enumerate() {
                                    if ui.selectable_label(self.schedule_form_day == idx, day.to_string()).clicked() {
                                        self.schedule_form_day = idx;
                                    }
                                }
                            });
                            ui.add_space(10.0);

                     
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Time:").size(14.0).strong());
                                ui.text_edit_singleline(&mut self.schedule_form_hour);
                                ui.label(":");
                                ui.text_edit_singleline(&mut self.schedule_form_minute);
                            });
                            ui.add_space(10.0);

                       
                            ui.label(RichText::new("Course:").size(14.0).strong());
                            if self.courses.is_empty() {
                                ui.label("No courses available");
                            } else {
                                ui.horizontal(|ui| {
                                    for (idx, course) in self.courses.iter().enumerate() {
                                        if ui.selectable_label(self.schedule_form_course == idx, &course.name).clicked() {
                                            self.schedule_form_course = idx;
                                        }
                                    }
                                });
                            }
                            ui.add_space(10.0);

                          
                            ui.label(RichText::new("Assignment Name:").size(14.0).strong());
                            ui.text_edit_singleline(&mut self.schedule_form_assignment);
                            ui.add_space(15.0);

                         
                            let add_button = egui::Button::new(
                                RichText::new("➕ Add to Schedule")
                                    .size(14.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(Color32::from_rgb(46, 204, 113))
                            .min_size(Vec2::new(150.0, 45.0));

                            if ui.add(add_button).clicked() {
                                if !self.courses.is_empty() && !self.schedule_form_assignment.is_empty() {
                                    if let (Ok(hour), Ok(minute)) = (self.schedule_form_hour.parse::<u8>(), self.schedule_form_minute.parse::<u8>()) {
                                        if hour < 24 && minute < 60 {
                                            let course_name = self.courses[self.schedule_form_course].name.clone();
                                            let entry = ScheduleEntry {
                                                day_of_week: self.schedule_form_day as u8,
                                                hour,
                                                minute,
                                                course: course_name,
                                                assignment: self.schedule_form_assignment.clone(),
                                            };
                                            // Save to database
                                            if let Err(e) = server::save_schedule_entry(&self.current_user_id, &entry) {
                                                eprintln!("Failed to save schedule entry: {}", e);
                                            }
                                            self.schedule_entries.push(entry);
                                            self.schedule_form_assignment.clear();
                                            self.schedule_form_hour = "09".to_string();
                                            self.schedule_form_minute = "00".to_string();
                                        }
                                    }
                                }
                            }
                        });
                        ui.add_space(25.0);

                        ui.heading(
                            RichText::new("Weekly Schedule")
                                .size(18.0)
                                .color(Color32::from_rgb(30, 80, 150)),
                        );
                        ui.add_space(15.0);

                        let days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
                        for (day_idx, day_name) in days.iter().enumerate() {
                            let day_entries: Vec<_> = self.schedule_entries
                                .iter()
                                .filter(|e| e.day_of_week as usize == day_idx)
                                .collect();

                            if !day_entries.is_empty() {
                                ui.group(|ui| {
                                    ui.set_width(ui.available_width());
                                    ui.heading(
                                        RichText::new(*day_name)
                                            .size(16.0)
                                            .strong()
                                            .color(Color32::from_rgb(30, 120, 60)),
                                    );

                                    for entry in day_entries {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                RichText::new(format!("⏰ {:02}:{:02}", entry.hour, entry.minute))
                                                    .size(13.0)
                                                    .strong(),
                                            );
                                            ui.separator();
                                            ui.label(
                                                RichText::new(&entry.course)
                                                    .size(13.0)
                                                    .color(Color32::from_rgb(52, 152, 219)),
                                            );
                                            ui.separator();
                                            ui.label(
                                                RichText::new(&entry.assignment)
                                                    .size(13.0)
                                                    .color(Color32::from_rgb(80, 80, 80)),
                                            );
                                        });
                                    }
                                });
                                ui.add_space(10.0);
                            }
                        }
                    });
            }
            if self.state == "Grades" {
           
                ui.horizontal(|ui| {
                    let back_button = egui::Button::new(
                        RichText::new("← Back to Main")
                            .size(16.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(70, 100, 200))
                    .min_size(Vec2::new(150.0, 45.0));
                    
                    if ui.add(back_button).clicked() {
                        self.state = "Main".to_string();
                    }
                });
                ui.add_space(20.0);

      
                ui.heading(
                    RichText::new("Your Grades")
                        .size(32.0)
                        .strong()
                        .color(Color32::from_rgb(40, 40, 40)),
                );
                ui.add_space(15.0);

               
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                    
                        for course in self.courses.iter() {
                  
                    let course_rect = ui.available_rect_before_wrap();
                    let painter = ui.painter();
                    painter.rect_filled(
                        egui::Rect::from_min_max(
                            course_rect.left_top(),
                            egui::pos2(course_rect.right(), course_rect.top() + 50.0),
                        ),
                        8.0,
                        Color32::from_rgba_unmultiplied(200, 220, 240, 200),
                    );

                    ui.vertical(|ui| {
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.heading(
                                RichText::new(&course.name)
                                    .size(22.0)
                                    .strong()
                                    .color(Color32::from_rgb(30, 60, 120)),
                            );
                        });
                        ui.add_space(10.0);
                    });
                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        ui.vertical(|ui| {
                       
                            ui.label(
                                RichText::new("Current Grade:")
                                    .size(15.0)
                                    .color(Color32::from_rgb(80, 80, 80)),
                            );
                            
                           
                            let grade_text = course.grade.clone().unwrap_or_else(|| "N/A".to_string());
                            let grade_result = gradething(grade_text.parse::<i32>().unwrap_or(-1));
                            let grade_color = match grade_text.as_str() {
                                "A" | "A+" => Color32::from_rgb(34, 177, 76),
                                "B" | "B+" => Color32::from_rgb(52, 152, 219),
                                "C" | "C+" => Color32::from_rgb(241, 196, 15),
                                "D" | "D+" => Color32::from_rgb(230, 126, 34),
                                "F" => Color32::from_rgb(231, 76, 60),
                                _ => Color32::from_rgb(149, 165, 166),
                            };
                            
                            let grade_button = egui::Button::new(
                                RichText::new(&grade_text)
                                    .size(32.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(grade_color)
                            .min_size(Vec2::new(110.0, 90.0));
                            
                            ui.add(grade_button);
                        });
                        ui.add_space(40.0);
                    });
                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(20.0);
                    }
                    });
            }
            if self.state == "Main" {
           
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() / 2.0 - 160.0);
                    
                    let schedule_button = egui::Button::new(
                        RichText::new("📅 Schedule")
                            .size(16.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(46, 204, 113))
                    .min_size(Vec2::new(130.0, 45.0));
                    
                    if ui.add(schedule_button).clicked() {
                        self.state = "Schedule".to_string();
                    }
                    
                    ui.add_space(15.0);
                    
                    let grades_button = egui::Button::new(
                        RichText::new("📊 Grades")
                            .size(16.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(231, 76, 60))
                    .min_size(Vec2::new(130.0, 45.0));
                    
                    if ui.add(grades_button).clicked() {
                        self.state = "Grades".to_string();
                    }
                });
                ui.add_space(20.0);

              
                ui.heading(
                    RichText::new("Your Courses")
                        .size(28.0)
                        .strong()
                        .color(Color32::from_rgb(40, 40, 40)),
                );
                ui.add_space(15.0);

            
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                      
                        for (i, course) in self.courses.iter().enumerate() {
                            ui.collapsing(
                                RichText::new(&course.name)
                                    .size(18.0)
                                    .strong()
                                    .color(Color32::from_rgb(30, 80, 150)),
                                |ui| {
                            ui.add_space(12.0);
                       
                            ui.horizontal(|ui| {
                                let tab1_button = egui::Button::new(
                                    RichText::new("📝 Latest Assignment")
                                        .size(14.0)
                                        .strong()
                                        .color(if self.course_tabs[i] == 0 { Color32::WHITE } else { Color32::from_rgb(60, 60, 60) }),
                                )
                                .fill(if self.course_tabs[i] == 0 { Color32::from_rgb(52, 152, 219) } else { Color32::from_rgba_unmultiplied(220, 220, 220, 200) })
                                .min_size(Vec2::new(160.0, 42.0));
                                
                                if ui.add(tab1_button).clicked() {
                                    self.course_tabs[i] = 0;
                                }
                                
                                ui.add_space(12.0);
                                
                                let tab2_button = egui::Button::new(
                                    RichText::new("⭐ Grade")
                                        .size(14.0)
                                        .strong()
                                        .color(if self.course_tabs[i] == 1 { Color32::WHITE } else { Color32::from_rgb(60, 60, 60) }),
                                )
                                .fill(if self.course_tabs[i] == 1 { Color32::from_rgb(155, 89, 182) } else { Color32::from_rgba_unmultiplied(220, 220, 220, 200) })
                                .min_size(Vec2::new(160.0, 42.0));
                                
                                if ui.add(tab2_button).clicked() {
                                    self.course_tabs[i] = 1;
                                }
                            });
                            ui.add_space(12.0);
                            ui.separator();
                            ui.add_space(12.0);
                            
                            match self.course_tabs[i] {
                                0 => {
                                    ui.label(
                                        RichText::new(course.latest_assignment.clone().unwrap_or_else(|| "No assignments".to_string()))
                                            .size(14.0)
                                            .color(Color32::from_rgb(50, 50, 50)),
                                    );
                                }
                                1 => {
                                    ui.label(
                                        RichText::new(course.grade.clone().unwrap_or_else(|| "N/A".to_string()))
                                            .size(14.0)
                                            .color(Color32::from_rgb(50, 50, 50)),
                                    );
                                }
                                _ => {}
                            }
                            ui.add_space(12.0);
                        }
                        );
                        ui.add_space(15.0);
                    }
                    });
            }
      
        });
    }
}
