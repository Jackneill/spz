# SPDX-License-Identifier: Apache-2.0 OR MIT
"""Extra Python utilities for the spz library.

This module provides Pythonic conveniences like context managers
and helper functions that complement the Rust-based core.
"""

from __future__ import annotations

import tempfile
from collections.abc import Iterator
from contextlib import contextmanager
from os import PathLike
from pathlib import Path

from spz import CoordinateSystem, GaussianSplat

__all__ = [
    "SplatReader",
    "SplatWriter",
    "temp_splat",
    "modified_splat",
]


@contextmanager
def open_spz(
    path: PathLike,
    coordinate_system=CoordinateSystem.UNSPECIFIED,
) -> Iterator[GaussianSplat]:
    """Context manager for loading an SPZ file.

    This provides a clean way to work with SPZ files using the
    `with` statement, ensuring proper resource handling.

    Args:
        path: Path to the SPZ file.
        coordinate_system: Target coordinate system for the loaded data.

    Yields:
        The loaded GaussianSplat object.

    Example:
        >>> with open_spz("scene.spz") as splat:
        ...     print(f"Loaded {len(splat)} gaussians")
        ...     # Work with splat...
    """
    from spz import GaussianSplat

    splat = GaussianSplat.load(str(path), coordinate_system)
    try:
        yield splat
    finally:
        pass


class SplatReader:
    """Context manager for opening and reading SPZ files.

    This provides a convenient way to read a GaussianSplat and automatically
    free it when the context exits.

    Args:
        path: Path to the SPZ file.
        from_coordinate_system: Source coordinate system when reading.

    Example:
        >>> with SplatReader("input.spz") as ctx:
        ...     splat = ctx.splat
        ...
        ...     # Work with splat...
    """

    def __init__(
        self,
        path: PathLike,
        from_coordinate_system=CoordinateSystem.UNSPECIFIED,
    ) -> None:
        """Initialize the SplatReader.

        Args:
            path: Path to the SPZ file.
            from_coordinate_system: Source coordinate system.
        """
        self.path = Path(path)
        self.from_coordinate_system = from_coordinate_system
        self.splat: GaussianSplat | None = None

        self._save_on_exit = True

    def __enter__(self) -> SplatReader:
        """Enter the context."""
        return self

    def __exit__(self, exc_type, _exc_val, _exc_tb) -> None:  # noqa: ANN001
        pass

    def cancel(self) -> None:
        pass


class SplatWriter:
    """Context manager for creating and saving SPZ files.

    This provides a convenient way to create a GaussianSplat, modify it,
    and automatically save it when the context exits.

    Args:
        path: Path where the SPZ file will be saved.
        from_coordinate_system: Source coordinate system when saving.

    Example:
        >>> with SplatWriter("output.spz") as ctx:
        ...     ctx.splat = spz.GaussianSplat(
        ...         positions=[0.0, 0.0, 0.0],
        ...         scales=[-5.0, -5.0, -5.0],
        ...         rotations=[1.0, 0.0, 0.0, 0.0],
        ...         alphas=[0.5],
        ...         colors=[255.0, 0.0, 0.0],
        ...     )
        ...     # Modify ctx.splat as needed...
        ... # Automatically saved on exit
    """

    def __init__(
        self,
        path: PathLike,
        from_coordinate_system=CoordinateSystem.UNSPECIFIED,
    ) -> None:
        """Initialize the SplatWriter.

        Args:
            path: Path where the SPZ file will be saved.
            from_coordinate_system: Source coordinate system when saving.
        """
        self.path = Path(path)
        self.from_coordinate_system = from_coordinate_system
        self.splat: GaussianSplat | None = None

        self._save_on_exit = True

    def __enter__(self) -> SplatWriter:
        """Enter the context."""
        return self

    def __exit__(self, exc_type, _exc_val, _exc_tb) -> None:  # noqa: ANN001
        """Exit the context and save the splat if no exception occurred."""
        if exc_type is None and self._save_on_exit and self.splat is not None:
            self.splat.save(str(self.path), self.from_coordinate_system)

    def cancel(self) -> None:
        """Cancel the automatic save on exit."""
        self._save_on_exit = False


@contextmanager
def temp_splat(
    splat: GaussianSplat,
    from_coordinate_system=CoordinateSystem.UNSPECIFIED,
    suffix: str = ".spz",
) -> Iterator[Path]:
    """Context manager that saves a GaussianSplat to a temporary file.

    Useful for passing splat data to external tools that require file paths.
    The temporary file is automatically deleted when the context exits.

    Args:
        splat: The GaussianSplat to save.
        from_coordinate_system: Source coordinate system when saving.
        suffix: File suffix for the temporary file.

    Yields:
        Path to the temporary SPZ file.

    Example:
        >>> with temp_splat(my_splat) as temp_path:
        ...     # Pass temp_path to external tool
        ...     subprocess.run(["viewer", str(temp_path)])
        ... # Temp file is deleted
    """
    with tempfile.NamedTemporaryFile(suffix=suffix, delete=False) as f:
        temp_path = Path(f.name)

    try:
        splat.save(str(temp_path), from_coordinate_system)

        yield temp_path
    finally:
        temp_path.unlink(missing_ok=True)


@contextmanager
def modified_splat(
    path: PathLike,
    output_path: PathLike | None = None,
    from_coordinate_system=CoordinateSystem.UNSPECIFIED,
    to_coordinate_system=CoordinateSystem.UNSPECIFIED,
) -> Iterator[GaussianSplat]:
    """Context manager for loading, modifying, and saving an SPZ file.

    Loads an SPZ file, yields it for modification, and saves it when
    the context exits. If output_path is None, overwrites the original file.

    Args:
        path: Path to the input SPZ file.
        output_path: Path to save the modified file. If None, overwrites input.
        from_coordinate_system: Coordinate system for loading.
        to_coordinate_system: Coordinate system for saving.

    Yields:
        The loaded GaussianSplat for modification.

    Example:
        >>> with modified_splat("scene.spz", "scene_rotated.spz") as splat:
        ...     splat.rotate_180_deg_about_x()
        ... # Automatically saved to scene_rotated.spz
    """
    from spz import GaussianSplat

    splat = GaussianSplat.load(str(path), from_coordinate_system)

    try:
        yield splat
    finally:
        save_path = output_path if output_path is not None else path
        splat.save(str(save_path), to_coordinate_system)
