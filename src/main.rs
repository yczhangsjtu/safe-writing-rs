use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use base64::{engine::general_purpose, Engine as _};
use hmac::{digest::MacError, Hmac, Mac};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use std::path::PathBuf;

use eframe::egui;
use egui::{Color32, FontFamily, FontId, FontSelection, RichText, TextEdit, Vec2};
use font_kit::source::SystemSource;
use serde::{Deserialize, Serialize};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        min_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Safe Writing",
        options,
        Box::new(|cc: &eframe::CreationContext<'_>| Box::<MyApp>::new(MyApp::new(cc))),
    )
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Config {
    font_size: f32,
    data_dir: String,
}

#[derive(Default, Clone)]
enum Content<T: Clone, P: Clone> {
    #[default]
    None,
    NewFile(T),
    Encrypted(T, T, T, T),
    PlainText(T, P, usize),
    Error(T),
}

impl<T: Clone, P: Clone> Content<T, P> {
    #[allow(unused)]
    fn as_ref(&self) -> Content<&T, &P> {
        match self {
            Content::Encrypted(ref title, ref a, ref b, ref c) => {
                Content::Encrypted(title, a, b, c)
            }
            Content::PlainText(ref title, ref a, index) => Content::PlainText(title, a, *index),
            Content::Error(ref a) => Content::Error(a),
            Content::None => Content::None,
            Content::NewFile(ref title) => Content::NewFile(title),
        }
    }

    fn get_plaintext_mut(&mut self) -> Option<&mut P> {
        match self {
            Content::PlainText(_, ref mut a, _) => Some(a),
            _ => None,
        }
    }
}

