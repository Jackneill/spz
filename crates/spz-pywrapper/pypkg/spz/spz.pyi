# SPDX-License-Identifier: Apache-2.0 OR MIT

"""SPZ - Gaussian Splat file format handling.

Python implementation (in Rust) of the .SPZ file format.
"""

import numpy as np
import numpy.typing as npt

class CoordinateSystem:
    """Coordinate system enumeration.

    Each coordinate system is defined by three axes:
        - L/R: Left/Right for X axis
        - U/D: Up/Down for Y axis
        - F/B: Front/Back for Z axis

    Common systems:
        - RUB: Three.js coordinate system (Right, Up, Back)
        - RDF: PLY file coordinate system (Right, Down, Front)
        - LUF: GLB/glTF coordinate system (Left, Up, Front)
        - RUF: Unity coordinate system (Right, Up, Front)
    """

    LDB: CoordinateSystem
    """Left Down Back."""

    RDB: CoordinateSystem
    """Right Down Back."""

    LUB: CoordinateSystem
    """Left Up Back."""

    RUB: CoordinateSystem
    """Right Up Back (Three.js coordinate system)."""

    LDF: CoordinateSystem
    """Left Down Front."""

    RDF: CoordinateSystem
    """Right Down Front (PLY file coordinate system)."""

    LUF: CoordinateSystem
    """Left Up Front (GLB/glTF coordinate system)."""

    RUF: CoordinateSystem
    """Right Up Front (Unity coordinate system)."""

    UNSPECIFIED: CoordinateSystem
    """Unspecified coordinate system (no conversion)."""

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...

class BoundingBox:
    """Bounding box of a Gaussian splat.

    This class cannot be instantiated directly.
    It is returned by the ``bbox`` property of ``GaussianSplat``.
    """

    min_x: float
    """Minimum X coordinate."""
    max_x: float
    """Maximum X coordinate."""
    min_y: float
    """Minimum Y coordinate."""
    max_y: float
    """Maximum Y coordinate."""
    min_z: float
    """Minimum Z coordinate."""
    max_z: float
    """Maximum Z coordinate."""

    @property
    def size(self) -> tuple[float, float, float]:
        """Get the size (extent) of the bounding box in each dimension.

        Returns:
            A tuple of (width, height, depth).
        """
        ...

    @property
    def center(self) -> tuple[float, float, float]:
        """Get the center of the bounding box.

        Returns:
            A tuple of (x, y, z) center coordinates.
        """
        ...

    def __repr__(self) -> str: ...

