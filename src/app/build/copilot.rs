use crate::app::config::Config;
use crate::data_structures::PlainText;
use std::io::BufRead;
use std::sync::mpsc::{channel, Receiver, Sender};

use eframe::egui;
use egui::containers::menu::MenuButton;
use egui::{Color32, FontFamily, FontId, FontSelection, RichText, ScrollArea, TextEdit, Widget};

pub const AI_PASSAGE_NAME: &str = ".ai";

fn make_brief_summary(text: &str, max_len: usize) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    let len = chars.len();
    if len <= max_len {
        text.to_string()
    } else {
        let half = max_len / 2;
        let start = chars[..half].iter().collect::<String>();
        let end = chars[len - half..].iter().collect::<String>();
        format!("{}...{}", start, end)
    }
}

#[derive(Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub display: String,
}

#[derive(Clone, Debug)]
pub struct FavoritePrompt {
    pub name: String,
    pub prompt: String,
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
    pub favorite_prompts: Vec<FavoritePrompt>,
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn unescape_xml(s: &str) -> String {
    s.replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}

fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);
    let start = xml.find(&start_tag)? + start_tag.len();
    let end = xml[start..].find(&end_tag)?;
    Some(xml[start..start + end].to_string())
}

fn extract_favorite_prompts(xml: &str) -> Option<Vec<(String, String)>> {
    let favs_section = extract_tag(xml, "favorite_prompts")?;
    let mut result = Vec::new();
    let mut remaining = favs_section.as_str();
    while let Some(prompt_start) = remaining.find("<prompt>") {
        let after_prompt_start = &remaining[prompt_start + 8..];
        if let Some(prompt_end) = after_prompt_start.find("</prompt>") {
            let prompt_content = &after_prompt_start[..prompt_end];
            if let (Some(name), Some(content)) = (
                extract_tag(prompt_content, "name"),
                extract_tag(prompt_content, "content"),
            ) {
                result.push((name, content));
            }
            remaining = &after_prompt_start[prompt_end + 9..];
        } else {
            break;
        }
    }
    Some(result)
}

fn extract_buffers(xml: &str) -> Option<Vec<String>> {
    let buffers_section = extract_tag(xml, "buffers")?;
    let mut buffers = vec![String::new(); 10];
    let mut remaining = buffers_section.as_str();
    while let Some(buffer_start) = remaining.find("<buffer") {
        let after_buffer_start = &remaining[buffer_start..];
        if let Some(gt_pos) = after_buffer_start.find('>') {
            let attr_part = &after_buffer_start[7..gt_pos];
            if let Some(index_str) = attr_part
                .strip_prefix(" index=\"")
                .and_then(|s| s.strip_suffix("\""))
            {
                if let Ok(index) = index_str.parse::<usize>() {
                    let content_start = gt_pos + 1;
                    if let Some(end_tag) = after_buffer_start.find("</buffer>") {
                        let content = &after_buffer_start[content_start..end_tag];
                        if index < 10 {
                            buffers[index] = unescape_xml(content);
                        }
                        remaining = &after_buffer_start[end_tag + 9..];
                        continue;
                    }
                }
            }
        }
        break;
    }
    Some(buffers)
}

impl Default for CopilotState {
    fn default() -> Self {
        Self {
            visible: false,
            system_prompt: "You are a helpful writing assistant.".to_string(),
            user_input: String::new(),
            output: String::new(),
            buffers: vec![String::new(); 10],
            messages: Vec::new(),
            stream_receiver: None,
            abort_sender: None,
            waiting: false,
            favorite_prompts: Vec::new(),
        }
    }
}

