// Copyright 2019 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: MIT-0

// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify,
// merge, publish, distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
// HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
// SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
pub const MAX_NUMBER_OF_PLANES: usize = 4;

/// An enumeration of supported pixel formats.
#[derive(Copy, Clone)]
#[repr(C)]
pub enum PixelFormat {
    /// Packed 8-bit RGB with alpha channel first.
    ///
    /// Each pixel is a four-byte little-endian value.
    /// A, R, G and B are found in bits `7:0`, `15:8`, `23:16` and `31:24` respectively.
    ///
    /// 32 bits per pixel
    Argb,
    /// Packed 8-bit reverse RGB with alpha channel last.
    ///
    /// Each pixel is a four-byte little-endian value.
    /// B, G, R and A are found in bits `7:0`, `15:8`, `23:16` and `31:24` respectively.
    ///
    /// 32 bits per pixel
    Bgra,
    /// Packed 8-bit reverse RGB without alpha channel.
    ///
    /// 24 bits per pixel
    Bgr,
    /// Packed 8-bit RGB with alpha channel last.
    ///
    /// Each pixel is a four-byte little-endian value.
    /// R, G, B and A are found in bits `7:0`, `15:8`, `23:16` and `31:24` respectively.
    ///
    /// 32 bits per pixel
    Rgba,
    /// Packed 8-bit RGB without alpha channel.
    ///
    /// 24 bits per pixel
    Rgb,
    /// Packed 10-bit reverse RGB with alpha channel last.
    ///
    /// Each pixel is a four-byte little-endian value.
    /// B, G, R and A are found in bits `9:0`, `19:10`, `29:20` and `31:30` respectively.
    ///
    /// 32 bits per pixel
    Bgra30,
    /// Packed 10-bit RGB with alpha channel last.
    ///
    /// Each pixel is a four-byte little-endian value.
    /// R, G, B and A are found in bits `9:0`, `19:10`, `29:20` and `31:30` respectively.
    ///
    /// 32 bits per pixel
    Rgba30,
    /// Planar 8-bit YUV with one luma plane Y then 2 chroma planes U and V.
    ///
    /// Chroma planes are not sub-sampled.
    ///
    /// 24 bits per pixel
    I444,
    /// Planar 8-bit YUV with one luma plane Y then 2 chroma planes U, V.
    ///
    /// Chroma planes are sub-sampled in the horizontal dimension, by a factor of 2.
    ///
    /// 16 bits per pixel
    I422,
    /// Planar 8-bit YUV with one luma plane Y then U chroma plane and last the V chroma plane.
    ///
    /// The two chroma planes are sub-sampled in both the horizontal and vertical dimensions by a factor of 2.
    ///
    /// 12 bits per pixel
    I420,
    /// Planar 8-bit YUV with one luma plane Y then one plane with interleaved U and V values.
    ///
    /// Chroma planes are subsampled in both the horizontal and vertical dimensions by a factor of 2.
    ///
    /// Samples in the UV plane are two-byte little-endian values.
    /// U and V are found in bits `7:0` and `15:8` respectively.
    ///
    /// 12 bits per pixel
    Nv12,
    /// Planar 10-bit YUV with one luma plane Y then 2 chroma planes U and V.
    ///
    /// Chroma planes are not sub-sampled.
    ///
    /// Each sample is a two-byte little-endian value.
    /// S is found in bits `9:0`, with bits `15:10` ignored.
    ///
    /// 48 bits per pixel
    P410,
    /// Planar 10-bit YUV with one luma plane Y then 2 chroma planes U and V.
    ///
    /// The two chroma planes are sub-sampled in both the horizontal and vertical
    /// dimensions by a factor of 2.
    ///
    /// Each sample is a two-byte little-endian value.
    /// S is found in bits `9:0`, with bits `15:10` ignored.
    ///
    /// 24 bits per pixel
    P010,
}

/// If a plane stride is assigned to this constant, the plane will be assumed to contain packed data
pub const STRIDE_AUTO: usize = 0;

pub const DEFAULT_STRIDES: [usize; MAX_NUMBER_OF_PLANES] = [STRIDE_AUTO; MAX_NUMBER_OF_PLANES];

const fn make_pf_spec(planes: u32, width: u32, height: u32, byte_count: u32) -> u32 {
    (byte_count << 4) | (height << 3) | (width << 2) | planes
}

const fn make_plane_spec(plane0: u32, plane1: u32, plane2: u32, plane3: u32) -> u32 {
    (plane3 << 18) | (plane2 << 12) | (plane1 << 6) | plane0
}

const I_P: u32 = 32;

const PF_SPECS: [u32; 13] = [
    make_pf_spec(0, 0, 0, 4), // Argb
    make_pf_spec(0, 0, 0, 4), // Bgra
    make_pf_spec(0, 0, 0, 3), // Bgr
    make_pf_spec(0, 0, 0, 4), // Rgba
    make_pf_spec(0, 0, 0, 3), // Rgb
    make_pf_spec(0, 0, 0, 4), // Bgra30
    make_pf_spec(0, 0, 0, 4), // Rgba30
    make_pf_spec(2, 0, 0, 1), // I444
    make_pf_spec(2, 1, 0, 1), // I422
    make_pf_spec(2, 1, 1, 1), // I420
    make_pf_spec(1, 1, 1, 1), // Nv12
    make_pf_spec(2, 0, 0, 2), // P410
    make_pf_spec(2, 1, 1, 2), // P010
];

