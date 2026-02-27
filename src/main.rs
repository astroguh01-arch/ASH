use std::vec;
use image::GenericImageView;
use eframe::egui::{ColorImage, TextureHandle};
use eframe::egui;
mod server;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        ..Default::default()
    };
  

    eframe::run_native(
        "Login",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

struct MyApp {
  state : String,
  Info : Vec<String>,
  texture: Option<TextureHandle>,
}

impl MyApp {
    fn load_image(ctx: &egui::Context, path: &str) -> TextureHandle {
        // Load image using image crate
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
           Info: vec![],
           texture: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.state == "login" {
                ui.heading("Login");
            
 use egui::{Color32, Frame, RichText, Vec2};

 
    let panel_rect = ui.max_rect();
    let painter = ui.painter();

   
    let top_color = Color32::from_rgb(150, 255, 180); 
    let bottom_color = Color32::from_rgb(180, 220, 255);

    
    let steps = 100; 
    let step_height = panel_rect.height() / steps as f32;
    for i in 0..steps {
        let t = i as f32 / (steps - 1) as f32; // 0.0 → 1.0
       
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
             let a = server::func().unwrap();
             MyApp::default().Info = a.Info;
            self.state = "Main".to_string();
        }

        ui.add_space(20.0);

      
      
    });
  
      
                
        }
             if self.state == "Schedule" {
                ui.heading("Schedule");
                   if ui.button("Main").clicked() {
                    self.state = "Main".to_string();
            }
            }
            if self.state == "Grades" {
                ui.heading("Grades");
                   if ui.button("Main").clicked() {
                    self.state = "Main".to_string();
            }
            }
            if self.state == "Main" {
               
                self.texture = Some(Self::load_image(ctx, "ASH.png"));
     if let Some(tex) = &self.texture {
    ui.with_layout(
        egui::Layout::right_to_left(egui::Align::TOP),
        |ui| {
            let size = tex.size_vec2() * 0.4; // 40% of original

            ui.add(
                egui::Image::new(tex)
                    .fit_to_exact_size(size)
            );
        },
    );
}
 //ui.heading("Main");
                
                   if ui.button("Schedule").clicked() {
                    self.state = "Schedule".to_string();
            }
              if ui.button("Grades").clicked() {
                    self.state = "Grades".to_string();
            }
            }
      
        });
    }
}