#[derive(Default)]
struct MyApp {
    content: Content<String, PlainText>,
    edited_text: String,
    dirty: bool,
    font_size: f32,
    file_names: Vec<String>,
    data_dir: String,
    creating_new_file: Option<String>,
    error_creating_new_file: Option<String>,
    password: String,
    add_new_passage: Option<(String, usize)>,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config_path = std::env::var("SAFE_WRITING_CONFIG_DIR")
            .map(|p| std::path::PathBuf::from(p))
            .unwrap_or(
                std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".safe_writing"),
            );
        if !config_path.is_dir() {
            if config_path.exists() {
                panic!("Config path {} is not a directory", config_path.display());
            }
            std::fs::create_dir_all(&config_path).unwrap();
        }

        let config_file = config_path.join("config.toml");
        let config = if !config_file.exists() {
            let config = Config {
                font_size: 24.0,
                data_dir: config_path.to_str().unwrap().to_owned(),
            };
            std::fs::write(config_file.clone(), toml::to_string(&config).unwrap()).unwrap();
            config
        } else {
            toml::from_str(&std::fs::read_to_string(config_file).unwrap()).unwrap()
        };

        let data_dir = std::path::PathBuf::from(&config.data_dir);
        if !data_dir.is_dir() {
            if data_dir.exists() {
                panic!("Data dir {} is not a directory", data_dir.display());
            }
            std::fs::create_dir_all(&data_dir).unwrap();
        }
        let file_names = std::fs::read_dir(&data_dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .filter_map(|filename| {
                if filename.ends_with(".safe") {
                    Some(filename[0..filename.len() - 5].to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let mut fonts = egui::FontDefinitions::default();

        let font = SystemSource::new()
            .select_by_postscript_name("Hei".into())
            .unwrap()
            .load()
            .unwrap()
            .copy_font_data()
            .unwrap();

        // Install my own font (maybe supporting non-latin characters).
        // .ttf and .otf files supported.
        fonts.font_data.insert(
            "heiti".to_owned(),
            egui::FontData::from_owned(font.to_vec()),
        );

        let font = SystemSource::new()
            .select_by_postscript_name("Heiti SC".into())
            .unwrap()
            .load()
            .unwrap()
            .copy_font_data()
            .unwrap();
        fonts.font_data.insert(
            "heiti sc".to_owned(),
            egui::FontData::from_owned(font.to_vec()),
        );

        let font = SystemSource::new()
            .select_by_postscript_name("PingFang SC".into())
            .unwrap()
            .load()
            .unwrap()
            .copy_font_data()
            .unwrap();
        fonts.font_data.insert(
            "pingfang sc".to_owned(),
            egui::FontData::from_owned(font.to_vec()),
        );

        let font = SystemSource::new()
            .select_by_postscript_name("SimSong".into())
            .unwrap()
            .load()
            .unwrap()
            .copy_font_data()
            .unwrap();
        fonts.font_data.insert(
            "simsong".to_owned(),
            egui::FontData::from_owned(font.to_vec()),
        );

        let font = SystemSource::new()
            .select_by_postscript_name("Songti SC".into())
            .unwrap()
            .load()
            .unwrap()
            .copy_font_data()
            .unwrap();
        fonts.font_data.insert(
            "songti sc".to_owned(),
            egui::FontData::from_owned(font.to_vec()),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "heiti".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "heiti sc".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "pingfang sc".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(1, "songti sc".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(1, "simsong".to_owned());

        // Put my font as last fallback for monospace:
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("heiti".to_owned());

        // Tell egui to use these fonts:
        cc.egui_ctx.set_fonts(fonts);

        Self {
            font_size: config.font_size,
            file_names,
            data_dir: config.data_dir,
            ..Default::default()
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(self.data_dir.as_str());
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                egui::Frame::none()
                    .fill(Color32::LIGHT_GRAY)
                    .inner_margin(5.0)
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(egui::WidgetText::RichText(
                                        RichText::from("Create New File").size(18.0),
                                    ))
                                    .min_size(Vec2::new(200.0, 24.0))
                                    .fill(Color32::LIGHT_GREEN),
                                )
                                .clicked()
                            {
                                if self.creating_new_file.is_none() {
                                    self.creating_new_file = Some("".to_string());
                                } else {
                                    self.creating_new_file = None;
                                }
                                self.error_creating_new_file = None;
                            }
                            if let Some(ref mut filename) = self.creating_new_file {
                                ui.add(
                                    egui::TextEdit::singleline(filename)
                                        .font(FontSelection::FontId(FontId::new(
                                            18.0,
                                            FontFamily::Proportional,
                                        )))
                                        .desired_width(200.0),
                                );
                                if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                                    println!("Create file {}", filename);
                                    let path = PathBuf::from(self.data_dir.clone())
                                        .join(format!("{}.safe", filename));
                                    if path.exists() {
                                        self.error_creating_new_file =
                                            Some(format!("File {} already exists", filename));
                                    } else {
                                        std::fs::write(path, "").unwrap();
                                        self.file_names.push(filename.clone());
                                    }
                                    self.creating_new_file = None;
                                }
                            }
                            if let Some(error) = &self.error_creating_new_file {
                                ui.add(egui::Label::new(egui::WidgetText::RichText(
                                    RichText::from(error).color(Color32::RED),
                                )));
                            }
                            egui::ScrollArea::vertical()
                                .id_source("file_name_list")
                                .max_height(f32::INFINITY)
                                .auto_shrink([true, false])
                                .max_width(200.0)
                                .show(ui, |ui| {
                                    self.file_names.iter().for_each(|file_name| {
                                        if ui
                                            .add(
                                                egui::Button::new(egui::WidgetText::RichText(
                                                    RichText::from(file_name).size(18.0),
                                                ))
                                                .min_size(Vec2::new(200.0, 24.0))
                                                .fill(Color32::LIGHT_BLUE),
                                            )
                                            .clicked()
                                        {
                                            println!("Clicked on {}", file_name);
                                            self.password = "".to_string();
                                            let path = PathBuf::from(self.data_dir.clone())
                                                .join(format!("{}.safe", file_name));
                                            match std::fs::read(path) {
                                                Ok(content) => {
                                                    let content =
                                                        String::from_utf8(content).unwrap();
                                                    if content.is_empty() {
                                                        self.content =
                                                            Content::NewFile(file_name.clone());
                                                    } else {
                                                        let content: Vec<_> =
                                                            content.split("\n").collect();
                                                        if content.len() < 3 {
                                                            self.content = Content::Error(
                                                                "Invalid format".to_string(),
                                                            );
                                                        } else {
                                                            self.content = Content::Encrypted(
                                                                file_name.clone(),
                                                                content[0].to_string(),
                                                                content[1].to_string(),
                                                                content[2].to_string(),
                                                            );
                                                        }
                                                    }
                                                }
                                                Err(err) => {
                                                    println!(
                                                        "Failed to open file {}: {:?}",
                                                        file_name, err
                                                    );
                                                }
                                            }
                                        }
                                    });
                                });
                        });
                    });
                match self.content.clone() {
                    Content::NewFile(filename) => {
                        ui.add(
                            TextEdit::singleline(&mut self.password)
                                .password(true)
                                .hint_text("New Password"),
                        );
                        if ui.button("Create").clicked() {
                            if self.password.len() > 0 {
                                self.content =
                                    Content::PlainText(filename, PlainText { content: vec![] }, 0)
                            }
                        }
                    }
                    Content::Encrypted(ref filename, ref iv, ref data, ref mac) => {
                        ui.add(
                            TextEdit::singleline(&mut self.password)
                                .password(true)
                                .hint_text("Password"),
                        );
                        if ui.button("Decrypt").clicked() {
                            match decrypt(&self.password, iv, data, mac) {
                                Ok(plaintext) => {
                                    if plaintext.content.len() > 0 {
                                        self.edited_text = plaintext.content[0].content.clone();
                                    }
                                    self.content =
                                        Content::PlainText(filename.clone(), plaintext, 0);
                                    self.dirty = false;
                                }
                                Err(err) => {
                                    self.content = Content::Error(format!("{:?}", err));
                                }
                            }
                        }
                    }
                    Content::None => {
                        ui.add(egui::Label::new(egui::WidgetText::RichText(
                            RichText::from("Please select a file to open").size(18.0),
                        )));
                    }
                    Content::PlainText(filename, plaintext, selected_index) => {
                        egui::Frame::none()
                            .fill(Color32::LIGHT_GRAY)
                            .inner_margin(5.0)
                            .show(ui, |ui| {
                                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(egui::WidgetText::RichText(
                                                RichText::from("Add").size(18.0),
                                            ))
                                            .min_size(Vec2::new(100.0, 24.0))
                                            .fill(Color32::LIGHT_GREEN),
                                        )
                                        .clicked()
                                    {
                                        self.add_new_passage =
                                            Some(("".to_string(), selected_index + 1))
                                    }
                                    if ui
                                        .add(
                                            egui::Button::new(egui::WidgetText::RichText(
                                                RichText::from("Save").size(18.0).color(
                                                    if self.dirty {
                                                        Color32::BLACK
                                                    } else {
                                                        Color32::LIGHT_GRAY
                                                    },
                                                ),
                                            ))
                                            .min_size(Vec2::new(100.0, 24.0))
                                            .fill(Color32::LIGHT_GREEN),
                                        )
                                        .clicked()
                                    {
                                        let path = PathBuf::from(self.data_dir.clone())
                                            .join(format!("{}.safe", filename));
                                        std::fs::write(
                                            path,
                                            encrypt(&self.password, plaintext.clone()),
                                        )
                                        .unwrap();
                                        self.dirty = false;
                                    }
                                    egui::ScrollArea::vertical()
                                        .id_source("passage_list")
                                        .max_height(f32::INFINITY)
                                        .auto_shrink([true, false])
                                        .max_width(100.0)
                                        .show(ui, |ui| {
                                            if plaintext.content.is_empty() {
                                                if let Some((ref mut title, _)) =
                                                    self.add_new_passage
                                                {
                                                    ui.add(
                                                        egui::TextEdit::singleline(title)
                                                            .font(FontSelection::FontId(
                                                                FontId::new(
                                                                    18.0,
                                                                    FontFamily::Proportional,
                                                                ),
                                                            ))
                                                            .desired_width(100.0),
                                                    );
                                                    if ctx
                                                        .input(|i| i.key_pressed(egui::Key::Enter))
                                                    {
                                                        println!("Create passage {}", title);
                                                        self.content
                                                            .get_plaintext_mut()
                                                            .unwrap()
                                                            .content
                                                            .insert(
                                                                0,
                                                                Passage {
                                                                    title: title.clone(),
                                                                    content: "".to_string(),
                                                                },
                                                            );
                                                        self.add_new_passage = None;
                                                        self.dirty = true;
                                                    }
                                                }
                                            }
                                            plaintext.content.iter().enumerate().for_each(
                                                |(i, passage)| {
                                                    if ui
                                                        .add(
                                                            egui::Button::new(
                                                                egui::WidgetText::RichText(
                                                                    RichText::from(
                                                                        passage.title.clone(),
                                                                    )
                                                                    .size(18.0),
                                                                ),
                                                            )
                                                            .min_size(Vec2::new(100.0, 24.0))
                                                            .fill(if i == selected_index {
                                                                Color32::LIGHT_BLUE
                                                            } else {
                                                                Color32::LIGHT_GRAY
                                                            }),
                                                        )
                                                        .clicked()
                                                    {
                                                        if i != selected_index {
                                                            self.content = Content::PlainText(
                                                                filename.clone(),
                                                                plaintext.clone(),
                                                                i,
                                                            );
                                                            self.edited_text = plaintext.content[i]
                                                                .content
                                                                .clone();
                                                        }
                                                    }
                                                    if let Some((ref mut title, index)) =
                                                        self.add_new_passage
                                                    {
                                                        if index == i + 1 {
                                                            ui.add(
                                                            egui::TextEdit::singleline(title)
                                                                .font(FontSelection::FontId(
                                                                    FontId::new(
                                                                        18.0,
                                                                        FontFamily::Proportional,
                                                                    ),
                                                                ))
                                                                .desired_width(100.0),
                                                        );
                                                            if ctx.input(|i| {
                                                                i.key_pressed(egui::Key::Enter)
                                                            }) {
                                                                println!(
                                                                    "Create passage {}",
                                                                    title
                                                                );
                                                                self.content
                                                                    .get_plaintext_mut()
                                                                    .unwrap()
                                                                    .content
                                                                    .insert(
                                                                        index,
                                                                        Passage {
                                                                            title: title.clone(),
                                                                            content: "".to_string(),
                                                                        },
                                                                    );
                                                                self.add_new_passage = None;
                                                            }
                                                        }
                                                    }
                                                },
                                            );
                                        });
                                })
                            });

                        egui::ScrollArea::vertical()
                            .id_source("editor")
                            .show(ui, |ui| {
                                if plaintext.content.is_empty() {
                                    ui.add(egui::Label::new(egui::WidgetText::RichText(
                                        RichText::from("Empty file").size(18.0),
                                    )));
                                } else {
                                    if ui
                                        .add(
                                            TextEdit::multiline(&mut self.edited_text)
                                                .frame(false)
                                                .desired_width(f32::INFINITY)
                                                .desired_rows(1000) // Infinite
                                                .font(FontSelection::FontId(FontId::new(
                                                    self.font_size,
                                                    FontFamily::Proportional,
                                                ))),
                                        )
                                        .changed()
                                    {
                                        self.content.get_plaintext_mut().map(|plaintext| {
                                            plaintext.content[selected_index].content =
                                                self.edited_text.clone();
                                            self.dirty = true;
                                        });
                                    }
                                }
                            });
                    }
                    Content::Error(err) => {
                        ui.add(egui::Label::new(egui::WidgetText::RichText(
                            RichText::from(err).size(18.0).color(Color32::RED),
                        )));
                    }
                }
            })
        });
    }
}

