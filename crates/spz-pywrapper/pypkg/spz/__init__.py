# SPDX-License-Identifier: Apache-2.0 OR MIT
"""SPZ - Gaussian Splat file format library.

A fast, efficient library for reading and writing SPZ files,
which store 3D Gaussian Splat point clouds in a compressed format.
"""

from .context_managers import (
    SplatReader,
    SplatWriter,
    modified_splat,
    temp_splat,
)
from .spz import (
    BoundingBox,
    CoordinateSystem,
    GaussianSplat,
    load,
)

__all__ = [
    "GaussianSplat",
    "CoordinateSystem",
    "BoundingBox",
    "load",
    # Context managers
    "SplatReader",
    "SplatWriter",
    "temp_splat",
    "modified_splat",
]
