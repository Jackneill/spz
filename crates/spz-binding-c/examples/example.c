// SPDX-License-Identifier: Apache-2.0 OR MIT

/**
 * SPZ C API Example - Loading and inspecting a Gaussian Splat file
 *
 * Compile with:
 *   gcc -o example example.c -L../target/release -lspz_capi -lpthread -ldl -lm
 *
 * Run with:
 *   LD_LIBRARY_PATH=../target/release ./example path/to/file.spz
 */

#include <stdio.h>
#include <stdlib.h>
#include "../include/spz.h"

int main(int argc, char *argv[]) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <spz_file>\n", argv[0]);
        return 1;
    }

    const char *filepath = argv[1];

    // Print library version
    printf("SPZ C API version: %s\n\n", spz_version());

    // -----------------------------------------------------------------------
    // Quick header inspection (does NOT decompress the full file)
    // -----------------------------------------------------------------------
    SpzHeader *header = spz_header_from_file(filepath);
    if (header == NULL) {
        const char *error = spz_last_error();
        fprintf(stderr, "Error reading header: %s\n",
                error ? error : "unknown error");
        return 1;
    }

    printf("Header-only inspection:\n");
    printf("  Version:         v%d\n", (int)spz_header_version(header));
    printf("  Num points:      %d\n", spz_header_num_points(header));
    printf("  SH degree:       %d\n", spz_header_sh_degree(header));
    printf("  Fractional bits: %d\n", spz_header_fractional_bits(header));
    printf("  Antialiased:     %s\n",
           spz_header_antialiased(header) ? "yes" : "no");
    printf("  Valid:           %s\n",
           spz_header_is_valid(header) ? "yes" : "no");

    // Pretty-printed header summary (caller must free with spz_free_string)
    char *header_summary = spz_header_pretty_fmt(header);
    if (header_summary) {
        printf("\n%s\n", header_summary);
        spz_free_string(header_summary);
    }

    spz_header_free(header);

    // -----------------------------------------------------------------------
    // Full load
    // -----------------------------------------------------------------------
    SpzGaussianSplat *splat = spz_gaussian_splat_load(
        filepath,
        SpzCoordinateSystem_RightUpBack
    );

    if (splat == NULL) {
        const char *error = spz_last_error();
        fprintf(stderr, "Error loading SPZ file: %s\n",
                error ? error : "unknown error");
        return 1;
    }

    // Scalar accessors
    int32_t num_points = spz_gaussian_splat_num_points(splat);
    uint8_t sh_degree  = spz_gaussian_splat_sh_degree(splat);
    bool antialiased   = spz_gaussian_splat_antialiased(splat);

    printf("Loaded: %s\n", filepath);
    printf("  Number of points:  %d\n", num_points);
    printf("  SH degree:         %d\n", sh_degree);
    printf("  Version:           v%d\n",
           (int)spz_gaussian_splat_version(splat));
    printf("  Fractional bits:   %d\n",
           spz_gaussian_splat_fractional_bits(splat));
    printf("  Antialiased:       %s\n", antialiased ? "yes" : "no");
    printf("  Sizes consistent:  %s\n",
           spz_gaussian_splat_check_sizes(splat) ? "yes" : "no");

    // Bounding box
    SpzBoundingBox bbox = spz_gaussian_splat_bbox(splat);
    printf("  Bounding box:\n");
    printf("    X: [%.3f, %.3f]\n", bbox.min_x, bbox.max_x);
    printf("    Y: [%.3f, %.3f]\n", bbox.min_y, bbox.max_y);
    printf("    Z: [%.3f, %.3f]\n", bbox.min_z, bbox.max_z);

    // Median volume
    float median_vol = spz_gaussian_splat_median_volume(splat);
    printf("  Median ellipsoid volume: %.6f\n", median_vol);

    // Pretty-printed full summary
    char *splat_summary = spz_gaussian_splat_pretty_fmt(splat);
    if (splat_summary) {
        printf("\n%s\n", splat_summary);
        spz_free_string(splat_summary);
    }

    // Access position data
    uintptr_t positions_len;
    const float *positions = spz_gaussian_splat_positions(splat, &positions_len);
    if (positions && positions_len > 0) {
        printf("  First 3 positions (x, y, z):\n");
        for (int i = 0; i < 3 && i < num_points; i++) {
            printf("    [%d]: (%.4f, %.4f, %.4f)\n",
                   i,
                   positions[i * 3 + 0],
                   positions[i * 3 + 1],
                   positions[i * 3 + 2]);
        }
    }

    // Example: serialize to bytes and back
    uint8_t *data = NULL;
    uintptr_t data_len = 0;
    SpzResult result = spz_gaussian_splat_to_bytes(
        splat,
        SpzCoordinateSystem_Unspecified,
        &data,
        &data_len
    );
    if (result == SpzResult_Success) {
        printf("\n  Serialized size: %zu bytes\n", (size_t)data_len);
        spz_free_bytes(data, data_len);
    } else {
        fprintf(stderr, "  Serialize error: %s\n", spz_last_error());
    }

    // Clean up
    spz_gaussian_splat_free(splat);

    printf("\nDone!\n");
    return 0;
}
