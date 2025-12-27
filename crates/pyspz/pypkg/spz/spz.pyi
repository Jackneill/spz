# SPDX-License-Identifier: Apache-2.0 OR MIT
"""SPZ - Gaussian Splat file format handling.

Python implementation (in Rust) of the .SPZ file format.
"""


class CoordinateSystem:
    """Coordinate system enumeration.

    Each coordinate system is defined by three axes:
        - L/R: Left/Right for X axis
        - U/D: Up/Down for Y axislibrary
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

    def __init__(self, name: str = "UNSPECIFIED") -> None:
        """Create a coordinate system from its name.

        Args:
            name: Name of the coordinate system (e.g., "RUB", "RDF").
        """
        ...

    def __repr__(self) -> str: ...
    def __str__(self) -> str: ...
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...


class BoundingBox:
    """Bounding box of a Gaussian splat.

    This class cannot be instantiated directly. It is returned by
    the ``bbox`` property of ``GaussianSplat``.

    Attributes:
        min_x: Minimum X coordinate.
        max_x: Maximum X coordinate.
        min_y: Minimum Y coordinate.
        max_y: Maximum Y coordinate.
        min_z: Minimum Z coordinate.
        max_z: Maximum Z coordinate.
    """

    min_x: float
    max_x: float
    min_y: float
    max_y: float
    min_z: float
    max_z: float

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

    Example:
        Load from file::

            splat = spz.GaussianSplat.load("scene.spz")
            print(f"Loaded {splat.num_points} gaussians")

        Create from data::

            splat = spz.GaussianSplat(
                positions=[0.0, 0.0, 0.0, 1.0, 0.0, 0.0],  # 2 points
                scales=[-5.0] * 6,
                rotations=[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
                alphas=[0.0, 0.0],
                colors=[0.0] * 6,
                sh_degree=0
            )
    """

    def __init__(
        self,
        positions: list[float],
        scales: list[float],
        rotations: list[float],
        alphas: list[float],
        colors: list[float],
        sh_degree: int = 0,
        spherical_harmonics: list[float] | None = None,
        antialiased: bool = False,
    ) -> None:
        """Create a new GaussianSplat from arrays.

        Args:
            positions: Flattened (x, y, z) positions. Length must be
                num_points * 3.
            scales: Flattened (x, y, z) log-scale values. Length must be
                num_points * 3.
            rotations: Flattened (x, y, z, w) quaternion rotations. Length
                must be num_points * 4.
            alphas: Inverse-sigmoid opacity values. Length must be num_points.
            colors: Flattened (r, g, b) SH0 color values. Length must be
                num_points * 3.
            sh_degree: Spherical harmonics degree (0-3). Defaults to 0.
            spherical_harmonics: Flattened SH coefficients. Length depends
                on sh_degree. Defaults to None.
            antialiased: Whether the splat was trained with antialiasing.
                Defaults to False.
        """
        ...

    @staticmethod
    def load(
        path: str, coordinate_system = CoordinateSystem.UNSPECIFIED
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
        data: bytes, coordinate_system = CoordinateSystem.UNSPECIFIED,
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
        from_coordinate_system = CoordinateSystem.UNSPECIFIED,
    ) -> None:
        """Save the GaussianSplat to an SPZ file.

        Args:
            path: Path to save the SPZ file.
            from_coordinate_system: Source coordinate system of the data.
                Defaults to UNSPECIFIED (no conversion).
        """
        ...

    def to_bytes(
        self, from_coordinate_system = CoordinateSystem.UNSPECIFIED,
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

    def rotate_180_deg_about_x(self) -> None:
        """Rotate 180 degrees about the X axis (RUB <-> RDF conversion)."""
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
    def positions(self) -> list[float]:
        """Flattened (x, y, z) positions. Length is num_points * 3."""
        ...

    @property
    def scales(self) -> list[float]:
        """Flattened (x, y, z) log-scale values. Length is num_points * 3."""
        ...

    @property
    def rotations(self) -> list[float]:
        """Flattened (x, y, z, w) quaternion rotations. Length is num_points * 4."""
        ...

    @property
    def alphas(self) -> list[float]:
        """Inverse-sigmoid opacity values. Length is num_points."""
        ...

    @property
    def colors(self) -> list[float]:
        """Flattened (r, g, b) SH0 color values. Length is num_points * 3."""
        ...

    @property
    def spherical_harmonics(self) -> list[float]:
        """Flattened spherical harmonics coefficients."""
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
    path: str, coordinate_system = CoordinateSystem.UNSPECIFIED,
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
