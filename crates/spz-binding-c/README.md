# SPZ C Bindings

C bindings/api for the SPZ Gaussian Splat file format library.

## Development

```bash
../../just cbuild
```

This produces:

- `target/release/libspz_capi.so` (Linux)
- `target/release/libspz_capi.dylib` (macOS)
- `target/release/spz_capi.dll` (Windows)

And the header file is generated at `include/spz.h`.

## Usage

### Linux/macOS

```bash
gcc -o myapp myapp.c -L/path/to/lib -lspz_capi -lpthread -ldl -lm
```

### CMake

```cmake
find_library(SPZ_CAPI spz_capi PATHS /path/to/lib)
target_link_libraries(myapp ${SPZ_CAPI})
```

### Meson

```meson
project('myapp', 'c')

spz_inc = include_directories('/path/to/spz-binding-c/include')
spz_lib = meson.get_compiler('c').find_library('spz_capi',
  dirs: '/path/to/lib',
)

executable('myapp', 'myapp.c',
  include_directories: spz_inc,
  dependencies: spz_lib,
)
```

If you install `libspz_capi` somewhere on the default library search path
(e.g. `/usr/local/lib`) and the header to `/usr/local/include`, you can
simplify this:

```meson
project('myapp', 'c')

spz_dep = dependency('spz_capi', required: false)
if not spz_dep.found()
  spz_dep = meson.get_compiler('c').find_library('spz_capi')
endif

executable('myapp', 'myapp.c',
  dependencies: spz_dep,
)
```

## Examples

### Loading an SPZ file

```c
#include "spz.h"

// Load with coordinate system conversion
SpzGaussianSplat *splat = spz_gaussian_splat_load(
    "scene.spz",
    SpzCoordinateSystem_RightUpBack  // Three.js coordinates
);

if (splat == NULL) {
    fprintf(stderr, "Error: %s\n", spz_last_error());
    return 1;
}

// Use the splat...
int32_t num_points = spz_gaussian_splat_num_points(splat);

// Don't forget to free
spz_gaussian_splat_free(splat);
```

### Loading from memory

```c
uint8_t *buffer = ...;  // SPZ file contents
size_t buffer_len = ...;

SpzGaussianSplat *splat = spz_gaussian_splat_load_from_bytes(
    buffer,
    buffer_len,
    SpzCoordinateSystem_Unspecified
);
```

### Accessing data

```c
// Get positions array (num_points * 3 floats)
uintptr_t len;
const float *positions = spz_gaussian_splat_positions(splat, &len);

// Get bounding box
SpzBoundingBox bbox = spz_gaussian_splat_bbox(splat);
printf("Size: %.2f x %.2f x %.2f\n",
       bbox.max_x - bbox.min_x,
       bbox.max_y - bbox.min_y,
       bbox.max_z - bbox.min_z);
```

### Saving

```c
// Save to file
int result = spz_gaussian_splat_save(splat, "output.spz",
    SpzCoordinateSystem_RightUpBack);

// Or serialize to bytes
uint8_t *data;
uintptr_t data_len;
if (spz_gaussian_splat_to_bytes(splat, SpzCoordinateSystem_Unspecified,
                                 &data, &data_len) == 0) {
    // Use data...
    spz_free_bytes(data, data_len);
}
```

### Coordinate conversion

```c
// Convert in-place from PLY coordinates to Unity coordinates
spz_gaussian_splat_convert_coordinates(
    splat,
    SpzCoordinateSystem_RightDownFront,  // PLY format
    SpzCoordinateSystem_RightUpFront     // Unity
);
```

## Error Handling

All functions that can fail return `NULL` (for pointers) or `-1` (for int).
Use `spz_last_error()` to get the error message:

```c
SpzGaussianSplat *splat = spz_gaussian_splat_load("missing.spz",
    SpzCoordinateSystem_Unspecified);
if (splat == NULL) {
    const char *error = spz_last_error();
    fprintf(stderr, "Failed to load: %s\n", error ? error : "unknown");
}
```
