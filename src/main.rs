#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")] // For hiding the window in release builds, `cargo run --release --no-default-features`.

pub mod analyzer;
pub use analyzer::AnalyzerResults;
pub use analyzer::MicrophoneAnalyzer;
pub mod repeat_timer;
pub use repeat_timer::RepeatTimer;
pub mod binding;
pub use binding::Binding;
pub mod active_window;
pub use active_window::ActiveWindow;
pub mod key_end;
pub use key_end::key_end;

use eframe::egui;
use enigo::*;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;

const DEFAULT_LOOP_RATE: u32 = 90;
const NO_BINDING_SELECTED: &str = "No Binding Selected";

fn main() -> Result<(), eframe::Error> {
    let (t_mic_note, r_mic_note) = mpsc::sync_channel::<AnalyzerResults>(1);
    let (t_mic, r_mic) = mpsc::channel::<AnalyzerMessage>();
    let (t_gui, r_gui) = mpsc::channel::<GuiMessage>();

    // LOGIC

    // Handles microphone input and note detection.
    // This is spawned on a seperate thread to avoid fixing the input detection to a specific refresh rate GUIs usually have.
    let analyzer_thread = std::thread::spawn(move || {
        let mut mic = MicrophoneAnalyzer::new();
        let mut enigo = Enigo::new();
        let mut active = true;

        let mut binding = None;

        let mut fps_timer = RepeatTimer::new(Duration::from_secs_f64(1.0));
        let mut frames = 0;

        // For limiting the frame rate.
        let mut loop_start;
        let mut loop_rate = Duration::from_secs_f64(1.0 / DEFAULT_LOOP_RATE as f64);

        loop {
            loop_start = Instant::now();

            frames += 1;
            if fps_timer.tick() {
                t_mic.send(AnalyzerMessage::Fps { fps: frames }).unwrap();
                frames = 0;
            }

            // handles messages
            while let Ok(msg) = r_gui.try_recv() {
                match msg {
                    GuiMessage::ActiveChanged { active: new_active } => active = new_active,
                    GuiMessage::BindingChanged {
                        binding: new_binding,
                    } => binding = Some(new_binding),
                    GuiMessage::LoopRateChanged {
                        loop_rate: new_loop_rate,
                    } => loop_rate = Duration::from_secs_f64(1.0 / new_loop_rate as f64),
                    GuiMessage::Exiting => return (),
                }
            }

            // handles note processing
            let note = mic.update_and_analyze();
            if active {
                let mut err = None;
                if let Some(binding) = &mut binding {
                    if let Err(error) = binding.process(&note, &mut enigo) {
                        err = Some(error.to_string());
                    }
                }
                if let Some(err) = err {
                    binding = None;
                    t_mic.send(AnalyzerMessage::ScriptError { err }).unwrap();
                }
            }

            // It is okay if the send fails.
            #[allow(unused_must_use)]
            {
                t_mic_note.try_send(note.clone());
            }

            // how_long_it_should_take - how_long_it_actually_took
            if let Some(delta) = loop_rate.checked_sub(loop_start.elapsed()) {
                // Sleep here to prevent consuming too much processing power
                spin_sleep::sleep(delta); // `spin_sleep::sleep` because `std::thread::sleep` is fairly un-accurate
            }
        }
    });

    // GUI

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(370.0, 300.0)),
        icon_data: Some(
            eframe::IconData::try_from_png_bytes(include_bytes!("../icon.png")).unwrap(),
        ),
        drag_and_drop_support: true,
        ..Default::default()
    };

    // Application state:
    let mut note = pitch_detector::core::NoteName::C;
    let mut pitch = 0.0;
    let mut microphone_power = 1.0;
    let mut active = true;
    let mut current_binding = NO_BINDING_SELECTED.to_string();
    let mut loop_rate = DEFAULT_LOOP_RATE;
    let mut fps = DEFAULT_LOOP_RATE;
    let mut show_analyzer = true;
    let mut always_on_top = false;
    let t_gui_egui = t_gui.clone();

    eframe::run_simple_native("Musical Bindings", options, move |ctx, frame| {
        if let Ok(analyzed_note) = r_mic_note.try_recv() {
            microphone_power = analyzed_note.power;
            // A minimum threshold of `0.025` to prevent stuttering artifacts in the GUI.
            if microphone_power > 0.025 {
                pitch = analyzed_note.pitch;
                if let Some(analyzed_note) = &analyzed_note.note {
                    note = analyzed_note.note_name.clone();
                }
            }
        }

        // handles messages
        while let Ok(msg) = r_mic.try_recv() {
            match msg {
                AnalyzerMessage::Fps { fps: new_fps } => fps = new_fps,
                AnalyzerMessage::ScriptError { err } => {
                    current_binding = NO_BINDING_SELECTED.to_string();
                    rfd::MessageDialog::new()
                        .set_level(rfd::MessageLevel::Error)
                        .set_title("Error in Script Binding")
                        .set_description(&err)
                        .show();
                }
            }
        }

        if active && key_end() {
            active = false;
            t_gui_egui
                .send(GuiMessage::ActiveChanged { active })
                .unwrap();
        }

        if show_analyzer {
            ctx.request_repaint();
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            // BINDING
            // Handles drag and drop
            ctx.input(|i| {
                if !i.raw.dropped_files.is_empty() {
                    for file in &i.raw.dropped_files {
                        if let Some(path) = &file.path {
                            match Binding::from_path(&path) {
                                Ok(binding) => {
                                    current_binding = binding.name().to_string();
                                    t_gui_egui
                                        .send(GuiMessage::BindingChanged { binding })
                                        .unwrap();
                                }
                                Err(err) => {
                                    rfd::MessageDialog::new()
                                        .set_level(rfd::MessageLevel::Error)
                                        .set_title("Error Loading Script Binding")
                                        .set_description(&err.to_string())
                                        .show();
                                }
                            }
                            break;
                        }
                    }
                }
            });
            // Handles binding button
            ui.horizontal(|ui| {
                if ui.button("Load Binding").clicked() {
                    if let Some(file) = rfd::FileDialog::new()
                        .add_filter("lua", &["lua"])
                        .set_directory("/")
                        .pick_file()
                    {
                        match Binding::from_path(&file) {
                            Ok(binding) => {
                                current_binding = binding.name().to_string();
                                t_gui_egui
                                    .send(GuiMessage::BindingChanged { binding })
                                    .unwrap();
                            }
                            Err(err) => {
                                rfd::MessageDialog::new()
                                    .set_level(rfd::MessageLevel::Error)
                                    .set_title("Error Loading Script Binding")
                                    .set_description(&err.to_string())
                                    .show();
                            }
                        }
                    }
                }
                ui.label(&current_binding);
            });
            if ui.checkbox(&mut active, "Active").changed() {
                t_gui_egui
                    .send(GuiMessage::ActiveChanged { active })
                    .unwrap();
            }
            ui.checkbox(&mut show_analyzer, "Show Analyzer Feedback");
            ui.separator();

            // ANALYZER
            if show_analyzer {
                ui.heading("Analyzer Feedback");
                ui.add(egui::Slider::new(&mut microphone_power, 0.0..=1.0).text("power"));
                ui.add(egui::Slider::new(&mut pitch, 0.0..=999.0).text("pitch"));
                ui.label(format!("note: {note}"));
                ui.add(
                    egui::Slider::new(&mut fps, 30..=255).text("Actual `process` calls per second"),
                );
                ui.separator();
            }

            // SETTINGS
            ui.heading("Settings");
            if ui
                .add(egui::Slider::new(&mut loop_rate, 30..=255).text("`process` calls per second"))
                .changed()
            {
                t_gui_egui
                    .send(GuiMessage::LoopRateChanged { loop_rate })
                    .unwrap();
            }
            if ui
                .checkbox(&mut always_on_top, "Window Always on Top")
                .changed()
            {
                frame.set_always_on_top(always_on_top);
            }
            ui.label("Press `END` to deactivate musical bindings.");
        });
    })?;

    t_gui.send(GuiMessage::Exiting).unwrap();
    analyzer_thread.join().unwrap();

    Ok(())
}

enum GuiMessage {
    ActiveChanged { active: bool },
    BindingChanged { binding: Binding },
    LoopRateChanged { loop_rate: u32 },
    Exiting,
}

enum AnalyzerMessage {
    Fps { fps: u32 },
    ScriptError { err: String },
}