class GaussianSplat:
    """A 3D Gaussian Splat point cloud.

    This class represents a collection of 3D Gaussians used for
    Gaussian Splatting rendering. Each Gaussian has a position,
    rotation, scale, color, alpha (opacity), and spherical harmonics
    coefficients for view-dependent appearance.

    All array data is returned as numpy arrays for efficient processing.

    Example:
        Load from file::

            splat = spz.GaussianSplat.load("scene.spz")
            print(f"Loaded {splat.num_points} gaussians")
            positions = splat.positions  # shape: (num_points, 3)

        Create from numpy arrays::

            import numpy as np

            splat = spz.GaussianSplat(
                positions=np.zeros((100, 3), dtype=np.float32),
                scales=np.full((100, 3), -5.0, dtype=np.float32),
                rotations=np.tile([1, 0, 0, 0], (100, 1)).astype(np.float32),
                alphas=np.zeros(100, dtype=np.float32),
                colors=np.zeros((100, 3), dtype=np.float32),
            )
    """

    def __init__(
        self,
        /,
        positions: npt.NDArray[np.float32],
        scales: npt.NDArray[np.float32],
        rotations: npt.NDArray[np.float32],
        alphas: npt.NDArray[np.float32],
        colors: npt.NDArray[np.float32],
        sh_degree: int = 0,
        spherical_harmonics: npt.NDArray[np.float32] | None = None,
        antialiased: bool = False,
    ) -> None:
        """Create a new GaussianSplat from numpy arrays.

        Args:
            positions: (N, 3) array of (x, y, z) positions.
            scales: (N, 3) array of (x, y, z) log-scale values.
            rotations: (N, 4) array of (w, x, y, z) quaternion rotations.
            alphas: (N,) array of inverse-sigmoid opacity values.
            colors: (N, 3) array of (r, g, b) SH0 color values.
            sh_degree: Spherical harmonics degree (0-3). Defaults to 0.
            spherical_harmonics: (N, sh_dim, 3) array of SH coefficients.
                Defaults to None.
            antialiased: Whether the splat was trained with antialiasing.
                Defaults to False.
        """
        ...

    @staticmethod
    def load(
        path: str, coordinate_system=CoordinateSystem.UNSPECIFIED
    ) -> GaussianSplat:
        """Load a GaussianSplat from an SPZ file.

        Args:
            path: Path to the SPZ file.
            coordinate_system: Target coordinate system for the loaded data.
                Defaults to UNSPECIFIED (no conversion).

        Returns:
            The loaded Gaussian splat.

        Raises:
            ValueError: If the file cannot be read or is invalid.
        """
        ...

    @staticmethod
    def from_bytes(
        data: bytes,
        coordinate_system=CoordinateSystem.UNSPECIFIED,
    ) -> GaussianSplat:
        """Load a GaussianSplat from bytes.

        Args:
            data: The SPZ file contents as bytes.
            coordinate_system: Target coordinate system for the loaded data.
                Defaults to UNSPECIFIED (no conversion).

        Returns:
            The loaded Gaussian splat.
        """
        ...

    def save(
        self,
        path: str,
        from_coordinate_system=CoordinateSystem.UNSPECIFIED,
    ) -> None:
        """Save the GaussianSplat to an SPZ file.

        Args:
            path: Path to save the SPZ file.
            from_coordinate_system: Source coordinate system of the data.
                Defaults to UNSPECIFIED (no conversion).
        """
        ...

    def to_bytes(
        self,
        from_coordinate_system=CoordinateSystem.UNSPECIFIED,
    ) -> bytes:
        """Serialize the GaussianSplat to bytes.

        Args:
            from_coordinate_system: Source coordinate system of the data.
                Defaults to UNSPECIFIED (no conversion).

        Returns:
            The SPZ file contents as bytes.
        """
        ...

    def convert_coordinates(
        self, from_system: CoordinateSystem, to_system: CoordinateSystem
    ) -> None:
        """Convert coordinates to a different coordinate system.

        Args:
            from_system: The current coordinate system of the data.
            to_system: The target coordinate system.
        """
        ...

    @property
    def num_points(self) -> int:
        """The number of Gaussian points."""
        ...

    @property
    def sh_degree(self) -> int:
        """The spherical harmonics degree (0-3)."""
        ...

    @property
    def antialiased(self) -> bool:
        """Whether the splat was trained with antialiasing."""
        ...

    @property
    def positions(self) -> npt.NDArray[np.float32]:
        """(N, 3) array of (x, y, z) positions."""
        ...

    @property
    def scales(self) -> npt.NDArray[np.float32]:
        """(N, 3) array of (x, y, z) log-scale values."""
        ...

    @property
    def rotations(self) -> npt.NDArray[np.float32]:
        """(N, 4) array of (w, x, y, z) quaternion rotations."""
        ...

    @property
    def alphas(self) -> npt.NDArray[np.float32]:
        """(N,) array of inverse-sigmoid opacity values."""
        ...

    @property
    def colors(self) -> npt.NDArray[np.float32]:
        """(N, 3) array of (r, g, b) SH0 color values."""
        ...

    @property
    def spherical_harmonics(self) -> npt.NDArray[np.float32]:
        """(N, sh_dim * 3) array of spherical harmonics coefficients.

        The sh_dim depends on sh_degree: 0→0, 1→3, 2→8, 3→15.

        Returns an empty (N, 0) array if sh_degree is 0.
        """
        ...

    @property
    def bbox(self) -> BoundingBox:
        """The bounding box of the splat."""
        ...

    @property
    def median_volume(self) -> float:
        """The median ellipsoid volume of the Gaussians.

        This is useful for understanding the typical size of the
        Gaussians in the point cloud.
        """
        ...

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __len__(self) -> int: ...

def load(
    path: str,
    coordinate_system=CoordinateSystem.UNSPECIFIED,
) -> GaussianSplat:
    """Load a GaussianSplat from an SPZ file.

    This is a convenience function equivalent to ``GaussianSplat.load()``.

    Args:
        path: Path to the SPZ file.
        coordinate_system: Target coordinate system for the loaded data.

    Returns:
        The loaded Gaussian splat.

    Example:
        >>> import spz
        >>> splat = spz.load("scene.spz")
        >>> print(f"Loaded {len(splat)} gaussians")
    """
    ...
