#![windows_subsystem = "windows"]
mod mem_lib;
use std::env;
use std::io;
use std::path;

use cgmath;
use fluent_bundle::{FluentBundle, FluentResource};
use ggez::event::{self, Axis, Button, GamepadId, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{self, Align, Color, DrawParam, Font, Scale, Text, TextFragment};
use ggez::*;
use image;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::f32;
use webbrowser;

#[derive(PartialEq)]
enum SCState {
    FindingProcess,
    FindingModule,
    FailToReadMem,
    FindingUmsbankMap,
}

struct MainState<'a> {
    font: graphics::Font,
    mouse_down: bool,
    pos_x: f32,
    pos_y: f32,
    locale: &'a str,
    fluent_bundles: HashMap<&'a str, FluentBundle<'a>, RandomState>,
    assets: Assets,
    state: SCState,
    wait: u8,
}
struct Assets {
    background_image: graphics::Image,
    update_button: graphics::Image,
    language_button: graphics::Image,
    homepage_button: graphics::Image,
    folder_button: graphics::Image,
    hover_button: graphics::Image,
}

impl Assets {
    fn new(ctx: &mut Context) -> GameResult<Assets> {
        // TODO: compress assets in binary
        let background = image::load_from_memory(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/bc2017console.jpg"
        )))?
        .to_rgba();
        let (width, height) = background.dimensions();
        let background_image =
            graphics::Image::from_rgba8(ctx, width as u16, height as u16, &background)?;

        let update = image::load_from_memory(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/update.png"
        )))?
        .to_rgba();
        let (width, height) = update.dimensions();
        let update_button = graphics::Image::from_rgba8(ctx, width as u16, height as u16, &update)?;

        let language = image::load_from_memory(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/language.png"
        )))?
        .to_rgba();
        let (width, height) = language.dimensions();
        let language_button =
            graphics::Image::from_rgba8(ctx, width as u16, height as u16, &language)?;

        let homepage = image::load_from_memory(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/homepage.jpg"
        )))?
        .to_rgba();
        let (width, height) = homepage.dimensions();
        let homepage_button =
            graphics::Image::from_rgba8(ctx, width as u16, height as u16, &homepage)?;

        let folder = image::load_from_memory(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/folder.png"
        )))?
        .to_rgba();
        let (width, height) = folder.dimensions();
        let folder_button = graphics::Image::from_rgba8(ctx, width as u16, height as u16, &folder)?;

        let hover = image::load_from_memory(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/hover.png"
        )))?
        .to_rgba();
        let (width, height) = hover.dimensions();
        let hover_button = graphics::Image::from_rgba8(ctx, width as u16, height as u16, &hover)?;

        Ok(Assets {
            background_image,
            update_button,
            language_button,
            homepage_button,
            folder_button,
            hover_button,
        })
    }
}

impl<'a> MainState<'a> {
    pub fn get_text(&self, id: &str) -> String {
        let (value, errors) = self
            .fluent_bundles
            .get(self.locale)
            .unwrap()
            .format(id, None)
            .expect("Failed to format a message.");
        value
    }
}

impl<'a> event::EventHandler for MainState<'a> {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.wait > 0 {
            self.wait -= 1;
            return Ok(());
        }
        // TODO: reduce winapi calls
        let proc = match mem_lib::get_proc_by_name("StarCraft.exe") {
            Ok(proc) => proc,
            Err(_) => {
                self.state = SCState::FindingProcess;
                self.wait = 127;
                return Ok(());
            }
        };
        let mut module = match proc.get_module("StarCraft.exe") {
            Ok(module) => module,
            Err(_) => {
                self.state = SCState::FindingModule;
                return Ok(());
            }
        };
        let use_umsbank = match module.read::<u32>(0xC04E28 + 212, &proc) {
            Ok(value) => value,
            Err(_) => {
                self.state = SCState::FailToReadMem;
                return Ok(());
            }
        };
        if use_umsbank != 0x5537F23B {
            self.state = SCState::FindingUmsbankMap;
            return Ok(());
        }
        module
            .write::<u32>(&proc, 0xC04E28 + 212, 0x12341234)
            .unwrap();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        let assets = &mut self.assets;

        let background = &mut assets.background_image;
        let dst = cgmath::Point2::new(0.0, 0.0);
        graphics::draw(ctx, background, (dst,))?;

