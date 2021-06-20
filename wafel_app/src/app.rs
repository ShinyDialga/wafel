use std::collections::HashMap;

use imgui::{self as ig, im_str};
use wafel_api::{load_m64, SM64Version};

use crate::{
    config::{
        default_unlocked_game_version, is_game_version_unlocked, known_game_versions,
        locked_game_versions, unlocked_game_versions,
    },
    project::{Project, TasFileInfo},
};

/// The top level Wafel app state.
#[derive(Debug)]
pub struct App {
    pending_tas: Option<TasFileInfo>,
    project: Option<Project>,
    game_version_errors: HashMap<SM64Version, String>,
}

impl App {
    /// Create a new app state.
    pub fn open() -> Self {
        Self {
            pending_tas: None,
            project: None,
            game_version_errors: HashMap::new(),
        }
    }

    /// Render the app.
    pub fn render(&mut self, ui: &ig::Ui<'_>) {
        // Main app UI
        ig::Window::new(im_str!("Main"))
            .position([0.0, 0.0], ig::Condition::Always)
            .size(
                [ui.io().display_size[0], ui.io().display_size[1]],
                ig::Condition::Always,
            )
            .resizable(false)
            .title_bar(false)
            .menu_bar(true)
            .bring_to_front_on_focus(false)
            .build(&ui, || {
                // If no project is open and at least one libsm64 version has been unlocked,
                // create an empty project.
                if self.project.is_none() && self.pending_tas.is_none() {
                    if let Some(game_version) = default_unlocked_game_version() {
                        self.project = Some(Project::empty(game_version));
                    } else {
                        // No game versions are unlocked yet
                        ui.open_popup(im_str!("Game versions##game-versions"));
                    }
                }

                // If a TAS is waiting to be opened
                if let Some(pending_tas) = &self.pending_tas {
                    if is_game_version_unlocked(pending_tas.game_version) {
                        // If the required game version is unlocked, load the TAS
                        self.project = Some(Project::with_m64(pending_tas));
                        self.pending_tas = None;
                    } else {
                        // Otherwise show game version popup so that user can unlock it
                        ui.open_popup(im_str!("Game versions##game-versions"));
                    }
                }

                // Menu bar
                self.menu_bar(ui);

                // Project content
                if let Some(project) = &mut self.project {
                    project.render(ui);
                }

                // Controller bindings popup
                ui.popup_modal(im_str!("Controller##settings-controller"))
                    .opened(&mut true)
                    .resizable(false)
                    .build(|| {});

                // Keyboard bindings popup
                ui.popup_modal(im_str!("Key bindings##settings-key-bindings"))
                    .opened(&mut true)
                    .resizable(false)
                    .build(|| {});

                // Game versions popup
                let mut opened = true;
                ui.popup_modal(im_str!("Game versions##game-versions"))
                    .opened(&mut opened)
                    .resizable(false)
                    .always_auto_resize(true)
                    .build(|| self.game_versions_popup(ui));
                if !opened && self.pending_tas.is_some() {
                    self.pending_tas = None;
                }
            });
    }

    fn menu_bar(&mut self, ui: &ig::Ui<'_>) {
        let mut open_popup: Option<&str> = None;

        ui.main_menu_bar(|| {
            ui.menu(im_str!("File"), true, || {
                if ig::MenuItem::new(im_str!("New")).build(ui) {
                    self.new_project();
                }
                if ig::MenuItem::new(im_str!("Open")).build(ui) {
                    self.open_m64();
                }
                if ig::MenuItem::new(im_str!("Save"))
                    .enabled(self.project.is_some())
                    .build(ui)
                {
                    self.save_m64();
                }
                if ig::MenuItem::new(im_str!("Save as"))
                    .enabled(self.project.is_some())
                    .build(ui)
                {
                    self.save_m64_as();
                }
                ui.menu(im_str!("Game version"), true, || {
                    if let Some(name) = self.game_version_menu(ui) {
                        open_popup = Some(name);
                    }
                });
            });

            ui.menu(im_str!("Settings"), true, || {
                if ig::MenuItem::new(im_str!("Controller")).build(ui) {
                    open_popup = Some("Controller##settings-controller");
                }
                if ig::MenuItem::new(im_str!("Key bindings")).build(ui) {
                    open_popup = Some("Key bindings##settings-key-bindings");
                }
            });
        });

        if let Some(name) = open_popup {
            ui.open_popup(&im_str!("{}", name));
        }
    }

