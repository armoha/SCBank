use std::io::{self, prelude::*};

use flate2::bufread::ZlibDecoder;
use ggez::{*, audio, graphics};
use image;

pub struct Assets {
    pub background_image: graphics::Image,
    pub update_button: graphics::Image,
    pub language_button: graphics::Image,
    pub homepage_button: graphics::Image,
    pub folder_button: graphics::Image,
    pub hover_button: graphics::Image,
    pub mousedown_sound: audio::Source,
}

pub fn decode_reader(bytes: &[u8]) -> io::Result<Vec<u8>> {
    let mut z = ZlibDecoder::new(bytes);
    let mut v = Vec::new();
    z.read_to_end(&mut v)?;
    Ok(v)
}

pub fn decode_string(bytes: &[u8]) -> io::Result<String> {
    let mut z = ZlibDecoder::new(bytes);
    let mut s = String::new();
    z.read_to_string(&mut s)?;
    Ok(s)
}

impl Assets {
    pub fn new(ctx: &mut Context) -> GameResult<Assets> {
        let background = decode_reader(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/cmp/bc2017console.jpg"
        )))?;
        let background = image::load_from_memory(&background)?.to_rgba();
        let (width, height) = background.dimensions();
        let background_image =
            graphics::Image::from_rgba8(ctx, width as u16, height as u16, &background)?;

        let update = decode_reader(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/cmp/update.png"
        )))?;
        let update = image::load_from_memory(&update)?.to_rgba();
        let (width, height) = update.dimensions();
        let update_button = graphics::Image::from_rgba8(ctx, width as u16, height as u16, &update)?;

        let language = decode_reader(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/cmp/language.png"
        )))?;
        let language = image::load_from_memory(&language)?.to_rgba();
        let (width, height) = language.dimensions();
        let language_button =
            graphics::Image::from_rgba8(ctx, width as u16, height as u16, &language)?;

        let homepage = decode_reader(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/cmp/homepage.jpg"
        )))?;
        let homepage = image::load_from_memory(&homepage)?.to_rgba();
        let (width, height) = homepage.dimensions();
        let homepage_button =
            graphics::Image::from_rgba8(ctx, width as u16, height as u16, &homepage)?;

        let folder = decode_reader(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/cmp/folder.png"
        )))?;
        let folder = image::load_from_memory(&folder)?.to_rgba();
        let (width, height) = folder.dimensions();
        let folder_button = graphics::Image::from_rgba8(ctx, width as u16, height as u16, &folder)?;

        let hover = decode_reader(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/cmp/hover.png"
        )))?;
        let hover = image::load_from_memory(&hover)?.to_rgba();
        let (width, height) = hover.dimensions();
        let hover_button = graphics::Image::from_rgba8(ctx, width as u16, height as u16, &hover)?;

        let mousedown = decode_reader(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/cmp/mousedown2.wav"
        )))?;
        let mousedown_sound = audio::Source::from_data(ctx, audio::SoundData::from(mousedown))?;

        Ok(Assets {
            background_image,
            update_button,
            language_button,
            homepage_button,
            folder_button,
            hover_button,
            mousedown_sound,
        })
    }
}
