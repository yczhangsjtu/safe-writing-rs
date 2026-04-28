use crate::app::config::Config;
use crate::consts::COPILOT_PANEL_WIDTH;
use std::io::BufRead;
use std::sync::mpsc::{channel, Receiver, Sender};

use eframe::egui;
use egui::{Color32, FontFamily, FontId, FontSelection, RichText, ScrollArea, TextEdit};

#[derive(Clone)]
pub struct Message {
    pub role: String,
    pub content: String, // XML-structured content sent to API
    pub display: String, // Raw prompt shown in UI
}

pub struct CopilotState {
    pub visible: bool,
    pub system_prompt: String,
    pub user_input: String,
    pub output: String,
    pub buffers: Vec<String>,
    pub messages: Vec<Message>,
    pub stream_receiver: Option<Receiver<String>>,
    pub abort_sender: Option<Sender<()>>,
    pub waiting: bool,
}

impl Default for CopilotState {
    fn default() -> Self {
        Self {
            visible: false,
            system_prompt: "You are a helpful writing assistant.".to_string(),
            user_input: String::new(),
            output: String::new(),
            buffers: Vec::new(),
            messages: Vec::new(),
            stream_receiver: None,
            abort_sender: None,
            waiting: false,
        }
    }
}

impl CopilotState {
    pub fn send_message(&mut self, config: &Config) {
        if self.user_input.trim().is_empty() {
            return;
        }

        let prompt = self.user_input.clone();
        self.user_input.clear();

        // Build XML-structured content
        let xml_content = Self::build_prompt(&prompt, &self.buffers);

        // Add user message to history (store both raw and XML)
        self.messages.push(Message {
            role: "user".to_string(),
            content: xml_content.clone(),
            display: prompt.clone(),
        });

        // Build the full messages array for the API
        let mut api_messages = vec![serde_json::json!({
            "role": "system",
            "content": self.system_prompt,
        })];

        for msg in &self.messages {
            api_messages.push(serde_json::json!({
                "role": msg.role,
                "content": msg.content,
            }));
        }

        let body = serde_json::json!({
            "model": "local",
            "messages": api_messages,
            "stream": true,
        });

        let url = format!("{}/v1/chat/completions", config.llamacpp_url);
        let (tx, rx) = channel::<String>();
        let (abort_tx, abort_rx) = channel::<()>();

        self.stream_receiver = Some(rx);
        self.abort_sender = Some(abort_tx);
        self.waiting = true;
        self.output.clear();

        std::thread::spawn(move || {
            let agent = ureq::Agent::new_with_defaults();
            let body_json = body.to_string();
            let resp: ureq::http::Response<ureq::Body> = match agent
                .post(&url)
                .header("Content-Type", "application/json")
                .send(body_json.as_bytes())
            {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(format!("Error: {}", e));
                    let _ = tx.send("__DONE__".to_string());
                    return;
                }
            };

            let mut resp_body = resp.into_body();
            let reader = std::io::BufReader::new(resp_body.as_reader());
            for line in reader.lines() {
                if abort_rx.try_recv().is_ok() {
                    let _ = tx.send("__DONE__".to_string());
                    return;
                }
                let line: String = match line {
                    Ok(l) => l,
                    Err(e) => {
                        let _ = tx.send(format!("Error reading stream: {}", e));
                        let _ = tx.send("__DONE__".to_string());
                        return;
                    }
                };
                if line.is_empty() {
                    continue;
                }
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        let _ = tx.send("__DONE__".to_string());
                        return;
                    }
                    match serde_json::from_str::<serde_json::Value>(data) {
                        Ok(json) => {
                            if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                                let _ = tx.send(content.to_string());
                            }
                        }
                        Err(_) => continue,
                    }
                }
            }
            let _ = tx.send("__DONE__".to_string());
        });
    }

    fn build_prompt(prompt: &str, buffers: &[String]) -> String {
        let mut result = String::new();
        result.push_str("<prompt>\n");
        result.push_str(prompt);
        result.push_str("\n</prompt>\n");

        // Only include buffers that are referenced in the prompt
        for (i, buf) in buffers.iter().enumerate() {
            let id = format!("#{}", i);
            if prompt.contains(&id) {
                result.push_str(&format!("<buffer id=\"{}\">\n", i));
                result.push_str(buf);
                result.push_str("\n</buffer>\n");
            }
        }

        result
    }

    pub fn stop_generation(&mut self) {
        if let Some(sender) = self.abort_sender.take() {
            let _ = sender.send(());
        }
        self.waiting = false;
    }

    pub fn poll_stream(&mut self) {
        if let Some(ref rx) = self.stream_receiver {
            loop {
                match rx.try_recv() {
                    Ok(chunk) => {
                        if chunk == "__DONE__" {
                            self.waiting = false;
                            self.stream_receiver = None;
                            self.abort_sender = None;
                            // Add assistant response to history
                            if !self.output.is_empty() {
                                self.messages.push(Message {
                                    role: "assistant".to_string(),
                                    content: self.output.clone(),
                                    display: self.output.clone(),
                                });
                            }
                            break;
                        } else {
                            self.output.push_str(&chunk);
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        self.waiting = false;
                        self.stream_receiver = None;
                        self.abort_sender = None;
                        break;
                    }
                }
            }
        }
    }

    pub fn add_buffer(&mut self, text: String) {
        self.buffers.push(text);
    }

    pub fn remove_buffer(&mut self, index: usize) {
        if index < self.buffers.len() {
            self.buffers.remove(index);
        }
    }

    pub fn clear_history(&mut self) {
        self.messages.clear();
        self.output.clear();
    }
}

