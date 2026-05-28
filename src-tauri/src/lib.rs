pub mod cipher;
pub mod commands;
pub mod consts;
pub mod data_structures;
pub mod encode;
pub mod error;
pub mod png;
pub mod safe_note;
pub mod config;
pub mod state;
pub mod agent;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::file::list_files,
            commands::file::create_file,
            commands::file::open_file,
            commands::file::delete_file,
            commands::file::refresh_files,
            commands::cipher::decrypt_file,
            commands::cipher::encrypt_and_save,
            commands::cipher::change_password,
            commands::cipher::close_file,
            commands::cipher::import_safe_notes,
            commands::editor::get_passages,
            commands::editor::update_passage_content,
            commands::editor::update_passage_title,
            commands::editor::add_passage,
            commands::editor::remove_passage,
            commands::editor::move_passage,
            commands::editor::insert_image,
            commands::editor::get_images,
            commands::editor::get_image_metadata,
            commands::editor::delete_image,
            commands::editor::find_referenced_digests,
            commands::editor::get_current_file,
            commands::editor::set_current_passage,
            commands::editor::get_current_passage_index,
            commands::editor::is_dirty,
            commands::editor::append_file,
            commands::editor::read_clipboard_images,
            commands::editor::read_image_files,
            commands::config::get_config,
            commands::config::update_config,
            commands::config::get_data_dir,
            commands::copilot::send_message,
            commands::copilot::abort_generation,
            commands::copilot::update_copilot_settings,
            commands::copilot::save_ai_settings,
            commands::copilot::load_ai_settings,
            commands::copilot::clear_copilot,
            commands::copilot::reset_copilot_settings,
            commands::state::get_app_state,
            // Agent commands
            commands::agent::get_workspace,
            commands::agent::get_characters,
            commands::agent::add_character,
            commands::agent::update_character,
            commands::agent::remove_character,
            commands::agent::get_relationships,
            commands::agent::add_relationship,
            commands::agent::update_relationship,
            commands::agent::remove_relationship,
            commands::agent::get_kv_store,
            commands::agent::set_kv,
            commands::agent::delete_kv,
            commands::agent::get_session,
            commands::agent::edit_session_message,
            commands::agent::delete_session_message,
            commands::agent::compress_session,
            commands::agent::clear_session,
            commands::agent::get_tool_definitions,
            commands::agent::confirm_tool_call,
            commands::agent::execute_confirmed_tool,
            commands::agent::get_pending_tool_calls,
            commands::agent::send_agent_message,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}