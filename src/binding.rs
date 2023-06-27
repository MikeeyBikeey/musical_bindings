//! Musical bindings and scripting
use crate::{ActiveWindow, AnalyzerResults};
use enigo::*;
use mlua::{Function, Lua, Result, String as LuaString};
use std::path::Path;

pub struct Binding {
    lua: Lua,
    name: String,
    /// Whether or not the script accepts the currently active window.
    script_accepts_window: bool,
    active_window: ActiveWindow,
}

impl Binding {
    pub fn from_path(path: &Path) -> Result<Self> {
        Binding::from_bytes(
            &std::fs::read(path)?,
            path.file_name()
                .map(|file_name| file_name.to_str())
                .flatten()
                .unwrap_or("expected valid file name"),
        )
    }

    pub fn from_bytes(bytes: &[u8], name: &str) -> Result<Self> {
        let lua = Lua::new();
        lua.load(bytes).set_name(name)?.exec()?;
        Ok(Binding {
            lua,
            name: name.to_string(),
            script_accepts_window: false,
            active_window: Default::default(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn process(&mut self, note: &AnalyzerResults, enigo: &mut Enigo) -> Result<()> {
        let globals = self.lua.globals();
        if let Some(note) = &note.note {
            globals.set("note", note.note_name.to_string())?;
            globals.set("note_frequency", note.note_freq)?;
            globals.set("frequency", note.actual_freq)?;
            globals.set("octave", note.octave)?;
            globals.set("cents_offset", note.cents_offset)?;
            globals.set("in_tune", note.in_tune)?;
        }
        globals.set("pitch", note.pitch)?;
        globals.set("power", note.power)?;

        self.lua.scope(|scope| {
            globals.set(
                "keys",
                scope.create_function_mut(|_, (key, is_up): (LuaString, bool)| {
                    let input_mode: LuaString = globals.get("input_mode")?;
                    let input_mode = input_mode.to_str().unwrap_or("keyboard");
                    for key in key.to_str()?.chars() {
                        // WINDOW CHECK
                        if self.active_window.changed() {
                            let accepts_window: Function = globals.get("accepts")?;
                            self.script_accepts_window =
                                accepts_window.call::<_, bool>(self.active_window.name())?;
                        }
                        if !self.script_accepts_window {
                            continue;
                        }

                        // KEY INPUTS
                        match input_mode {
                            "character" => {
                                let mut b = [0; 4];
                                let key = key.encode_utf8(&mut b);
                                // `key_sequence` works differently from `key_up` and `key_down` (which is why there is a setting here)
                                // Look at the Windows implementation of `Enigo`
                                enigo.key_sequence(key);
                            }
                            _ => {
                                // "keyboard" => {
                                let key = key.to_ascii_lowercase();
                                if is_up {
                                    enigo.key_up(enigo::Key::Layout(key));
                                } else {
                                    enigo.key_down(enigo::Key::Layout(key));
                                }
                            }
                        }
                    }
                    Ok(())
                })?,
            )?;

            globals.set(
                "keys_down",
                scope.create_function_mut(|_, key: LuaString| {
                    let keys: Function = globals.get("keys")?;
                    keys.call::<_, ()>((key, false))?;
                    Ok(())
                })?,
            )?;

            globals.set(
                "keys_up",
                scope.create_function_mut(|_, key: LuaString| {
                    let keys: Function = globals.get("keys")?;
                    keys.call::<_, ()>((key, true))?;
                    Ok(())
                })?,
            )?;

            let process_function: Function = globals.get("process")?;
            process_function.call::<_, ()>(())?;

            Ok(())
        })?;

        Ok(())
    }
}