        let update = &mut assets.update_button;
        let dst = cgmath::Point2::new(128.0, 22.0);
        graphics::draw(ctx, update, (dst,))?;

        let language = &mut assets.language_button;
        let dst = cgmath::Point2::new(149.0, 22.0);
        graphics::draw(ctx, language, (dst,))?;

        let homepage = &mut assets.homepage_button;
        let dst = cgmath::Point2::new(170.0, 22.0);
        graphics::draw(ctx, homepage, (dst,))?;

        let folder = &mut assets.folder_button;
        let dst = cgmath::Point2::new(191.0, 22.0);
        graphics::draw(ctx, folder, (dst,))?;

        if self.pos_y >= 22.0 && self.pos_y <= 42.0 {
            let dst = match self.pos_x {
                128.0...148.0 => Some(("update", 128.0)),
                149.0...169.0 => Some(("change_language", 149.0)),
                170.0...190.0 => Some(("homepage", 170.0)),
                191.0...211.0 => Some(("open_folder", 191.0)),
                _ => None,
            };
            match dst {
                Some((text, p)) => {
                    let hover = &mut assets.hover_button;
                    let dst = cgmath::Point2::new(p, 22.0);
                    graphics::draw(ctx, hover, (dst,))?;
                    let green = graphics::Color::new(0.03, 0.9, 0.03, 1.0);
                    let text = self.get_text(text);
                    let mut text = graphics::Text::new((text, self.font, 12.0));
                    let txtdst = cgmath::Point2::new(332.0, 25.0);
                    text.set_bounds(cgmath::Point2::new(70.0, f32::INFINITY), Align::Center);
                    graphics::draw(ctx, &text, (txtdst, green))?;
                }
                None => (),
            }
        }

