use serde::{Deserialize, Serialize};

fn default_llamacpp_url() -> String {
    "http://localhost:8080".to_string()
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub(crate) struct Config {
    pub(super) font_size: f32,
    pub(super) data_dir: String,
    #[serde(default = "default_llamacpp_url")]
    pub(super) llamacpp_url: String,
}
