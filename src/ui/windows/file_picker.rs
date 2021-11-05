use std::env;
use std::path::PathBuf;

use imgui::*;

pub struct FilePickerWindow {
    current_path: PathBuf,
    show_dot_entries: bool
}

impl FilePickerWindow {
    pub fn init(current_path: PathBuf) -> FilePickerWindow {
        let current_path = if current_path.exists() {current_path} else {env::current_dir().unwrap_or_else(|_| PathBuf::new())};
        
        FilePickerWindow {
            current_path,
            show_dot_entries: false
        }
    }

    pub fn draw(&mut self, ui: &Ui) -> Option<PathBuf> {
        let mut chosen_file = None;

        if let Some(_token) = PopupModal::new("File Picker").begin_popup(ui) {
            if self.current_path.exists() {
                ListBox::new("").size([400.0, 200.0]).build(ui, || {
                    if let Ok(mut entries) = self.current_path.read_dir() {
                        let mut dirs = Vec::new();
                        let mut files = Vec::new();

                        while let Some(Ok(entry)) = entries.next() {
                            if let Ok(metadata) = entry.metadata() {
                                if metadata.is_dir() {
                                    if entry.file_name().to_string_lossy().starts_with('.') {
                                        if self.show_dot_entries {
                                            dirs.push(entry);
                                        }
                                    }
                                    else {
                                        dirs.push(entry);
                                    }
                                }
                                else if entry.file_name().to_string_lossy().starts_with('.') {
                                    if self.show_dot_entries {
                                        files.push(entry);
                                    }
                                }
                                else {
                                    files.push(entry);
                                }
                            }
                        }

                        if let Some(parent) = self.current_path.parent() {
                            if Selectable::new("../").build(ui) {
                                self.current_path = PathBuf::from(parent);
                            }
                        }

                        dirs.sort_by_key(|a| a.path().as_os_str().to_ascii_lowercase());
                        files.sort_by_key(|a| a.path().as_os_str().to_ascii_lowercase());

                        for dir in dirs {
                            if let Some(path) = dir.file_name().to_str() {
                                let path = format!("{}/", path);

                                if Selectable::new(&ImString::from(path)).build(ui) {
                                    self.current_path = dir.path();
                                }
                            }
                        }

                        for file in files {
                            if let Some(path) = file.file_name().to_str() {
                                let path = path.to_string();

                                if Selectable::new(&ImString::from(path)).build(ui) {
                                    chosen_file = Some(file.path());
                                    ui.close_current_popup();
                                }
                            }
                        }
                    }
                });

                ui.checkbox("Show entries starting with .", &mut self.show_dot_entries);
            }
            else {
                ui.text_colored([1.0, 0.0, 0.0, 1.0], "Couldn't open current path.");
            }
        };

        ui.open_popup("File Picker");

        chosen_file
    }
}
