#![windows_subsystem = "windows"]

use std::{collections::hash_map::RandomState, collections::HashMap, env, f32, io, mem, path};

use cgmath;
use fluent_bundle::{FluentBundle, FluentResource};
use ggez::{
    *,
    audio::SoundSource,
    event::{self, MouseButton},
    graphics::{self, Align, Color, Font, Text},
};
use webbrowser;

mod asset;
mod get_time;
mod mem_lib;
// mod scr;

const STARCRAFT_VERSION: &str = "1.22.4.5993";
const BUFFER_PTR: u32 = 0xBFD6E8;

#[derive(PartialEq)]
enum SCState {
    FindingProcess,
    FindingModule,
    FindingSCBankMap,
    RequestFilename,
    CheckingLatestVersion,
}

enum TextColor {
    Green,
    LightBlue,
    Tan,
}

struct MouseInfo {
    button: MouseButton,
    down: bool,
    x: f32,
    y: f32,
}

struct MainState<'a> {
    font: Font,
    mouse_info: MouseInfo,
    locale: &'a str,
    fluent_bundles: HashMap<&'a str, FluentBundle<'a>, RandomState>,
    assets: asset::Assets,
    state: SCState,
    wait: u8,
    process: mem_lib::GameProcess,
    module: mem_lib::Module,
}

impl<'a> MainState<'a> {
    pub fn get_text(&self, id: &str) -> String {
        let (value, _errors) = self
            .fluent_bundles
            .get(self.locale)
            .unwrap()
            .format(id, None)
            .expect("Failed to format a message.");
        value
    }

    pub fn update_app(&mut self) -> Result<(), Box<::std::error::Error>> {
        use self_update::{self, cargo_crate_version};
        let mut state = SCState::CheckingLatestVersion;
        mem::swap(&mut self.state, &mut state);
        let target = self_update::get_target()?;
        let _releases = self_update::backends::github::ReleaseList::configure()
            .repo_owner("armoha")
            .repo_name("SCBank")
            .with_target(&target)
            .build()?
            .fetch()?;

        let _status = self_update::backends::github::Update::configure()?
            .repo_owner("armoha")
            .repo_name("SCBank")
            .target(&target)
            .bin_name("SCBank.exe")
            .show_download_progress(true)
            .current_version(cargo_crate_version!())
            .build()?
            .update()?;
        mem::swap(&mut self.state, &mut state);
        Ok(())
    }

    pub fn get_sc_proc(&mut self) -> SCState {
        self.process = match mem_lib::get_proc_by_name("StarCraft.exe") {
            Ok(proc) => proc,
            Err(_) => {
                self.wait = 127;
                return SCState::FindingProcess;
            }
        };
        SCState::FindingModule
    }

    pub fn get_sc_module(&mut self) -> SCState {
        self.module = match self.process.get_module("StarCraft.exe") {
            Ok(module) => module,
            Err(_) => {
                return SCState::FindingProcess;
            }
        };
        SCState::FindingSCBankMap
    }

    pub fn check_scbank_map(&mut self) -> SCState {
        self.module
            .write::<u32>(&self.process, 0xBEFB88, 0)
            .unwrap();
        match self.module.read::<u32>(BUFFER_PTR + 212, &self.process) {
            Ok(value) => {
                if value == 0x5537F23B {
                    return SCState::RequestFilename;
                }
                SCState::FindingSCBankMap
            }
            Err(_) => SCState::FindingProcess,
        }
    }
}

trait InRange {
    fn in_range(&self, begin: Self, end: Self) -> bool;
}

impl InRange for f32 {
    fn in_range(&self, begin: f32, end: f32) -> bool {
        *self >= begin && *self < end
    }
}