pub fn build_copilot_panel(
    copilot_state: &mut CopilotState,
    config: &Config,
    selected_text: Option<&str>,
    ui: &mut egui::Ui,
) {
    egui::Frame::new()
        .fill(Color32::from_rgb(30, 30, 40))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_min_width(COPILOT_PANEL_WIDTH);
            ui.set_max_width(COPILOT_PANEL_WIDTH);

            // Poll stream every frame
            copilot_state.poll_stream();

            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("AI Copilot")
                            .size(16.0)
                            .color(Color32::LIGHT_BLUE),
                    );
                    if ui
                        .add(egui::Button::new(
                            RichText::new("Clear").size(14.0).color(Color32::WHITE),
                        ))
                        .clicked()
                    {
                        copilot_state.clear_history();
                    }
                });
                ui.separator();

                // System prompt
                ui.collapsing("System Prompt", |ui| {
                    ui.add(
                        TextEdit::multiline(&mut copilot_state.system_prompt)
                            .desired_width(COPILOT_PANEL_WIDTH - 20.0)
                            .desired_rows(3)
                            .font(FontSelection::FontId(FontId::new(
                                14.0,
                                FontFamily::Proportional,
                            )))
                            .text_color(Color32::WHITE),
                    );
                });

                // Buffers
                ui.collapsing(format!("Buffers ({})", copilot_state.buffers.len()), |ui| {
                    if let Some(text) = selected_text {
                        if !text.is_empty()
                            && ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Add Selection to Buffer")
                                            .size(14.0)
                                            .color(Color32::WHITE),
                                    )
                                    .fill(Color32::DARK_BLUE),
                                )
                                .clicked()
                        {
                            copilot_state.add_buffer(text.to_string());
                        }
                    }
                    let mut to_remove = None;
                    for (i, buf) in copilot_state.buffers.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("#{}: {} chars", i, buf.chars().count()))
                                    .size(14.0)
                                    .color(Color32::LIGHT_GRAY),
                            );
                            if ui
                                .add(egui::Button::new(
                                    RichText::new("×").size(14.0).color(Color32::RED),
                                ))
                                .clicked()
                            {
                                to_remove = Some(i);
                            }
                        });
                    }
                    if let Some(i) = to_remove {
                        copilot_state.remove_buffer(i);
                    }
                });

                ui.separator();

                // Output area
                ScrollArea::vertical()
                    .id_salt("copilot_output")
                    .auto_shrink([false, false])
                    .max_height(ui.available_height() - 120.0)
                    .show(ui, |ui| {
                        // Show conversation history
                        for msg in &copilot_state.messages {
                            let (label, color) = match msg.role.as_str() {
                                "user" => ("User", Color32::LIGHT_GREEN),
                                "assistant" => ("Assistant", Color32::LIGHT_BLUE),
                                _ => ("Unknown", Color32::GRAY),
                            };
                            ui.label(
                                RichText::new(format!("{}:", label))
                                    .size(14.0)
                                    .color(color)
                                    .strong(),
                            );
                            let display_text = if msg.role == "user" {
                                &msg.display
                            } else {
                                &msg.content
                            };
                            ui.add(
                                TextEdit::multiline(&mut display_text.clone())
                                    .desired_width(COPILOT_PANEL_WIDTH - 20.0)
                                    .font(FontSelection::FontId(FontId::new(
                                        14.0,
                                        FontFamily::Proportional,
                                    )))
                                    .text_color(Color32::WHITE)
                                    .interactive(false),
                            );
                            ui.separator();
                        }

                        // Show streaming output
                        if copilot_state.waiting || !copilot_state.output.is_empty() {
                            ui.label(
                                RichText::new("Assistant:")
                                    .size(14.0)
                                    .color(Color32::LIGHT_BLUE)
                                    .strong(),
                            );
                            ui.add(
                                TextEdit::multiline(&mut copilot_state.output.clone())
                                    .desired_width(COPILOT_PANEL_WIDTH - 20.0)
                                    .font(FontSelection::FontId(FontId::new(
                                        14.0,
                                        FontFamily::Proportional,
                                    )))
                                    .text_color(Color32::WHITE)
                                    .interactive(false),
                            );
                        }
                    });

                // Input area
                ui.horizontal(|ui| {
                    ui.add(
                        TextEdit::singleline(&mut copilot_state.user_input)
                            .desired_width(COPILOT_PANEL_WIDTH - 80.0)
                            .font(FontSelection::FontId(FontId::new(
                                14.0,
                                FontFamily::Proportional,
                            )))
                            .hint_text("Type prompt, use #0 #1...")
                            .text_color(Color32::WHITE),
                    );
                    if copilot_state.waiting {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Stop").size(14.0).color(Color32::WHITE),
                                )
                                .fill(Color32::RED),
                            )
                            .clicked()
                        {
                            copilot_state.stop_generation();
                        }
                    } else {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Send").size(14.0).color(Color32::WHITE),
                                )
                                .fill(Color32::DARK_GREEN),
                            )
                            .clicked()
                            || ui
                                .ctx()
                                .input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift)
                        {
                            copilot_state.send_message(config);
                        }
                    }
                });
            });
        });
}