        let text = match self.state {
            SCState::FindingProcess => Some("waiting_sc_process"),
            SCState::FindingModule => Some("waiting_sc_module"),
            SCState::FailToReadMem => Some("fail_to_read_memory"),
            SCState::FindingUmsbankMap => Some("waiting_map_using_umsbank"),
            _ => None,
        };
        match text {
            Some(text) => {
                let text = self.get_text(text);
                let mut text = graphics::Text::new((text, self.font, 40.0));
                let txtdst = cgmath::Point2::new(24.0, 68.0);
                text.set_bounds(cgmath::Point2::new(432.0, f32::INFINITY), Align::Center);
                // let green = graphics::Color::new(0.03, 0.9, 0.03, 1.0);
                let light_blue = graphics::Color::new(0.71875, 0.71875, 0.90234375, 1.0);
                graphics::draw(ctx, &text, (txtdst, light_blue))?;
            }
            None => (),
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        self.mouse_down = true;
        if self.pos_y >= 22.0 && self.pos_y <= 42.0 {
            match self.pos_x {
                128.0...148.0 => {
                    // TODO: Implement (Auto)Update
                    println!("Update is not implemented yet!");
                }
                149.0...169.0 => {
                    self.locale = match self.locale {
                        "ko-KR" => "en-US",
                        "en-US" => "ko-KR",
                        _ => "en-US",
                    }
                }
                170.0...190.0 => {
                    webbrowser::open("http://blog.naver.com/kein0011").unwrap();
                }
                191.0...211.0 => {
                    #[cfg(not(windows))]
                    fn open_browser(path: &path::Path) -> io::Result<bool> {
                        use std::process::{Command, Stdio};

                        let env_browser = env::var_os("BROWSER")
                            .map(|b| env::split_paths(&b).collect::<Vec<_>>());
                        let env_commands: Vec<&str> = env_browser
                            .as_ref()
                            .map(|cmds| cmds.iter().by_ref().filter_map(|b| b.to_str()).collect())
                            .unwrap_or_default();

                        let commands = [
                            "xdg-open",
                            "open",
                            "firefox",
                            "chromium",
                            "sensible-browser",
                        ];
                        if let Some(cmd) = find_cmd(&env_commands).or_else(|| find_cmd(&commands)) {
                            Command::new(cmd)
                                .arg(path)
                                .stdin(Stdio::null())
                                .stdout(Stdio::null())
                                .stderr(Stdio::null())
                                .spawn()
                                .map(|_| true)
                        } else {
                            Ok(false)
                        }
                    }
                    #[cfg(windows)]
                    fn open_browser(path: &path::Path) -> io::Result<bool> {
                        use std::ptr;
                        use winapi::ctypes;
                        use winapi::shared::minwindef::HINSTANCE;
                        use winapi::shared::ntdef::LPCWSTR;
                        use winapi::shared::windef::HWND;

                        // FIXME: When winapi has this function, use their version
                        extern "system" {
                            pub fn ShellExecuteW(
                                hwnd: HWND,
                                lpOperation: LPCWSTR,
                                lpFile: LPCWSTR,
                                lpParameters: LPCWSTR,
                                lpDirectory: LPCWSTR,
                                nShowCmd: ctypes::c_int,
                            ) -> HINSTANCE;
                        }
                        const SW_SHOW: ctypes::c_int = 5;

                        let path = windows::to_u16s(path)?;
                        let operation = windows::to_u16s("open")?;
                        let result = unsafe {
                            ShellExecuteW(
                                ptr::null_mut(),
                                operation.as_ptr(),
                                path.as_ptr(),
                                ptr::null(),
                                ptr::null(),
                                SW_SHOW,
                            )
                        };
                        Ok(result as usize > 32)
                    }
                    let path = env::current_dir().unwrap();
                    // path.push("resources");
                    open_browser(&path).unwrap();
                }
                _ => (),
            }
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        self.mouse_down = false;
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, xrel: f32, yrel: f32) {
        self.pos_x = x;
        self.pos_y = y;
    }
}

pub fn main() -> GameResult {
    let resource_dir = path::PathBuf::from("./resources");
    let cb = ContextBuilder::new("umsbank", "Armoha")
        .window_setup(conf::WindowSetup::default().title(&format!(
            "umsbank {} (SC:R 1.22.3.5482)",
            env!("CARGO_PKG_VERSION")
        )))
        .window_mode(conf::WindowMode::default().dimensions(480.0, 224.0))
        .add_resource_path(resource_dir);
    let (ctx, event_loop) = &mut cb.build()?;

    use std::fs;
    let mut path = env::current_dir().unwrap();
    path.push("resources");
    path.push("ko-KR.ftl");
    let ftl_string = fs::read_to_string(&path).expect("Fail to parse .ftl file.");
    let res = FluentResource::try_new(ftl_string).expect("Could not parse an FTL string.");
    let mut ko_kr_bundle = FluentBundle::new(&["ko-KR"]);
    ko_kr_bundle
        .add_resource(&res)
        .expect("Failed to add FTL resources to the bundle.");
    path.pop();
    path.push("en-US.ftl");
    let ftl_string = fs::read_to_string(&path).expect("Fail to parse .ftl file.");
    let res = FluentResource::try_new(ftl_string).expect("Could not parse an FTL string.");
    let mut en_us_bundle = FluentBundle::new(&["en-US"]);
    en_us_bundle
        .add_resource(&res)
        .expect("Failed to add FTL resources to the bundle.");

    let mut fluent_bundles = HashMap::new();
    fluent_bundles.insert("ko-KR", ko_kr_bundle);
    fluent_bundles.insert("en-US", en_us_bundle);
    /*
    let (value, errors) = bundle.format(id, None)
        .expect("Failed to format a message.");
    value*/

    let font = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/bl.ttf"));
    let font = graphics::Font::new_glyph_font_bytes(ctx, font).unwrap_or_default();
    let assets = Assets::new(ctx)?;

    let state = &mut MainState {
        font,
        mouse_down: false,
        pos_x: 100.0,
        pos_y: 100.0,
        locale: "ko-KR",
        fluent_bundles: fluent_bundles,
        assets: assets,
        state: SCState::FindingProcess,
        wait: 0,
    };
    event::run(ctx, event_loop, state)
}

#[cfg(windows)]
pub mod windows {
    use std::ffi::OsStr;
    use std::io;
    use std::os::windows::ffi::OsStrExt;

    pub fn to_u16s<S: AsRef<OsStr>>(s: S) -> io::Result<Vec<u16>> {
        fn inner(s: &OsStr) -> io::Result<Vec<u16>> {
            let mut maybe_result: Vec<u16> = s.encode_wide().collect();
            if maybe_result.iter().any(|&u| u == 0) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "strings passed to WinAPI cannot contain NULs",
                ));
            }
            maybe_result.push(0);
            Ok(maybe_result)
        }
        inner(s.as_ref())
    }
}
