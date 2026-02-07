// SPDX-License-Identifier: Apache-2.0 OR MIT

#ifndef SPZ_H
#define SPZ_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * SPZ file format version.
 *
 * Currently only V2 and V3 are supported. V3 is the default and recommended
 * version.
 */
typedef enum SpzVersion
{
	/**
         * Version 1 (unsupported).
         */
	SpzVersion_V1 = 1,
	/**
         * Version 2.
         */
	SpzVersion_V2 = 2,
	/**
         * Version 3 (default).
         */
	SpzVersion_V3 = 3,
} SpzVersion;

/**
 * Coordinate system enumeration for 3D data.
 *
 * The SPZ format internally uses RightUpBack (RUB) coordinates.
 * Specify your source/target coordinate system to enable automatic conversion.
 */
typedef enum SpzCoordinateSystem
{
	SpzCoordinateSystem_Unspecified = 0,
	SpzCoordinateSystem_LeftDownBack = 1,
	SpzCoordinateSystem_RightDownBack = 2,
	SpzCoordinateSystem_LeftUpBack = 3,
	SpzCoordinateSystem_RightUpBack = 4,
	SpzCoordinateSystem_LeftDownFront = 5,
	SpzCoordinateSystem_RightDownFront = 6,
	SpzCoordinateSystem_LeftUpFront = 7,
	SpzCoordinateSystem_RightUpFront = 8,
} SpzCoordinateSystem;

/**
 * Status code returned by fallible SPZ functions.
 *
 * Check with `result == SpzResult_Success`. On failure, call
 * `spz_last_error()` for a descriptive message.
 */
typedef enum SpzResult
{
	/**
         * Operation completed successfully.
         */
	SpzResult_Success = 0,
	/**
         * A null pointer was passed where a valid pointer was expected.
         */
	SpzResult_NullPointer = 1,
	/**
         * A function argument was invalid (e.g. non-UTF-8 path).
         */
	SpzResult_InvalidArgument = 2,
	/**
         * An I/O or parsing error occurred.
         */
	SpzResult_IoError = 3,
} SpzResult;

/**
 * Opaque handle to a GaussianSplat object.
 *
 * Must be freed with `spz_gaussian_splat_free`.
 */
typedef struct SpzGaussianSplat SpzGaussianSplat;

/**
 * Opaque handle to an SPZ file header.
 *
 * A header can be read from a file or from bytes *without* loading the full
 * splat data. This is useful for quick file inspection.
 *
 * Must be freed with `spz_header_free`.
 */
typedef struct SpzHeader SpzHeader;

/**
 * Axis-aligned bounding box of a Gaussian Splat.
 */
typedef struct SpzBoundingBox
{
	float min_x;
	float max_x;
	float min_y;
	float max_y;
	float min_z;
	float max_z;
} SpzBoundingBox;

