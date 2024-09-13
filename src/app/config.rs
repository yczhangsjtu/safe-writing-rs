use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct Config {
    pub(super) font_size: f32,
    pub(super) data_dir: String,
}
