use std::fs::File;
use std::num::NonZeroU32;

use image::codecs::png::PngEncoder;
use image::io::Reader as ImageReader;
use image::{ColorType, DynamicImage, GenericImageView};

use fast_image_resize::pixels::*;
use fast_image_resize::{CpuExtensions, Image, PixelType};

pub fn image_checksum<const N: usize>(buffer: &[u8]) -> [u32; N] {
    let mut res = [0u32; N];
    for pixel in buffer.chunks_exact(N) {
        res.iter_mut().zip(pixel).for_each(|(d, &s)| *d += s as u32);
    }
    res
}

pub trait PixelExt: Pixel {
    fn pixel_type_str() -> &'static str {
        match Self::pixel_type() {
            PixelType::U8 => "u8",
            PixelType::U8x3 => "u8x3",
            PixelType::U8x4 => "u8x4",
            PixelType::I32 => "i32",
            PixelType::F32 => "f32",
        }
    }

    fn load_big_src_image() -> Image<'static> {
        let img = ImageReader::open("./data/nasa-4928x3279.png")
            .unwrap()
            .decode()
            .unwrap();
        Image::from_vec_u8(
            NonZeroU32::new(img.width()).unwrap(),
            NonZeroU32::new(img.height()).unwrap(),
            Self::img_into_bytes(img),
            Self::pixel_type(),
        )
        .unwrap()
    }

    fn load_small_src_image() -> Image<'static> {
        let img = ImageReader::open("./data/nasa-852x567.png")
            .unwrap()
            .decode()
            .unwrap();
        Image::from_vec_u8(
            NonZeroU32::new(img.width()).unwrap(),
            NonZeroU32::new(img.height()).unwrap(),
            Self::img_into_bytes(img),
            Self::pixel_type(),
        )
        .unwrap()
    }

    fn load_small_rgba_image() -> Image<'static> {
        let img = ImageReader::open("./data/nasa-852x567-rgba.png")
            .unwrap()
            .decode()
            .unwrap();
        Image::from_vec_u8(
            NonZeroU32::new(img.width()).unwrap(),
            NonZeroU32::new(img.height()).unwrap(),
            Self::img_into_bytes(img),
            Self::pixel_type(),
        )
        .unwrap()
    }

    fn img_into_bytes(img: DynamicImage) -> Vec<u8>;
}

impl PixelExt for U8 {
    fn img_into_bytes(img: DynamicImage) -> Vec<u8> {
        img.to_luma8().into_raw()
    }
}

impl PixelExt for U8x3 {
    fn img_into_bytes(img: DynamicImage) -> Vec<u8> {
        img.to_rgb8().into_raw()
    }
}

impl PixelExt for U8x4 {
    fn img_into_bytes(img: DynamicImage) -> Vec<u8> {
        img.to_rgba8().into_raw()
    }
}

impl PixelExt for I32 {
    fn img_into_bytes(img: DynamicImage) -> Vec<u8> {
        img.to_luma16()
            .as_raw()
            .iter()
            .map(|&p| p as u32 * (i16::MAX as u32 + 1))
            .flat_map(|val| val.to_le_bytes())
            .collect()
    }
}

impl PixelExt for F32 {
    fn img_into_bytes(img: DynamicImage) -> Vec<u8> {
        img.to_luma16()
            .as_raw()
            .iter()
            .map(|&p| p as f32 * (i16::MAX as f32 + 1.0))
            .flat_map(|val| val.to_le_bytes())
            .collect()
    }
}

pub fn save_result(image: &Image, name: &str) {
    if std::env::var("DONT_SAVE_RESULT").unwrap_or_else(|_| "".to_owned()) == "1" {
        return;
    }
    std::fs::create_dir_all("./data/result").unwrap();
    let mut file = File::create(format!("./data/result/{}.png", name)).unwrap();
    let color_type = match image.pixel_type() {
        PixelType::U8x3 => ColorType::Rgb8,
        PixelType::U8x4 => ColorType::Rgba8,
        PixelType::U8 => ColorType::L8,
        _ => panic!("Unsupported type of pixels"),
    };
    PngEncoder::new(&mut file)
        .encode(
            image.buffer(),
            image.width().get(),
            image.height().get(),
            color_type,
        )
        .unwrap();
}

pub fn cpu_ext_into_str(cpu_extensions: CpuExtensions) -> &'static str {
    match cpu_extensions {
        CpuExtensions::None => "native",
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Sse4_1 => "sse41",
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Avx2 => "avx2",
    }
}