fn key_derive(password: &str) -> [u8; 16] {
    let mut out = [0u8; 16]; // We will use 128 bits key
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(password.as_bytes(), b"safe_write", 100, &mut out);
    out
}

fn encrypt(password: &str, data: PlainText) -> String {
    let data = data.encode();
    let key = key_derive(password);

    let mut iv = [0u8; 16];
    StdRng::from_entropy().fill_bytes(&mut iv);

    let encrypted = cbc::Encryptor::<aes::Aes128>::new(&key.into(), &iv.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&data);
    let mut mac =
        Hmac::<sha2::Sha256>::new_from_slice(&key).expect("HMAC can take key of any size");
    mac.update(encrypted.as_slice());

    general_purpose::STANDARD.encode(iv)
        + "\n"
        + &general_purpose::STANDARD.encode(encrypted)
        + "\n"
        + &general_purpose::STANDARD.encode(mac.finalize().into_bytes())
}

#[derive(Debug, Clone)]
struct Passage {
    title: String,
    content: String,
}

impl Passage {
    fn encode(&self) -> String {
        let title = base64_encode(self.title.as_bytes().to_vec());
        let content = base64_encode(self.content.as_bytes().to_vec());
        title + "-" + &content
    }
}

#[derive(Debug, Clone)]
struct PlainText {
    content: Vec<Passage>,
}

