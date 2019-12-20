use criterion::*;
use std::error;
use std::fmt;
use std::fs::{remove_file, OpenOptions};
use std::io::BufRead;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::Duration;
use std::time::Instant;

use dcp::*;

const NV12_OUTPUT: &str = &"./output.nv12";
const BGRA_OUTPUT: &str = &"./output.bgra";

const SAMPLE_SIZE: usize = 22;

#[derive(Debug, Clone)]
struct BenchmarkError;

impl fmt::Display for BenchmarkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Generic benchmark error")
    }
}

impl error::Error for BenchmarkError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

type BenchmarkResult<T> = std::result::Result<T, Box<dyn error::Error>>;

fn skip_line(file: &mut Cursor<&[u8]>) -> BenchmarkResult<()> {
    let mut byte = [0; 1];
    while byte[0] != 0xA {
        file.read(&mut byte)?;
    }

    Ok(())
}

fn read_line(file: &mut Cursor<&[u8]>) -> BenchmarkResult<String> {
    let mut string = String::new();
    let written_chars = file.read_line(&mut string)?;

    let result = match written_chars {
        0 => Err(BenchmarkError),
        _ => Ok(string),
    };

    result.map_err(|e| e.into())
}

fn pnm_size(file: &mut Cursor<&[u8]>) -> BenchmarkResult<(u32, u32)> {
    file.seek(SeekFrom::Start(0))?;
    skip_line(file)?;

    let dimensions: Vec<_> = read_line(file)?
        .split_whitespace()
        .map(|s| s.parse::<u32>().unwrap())
        .collect();

    let width = dimensions.get(0).ok_or(BenchmarkError)?;
    let height = dimensions.get(1).ok_or(BenchmarkError)?;
    Ok((*width, *height))
}

fn pnm_data(file: &mut Cursor<&[u8]>) -> BenchmarkResult<(u32, u32, Vec<u8>)> {
    file.seek(SeekFrom::Start(0))?;
    let (width, height) = pnm_size(file)?;

    let size: usize = (width as usize) * (height as usize);
    let mut x: Vec<u8> = Vec::with_capacity(size);
    skip_line(file)?;
    file.read_to_end(&mut x)?;

    Ok((width, height, x))
}

fn bgra_nv12_dcp(
    mut input_file: &mut Cursor<&[u8]>,
    output_path: &str,
) -> BenchmarkResult<Duration> {
    let (mut width, height, input_buffer) = { pnm_data(&mut input_file)? };
    width /= 4;

    // Allocate output
    let dst_size: usize = 3 * (width as usize) * (height as usize) / 2;
    let mut output_buffer: Vec<u8> = vec![0; dst_size];

    // Setup dcp
    let input_data: &[&[u8]] = &[&input_buffer];
    let output_data: &mut [&mut [u8]] = &mut [&mut output_buffer[..]];

    let src_format = ImageFormat {
        pixel_format: PixelFormat::Bgra,
        color_space: ColorSpace::Lrgb,
        num_planes: 1,
    };

    let dst_format = ImageFormat {
        pixel_format: PixelFormat::Nv12,
        color_space: ColorSpace::Bt601,
        num_planes: 1,
    };

    let start = Instant::now();
    convert_image(
        width,
        height,
        &src_format,
        None,
        input_data,
        &dst_format,
        None,
        output_data,
    )?;

    let elapsed = start.elapsed();

    if !Path::new(output_path).exists() {
        let mut buffer = OpenOptions::new()
            .write(true)
            .create(true)
            .open(output_path)?;
        write!(buffer, "P5\n{} {}\n255\n", width, height + height / 2)?;
        buffer.write(&output_buffer)?;
    }

    Ok(elapsed)
}

fn nv12_bgra_dcp(
    mut input_file: &mut Cursor<&[u8]>,
    output_path: &str,
) -> BenchmarkResult<Duration> {
    let (width, mut height, input_buffer) = { pnm_data(&mut input_file)? };
    height = 2 * height / 3;

    // Allocate output
    let dst_size: usize = 4 * (width as usize) * (height as usize);
    let mut output_buffer: Vec<u8> = vec![0; dst_size];

    // Setup dcp
    let input_data: &[&[u8]] = &[&input_buffer];
    let output_data: &mut [&mut [u8]] = &mut [&mut output_buffer[..]];

    let src_format = ImageFormat {
        pixel_format: PixelFormat::Nv12,
        color_space: ColorSpace::Bt601,
        num_planes: 1,
    };

    let dst_format = ImageFormat {
        pixel_format: PixelFormat::Bgra,
        color_space: ColorSpace::Lrgb,
        num_planes: 1,
    };

    let start = Instant::now();
    convert_image(
        width,
        height,
        &src_format,
        None,
        input_data,
        &dst_format,
        None,
        output_data,
    )?;

    let elapsed = start.elapsed();

    // Write to file
    if !Path::new(output_path).exists() {
        let mut buffer = OpenOptions::new()
            .write(true)
            .create(true)
            .open(output_path)?;
        write!(buffer, "P5\n{} {}\n255\n", 4 * width, height)?;
        buffer.write(&output_buffer)?;
    }

    Ok(elapsed)
}

fn bench(c: &mut Criterion) {
    initialize();

    let mut group = c.benchmark_group("dcp");
    group.sample_size(SAMPLE_SIZE);

    {
        let output_path = &NV12_OUTPUT;
        if Path::new(output_path).exists() {
            remove_file(Path::new(output_path)).expect("Unable to delete benchmark output");
        }

        let mut input_file: Cursor<&[u8]> = Cursor::new(include_bytes!("input.bgra"));
        let (width, height) =
            { pnm_size(&mut input_file).expect("Malformed benchmark input file") };
        group.throughput(Throughput::Elements((width as u64) * (height as u64)));
        group.bench_function("bgra>nv12", move |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::new(0, 0);
                for _i in 0..iters {
                    total += bgra_nv12_dcp(&mut input_file, output_path)
                        .expect("Benchmark iteration failed");
                }

                total
            });
        });
    }

    {
        let output_path = &BGRA_OUTPUT;
        if Path::new(output_path).exists() {
            remove_file(Path::new(output_path)).expect("Unable to delete benchmark output");
        }

        let mut input_file: Cursor<&[u8]> = Cursor::new(include_bytes!("input.nv12"));
        let (width, height) =
            { pnm_size(&mut input_file).expect("Malformed benchmark input file") };
        group.throughput(Throughput::Elements((width as u64) * (height as u64)));
        group.bench_function("nv12>bgra", move |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::new(0, 0);
                for _i in 0..iters {
                    total += nv12_bgra_dcp(&mut input_file, output_path)
                        .expect("Benchmark iteration failed");
                }

                total
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);