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

    // Load the SPZ file
    // Use RightUpBack coordinate system (Three.js convention)
    SpzGaussianSplat *splat = spz_gaussian_splat_load(
        filepath,
        SpzCoordinateSystem_RightUpBack
    );

    if (splat == NULL) {
        const char *error = spz_last_error();
        fprintf(stderr, "Error loading SPZ file: %s\n", error ? error : "unknown error");
        return 1;
    }

    // Get basic properties
    int32_t num_points = spz_gaussian_splat_num_points(splat);
    int32_t sh_degree = spz_gaussian_splat_sh_degree(splat);
    bool antialiased = spz_gaussian_splat_antialiased(splat);

    printf("Loaded: %s\n", filepath);
    printf("  Number of points: %d\n", num_points);
    printf("  SH degree: %d\n", sh_degree);
    printf("  Antialiased: %s\n", antialiased ? "yes" : "no");

    // Get bounding box
    SpzBoundingBox bbox = spz_gaussian_splat_bbox(splat);
    printf("  Bounding box:\n");
    printf("    X: [%.3f, %.3f]\n", bbox.min_x, bbox.max_x);
    printf("    Y: [%.3f, %.3f]\n", bbox.min_y, bbox.max_y);
    printf("    Z: [%.3f, %.3f]\n", bbox.min_z, bbox.max_z);

    // Get median volume
    float median_vol = spz_gaussian_splat_median_volume(splat);
    printf("  Median ellipsoid volume: %.6f\n", median_vol);

    // Access position data
    uintptr_t positions_len;
    const float *positions = spz_gaussian_splat_positions(splat, &positions_len);
    if (positions && positions_len > 0) {
        printf("\n  First 3 positions (x, y, z):\n");
        for (int i = 0; i < 3 && i < num_points; i++) {
            printf("    [%d]: (%.4f, %.4f, %.4f)\n",
                   i,
                   positions[i * 3 + 0],
                   positions[i * 3 + 1],
                   positions[i * 3 + 2]);
        }
    }

    // Example: Save to a new file with coordinate conversion
    // (commented out to avoid creating files during example run)
    /*
    int result = spz_gaussian_splat_save(
        splat,
        "output.spz",
        SpzCoordinateSystem_RightUpBack
    );
    if (result != 0) {
        fprintf(stderr, "Error saving: %s\n", spz_last_error());
    }
    */

    // Example: Serialize to bytes
    uint8_t *data = NULL;
    uintptr_t data_len = 0;
    int result = spz_gaussian_splat_to_bytes(
        splat,
        SpzCoordinateSystem_Unspecified,
        &data,
        &data_len
    );
    if (result == 0) {
        printf("\n  Serialized size: %zu bytes\n", (size_t)data_len);
        spz_free_bytes(data, data_len);
    }

    // Clean up
    spz_gaussian_splat_free(splat);

    printf("\nDone!\n");
    return 0;
}