impl PlainText {
    fn encode(&self) -> Vec<u8> {
        (self
            .content
            .iter()
            .map(|p| p.encode())
            .collect::<Vec<_>>()
            .join("|")
            + ":FontSize=24")
            .as_bytes()
            .to_vec()
    }
}

fn decrypt(password: &str, iv: &str, data: &str, mac: &str) -> Result<PlainText, Error> {
    let key = key_derive(password);
    let iv = general_purpose::STANDARD
        .decode(iv)
        .map_err(|_| Error::Base64DecodeFail)?;
    let data = general_purpose::STANDARD
        .decode(data)
        .map_err(|_| Error::Base64DecodeFail)?;
    let mac = general_purpose::STANDARD
        .decode(mac)
        .map_err(|_| Error::Base64DecodeFail)?;
    let mut mac_calculated =
        Hmac::<sha2::Sha256>::new_from_slice(&key).expect("HMAC can take key of any size");
    mac_calculated.update(data.as_slice());
    mac_calculated
        .verify_slice(&mac)
        .map_err(|err| Error::MacFail(err))?;

    let plaintext = cbc::Decryptor::<aes::Aes128>::new(&key.into(), iv.as_slice().into())
        .decrypt_padded_vec_mut::<Pkcs7>(&data)
        .map_err(|_| Error::DecryptionFail)
        .and_then(|s| String::from_utf8(s).map_err(|_| Error::InvalidUTF8))?;

    let plaintexts: Vec<_> = plaintext.split(":").collect();
    if plaintexts.len() < 2 {
        return Err(Error::InvalidPlaintextFormat);
    }
    let plaintext_encodings = plaintexts[0];
    // let font_size = plaintexts[1];

    let plaintext_encodings: Vec<_> = plaintext_encodings.split("|").collect();
    let passages = plaintext_encodings
        .iter()
        .map(|s| {
            let contents: Vec<_> = s.split("-").collect();
            if contents.len() < 2 {
                return Err(Error::InvalidPlaintextFormat);
            }
            let title = contents[0];
            let content = contents[1];
            Ok(Passage {
                title: base64_decode(title)?,
                content: base64_decode(content)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(PlainText { content: passages })
}

fn base64_encode(data: Vec<u8>) -> String {
    general_purpose::STANDARD.encode(data)
}

fn base64_decode(data: &str) -> Result<String, Error> {
    String::from_utf8(
        general_purpose::STANDARD
            .decode(data)
            .map_err(|_| Error::Base64DecodeFail)?,
    )
    .map_err(|_| Error::InvalidUTF8)
}

#[derive(Debug)]
enum Error {
    Base64DecodeFail,
    DecryptionFail,
    MacFail(MacError),
    InvalidUTF8,
    InvalidPlaintextFormat,
}
