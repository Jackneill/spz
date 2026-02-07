# SPDX-License-Identifier: Apache-2.0 OR MIT

"""SPZ - Gaussian Splat file format library.

A fast, efficient library for reading and writing SPZ files,
which store 3D Gaussian Splat point clouds in a compressed format.
"""

from ._context_managers import (
    SplatReader,
    SplatWriter,
    modified_splat,
    temp_save,
)
from ._spz import (
    BoundingBox,
    CoordinateSystem,
    GaussianSplat,
    Header,
    Version,
    load,
    read_header,
)

__all__ = [
    "GaussianSplat",
    "CoordinateSystem",
    "BoundingBox",
    "Header",
    "Version",
    "load",
    "read_header",
    # Context managers
    "SplatReader",
    "SplatWriter",
    "temp_save",
    "modified_splat",
]
