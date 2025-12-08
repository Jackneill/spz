/// Scale factor for DC color components.
///
/// To convert to RGB, we should multiply by 0.282, but it can
/// be useful to represent base colors that are out of range if the higher
/// spherical harmonics bands bring them back into range so we multiply by a
/// smaller value.
pub static COLOR_SCALE: f64 = 0.15;
pub static SQRT_1_2: f64 = 0.707106781186547524401; // 1/sqrt(2)

pub static FLAG_ANTIALIASED: u8 = 0x1;

/// "NGSP" in little-endian.
pub static PACKED_GAUSSIAN_HEADER_MAGIC: u32 = 0x5053474e;

pub static PACKED_GAUSSIAN_HEADER_VERSION: u32 = 3;
