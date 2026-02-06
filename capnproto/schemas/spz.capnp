# SPDX-License-Identifier: Apache-2.0 OR MIT

@0xed8f0b2c71c9f241;

struct GaussianSplat {
	header @0 :Header;
	body @1 :Body;
}

# SPZ file header containing metadata about the Gaussian Splat.
#
# Some fields are omitted (compared to the rust struct) since they are static
# such as the magic value or the reserved bytes which are always `0`.
struct Header {
	# SPZ file format version.
	version @0 :Version;

	# Number of Gaussians in the splat.
	numPoints @1 :UInt64;

	# Spherical harmonics degree (0, 1, 2, or 3).
	#
	# Controls view-dependent color complexity:
	#   - Degree 0: No view dependence (color only, stored in colors array)
	#   - Degree 1: 3 coefficients/channel (9 total)
	#   - Degree 2: 8 coefficients/channel (24 total)
	#   - Degree 3: 15 coefficients/channel (45 total)
	sphericalHarmonicsDegree @2 :UInt8;

	# Number of bits for the fractional part in fixed-point position encoding.
	fractionalBits @3 :UInt8;

	# Bit flags containing metadata like antialiasing.
	flags @4 :Flags;
}

# A complete set of Gaussian Splats representing a 3D scene.
struct Body {
	# Size: n × 3 floats.
	positions @0 :List(Float32);
	# Size: n × 3 floats.
	scales @1 :List(Float32);
	# Size: n × 4 floats.
	rotations @2 :List(Float32);
	# Size: n floats.
	alphas @3 :List(Float32);
	# Size: n × 3 floats.
	colors @4 :List(Float32);
	# Size: n × coefficientsPerGaussian floats.
	# coefficientsPerGaussian depends on sphericalHarmonicsDegree in the
	# header.
	sphericalHarmonics @5 :List(Float32);
}

enum Version {
	v1 @0; # Deprecated
	v2 @1; # Supported (first-three quaternions)
	v3 @2; # Current (smallest-three quaternions)
}

# Bit flags for SPZ metadata.
struct Flags {
	# Whether the Gaussian Splat was trained with `antialiasing`.
	antialiased @0 :Bool;
}

struct BoundingBox {
	minX @0 :Float32;
	maxX @1 :Float32;

	minY @2 :Float32;
	maxY @3 :Float32;

	minZ @4 :Float32;
	maxZ @5 :Float32;
}

enum CoordinateSystem {
	unspecified @0;      # No conversion

	leftDownBack @1;     # LDB
	rightDownBack @2;    # RDB
	leftUpBack @3;       # LUB
	rightUpBack @4;      # RUB - SPZ Internal, OpenGL, Three.js
	leftDownFront @5;    # LDF
	rightDownFront @6;   # RDF - PLY files
	leftUpFront @7;      # LUF - glTF/glB
	rightUpFront @8;     # RUF - Unity
}
