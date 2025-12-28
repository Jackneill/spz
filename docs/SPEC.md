# SPZ File Format Specification

**Version:** 3.0
**Status:** Stable
**Creator:** [Niantic Labs](https://github.com/nianticlabs)
**License:** [MIT](../LICENSE-MIT)

---

## Table of Contents

1. [Introduction](#introduction)
1. [Overview](#overview)
1. [Compression Layer](#compression-layer)
1. [File Extension](#file-extension)
1. [References](#references)

---

## Introduction

* `SPZ` (Splat Zip) is a compressed binary file format for storing 3D Gaussian Splat data, designed by Niantic.<br>
* The format prioritizes compact storage while maintaining sufficient precision for high-quality rendering.

## Overview

An `SPZ` file consists of:

1. **Outer compression**:
	The entire payload is gzip-compressed.
2. **Inner binary data**:
	A header followed by non-interleaved arrays of gaussian attributes.

## File Structure

```
┌─────────────────────────────────────────┐
│           GZIP Compressed Data          │
│  ┌───────────────────────────────────┐  │
│  │         Header (16 bytes)         │  │
│  ├───────────────────────────────────┤  │
│  │         Positions Array           │  │
│  ├───────────────────────────────────┤  │
│  │          Alphas Array             │  │
│  ├───────────────────────────────────┤  │
│  │          Colors Array             │  │
│  ├───────────────────────────────────┤  │
│  │          Scales Array             │  │
│  ├───────────────────────────────────┤  │
│  │         Rotations Array           │  │
│  ├───────────────────────────────────┤  │
│  │    Spherical Harmonics Array      │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

* The data is organized by attribute (Structure of Arrays),
 	* rather than by gaussian (Array of Structures) for better compression ratios.

## Compression Layer

* The entire payload is compressed using **gzip** ([RFC 1952](https://www.rfc-editor.org/rfc/rfc1952.html)).

---

## Header Structure

The header is exactly **16 bytes** using little-endian byte order.

### Header Layout

| Offset | Size | Type | Field | Value | Description |
|--------|------|------|-------|-------|-------------|
| 0 | 4 | uint32 | `magic` | `0x5053474e` | Magic number |
| 4 | 4 | uint32 | `version` | `2` \| `3` | Format version |
| 8 | 4 | uint32 | `num_points` | - | Number of gaussians |
| 12 | 1 | uint8 | `sh_degree` | `0` \| `1` \| `2` \| `3` | Spherical harmonics degree |
| 13 | 1 | uint8 | `fractional_bits` | - | Fixed-point fractional bits |
| 14 | 1 | uint8 | `flags` | - | Bit field flags |
| 15 | 1 | uint8 | `reserved` | Must be `0` | Reserved |

### Magic Number

The magic number `0x5053474e` encodes `NGSP` (**N**iantic **G**aussian **SP**lat) in ASCII:

| Byte Index | Hex Value | ASCII Character |
|------------|-----------|-----------------|
| 0 | 0x4E | N |
| 1 | 0x47 | G |
| 2 | 0x53 | S |
| 3 | 0x50 | P |

*Note: Stored as little-endian `int32`, so bytes appear reversed in memory.*

### Version Field

| Version | Status | Position Format | Rotation Format |
|---------|--------|-----------------|-----------------|
| 1 | Deprecated | float16 | First-three |
| 2 | Supported | Fixed-point 24-bit | First-three |
| 3 | **Current** | Fixed-point 24-bit | Smallest-three |

### Flags Field

| Bit | Mask | Name | Description |
|-----|------|------|-------------|
| 0 | `0x01` | `FLAG_ANTIALIASED` | Whether the splat was trained with antialiasing. |
| 1-7 | — | Reserved | Must be `0`. |

### C/C++ Header Definition

```C
struct Header {
    uint32_t magic;           // Always `0x5053474e`
    uint32_t version;         // 2 or 3
    uint32_t num_points;      // Number of gaussians
    uint8_t  sh_degree;       // SH degree: 0, 1, 2, or 3
    uint8_t  fractional_bits; // Fixed-point precision
    uint8_t  flags;           // Bit flags
    uint8_t  reserved;        // Must be 0
};
```

## File Extension

The standard file extension is `.spz`.

---

## References

1. <https://github.com/nianticlabs/spz>