const STRIDE_SPECS: [u32; 13] = [
    make_plane_spec(0, I_P, I_P, I_P), // Argb
    make_plane_spec(0, I_P, I_P, I_P), // Bgra
    make_plane_spec(0, I_P, I_P, I_P), // Bgr
    make_plane_spec(0, I_P, I_P, I_P), // Rgba
    make_plane_spec(0, I_P, I_P, I_P), // Rgb
    make_plane_spec(0, I_P, I_P, I_P), // Bgra30
    make_plane_spec(0, I_P, I_P, I_P), // Rgba30
    make_plane_spec(0, 0, 0, I_P),     // I444
    make_plane_spec(0, 1, 1, I_P),     // I422
    make_plane_spec(0, 1, 1, I_P),     // I420
    make_plane_spec(0, 0, I_P, I_P),   // Nv12
    make_plane_spec(0, 0, 0, I_P),     // P410
    make_plane_spec(0, 1, 1, I_P),     // P010
];

const HEIGHT_SPECS: [u32; 13] = [
    make_plane_spec(0, I_P, I_P, I_P), // Argb
    make_plane_spec(0, I_P, I_P, I_P), // Bgra
    make_plane_spec(0, I_P, I_P, I_P), // Bgr
    make_plane_spec(0, I_P, I_P, I_P), // Rgba
    make_plane_spec(0, I_P, I_P, I_P), // Rgb
    make_plane_spec(0, I_P, I_P, I_P), // Bgra30
    make_plane_spec(0, I_P, I_P, I_P), // Rgba30
    make_plane_spec(0, 0, 0, I_P),     // I444
    make_plane_spec(0, 0, 0, I_P),     // I422
    make_plane_spec(0, 1, 1, I_P),     // I420
    make_plane_spec(0, 1, I_P, I_P),   // Nv12
    make_plane_spec(0, 0, 0, I_P),     // P410
    make_plane_spec(0, 1, 1, I_P),     // P010
];

fn get_pf_width(pf: u32) -> u32 {
    (pf >> 2) & 1
}

fn get_pf_height(pf: u32) -> u32 {
    (pf >> 3) & 1
}

fn get_pf_byte_count(pf: u32) -> u32 {
    pf >> 4
}

fn get_pf_planes(pf: u32) -> u32 {
    pf & 3
}

fn get_plane_value(bpp: u32, plane: u32) -> u32 {
    (bpp >> (6 * plane)) & 0x3F
}

fn get_plane_mask(bpp: u32, plane: u32) -> usize {
    (I_P != get_plane_value(bpp, plane)) as usize
}

fn get_plane_spec(dimension: u32, bpp: u32, plane: u32) -> usize {
    (dimension.wrapping_shr(get_plane_value(bpp, plane))) as usize
}

pub fn is_compatible(pixel_format: u32, width: u32, height: u32, last_plane: u32) -> bool {
    let spec = PF_SPECS[pixel_format as usize];
    ((width & get_pf_width(spec))
        | (height & get_pf_height(spec))
        | last_plane.wrapping_mul(last_plane.wrapping_sub(get_pf_planes(spec))))
        == 0
}

pub fn get_buffers_size(
    pixel_format: u32,
    width: u32,
    height: u32,
    last_plane: u32,
    strides: &[usize],
    buffers_size: &mut [usize],
) -> bool {
    let last_plane = last_plane as usize;
    if last_plane >= MAX_NUMBER_OF_PLANES
        || last_plane >= strides.len()
        || last_plane >= buffers_size.len()
    {
        return false;
    }

    let stride = &mut [0usize; MAX_NUMBER_OF_PLANES];

    let pixel_format = pixel_format as usize;
    let pf_spec = PF_SPECS[pixel_format];
    let byte_count = get_pf_byte_count(pf_spec) as usize;

    let stride_spec = STRIDE_SPECS[pixel_format];
    for i in 0..MAX_NUMBER_OF_PLANES {
        stride[i] = if i >= strides.len() || strides[i] == STRIDE_AUTO {
            get_plane_mask(stride_spec, i as u32)
                * get_plane_spec(width, stride_spec, i as u32)
                * byte_count
        } else {
            strides[i]
        };
    }

    let height_spec = HEIGHT_SPECS[pixel_format];
    if last_plane == 0 {
        buffers_size[0] = ((stride[0] * get_plane_spec(height, height_spec, 0))
            + (stride[1] * get_plane_spec(height, height_spec, 1)))
            + ((stride[2] * get_plane_spec(height, height_spec, 2))
                + (stride[3] * get_plane_spec(height, height_spec, 3)));
    } else {
        let buffer_array = &mut buffers_size[..last_plane + 1];
        let stride_array = &stride[..last_plane + 1];

        for (buffer_size, (i, stride)) in
            buffer_array.iter_mut().zip(stride_array.iter().enumerate())
        {
            *buffer_size = *stride * get_plane_spec(height, height_spec, i as u32);
        }
    }

    true
}

pub fn are_planes_compatible(pixel_format: u32, num_planes: u32) -> bool {
    let last_plane = num_planes.wrapping_sub(1);
    let spec = PF_SPECS[pixel_format as usize];
    last_plane.wrapping_mul(last_plane.wrapping_sub(get_pf_planes(spec))) == 0
}
