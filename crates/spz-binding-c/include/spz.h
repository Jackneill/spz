// SPDX-License-Identifier: Apache-2.0 OR MIT

#ifndef SPZ_H
#define SPZ_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

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
 * Opaque handle to a GaussianSplat object.
 *
 * This handle must be freed with `spz_gaussian_splat_free` when no longer needed.
 */
typedef struct SpzGaussianSplat SpzGaussianSplat;

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
 * Returns the last error message, or NULL if no error occurred.
 *
 * The returned string is valid until the next SPZ function call on this thread.
 * The caller must NOT free this string.
 */
	const char *spz_last_error(void);

	/**
 * Creates a new empty GaussianSplat.
 *
 * Returns NULL on failure. Call `spz_last_error()` for error details.
 * The caller must free the returned handle with `spz_gaussian_splat_free`.
 */
	struct SpzGaussianSplat *spz_gaussian_splat_new(void);

	/**
 * Loads a GaussianSplat from an SPZ file.
 *
 * # Args
 *
 * - `filepath`: Path to the SPZ file (null-terminated UTF-8 string).
 * - `coord_sys`: Target coordinate system for the loaded data.
 *
 * Returns NULL on failure. Call `spz_last_error()` for error details.
 * The caller must free the returned handle with `spz_gaussian_splat_free`.
 */

	struct SpzGaussianSplat *spz_gaussian_splat_load(const char *filepath, enum SpzCoordinateSystem coord_sys);

	/**
 * Loads a GaussianSplat from a byte buffer containing SPZ data.
 *
 * # Args
 *
 * - `data`: Pointer to the SPZ data buffer.
 * - `len`: Length of the data buffer in bytes.
 * - `coord_sys`: Target coordinate system for the loaded data.
 *
 * Returns NULL on failure. Call `spz_last_error()` for error details.
 * The caller must free the returned handle with `spz_gaussian_splat_free`.
 */

	struct SpzGaussianSplat *
	spz_gaussian_splat_load_from_bytes(const uint8_t *data, uintptr_t len, enum SpzCoordinateSystem coord_sys);

	/**
 * Saves a GaussianSplat to an SPZ file.
 *
 * # Args
 *
 * - `splat`: Handle to the GaussianSplat.
 * - `filepath`: Path to save the SPZ file (null-terminated UTF-8 string).
 * - `coord_sys`: Source coordinate system of the data being saved.
 *
 * Returns 0 on success, -1 on failure. Call `spz_last_error()` for error details.
 */

	int spz_gaussian_splat_save(
	    const struct SpzGaussianSplat *splat, const char *filepath, enum SpzCoordinateSystem coord_sys);

	/**
 * Serializes a GaussianSplat to a byte buffer.
 *
 * # Args
 *
 * - `splat`: Handle to the GaussianSplat.
 * - `coord_sys`: Source coordinate system of the data being saved.
 * - `out_data`: Pointer to receive the allocated data buffer.
 * - `out_len`: Pointer to receive the buffer length.
 *
 * Returns 0 on success, -1 on failure. Call `spz_last_error()` for error details.
 * The caller must free the returned buffer with `spz_free_bytes`.
 */

	int spz_gaussian_splat_to_bytes(
	    const struct SpzGaussianSplat *splat,
	    enum SpzCoordinateSystem coord_sys,
	    uint8_t **out_data,
	    uintptr_t *out_len);

	/**
 * Frees a byte buffer allocated by `spz_gaussian_splat_to_bytes`.
 */
	void spz_free_bytes(uint8_t *data, uintptr_t len);

	/**
 * Frees a GaussianSplat handle.
 */
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
 * Returns whether the splat was trained with antialiasing.
 */
	bool spz_gaussian_splat_antialiased(const struct SpzGaussianSplat *splat);

	/**
 * Returns the bounding box of the splat.
 */
	struct SpzBoundingBox spz_gaussian_splat_bbox(const struct SpzGaussianSplat *splat);

	/**
 * Returns the median ellipsoid volume of the gaussians.
 */
	float spz_gaussian_splat_median_volume(const struct SpzGaussianSplat *splat);

	/**
 * Returns a pointer to the positions array.
 *
 * The array contains `num_points * 3` floats in [x0, y0, z0, x1, y1, z1, ...] order.
 * The pointer is valid until the splat is modified or freed.
 */

	const float *spz_gaussian_splat_positions(const struct SpzGaussianSplat *splat, uintptr_t *out_len);

	/**
 * Returns a pointer to the scales array.
 *
 * The array contains `num_points * 3` floats (log-encoded) in [x0, y0, z0, ...] order.
 * The pointer is valid until the splat is modified or freed.
 */
	const float *spz_gaussian_splat_scales(const struct SpzGaussianSplat *splat, uintptr_t *out_len);

	/**
 * Returns a pointer to the rotations array.
 *
 * The array contains `num_points * 4` floats (quaternions) in [x0, y0, z0, w0, ...] order.
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
 * The array contains `num_points * 3` floats (DC color) in [r0, g0, b0, ...] order.
 * The pointer is valid until the splat is modified or freed.
 */
	const float *spz_gaussian_splat_colors(const struct SpzGaussianSplat *splat, uintptr_t *out_len);

	/**
 * Returns a pointer to the spherical harmonics array.
 *
 * The array contains SH coefficients for degrees 1-3.
 * The number of coefficients per gaussian depends on the SH degree:
 * - Degree 0: 0 coefficients
 * - Degree 1: 9 coefficients (3 bands × 3 colors)
 * - Degree 2: 24 coefficients
 * - Degree 3: 45 coefficients
 *
 * The pointer is valid until the splat is modified or freed.
 */

	const float *spz_gaussian_splat_spherical_harmonics(const struct SpzGaussianSplat *splat, uintptr_t *out_len);

	/**
 * Converts the splat's coordinate system in-place.
 *
 * # Args
 *
 * - `splat`: Handle to the GaussianSplat.
 * - `from`: Source coordinate system.
 * - `to`: Target coordinate system.
 */

	void spz_gaussian_splat_convert_coordinates(
	    struct SpzGaussianSplat *splat, enum SpzCoordinateSystem from, enum SpzCoordinateSystem to);

	/**
 * Returns the library version as a static string.
 */
	const char *spz_version(void);

#ifdef __cplusplus
}  // extern "C"
#endif	// __cplusplus

#endif /* SPZ_H */