impl CopilotState {
    pub fn to_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str("<ai_settings>\n");
        xml.push_str(&format!(
            "  <system_prompt>{}</system_prompt>\n",
            escape_xml(&self.system_prompt)
        ));
        xml.push_str("  <favorite_prompts>\n");
        for fav in &self.favorite_prompts {
            xml.push_str("    <prompt>\n");
            xml.push_str(&format!("      <name>{}</name>\n", escape_xml(&fav.name)));
            xml.push_str(&format!(
                "      <content>{}</content>\n",
                escape_xml(&fav.prompt)
            ));
            xml.push_str("    </prompt>\n");
        }
        xml.push_str("  </favorite_prompts>\n");
        xml.push_str("  <buffers>\n");
        for (i, buf) in self.buffers.iter().enumerate().take(10) {
            if !buf.is_empty() {
                xml.push_str(&format!(
                    "    <buffer index=\"{}\">{}</buffer>\n",
                    i,
                    escape_xml(buf)
                ));
            }
        }
        xml.push_str("  </buffers>\n");
        xml.push_str("</ai_settings>");
        xml
    }

    pub fn from_xml(xml: &str) -> Option<Self> {
        let system_prompt = extract_tag(xml, "system_prompt")?;
        let favorite_prompts = extract_favorite_prompts(xml)?;
        let buffers = extract_buffers(xml).unwrap_or_else(|| vec![String::new(); 10]);
        Some(Self {
            visible: false,
            system_prompt: unescape_xml(&system_prompt),
            user_input: String::new(),
            output: String::new(),
            buffers,
            messages: Vec::new(),
            stream_receiver: None,
            abort_sender: None,
            waiting: false,
            favorite_prompts: favorite_prompts
                .into_iter()
                .map(|(name, prompt)| FavoritePrompt {
                    name: unescape_xml(&name),
                    prompt: unescape_xml(&prompt),
                })
                .collect(),
        })
    }

    pub fn load_from_plaintext(&mut self, plaintext: &PlainText, _filename: &str) {
        let mut found = false;
        for passage in plaintext.passages() {
            if passage.title() == AI_PASSAGE_NAME {
                if let Some(state) = Self::from_xml(passage.content()) {
                    self.system_prompt = state.system_prompt;
                    self.favorite_prompts = state.favorite_prompts;
                    self.buffers = state.buffers;
                }
                found = true;
                break;
            }
        }
        if !found {
            self.clear();
        }
    }

    pub fn refresh_from_plaintext(&mut self, plaintext: &PlainText) {
        let mut found = false;
        for passage in plaintext.passages() {
            if passage.title() == AI_PASSAGE_NAME {
                if let Some(state) = Self::from_xml(passage.content()) {
                    self.system_prompt = state.system_prompt;
                    self.favorite_prompts = state.favorite_prompts;
                    self.buffers = state.buffers;
                }
                found = true;
                break;
            }
        }
        if !found {
            self.clear();
        }
    }

    pub fn reload_from_plaintext(&mut self, plaintext: &PlainText) -> Result<(), String> {
        for passage in plaintext.passages() {
            if passage.title() == AI_PASSAGE_NAME {
                match Self::from_xml(passage.content()) {
                    Some(state) => {
                        self.system_prompt = state.system_prompt;
                        self.favorite_prompts = state.favorite_prompts;
                        self.buffers = state.buffers;
                        return Ok(());
                    }
                    None => {
                        return Err("Failed to parse .ai passage XML".to_string());
                    }
                }
            }
        }
        self.clear();
        Ok(())
    }

    pub fn save_to_plaintext(&self, plaintext: &mut PlainText) {
        let ai_passage_index = plaintext
            .passages()
            .iter()
            .position(|p| p.title() == AI_PASSAGE_NAME);
        let xml = self.to_xml();
        if let Some(index) = ai_passage_index {
            plaintext.set_content(index, xml);
        } else {
            plaintext.insert_new_passage(plaintext.num_passages(), AI_PASSAGE_NAME.to_string());
            plaintext.set_content(plaintext.num_passages() - 1, xml);
        }
    }

    pub fn send_message(&mut self, config: &Config, current_passage: &str) {
        if self.user_input.trim().is_empty() {
            return;
        }

        let prompt = self.user_input.clone();
        self.user_input.clear();

        let xml_content = Self::build_prompt(&prompt, &self.buffers, current_passage);

        // Add user message to history (store both raw and XML)
        self.messages.push(Message {
            role: "user".to_string(),
            content: xml_content.clone(),
            display: prompt.clone(),
        });

        // Build the full messages array for the API
        let buffer_instructions = "\n\n<buffer_instructions>\nThe user may reference text buffers using placeholders like <first>, <second>, etc. When such placeholders appear, the actual buffer content will be provided in a <referenced_buffers> section within the user's message.\n</buffer_instructions>";
        
        let mut api_messages = vec![serde_json::json!({
            "role": "system",
            "content": format!("{}{}", self.system_prompt, buffer_instructions),
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

    fn build_prompt(prompt: &str, buffers: &[String], current_passage: &str) -> String {
        let buffer_names = [
            "first", "second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth",
            "tenth",
        ];

        let mut result = String::new();
        let mut referenced_names: Vec<(&str, usize)> = Vec::new();
        let mut include_all = false;

        if prompt.contains("<all>") {
            include_all = true;
        }

        for (i, _) in buffers.iter().enumerate().take(10) {
            let id = format!("#{}", i);
            if prompt.contains(&id) {
                referenced_names.push((buffer_names[i], i));
            }
        }

        let mut processed_prompt = prompt.to_string();
        for (name, i) in &referenced_names {
            let id = format!("#{}", i);
            let replacement = format!("<{}>", name);
            processed_prompt = processed_prompt.replace(&id, &replacement);
        }

        result.push_str(&processed_prompt);

        if !referenced_names.is_empty() || include_all {
            result.push_str("\n\n<referenced_buffers>\n");
            for (name, i) in &referenced_names {
                result.push_str(&format!("<{}>\n", name));
                result.push_str(&buffers[*i]);
                result.push_str(&format!("\n</{}>\n", name));
            }
            if include_all {
                result.push_str("<all>\n");
                result.push_str(current_passage);
                result.push_str("\n</all>\n");
            }
            result.push_str("</referenced_buffers>\n");
        }

        result
    }

    pub fn stop_generation(&mut self) {
        if let Some(sender) = self.abort_sender.take() {
            let _ = sender.send(());
        }
        self.waiting = false;
    }

    pub fn poll_stream(&mut self, ctx: &egui::Context) -> bool {
        let mut received_data = false;
        if let Some(ref rx) = self.stream_receiver {
            loop {
                match rx.try_recv() {
                    Ok(chunk) => {
                        received_data = true;
                        if chunk == "__DONE__" {
                            self.waiting = false;
                            self.stream_receiver = None;
                            self.abort_sender = None;
                            if !self.output.is_empty() {
                                self.messages.push(Message {
                                    role: "assistant".to_string(),
                                    content: self.output.clone(),
                                    display: self.output.clone(),
                                });
                                self.output.clear();
                            }
                            break;
                        } else {
                            self.output.push_str(&chunk);
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        if self.waiting {
                            ctx.request_repaint_after(std::time::Duration::from_millis(100));
                        }
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        self.waiting = false;
                        self.stream_receiver = None;
                        self.abort_sender = None;
                        break;
                    }
                }
            }
        }
        received_data
    }

    pub fn add_buffer(&mut self, text: String) {
        for buf in &mut self.buffers {
            if buf.is_empty() {
                *buf = text;
                return;
            }
        }
    }

    pub fn remove_buffer(&mut self, index: usize) {
        if index < self.buffers.len() {
            self.buffers[index].clear();
        }
    }

    pub fn clear_history(&mut self) {
        self.messages.clear();
        self.output.clear();
        self.waiting = false;
        self.stream_receiver = None;
        self.abort_sender = None;
    }

    pub fn clear(&mut self) {
        self.clear_history();
        self.system_prompt = Self::default().system_prompt;
        self.favorite_prompts.clear();
        self.user_input.clear();
        self.buffers = vec![String::new(); 10];
    }
}

pub fn build_copilot_panel(
    copilot_state: &mut CopilotState,
    config: &Config,
    selected_text: Option<&str>,
    plaintext: &PlainText,
    current_passage: &str,
    reload_error: Option<&String>,
    ui: &mut egui::Ui,
) -> (Option<String>, bool) {
    let bg = Color32::from_rgb(30, 30, 35);
    let text_light = Color32::from_rgb(220, 220, 230);
    let text_gray = Color32::from_rgb(150, 150, 160);
    let user_color = Color32::from_rgb(100, 200, 140);
    let assistant_color = Color32::from_rgb(100, 150, 230);
    let input_bg = Color32::from_rgb(45, 45, 50);
    let button_primary = Color32::from_rgb(0, 120, 60);
    let button_danger = Color32::from_rgb(200, 40, 40);
    let button_secondary = Color32::from_rgb(60, 60, 70);
    let button_secondary_text = Color32::from_rgb(200, 200, 210);

    let mut output_to_insert = None;
    let mut needs_save_ai = false;

    egui::Frame::new()
        .fill(bg)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.visuals_mut().widgets.noninteractive.fg_stroke.color = text_light;
            ui.visuals_mut().widgets.noninteractive.bg_fill = bg;
            ui.visuals_mut().widgets.inactive.fg_stroke.color = text_light;
            ui.visuals_mut().widgets.inactive.bg_fill = input_bg;
            ui.visuals_mut().widgets.hovered.fg_stroke.color = text_light;
            ui.visuals_mut().widgets.hovered.bg_fill = Color32::from_rgb(50, 50, 60);
            ui.visuals_mut().widgets.active.fg_stroke.color = text_light;
            ui.visuals_mut().widgets.active.bg_fill = Color32::from_rgb(60, 100, 180);
            ui.visuals_mut().window_fill = bg;

            let available_width = ui.available_width();

            let ctx = ui.ctx().clone();
            if copilot_state.poll_stream(&ctx) {
                ctx.request_repaint();
            }

            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("AI Copilot")
                            .size(16.0)
                            .color(assistant_color)
                            .strong(),
                    );
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("↻").size(14.0).color(button_secondary_text),
                            )
                            .fill(button_secondary),
                        )
                        .clicked()
                    {
                        copilot_state.refresh_from_plaintext(plaintext);
                    }
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("Clear")
                                    .size(14.0)
                                    .color(button_secondary_text),
                            )
                            .fill(button_secondary),
                        )
                        .clicked()
                    {
                        copilot_state.clear_history();
                    }
                });

                if let Some(err) = reload_error {
                    ui.label(RichText::new(err).size(14.0).color(button_danger));
                }

                ui.separator();

                ui.collapsing(
                    RichText::new("System Prompt").color(text_light).strong(),
                    |ui| {
                        if ui
                            .add(
                                TextEdit::multiline(&mut copilot_state.system_prompt)
                                    .desired_width(available_width - 4.0)
                                    .desired_rows(3)
                                    .font(FontSelection::FontId(FontId::new(
                                        14.0,
                                        FontFamily::Proportional,
                                    )))
                                    .text_color(text_light)
                                    .background_color(input_bg),
                            )
                            .changed()
                        {
                            needs_save_ai = true;
                        }
                    },
                );

                ui.collapsing(
                    RichText::new(format!("Buffers ({})", copilot_state.buffers.len()))
                        .color(text_light)
                        .strong(),
                    |ui| {
                        if let Some(text) = selected_text {
                            if !text.is_empty()
                                && ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("Add Selection to Buffer")
                                                .size(14.0)
                                                .color(Color32::WHITE),
                                        )
                                        .fill(Color32::from_rgb(60, 100, 180)),
                                    )
                                    .clicked()
                            {
                                copilot_state.add_buffer(text.to_string());
                                needs_save_ai = true;
                            }
                        }
                        let buffer_names = [
                            "first", "second", "third", "fourth", "fifth", "sixth", "seventh",
                            "eighth", "ninth", "tenth",
                        ];
                        let mut to_remove = None;
                        for (i, buf) in copilot_state.buffers.iter().enumerate() {
                            if buf.is_empty() {
                                continue;
                            }
                            let name = buffer_names.get(i).copied().unwrap_or("?");
                            let summary = make_brief_summary(buf, 30);
                            ui.horizontal(|ui| {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("×").size(14.0).color(button_danger),
                                        )
                                        .fill(Color32::from_rgb(80, 40, 40)),
                                    )
                                    .clicked()
                                {
                                    to_remove = Some(i);
                                }
                                let text = format!("#{} ({}): {} ({})", i, name, summary, buf.chars().count());
                                ui.add_sized(
                                    egui::vec2(ui.available_width(), 0.0),
                                    egui::Label::new(RichText::new(text).size(14.0).color(text_gray)).wrap(),
                                );
                            });
                        }
                        if let Some(i) = to_remove {
                            copilot_state.remove_buffer(i);
                            needs_save_ai = true;
                        }
                    },
                );

                ui.separator();

                ScrollArea::vertical()
                    .id_salt("copilot_output")
                    .auto_shrink([false, false])
                    .max_height(ui.available_height() - 120.0)
                    .show(ui, |ui| {
                        let mut last_assistant_index = None;
                        for (msg_idx, msg) in copilot_state.messages.iter().enumerate() {
                            if msg.role == "assistant" {
                                last_assistant_index = Some(msg_idx);
                            }
                        }

                        let mut prompts_to_add: Vec<(String, String)> = Vec::new();
                        let mut outputs_to_insert_from_history: Vec<String> = Vec::new();
                        let mut message_to_delete: Option<usize> = None;

                        for (msg_idx, msg) in copilot_state.messages.iter().enumerate() {
                            let (label, color) = match msg.role.as_str() {
                                "user" => ("User", user_color),
                                "assistant" => ("Assistant", assistant_color),
                                _ => ("Unknown", text_gray),
                            };
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("{}:", label))
                                        .size(14.0)
                                        .color(color)
                                        .strong(),
                                );
                                if msg.role == "user" {
                                    let msg_display = msg.display.clone();
                                    let prompt_name = make_brief_summary(&msg_display, 20);
                                    let btn = ui.add(
                                        egui::Button::new(
                                            RichText::new("+").size(14.0).color(Color32::WHITE),
                                        )
                                        .fill(button_primary)
                                        .min_size(egui::Vec2::new(20.0, 20.0)),
                                    );
                                    if btn.clicked() {
                                        prompts_to_add.push((prompt_name, msg_display));
                                    }
                                }
                                let delete_btn = ui.add(
                                    egui::Button::new(
                                        RichText::new("Delete").size(12.0).color(Color32::WHITE),
                                    )
                                    .fill(button_danger),
                                );
                                if delete_btn.clicked() {
                                    message_to_delete = Some(msg_idx);
                                }
                                if msg.role == "assistant" && last_assistant_index == Some(msg_idx)
                                {
                                    let msg_content = msg.content.clone();
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new("Insert")
                                                    .size(12.0)
                                                    .color(Color32::WHITE),
                                            )
                                            .fill(Color32::from_rgb(60, 100, 180)),
                                        )
                                        .clicked()
                                    {
                                        outputs_to_insert_from_history.push(msg_content);
                                    }
                                }
                            });
                            let display_text = if msg.role == "user" {
                                &msg.display
                            } else {
                                &msg.content
                            };
                            ui.add(
                                TextEdit::multiline(&mut display_text.clone())
                                    .desired_width(available_width - 4.0)
                                    .font(FontSelection::FontId(FontId::new(
                                        14.0,
                                        FontFamily::Proportional,
                                    )))
                                    .text_color(text_light)
                                    .background_color(input_bg)
                                    .interactive(false),
                            );
                            ui.separator();
                        }

                        for (name, prompt) in prompts_to_add {
                            copilot_state
                                .favorite_prompts
                                .push(FavoritePrompt { name, prompt });
                            needs_save_ai = true;
                        }
                        if let Some(content) = outputs_to_insert_from_history.first() {
                            output_to_insert = Some(content.clone());
                        }
                        if let Some(msg_idx) = message_to_delete {
                            if msg_idx < copilot_state.messages.len() {
                                copilot_state.messages.remove(msg_idx);
                            }
                        }

                        if copilot_state.waiting || !copilot_state.output.is_empty() {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Assistant:")
                                        .size(14.0)
                                        .color(assistant_color)
                                        .strong(),
                                );
                                if !copilot_state.waiting && !copilot_state.output.is_empty() {
                                    let output_clone = copilot_state.output.clone();
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new("Insert")
                                                    .size(12.0)
                                                    .color(Color32::WHITE),
                                            )
                                            .fill(Color32::from_rgb(60, 100, 180)),
                                        )
                                        .clicked()
                                    {
                                        output_to_insert = Some(output_clone);
                                    }
                                }
                            });
                            ui.add(
                                TextEdit::multiline(&mut copilot_state.output.clone())
                                    .desired_width(available_width - 4.0)
                                    .font(FontSelection::FontId(FontId::new(
                                        14.0,
                                        FontFamily::Proportional,
                                    )))
                                    .text_color(text_light)
                                    .background_color(input_bg)
                                    .interactive(false),
                            );
                        }
                    });

                ui.horizontal(|ui| {
                    MenuButton::from_button(
                        egui::Button::new(RichText::new("Favs").color(text_light)).fill(input_bg),
                    )
                    .ui(ui, |ui| {
                        for fav in &copilot_state.favorite_prompts {
                            if egui::Button::new(RichText::new(&fav.name).color(text_light))
                                .fill(bg)
                                .ui(ui)
                                .clicked()
                            {
                                copilot_state.user_input = fav.prompt.clone();
                                ui.close();
                            }
                        }
                    });
                    ui.label(
                        RichText::new("#0-#9 buffers, <all> current passage")
                            .size(12.0)
                            .color(text_gray),
                    );
                });

                ui.add(
                    TextEdit::multiline(&mut copilot_state.user_input)
                        .desired_width(available_width - 4.0)
                        .desired_rows(3)
                        .font(FontSelection::FontId(FontId::new(
                            14.0,
                            FontFamily::Proportional,
                        )))
                        .hint_text("Ctrl+Enter to send")
                        .text_color(text_light)
                        .background_color(input_bg),
                );

                ui.horizontal(|ui| {
                    if copilot_state.waiting {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Stop").size(14.0).color(Color32::WHITE),
                                )
                                .fill(button_danger),
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
                                .fill(button_primary),
                            )
                            .clicked()
                            || ui
                                .ctx()
                                .input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl)
                        {
                            copilot_state.send_message(config, current_passage);
                        }
                    }
                });
            });
        });
    (output_to_insert, needs_save_ai)
}