    fn new_project(&mut self) {
        // Use the current project's game version if possible
        let game_version = self
            .project
            .take()
            .map(|p| p.game_version())
            .or_else(default_unlocked_game_version);
        if let Some(game_version) = game_version {
            self.project = Some(Project::empty(game_version));
        }
    }

    fn open_m64(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Mupen64 TAS", &["m64"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            // TODO: Error handling for invalid M64 file and unknown game version
            let filename = path.as_os_str().to_str().expect("invalid filename");
            let (metadata, inputs) = load_m64(filename);
            let game_version = metadata.version().expect("unknown game version");

            self.pending_tas = Some(TasFileInfo {
                game_version,
                filename: filename.to_string(),
                metadata,
                inputs,
            });
        }
    }

    fn save_m64(&mut self) {
        let project = self.project.as_ref().expect("no open project");
        if project.filename().is_some() {
            project.save_m64();
        } else {
            self.save_m64_as();
        }
    }

    fn save_m64_as(&mut self) {
        let project = self.project.as_mut().expect("no open project");
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Mupen64 TAS", &["m64"])
            .add_filter("All Files", &["*"])
            .save_file()
        {
            project.set_filename(path.as_os_str().to_str().expect("invalid filename"));
            project.save_m64();
        }
    }

    fn game_version_menu(&mut self, ui: &ig::Ui<'_>) -> Option<&'static str> {
        let mut open_popup = None;

        // Show unlocked game versions in the menu
        let loaded_version = self.project.as_ref().map(|p| p.game_version());
        for version in unlocked_game_versions() {
            if ig::MenuItem::new(&im_str!("{}", version))
                .selected(loaded_version == Some(version))
                .build(ui)
            {
                if let Some(project) = &mut self.project {
                    project.change_game_version(version);
                }
            }
        }
        // If other locked versions exist, show an option to open the game version
        // popup
        if !locked_game_versions().is_empty() && ig::MenuItem::new(im_str!("Other")).build(ui) {
            open_popup = Some("Game versions##game-versions");
        }

        open_popup
    }

    fn game_versions_popup(&mut self, ui: &ig::Ui<'_>) {
        if let Some(pending_tas) = &self.pending_tas {
            ui.text(im_str!(
                "Unlock version {} using a vanilla SM64 ROM to open the selected TAS",
                pending_tas.game_version
            ));
        } else if unlocked_game_versions().is_empty() {
            ui.text(im_str!("Wafel requires a vanilla SM64 ROM to run"));
        } else if !locked_game_versions().is_empty() {
            ui.text(im_str!(
                "Unlock additional game versions using a vanilla SM64 ROM"
            ));
        }

        ui.dummy([1.0, 5.0]);

        for version in known_game_versions() {
            let id_token = ui.push_id(&format!("version-{}", version));

            let is_locked = !is_game_version_unlocked(version);
            if !is_locked {
                self.game_version_errors.remove(&version);
            }

            ui.separator();
            ui.text(im_str!(
                "SM64 {} - {}",
                version,
                if is_locked { "locked" } else { "unlocked" }
            ));

            if is_locked {
                ui.same_line(0.0);
                ui.dummy([3.0, 1.0]);
                ui.same_line(0.0);

                if ui.button(im_str!("Select ROM"), [0.0, 0.0]) {
                    if let Some(rom_path) = rfd::FileDialog::new()
                        .add_filter("N64 ROM", &["n64", "z64"])
                        .add_filter("All Files", &["*"])
                        .pick_file()
                    {
                        // TODO: Game version unlocking
                        // TODO: Log detailed error message to log::error
                        let error_message =
                            format!("Error: ROM did not match vanilla {} ROM", version);
                        self.game_version_errors.insert(version, error_message);
                    }
                }
            }

            if let Some(error) = self.game_version_errors.get(&version) {
                ui.text(im_str!("{}", error));
            }

            id_token.pop(ui);
        }

        ui.separator();
        ui.dummy([1.0, 5.0]);
    }
}