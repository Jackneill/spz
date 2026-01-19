# SPZ File Format (v3) Specification

**SPZ File Format Creator:** [Niantic Labs](https://github.com/nianticlabs) //
**License:** [MIT](../LICENSE-MIT)

*Last updated: 2026-01-15*

---

## Table of Contents

1. [Introduction](#introduction)
1. [Overview](#overview)
1. [File Structure](#file-structure)
1. [Compression Layer](#compression-layer)
1. [Header Structure](#header-structure)
	1. [C/C++ Header Definition](#cc-header-definition)
1. [Arrays Description](#arrays-description)
1. [Coordinate Systems](#coordinate-systems)
1. [File Extension](#file-extension)
1. [References](#references)

---

## Introduction

* `SPZ` (Splat Zip) is a compressed binary file format for storing 3D Gaussian
	Splat data, designed by Niantic.<br>
* The format prioritizes compact storage while maintaining sufficient precision
	for high-quality rendering.

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

### Endianness

* All multi-byte values use **little-endian** (LE) byte order.

## Compression Layer

* The entire payload is compressed using **gzip** ([RFC 1952](https://www.rfc-editor.org/rfc/rfc1952.html)).

---

## Header Structure

* The header is exactly **16 bytes** using **little-endian** (LE) byte order.

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

* The magic number `0x5053474e` encodes `NGSP` (**N**iantic **G**aussian **SP**lat) in ASCII:

| Byte Index | Hex Value | ASCII Character |
|------------|-----------|-----------------|
| 0 | 0x4E | N |
| 1 | 0x47 | G |
| 2 | 0x53 | S |
| 3 | 0x50 | P |

*Note: Stored as little-endian `int32`, so bytes appear reversed in memory.*

### Version Field

| Version | Status |
|---------|--------|
| 1 | Deprecated |
| 2 | Supported |
| 3 | **Current** |

### Flags Field

| Bit | Mask | Name | Description |
|-----|------|------|-------------|
| 0 | `0x01` | `FLAG_ANTIALIASED` | Whether the splat was trained with antialiasing. |
| 1-7 | — | Reserved | Must be `0`. |

### C/C++ Header Definition

```C
// 16 bytes
struct Header {
    uint32_t magic;           // Always `0x5053474e`
    uint32_t version;         // `3`
    uint32_t num_points;      // Number of gaussians
    uint8_t  sh_degree;       // SH degree: 0, 1, 2, or 3
    uint8_t  fractional_bits; // Fixed-point precision
    uint8_t  flags;           // Bit flags
    uint8_t  reserved;        // Must be 0
};
```

---

## Arrays Description

<table>
	<thead>
		<tr>
			<th>Array (in order)</th>
			<th>Elem Size</th>
			<th>Components</th>
			<th>Type</th>
			<th>C Type</th>
			<th>Notes</th>
		</tr>
	</thead>
	<tbody>
		<tr>
			<td>Positions</td>
			<td>72 bits</td>
			<td>(x, y, z)</td>
			<td>
<code>(i24, i24, i24) [num_positions]</code>
			</td>
			<td>
				<pre><code>
struct position {
	int8_t x[3];
	int8_t y[3];
	int8_t z[3];
} positions[num_points];
				</code></pre>
			</td>
			<td>
				<ul>
					<li>
						Configurable precision: the number of fractional bits for the precision is determined by the <code>fractional_bits</code> field in the header.
					</li>
				</ul>
			</td>
		</tr>
		<tr>
			<td>Alphas</td>
			<td>8 bits</td>
			<td>alpha</td>
			<td><code>u8[num_points]</code></td>
			<td>
<pre><code>
uint8_t alphas[num_points];
</code></pre>
			</td>
			<td>
				<ul>
					<li>
						Quantized sigmoid-encoded opacity (0-255 → sigmoid⁻¹)
					</li>
				</ul>
			</td>
		</tr>
		<tr>
			<td>Colors</td>
			<td>24 bits</td>
			<td>(r, g, b)</td>
			<td><code>(u8, u8, u8) [num_points]</code></td>
			<td>
<pre><code>
struct color {
	uint8_t r;
	uint8_t g;
	uint8_t b;
} colors[num_points];
</code></pre>
			</td>
			<td>
				<ul>
					<li>
						DC spherical harmonic coefficient (RGB)
					</li>
				</ul>
			</td>
		</tr>
		<tr>
			<td>Scales</td>
			<td>24 bits</td>
			<td>(x, y, z)</td>
			<td><code>(u8, u8, u8) [num_points]</code></td>
			<td>
<pre><code>
struct scale {
	uint8_t x;
	uint8_t y;
	uint8_t z;
} scales[num_points];
</code></pre>
			</td>
			<td>
				<ul>
					<li>
						Log-encoded scale factors: <code>scale = (byte / 16) - 10</code>
					</li>
				</ul>
			</td>
		</tr>
		<tr>
			<td>Rotations</td>
			<td>32 bits</td>
			<td>(idx, x, y, z)</td>
			<td><code>(u2, i10, i10, i10) [num_points]</code></td>
			<td>
<pre><code>
struct rotation {
	uint8_t index : 2;
	int16_t x : 10;
	int16_t y : 10;
	int16_t z : 10;
} rotations[num_points];
</code></pre>
			</td>
			<td>
				<ul>
					<li>
						Rotations are represented as the smallest three components of the normalized
						rotation quaternion, for optimal rotation accuracy.
					</li>
					<li>
						The largest component can be derived from the others and is not stored.
					</li>
					<li>
						Its index is stored on 2 bits and each of the smallest three components
						is encoded as a 10-bit signed integer.
					</li>
				</ul>
			</td>
		</tr>
		<tr>
			<td>Spherical Harmonics</td>
			<td>variable</td>
			<td>(r, g, b) × sh_dim</td>
			<td><code>u8[num_points × sh_dim × 3]</code></td>
			<td>
<pre><code>
uint8_t spherical_harmonics[num_points * sh_dim * 3];
</code></pre>
			</td>
			<td>
				<ul>
					<li>
						Quantized SH coefficients (degrees 1-3).<br>
						Size = <code>num_points × sh_dim × 3</code> bytes.<br>
				- degree 0 → 0<br>
				- degree 1 → 3<br>
				- degree 2 → 8<br>
				- degree 3 → 15
					</li>
					<li>
						Spherical Harmonics (SH) coefficients store view-dependent color information.
					</li>
					<li>
						SPZ supports degrees 0, 1, 2 or 3.
					</li>
				</ul>
			</td>
		</tr>
	</tbody>
</table>

---

## Spherical Harmonics (SH)

### Coefficients per Degree

| Degree | Bands | Coefficients per Channel | Total (RGB) |
|--------|-------|--------------------------|-------------|
| 0      | 1     | 0 (DC in colors)         | 0           |
| 1      | 3     | 3                        | 9           |
| 2      | 5     | 3 + 5 = 8                | 24          |
| 3      | 7     | 3 + 5 + 7 = 15           | 45          |

**Note**: DC (degree 0) is stored in the `Colors array`, not in SH.

### Quantization Precision

| Degree | Bits | Step Size | Range          |
|--------|------|-----------|----------------|
| 1      | 5    | 8         | 32 levels      |
| 2      | 4    | 16        | 16 levels      |
| 3      | 4    | 16        | 16 levels      |

---

## Coordinate Systems

### Supported Coordinate System Definitions

| Code | Name | +X | +Y | +Z | Common Usage |
|------|------|----|----|----|----|
| **RUB** | Right-Up-Back | Right | Up | Back | **SPZ Internal**, OpenGL, Three.js |
| RDF | Right-Down-Front | Right | Down | Front | PLY files |
| LUF | Left-Up-Front | Left | Up | Front | GLB/glTF |
| RUF | Right-Up-Front | Right | Up | Front | Unity |
| LDB | Left-Down-Back | Left | Down | Back | — |
| RDB | Right-Down-Back | Right | Down | Back | — |
| LUB | Left-Up-Back | Left | Up | Back | — |
| LDF | Left-Down-Front | Left | Down | Front | — |

---

## File Extension

* The standard file extension is `.spz`.

---

## References

1. <https://github.com/nianticlabs/spz>