#ifdef __cplusplus
extern "C"
{
#endif	// __cplusplus

	/**
 * Reads a header from an SPZ file without loading the full splat data.
 *
 * Efficient for quickly inspecting SPZ file metadata.
 *
 * Returns NULL on failure. Call `spz_last_error()` for error details.
 * The caller must free the returned handle with `spz_header_free`.
 */
	struct SpzHeader *spz_header_from_file(const char *filepath);

	/**
 * Reads a header from compressed SPZ bytes without loading the full splat data.
 *
 * Returns NULL on failure. Call `spz_last_error()` for error details.
 * The caller must free the returned handle with `spz_header_free`.
 */
	struct SpzHeader *spz_header_from_bytes(const uint8_t *data, uintptr_t len);

	/**
 * Frees a header handle.
 */
	void spz_header_free(struct SpzHeader *header);

	/**
 * Returns the SPZ format version stored in the header.
 */
	enum SpzVersion spz_header_version(const struct SpzHeader *header);

	/**
 * Returns the number of Gaussian points recorded in the header.
 */
	int32_t spz_header_num_points(const struct SpzHeader *header);

	/**
 * Returns the spherical harmonics degree (0-3).
 */
	uint8_t spz_header_sh_degree(const struct SpzHeader *header);

	/**
 * Returns the number of fractional bits used in position encoding.
 *
 * Standard value is 12, giving ~0.25 mm resolution.
 */
	uint8_t spz_header_fractional_bits(const struct SpzHeader *header);

	/**
 * Returns whether the splat was trained with antialiasing.
 */
	bool spz_header_antialiased(const struct SpzHeader *header);

	/**
 * Validates the header (magic number, version, ranges, reserved bytes).
 *
 * Returns `true` if the header passes all validation checks.
 */
	bool spz_header_is_valid(const struct SpzHeader *header);

	/**
 * Returns a heap-allocated, human-readable summary of the header.
 *
 * The caller must free the returned string with `spz_free_string`.
 * Returns NULL if the handle is null.
 */
	char *spz_header_pretty_fmt(const struct SpzHeader *header);

	/**
 * Creates a new, empty GaussianSplat (zero points).
 *
 * Returns NULL on failure. Call `spz_last_error()` for error details.
 * The caller must free the returned handle with `spz_gaussian_splat_free`.
 */
	struct SpzGaussianSplat *spz_gaussian_splat_new(void);

	/**
 * Loads a GaussianSplat from an SPZ file.
 *
 * Returns NULL on failure. Call `spz_last_error()` for error details.
 * The caller must free the returned handle with `spz_gaussian_splat_free`.
 */

	struct SpzGaussianSplat *spz_gaussian_splat_load(const char *filepath, enum SpzCoordinateSystem coord_sys);

	/**
 * Loads a GaussianSplat from a byte buffer containing SPZ data.
 *
 * Returns NULL on failure. Call `spz_last_error()` for error details.
 * The caller must free the returned handle with `spz_gaussian_splat_free`.
 */

	struct SpzGaussianSplat *
	spz_gaussian_splat_load_from_bytes(const uint8_t *data, uintptr_t len, enum SpzCoordinateSystem coord_sys);

	/**
 * Saves a GaussianSplat to an SPZ file.
 *
 * Returns `SpzResult_Success` on success. Call `spz_last_error()` on failure.
 */

	enum SpzResult spz_gaussian_splat_save(
	    const struct SpzGaussianSplat *splat, const char *filepath, enum SpzCoordinateSystem coord_sys);

	/**
 * Serializes a GaussianSplat to a heap-allocated byte buffer.
 *
 * Returns `SpzResult_Success` on success. Call `spz_last_error()` on failure.
 * The caller must free the returned buffer with `spz_free_bytes`.
 */

	enum SpzResult spz_gaussian_splat_to_bytes(
	    const struct SpzGaussianSplat *splat,
	    enum SpzCoordinateSystem coord_sys,
	    uint8_t **out_data,
	    uintptr_t *out_len);

	/**
 * Frees a byte buffer previously returned by `spz_gaussian_splat_to_bytes`.
 */
	void spz_free_bytes(uint8_t *data, uintptr_t len);

	void spz_gaussian_splat_free(struct SpzGaussianSplat *splat);

	/**
 * Returns the number of points (gaussians) in the splat.
 */
	int32_t spz_gaussian_splat_num_points(const struct SpzGaussianSplat *splat);

	/**
 * Returns the spherical harmonics degree (0-3).
 */
	uint8_t spz_gaussian_splat_sh_degree(const struct SpzGaussianSplat *splat);

	/**
 * Returns the SPZ format version of the splat.
 */
	enum SpzVersion spz_gaussian_splat_version(const struct SpzGaussianSplat *splat);

	/**
 * Returns the number of fractional bits used in position encoding.
 *
 * Standard value is 12, giving ~0.25 mm resolution.
 */
	uint8_t spz_gaussian_splat_fractional_bits(const struct SpzGaussianSplat *splat);

	/**
 * Returns whether the splat was trained with antialiasing.
 */
	bool spz_gaussian_splat_antialiased(const struct SpzGaussianSplat *splat);

	/**
 * Returns the bounding box of the splat.
 *
 * Returns a zeroed bounding box if the handle is null.
 */
	struct SpzBoundingBox spz_gaussian_splat_bbox(const struct SpzGaussianSplat *splat);

	/**
 * Returns the median ellipsoid volume of the gaussians.
 */
	float spz_gaussian_splat_median_volume(const struct SpzGaussianSplat *splat);

	/**
 * Validates that all internal arrays have consistent sizes.
 *
 * Returns `true` if the splat passes all size checks.
 */
	bool spz_gaussian_splat_check_sizes(const struct SpzGaussianSplat *splat);

	/**
 * Returns a pointer to the positions array.
 *
 * The array contains `num_points * 3` floats in `[x0, y0, z0, x1, y1, z1, ...]` order.
 * The pointer is valid until the splat is modified or freed.
 *
 * If `out_len` is non-null it receives the total number of floats.
 */

	const float *spz_gaussian_splat_positions(const struct SpzGaussianSplat *splat, uintptr_t *out_len);

	/**
 * Returns a pointer to the scales array.
 *
 * The array contains `num_points * 3` floats (log-encoded) in `[x0, y0, z0, ...]` order.
 * The pointer is valid until the splat is modified or freed.
 */
	const float *spz_gaussian_splat_scales(const struct SpzGaussianSplat *splat, uintptr_t *out_len);

	/**
 * Returns a pointer to the rotations array.
 *
 * The array contains `num_points * 4` floats (quaternions) in
 * `[x0, y0, z0, w0, ...]` order.
 * The pointer is valid until the splat is modified or freed.
 */

	const float *spz_gaussian_splat_rotations(const struct SpzGaussianSplat *splat, uintptr_t *out_len);

	/**
 * Returns a pointer to the alphas (opacity) array.
 *
 * The array contains `num_points` floats (sigmoid-encoded opacity values).
 * The pointer is valid until the splat is modified or freed.
 */
	const float *spz_gaussian_splat_alphas(const struct SpzGaussianSplat *splat, uintptr_t *out_len);

	/**
 * Returns a pointer to the colors array.
 *
 * The array contains `num_points * 3` floats (DC colour) in `[r0, g0, b0, ...]` order.
 * The pointer is valid until the splat is modified or freed.
 */
	const float *spz_gaussian_splat_colors(const struct SpzGaussianSplat *splat, uintptr_t *out_len);

	/**
 * Returns a pointer to the spherical harmonics coefficients array.
 *
 * The number of coefficients per gaussian depends on the SH degree:
 * - Degree 0: 0 coefficients
 * - Degree 1: 9 coefficients (3 bands x 3 colours)
 * - Degree 2: 24 coefficients
 * - Degree 3: 45 coefficients
 *
 * The pointer is valid until the splat is modified or freed.
 */

	const float *spz_gaussian_splat_spherical_harmonics(const struct SpzGaussianSplat *splat, uintptr_t *out_len);

	/**
 * Converts the splat's coordinate system in-place.
 */

	void spz_gaussian_splat_convert_coordinates(
	    struct SpzGaussianSplat *splat, enum SpzCoordinateSystem from, enum SpzCoordinateSystem to);

	/**
 * Returns a heap-allocated, human-readable summary of the splat.
 *
 * Includes header information, median volume, and bounding box.
 * The caller must free the returned string with `spz_free_string`.
 * Returns NULL if the handle is null.
 */
	char *spz_gaussian_splat_pretty_fmt(const struct SpzGaussianSplat *splat);

	/**
 * Frees a string previously returned by `spz_gaussian_splat_pretty_fmt`
 * or `spz_header_pretty_fmt`.
 */
	void spz_free_string(char *s);

	/**
 * Returns the last error message, or NULL if no error has occurred.
 *
 * The returned string is valid until the next SPZ function call on the same
 * thread. The caller must NOT free this string.
 */
	const char *spz_last_error(void);

	/**
 * Returns the library version as a static null-terminated string.
 */
	const char *spz_version(void);

#ifdef __cplusplus
}  // extern "C"
#endif	// __cplusplus

#endif /* SPZ_H */
