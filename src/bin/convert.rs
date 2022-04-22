use dcv_color_primitives as dcp;

use std::env;
use std::path::Path;

use glob::glob;

use image::*;

use dssim_core::*;
use rgb::RGB8;
use std::path::PathBuf;

fn load_pixels(
    attr: &Dssim,
    pixels: &[u8],
    width: usize,
    height: usize,
) -> Option<dssim_core::DssimImage<f32>> {
    let stride = 4 * width;

    let px: Vec<_> = (0..height)
        .map(move |y| {
            (0..width).map(move |x| {
                let offset = (4 * x) + (stride * y);
                let bgra = &pixels[offset..offset + 3];

                RGB8::new(bgra[2], bgra[1], bgra[0])
            })
        })
        .flatten()
        .collect();

    attr.create_image_rgb(&px, width, height)
}

fn to_byte(i: f32) -> u8 {
    if i <= 0.0 {
        0
    } else if i >= 255.0 / 256.0 {
        255
    } else {
        (i * 256.0) as u8
    }
}

fn convert_to(img: &image::DynamicImage, format: dcp::PixelFormat, path: &PathBuf, attr: &Dssim) {
    let rgb_format = dcp::ImageFormat {
        pixel_format: dcp::PixelFormat::Bgra,
        color_space: dcp::ColorSpace::Lrgb,
        num_planes: 1,
    };
    let yuv_format = dcp::ImageFormat {
        pixel_format: format,
        color_space: dcp::ColorSpace::Bt601FR, // matches jpeg output
        num_planes: 3,
    };

    // get image dimensions
    let dimensions = img.dimensions();
    let width = dimensions.0 as usize;
    let height = dimensions.1 as usize;
    let stride = 4 * width;

    // allocate and fill rgb buffer
    let mut bgra_buffer: Vec<_> = vec![0u8; stride * height];
    for (x, y, pixel) in img.pixels() {
        let offset = 4 * (x as usize) + stride * (y as usize);
        let bgra = &mut bgra_buffer[offset..offset + 3];

        bgra[0] = pixel[2]; // blue
        bgra[1] = pixel[1]; // green
        bgra[2] = pixel[0]; // red
    }

    let source_image = load_pixels(attr, &bgra_buffer, width, height).unwrap();

    // allocate yuv buffer
    let yuv_size: &mut [usize] = &mut [0usize; 3];
    dcp::get_buffers_size(dimensions.0, dimensions.1, &yuv_format, None, yuv_size)
        .expect("get_buffers_size failed");
    let mut y_buffer: Vec<_> = vec![0u8; yuv_size[0]];
    let mut u_buffer: Vec<_> = vec![0u8; yuv_size[1]];
    let mut v_buffer: Vec<_> = vec![0u8; yuv_size[2]];

    // convert from rgb to yuv
    let src_buffers = &[&bgra_buffer[..]];
    let dst_buffers = &mut [&mut y_buffer[..], &mut u_buffer[..], &mut v_buffer[..]];
    dcp::convert_image(
        dimensions.0,
        dimensions.1,
        &rgb_format,
        None, // data is packed
        src_buffers,
        &yuv_format,
        None,
        dst_buffers,
    )
    .expect("Failed rgb>yuv conversion");

    // convert back to rgb
    let src_buffers = &[&y_buffer[..], &u_buffer[..], &v_buffer[..]];
    let dst_buffers = &mut [&mut bgra_buffer[..]];
    dcp::convert_image(
        dimensions.0,
        dimensions.1,
        &yuv_format,
        None, // data is packed
        src_buffers,
        &rgb_format,
        None,
        dst_buffers,
    )
    .expect("Failed yuv>rgb conversion");

    // create destination image
    let mut imgbuf = image::ImageBuffer::new(dimensions.0, dimensions.1);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let offset = 4 * (x as usize) + stride * (y as usize);
        let bgra = &bgra_buffer[offset..offset + 3];

        *pixel = image::Rgb::<u8>([bgra[2], bgra[1], bgra[0]]);
    }

    // save images
    imgbuf.save(path).unwrap();

    // compare images with dssim
    let derived_image = load_pixels(attr, &bgra_buffer, width, height).unwrap();
    let (dssim, ssim_maps) = attr.compare(&source_image, &derived_image);
    println!("{:?}: dssim={:.8}", path, dssim);

    // save maps
    let ssim_maps_iter = ssim_maps.iter();
    ssim_maps_iter.enumerate().for_each(|(n, map_meta)| {
        let mut mapbuf =
            image::ImageBuffer::new(map_meta.map.width() as u32, map_meta.map.height() as u32);

        let required_ext = format!("d{}.png", n);
        let map_path = path.with_extension(required_ext);

        for (ssim, (_, _, pixel)) in map_meta.map.pixels().zip(mapbuf.enumerate_pixels_mut()) {
            let max = (1_f32 - ssim).abs();
            let ssim = 8_f32 * max;

            *pixel = image::Rgb::<u8>([0, to_byte(ssim), 0]);
        }

        mapbuf.save(map_path).unwrap();
    });
}

fn main() {
    let mut args = env::args();
    args.next(); // skip command name

    let src_dir = match args.next() {
        None => {
            println!("Error: Input file path is not specified.");
            println!("Usage: cargo run /path/to/input/directory");
            return;
        }
        Some(s) => s,
    };

    dcp::initialize();
    let mut attr = dssim_core::Dssim::new();
    attr.set_save_ssim_maps(1);

    let path = Path::new(&src_dir);
    let path = Path::join(path, Path::new("*.ppm"));

    for e in glob(path.to_str().unwrap()).expect("Failed to read glob pattern") {
        let src_path = e.unwrap();
        let rgb_path = src_path.with_extension("rgb.png");
        let i420_path = src_path.with_extension("i420.png");
        let i444_path = src_path.with_extension("i444.png");

        // load source image
        let img = image::open(&src_path).unwrap();
        img.save(&rgb_path).unwrap();

        // convert and save image to yuv420 format
        convert_to(&img, dcp::PixelFormat::I420, &i420_path, &attr);

        // convert and save image to yuv444 format
        convert_to(&img, dcp::PixelFormat::I444, &i444_path, &attr);
    }
}