impl<'a> event::EventHandler for MainState<'a> {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.wait > 0 {
            self.wait -= 1;
            return Ok(());
        }
        self.state = match self.state {
            SCState::FindingProcess => self.get_sc_proc(),
            SCState::FindingModule => self.get_sc_module(),
            SCState::FindingSCBankMap => self.check_scbank_map(),
            SCState::RequestFilename => return Ok(()),
            _ => SCState::FindingProcess,
        };

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 0.0].into());

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

        let mouse = &mut self.mouse_info;
        let (x, y) = (mouse.x, mouse.y);
        if y >= 22.0 && y <= 42.0 && x >= 128.0 && x <= 211.0 {
            let dst = match x {
                t if t.in_range(128.0, 148.0) => Some(("update", 128.0)),
                t if t.in_range(149.0, 169.0) => Some(("change_language", 149.0)),
                t if t.in_range(170.0, 190.0) => Some(("homepage", 170.0)),
                t if t.in_range(191.0, 211.0) => Some(("open_folder", 191.0)),
                _ => None,
            };
            match dst {
                Some((text, p)) => {
                    let hover = &mut assets.hover_button;
                    let dst = cgmath::Point2::new(p, 22.0);
                    graphics::draw(ctx, hover, (dst,))?;

                    let green = Color::new(0.03, 0.9, 0.03, 1.0);
                    let text = self.get_text(text);
                    let (font_size, upper_pos) = match self.locale {
                        "en-US" => (12.0, 25.0),
                        _ => (14.0, 24.0),
                    };
                    let mut text = Text::new((text, self.font, font_size));
                    let txtdst = cgmath::Point2::new(332.0, upper_pos);
                    text.set_bounds(cgmath::Point2::new(70.0, f32::INFINITY), Align::Center);
                    graphics::draw(ctx, &text, (txtdst, green))?;
                }
                None => (),
            }
        } else {
            let tan = Color::new(0.953, 0.851, 0.796, 1.0);
            let text = format!("v{}\n{}", env!("CARGO_PKG_VERSION"), STARCRAFT_VERSION);
            let mut text = Text::new((text, self.font, 11.0));
            let txtdst = cgmath::Point2::new(332.0, 20.0);
            text.set_bounds(cgmath::Point2::new(70.0, f32::INFINITY), Align::Center);
            graphics::draw(ctx, &text, (txtdst, tan))?;
        }

        let text = match self.state {
            SCState::FindingProcess => Some(("waiting_sc_process", TextColor::LightBlue)),
            SCState::FindingModule => Some(("waiting_sc_module", TextColor::LightBlue)),
            SCState::FindingSCBankMap => Some(("waiting_map_using_scbank", TextColor::LightBlue)),
            SCState::RequestFilename => Some(("request_save_file_name", TextColor::LightBlue)),
            SCState::CheckingLatestVersion => Some(("check_latest", TextColor::Tan)),
            _ => None,
        };
        match text {
            Some((text, color)) => {
                let text = self.get_text(text);
                let mut text = Text::new((text, self.font, 40.0));
                let txtdst = cgmath::Point2::new(24.0, 68.0);
                text.set_bounds(cgmath::Point2::new(432.0, f32::INFINITY), Align::Center);
                let color = match color {
                    TextColor::Green => Color::new(0.03, 0.9, 0.03, 1.0),
                    TextColor::LightBlue => Color::new(0.71875, 0.71875, 0.90234375, 1.0),
                    _ => Color::new(1., 1., 1., 1.),
                };
                graphics::draw(ctx, &text, (txtdst, color))?;
            }
            None => (),
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        let mouse = &mut self.mouse_info;
        mouse.button = button;
        mouse.down = true;
        if y >= 22.0 && y <= 42.0 && x >= 128.0 && x <= 211.0 {
            match x {
                x if x < 148.0 => {
                    // TODO: Implement AutoUpdate
                    self.assets.mousedown_sound.play_detached().unwrap();
                    self.update_app().unwrap();
                }
                x if x.in_range(149.0, 169.0) => {
                    self.assets.mousedown_sound.play_detached().unwrap();
                    self.locale = match self.locale {
                        "ko-KR" => "en-US",
                        "en-US" => "zh-CN",
                        "zh-CN" => "ko-KR",
                        _ => "en-US",
                    }
                }
                x if x.in_range(170.0, 190.0) => {
                    self.assets.mousedown_sound.play_detached().unwrap();
                    webbrowser::open("http://blog.naver.com/kein0011").unwrap();
                }
                x if x.in_range(191.0, 211.0) => {
                    self.assets.mousedown_sound.play_detached().unwrap();
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

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) {
        let mouse = &mut self.mouse_info;
        mouse.down = false;
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, xrel: f32, yrel: f32) {
        let mouse = &mut self.mouse_info;
        if mouse.down
            && mouse.button == MouseButton::Right
            && (mouse.x != x || mouse.y != y)
            && (mouse.x - 2.0 * xrel != x || mouse.y - 2.0 * yrel != y)
        {
            let window = graphics::window(ctx);
            let mut pos = window.get_position().unwrap();
            pos.x += (2.0 * xrel) as f64;
            pos.y += (2.0 * yrel) as f64;
            window.set_position(pos);
        }
        mouse.x = x;
        mouse.y = y;
    }
}

pub fn main() -> GameResult {
    let resource_dir = path::PathBuf::from("./resources");
    let cb = ContextBuilder::new("SCBank", "Armoha")
        .window_setup(
            conf::WindowSetup::default()
                .title(&format!(
                    "SCBank {} (StarCraft: Remastered {})",
                    env!("CARGO_PKG_VERSION"),
                    STARCRAFT_VERSION
                ))
                .transparent(true),
        )
        .window_mode(
            conf::WindowMode::default()
                .dimensions(480.0, 224.0)
                .borderless(true), // TODO: movable window with mouse drag
        )
        .add_resource_path(resource_dir);
    let (ctx, event_loop) = &mut cb.build()?;
    let window = graphics::window(ctx);
    window.set_window_icon(asset::load_icon()?);

    let ftl_string = asset::decode_string(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/cmp/ko-KR.ftl"
    )))?;
    let res = FluentResource::try_new(ftl_string).expect("Could not parse an FTL string.");
    let mut ko_kr_bundle = FluentBundle::new(&["ko-KR"]);
    ko_kr_bundle
        .add_resource(&res)
        .expect("Failed to add FTL resources to the bundle.");

    let ftl_string = asset::decode_string(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/cmp/en-US.ftl"
    )))?;
    let res = FluentResource::try_new(ftl_string).expect("Could not parse an FTL string.");
    let mut en_us_bundle = FluentBundle::new(&["en-US"]);
    en_us_bundle
        .add_resource(&res)
        .expect("Failed to add FTL resources to the bundle.");

    let ftl_string = asset::decode_string(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/cmp/zh-CN.ftl"
    )))?;
    let res = FluentResource::try_new(ftl_string).expect("Could not parse an FTL string.");
    let mut zh_cn_bundle = FluentBundle::new(&["zh-CN"]);
    zh_cn_bundle
        .add_resource(&res)
        .expect("Failed to add FTL resources to the bundle.");

    let mut fluent_bundles = HashMap::new();
    fluent_bundles.insert("ko-KR", ko_kr_bundle);
    fluent_bundles.insert("en-US", en_us_bundle);
    fluent_bundles.insert("zh-CN", zh_cn_bundle);

    let font = asset::decode_reader(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/cmp/bl.ttf"
    )))
        .unwrap();
    let font = Font::new_glyph_font_bytes(ctx, &font).unwrap_or_default();
    let assets = asset::Assets::new(ctx)?;
    let proc = mem_lib::GameProcess::current_process();
    let module = proc.get_module("SCBank.exe").unwrap();

    // println!("{}", get_time::get_utc_tm());

    let state = &mut MainState {
        font,
        mouse_info: MouseInfo {
            button: MouseButton::Middle,
            down: false,
            x: 100.0,
            y: 100.0,
        },
        locale: "ko-KR",
        fluent_bundles: fluent_bundles,
        assets: assets,
        state: SCState::FindingProcess,
        wait: 0,
        process: proc,
        module: module,
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
