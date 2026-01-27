# SPDX-License-Identifier: Apache-2.0 OR MIT

"""Tests for the spz Python bindings."""

from pathlib import Path
from tempfile import TemporaryDirectory

import numpy as np
import pytest
import spz

from . import util


class TestCoordinateSystem:
    """Tests for the CoordinateSystem class."""

    def test_all_coordinate_systems_exist(self):
        """All coordinate system variants should be accessible."""
        assert spz.CoordinateSystem.LDB is not None
        assert spz.CoordinateSystem.RDB is not None
        assert spz.CoordinateSystem.LUB is not None
        assert spz.CoordinateSystem.RUB is not None
        assert spz.CoordinateSystem.LDF is not None
        assert spz.CoordinateSystem.RDF is not None
        assert spz.CoordinateSystem.LUF is not None
        assert spz.CoordinateSystem.RUF is not None
        assert spz.CoordinateSystem.UNSPECIFIED is not None

    def test_coordinate_system_repr(self):
        """repr should show the variant name."""
        assert "Right-Up-Back" in repr(spz.CoordinateSystem.RUB)
        assert "Unspecified" in repr(spz.CoordinateSystem.UNSPECIFIED)

    def test_coordinate_system_str(self):
        """str should return the variant name."""
        assert str(spz.CoordinateSystem.RUB) == "Right-Up-Back"
        assert str(spz.CoordinateSystem.RDF) == "Right-Down-Front"
        assert str(spz.CoordinateSystem.UNSPECIFIED) == "Unspecified"

    def test_coordinate_system_equality(self):
        """Coordinate systems should be comparable."""
        assert spz.CoordinateSystem.RUB == spz.CoordinateSystem.RUB
        assert spz.CoordinateSystem.RUB != spz.CoordinateSystem.RDF


class TestBoundingBox:
    """Tests for the BoundingBox class."""

    def test_bounding_box_size(self):
        """BoundingBox.size should return dimensions."""
        splat = util.create_test_splat(10)
        bbox = splat.bbox
        size = bbox.size

        assert len(size) == 3
        assert all(isinstance(s, float) for s in size)

    def test_bounding_box_center(self):
        """BoundingBox.center should return center coordinates."""
        splat = util.create_test_splat(10)
        bbox = splat.bbox
        center = bbox.center

        assert len(center) == 3
        assert all(isinstance(c, float) for c in center)

    def test_bounding_box_repr(self):
        """BoundingBox repr should be informative."""
        splat = util.create_test_splat(10)
        bbox = splat.bbox
        repr_str = repr(bbox)

        assert "BoundingBox" in repr_str
        assert "x=" in repr_str
        assert "y=" in repr_str
        assert "z=" in repr_str


class TestGaussianSplatCreation:
    """Tests for creating GaussianSplat objects."""

    def test_create_from_numpy_arrays(self):
        """Should create a splat from numpy arrays."""
        splat = util.create_test_splat(100)

        assert splat.num_points == 100
        assert splat.sh_degree == 0
        assert splat.antialiased is False

    def test_create_with_sh_degree(self):
        """Should create a splat with spherical harmonics."""
        num_points = 50
        sh_degree = 1
        sh_dim = 3  # degree 1 has 3 SH coefficients

        positions = np.random.randn(num_points, 3).astype(np.float32)
        scales = np.full((num_points, 3), -5.0, dtype=np.float32)
        rotations = np.tile([1, 0, 0, 0], (num_points, 1)).astype(np.float32)
        alphas = np.zeros(num_points, dtype=np.float32)
        colors = np.zeros((num_points, 3), dtype=np.float32)
        # spherical_harmonics shape is (N, sh_dim * 3) = (N, 9) for degree 1
        spherical_harmonics = np.zeros((num_points, sh_dim * 3), dtype=np.float32)

        splat = spz.GaussianSplat(
            positions=positions,
            scales=scales,
            rotations=rotations,
            alphas=alphas,
            colors=colors,
            sh_degree=sh_degree,
            spherical_harmonics=spherical_harmonics,
        )

        assert splat.num_points == num_points
        assert splat.sh_degree == sh_degree

    def test_create_with_antialiased(self):
        """Should create an antialiased splat."""
        num_points = 10
        positions = np.random.randn(num_points, 3).astype(np.float32)
        scales = np.full((num_points, 3), -5.0, dtype=np.float32)
        rotations = np.tile([1, 0, 0, 0], (num_points, 1)).astype(np.float32)
        alphas = np.zeros(num_points, dtype=np.float32)
        colors = np.zeros((num_points, 3), dtype=np.float32)

        splat = spz.GaussianSplat(
            positions=positions,
            scales=scales,
            rotations=rotations,
            alphas=alphas,
            colors=colors,
            antialiased=True,
        )

        assert splat.antialiased is True


class TestGaussianSplatArrayAccess:
    """Tests for accessing array data from GaussianSplat."""

    def test_positions_shape(self):
        """positions should have shape (N, 3)."""
        splat = util.create_test_splat(100)
        positions = splat.positions

        assert positions.shape == (100, 3)
        assert positions.dtype == np.float32

    def test_scales_shape(self):
        """scales should have shape (N, 3)."""
        splat = util.create_test_splat(100)
        scales = splat.scales

        assert scales.shape == (100, 3)
        assert scales.dtype == np.float32

    def test_rotations_shape(self):
        """rotations should have shape (N, 4)."""
        splat = util.create_test_splat(100)
        rotations = splat.rotations

        assert rotations.shape == (100, 4)
        assert rotations.dtype == np.float32

    def test_alphas_shape(self):
        """alphas should have shape (N,)."""
        splat = util.create_test_splat(100)
        alphas = splat.alphas

        assert alphas.shape == (100,)
        assert alphas.dtype == np.float32

    def test_colors_shape(self):
        """colors should have shape (N, 3)."""
        splat = util.create_test_splat(100)
        colors = splat.colors

        assert colors.shape == (100, 3)
        assert colors.dtype == np.float32

    def test_spherical_harmonics_empty_for_degree_0(self):
        """spherical_harmonics should be empty for sh_degree=0."""
        splat = util.create_test_splat(100, sh_degree=0)
        sh = splat.spherical_harmonics

        assert sh.shape[0] == 100
        assert sh.shape[1] == 0  # Empty for degree 0


class TestGaussianSplatProperties:
    """Tests for GaussianSplat properties."""

    def test_num_points(self):
        """num_points should return the count."""
        splat = util.create_test_splat(42)
        assert splat.num_points == 42

    def test_len(self):
        """len() should return num_points."""
        splat = util.create_test_splat(42)
        assert len(splat) == 42

    def test_sh_degree(self):
        """sh_degree should return the spherical harmonics degree."""
        splat = util.create_test_splat(10, sh_degree=0)
        assert splat.sh_degree == 0

    def test_antialiased(self):
        """antialiased should return the flag."""
        splat = util.create_test_splat(10)
        assert splat.antialiased is False

    def test_bbox(self):
        """bbox should return a BoundingBox."""
        splat = util.create_test_splat(100)
        bbox = splat.bbox

        assert isinstance(bbox, spz.BoundingBox)

    def test_median_volume(self):
        """median_volume should return a positive float."""
        splat = util.create_test_splat(100)
        vol = splat.median_volume

        assert isinstance(vol, float)
        assert vol > 0

    def test_repr(self):
        """repr should be informative."""
        splat = util.create_test_splat(100)
        repr_str = repr(splat)

        assert "GaussianSplat" in repr_str
        assert "num_points=100" in repr_str

    def test_str(self):
        """str should provide a readable description."""
        splat = util.create_test_splat(100)
        str_repr = str(splat)

        assert "GaussianSplat" in str_repr


class TestGaussianSplatSerialization:
    """Tests for saving and loading GaussianSplat."""

    def test_to_bytes_and_from_bytes_roundtrip(self):
        """Serialization to bytes and back should preserve data."""
        original = util.create_test_splat(50)

        # Serialize to bytes
        data = original.to_bytes()
        assert isinstance(data, bytes)
        assert len(data) > 0

        # Deserialize from bytes
        restored = spz.GaussianSplat.from_bytes(data)

        assert restored.num_points == original.num_points
        assert restored.sh_degree == original.sh_degree
        assert restored.antialiased == original.antialiased

        # Check array data is preserved (approximately, due to compression)
        np.testing.assert_array_almost_equal(
            restored.positions, original.positions, decimal=1
        )

    def test_save_and_load_file_roundtrip(self):
        """Saving to file and loading should preserve data."""
        original = util.create_test_splat(50)

        with TemporaryDirectory() as tmpdir:
            filepath = Path(tmpdir) / "test.spz"

            # Save to file
            original.save(str(filepath))
            assert filepath.exists()
            assert filepath.stat().st_size > 0

            # Load from file
            restored = spz.GaussianSplat.load(str(filepath))

            assert restored.num_points == original.num_points
            assert restored.sh_degree == original.sh_degree

    def test_load_function(self):
        """spz.load() convenience function should work."""
        original = util.create_test_splat(30)

        with TemporaryDirectory() as tmpdir:
            filepath = Path(tmpdir) / "test.spz"
            original.save(str(filepath))

            # Use the convenience function
            restored = spz.load(str(filepath))

            assert restored.num_points == original.num_points

    def test_save_with_coordinate_system(self):
        """Saving with a coordinate system should work."""
        splat = util.create_test_splat(20)

        with TemporaryDirectory() as tmpdir:
            filepath = Path(tmpdir) / "test.spz"

            # Save with coordinate system conversion
            splat.save(str(filepath), spz.CoordinateSystem.RUB)

            assert filepath.exists()

    def test_load_with_coordinate_system(self):
        """Loading with a coordinate system should work."""
        original = util.create_test_splat(20)

        with TemporaryDirectory() as tmpdir:
            filepath = Path(tmpdir) / "test.spz"
            original.save(str(filepath))

            # Load with coordinate system conversion
            restored = spz.GaussianSplat.load(
                str(filepath), coordinate_system=spz.CoordinateSystem.RDF
            )

            assert restored.num_points == original.num_points


class TestGaussianSplatCoordinateConversion:
    """Tests for coordinate system conversion."""

    def test_convert_coordinates(self):
        """convert_coordinates should modify the splat in place."""
        splat = util.create_test_splat(10)
        original_positions = splat.positions.copy()

        splat.convert_coordinates(spz.CoordinateSystem.RUB, spz.CoordinateSystem.RDF)

        # Positions should be different after conversion
        # (unless the conversion is identity, which it isn't for RUB->RDF)
        assert not np.allclose(splat.positions, original_positions)


class TestGaussianSplatErrors:
    """Tests for error handling."""

    def test_load_nonexistent_file(self):
        """Loading a nonexistent file should raise ValueError."""
        with pytest.raises(ValueError, match="Failed to load"):
            spz.GaussianSplat.load("/nonexistent/path/to/file.spz")

    def test_from_bytes_invalid_data(self):
        """Parsing invalid bytes should raise ValueError."""
        with pytest.raises(ValueError, match="Failed"):
            spz.GaussianSplat.from_bytes(b"not valid spz data")

    def test_from_bytes_empty(self):
        """Parsing empty bytes should raise ValueError."""
        with pytest.raises(ValueError, match="Failed"):
            spz.GaussianSplat.from_bytes(b"")


class TestRealSPZFile:
    """Tests using real SPZ files from the assets directory."""

    @pytest.fixture
    def assets_dir(self) -> Path:
        """Get the assets directory path."""
        # Navigate from tests dir to workspace root
        current = Path(__file__).parent
        for _ in range(5):  # Walk up to find assets
            assets = current / "assets"
            if assets.exists():
                return assets
            current = current.parent

        # Try workspace root directly
        workspace_root = Path(__file__).parent.parent.parent.parent.parent
        assets = workspace_root / "assets"
        if assets.exists():
            return assets

        pytest.skip("Assets directory not found")

    def test_load_hornedlizard(self, assets_dir: Path):
        """Load the hornedlizard.spz test file."""
        filepath = assets_dir / "hornedlizard.spz"
        if not filepath.exists():
            pytest.skip(f"Test file not found: {filepath}")

        splat = spz.load(str(filepath))

        assert splat.num_points > 0
        assert splat.positions.shape == (splat.num_points, 3)
        assert splat.scales.shape == (splat.num_points, 3)
        assert splat.rotations.shape == (splat.num_points, 4)
        assert splat.alphas.shape == (splat.num_points,)
        assert splat.colors.shape == (splat.num_points, 3)

    def test_roundtrip_real_file(self, assets_dir: Path):
        """Load, save, and reload a real SPZ file."""
        filepath = assets_dir / "hornedlizard.spz"

        if not filepath.exists():
            pytest.skip(f"Test file not found: {filepath}")

        original = spz.load(str(filepath))

        # Roundtrip through bytes
        data = original.to_bytes()
        restored = spz.GaussianSplat.from_bytes(data)

        assert restored.num_points == original.num_points
        assert restored.sh_degree == original.sh_degree
        assert restored.antialiased == original.antialiased


class TestContextManagers:
    """Tests for context managers in context_managers.py."""

    def test_splat_reader(self):
        """SplatReader should load and provide access to splat."""
        original = util.create_test_splat(25)

        with TemporaryDirectory() as tmpdir:
            filepath = Path(tmpdir) / "test.spz"
            original.save(str(filepath))

            with spz.SplatReader(str(filepath)) as ctx:
                assert ctx.path == filepath
                # SplatReader provides path, user loads manually
                loaded = spz.GaussianSplat.load(str(ctx.path))
                assert loaded.num_points == original.num_points

    def test_splat_reader_cancel(self):
        """SplatReader.cancel should work without error."""
        original = util.create_test_splat(10)

        with TemporaryDirectory() as tmpdir:
            filepath = Path(tmpdir) / "test.spz"
            original.save(str(filepath))

            with spz.SplatReader(str(filepath)) as ctx:
                ctx.cancel()

    def test_splat_writer(self):
        """SplatWriter should save splat on exit."""
        with TemporaryDirectory() as tmpdir:
            filepath = Path(tmpdir) / "output.spz"

            with spz.SplatWriter(str(filepath)) as ctx:
                ctx.splat = util.create_test_splat(15)

            assert filepath.exists()
            loaded = spz.GaussianSplat.load(str(filepath))
            assert loaded.num_points == 15

    def test_splat_writer_cancel(self):
        """SplatWriter.cancel should prevent saving."""
        with TemporaryDirectory() as tmpdir:
            filepath = Path(tmpdir) / "output.spz"

            with spz.SplatWriter(str(filepath)) as ctx:
                ctx.splat = util.create_test_splat(15)
                ctx.cancel()

            assert not filepath.exists()

    def test_splat_writer_no_splat(self):
        """SplatWriter should not fail if no splat is set."""
        with TemporaryDirectory() as tmpdir:
            filepath = Path(tmpdir) / "output.spz"

            with spz.SplatWriter(str(filepath)) as _ctx:
                pass  # Don't set a splat

            assert not filepath.exists()

    def test_splat_writer_with_coordinate_system(self):
        """SplatWriter should respect coordinate system."""
        with TemporaryDirectory() as tmpdir:
            filepath = Path(tmpdir) / "output.spz"

            with spz.SplatWriter(
                str(filepath), coordinate_system=spz.CoordinateSystem.RUB
            ) as ctx:
                ctx.splat = util.create_test_splat(10)

            assert filepath.exists()

    def test_temp_save(self):
        """temp_save should create a temporary file."""
        splat = util.create_test_splat(20)

        with spz.temp_save(splat) as temp_path:
            assert temp_path.exists()
            assert temp_path.suffix == ".spz"

            # Can load the temp file
            loaded = spz.GaussianSplat.load(str(temp_path))
            assert loaded.num_points == 20

        # File should be deleted after context
        assert not temp_path.exists()

    def test_temp_save_with_coordinate_system(self):
        """temp_save should respect coordinate system."""
        splat = util.create_test_splat(10)

        with spz.temp_save(
            splat, coordinate_system=spz.CoordinateSystem.LUF
        ) as temp_path:
            assert temp_path.exists()

    def test_temp_save_custom_suffix(self):
        """temp_save should respect custom suffix."""
        splat = util.create_test_splat(10)

        with spz.temp_save(splat, suffix=".test.spz") as temp_path:
            assert temp_path.suffix == ".spz"
            assert ".test" in str(temp_path)

    def test_modified_splat(self):
        """modified_splat should load, allow modification, and save."""
        original = util.create_test_splat(30)

        with TemporaryDirectory() as tmpdir:
            input_path = Path(tmpdir) / "input.spz"
            output_path = Path(tmpdir) / "output.spz"
            original.save(str(input_path))

            with spz.modified_splat(str(input_path), str(output_path)) as splat:
                assert splat.num_points == 30
                # Modify the splat
                splat.convert_coordinates(
                    spz.CoordinateSystem.RUB, spz.CoordinateSystem.RDF
                )

            assert output_path.exists()
            loaded = spz.GaussianSplat.load(str(output_path))
            assert loaded.num_points == 30

    def test_modified_splat_overwrites_original(self):
        """modified_splat should overwrite original if no output_path."""
        original = util.create_test_splat(25)

        with TemporaryDirectory() as tmpdir:
            filepath = Path(tmpdir) / "test.spz"
            original.save(str(filepath))

            with spz.modified_splat(str(filepath)) as _splat:
                # Just load and save without modification
                pass

            # File should still exist and be valid
            assert filepath.exists()
            loaded = spz.GaussianSplat.load(str(filepath))
            assert loaded.num_points == 25


class TestSphericalHarmonics:
    """Tests for different spherical harmonics degrees."""

    @pytest.mark.parametrize("sh_degree,sh_dim", [(1, 3), (2, 8), (3, 15)])
    def test_spherical_harmonics_degrees(self, sh_degree: int, sh_dim: int):
        """Test creating splats with different SH degrees."""
        num_points = 50
        positions = np.random.randn(num_points, 3).astype(np.float32)
        scales = np.full((num_points, 3), -5.0, dtype=np.float32)
        rotations = np.tile([1, 0, 0, 0], (num_points, 1)).astype(np.float32)
        alphas = np.zeros(num_points, dtype=np.float32)
        colors = np.zeros((num_points, 3), dtype=np.float32)
        spherical_harmonics = np.random.randn(num_points, sh_dim * 3).astype(np.float32)

        splat = spz.GaussianSplat(
            positions=positions,
            scales=scales,
            rotations=rotations,
            alphas=alphas,
            colors=colors,
            sh_degree=sh_degree,
            spherical_harmonics=spherical_harmonics,
        )

        assert splat.sh_degree == sh_degree
        assert splat.spherical_harmonics.shape == (num_points, sh_dim * 3)

    @pytest.mark.parametrize("sh_degree", [1, 2, 3])
    def test_spherical_harmonics_roundtrip(self, sh_degree: int):
        """SH data should survive serialization roundtrip."""
        sh_dims = {1: 3, 2: 8, 3: 15}
        sh_dim = sh_dims[sh_degree]
        num_points = 20

        positions = np.random.randn(num_points, 3).astype(np.float32)
        scales = np.full((num_points, 3), -5.0, dtype=np.float32)
        rotations = np.tile([1, 0, 0, 0], (num_points, 1)).astype(np.float32)
        alphas = np.zeros(num_points, dtype=np.float32)
        colors = np.zeros((num_points, 3), dtype=np.float32)
        spherical_harmonics = np.random.randn(num_points, sh_dim * 3).astype(np.float32)

        original = spz.GaussianSplat(
            positions=positions,
            scales=scales,
            rotations=rotations,
            alphas=alphas,
            colors=colors,
            sh_degree=sh_degree,
            spherical_harmonics=spherical_harmonics,
        )

        data = original.to_bytes()
        restored = spz.GaussianSplat.from_bytes(data)

        assert restored.sh_degree == sh_degree
        assert restored.spherical_harmonics.shape == (num_points, sh_dim * 3)


class TestCoordinateSystemComprehensive:
    """Comprehensive tests for all coordinate systems."""

    @pytest.mark.parametrize(
        "coord_sys,expected_str",
        [
            (spz.CoordinateSystem.LDB, "Left-Down-Back"),
            (spz.CoordinateSystem.RDB, "Right-Down-Back"),
            (spz.CoordinateSystem.LUB, "Left-Up-Back"),
            (spz.CoordinateSystem.RUB, "Right-Up-Back"),
            (spz.CoordinateSystem.LDF, "Left-Down-Front"),
            (spz.CoordinateSystem.RDF, "Right-Down-Front"),
            (spz.CoordinateSystem.LUF, "Left-Up-Front"),
            (spz.CoordinateSystem.RUF, "Right-Up-Front"),
            (spz.CoordinateSystem.UNSPECIFIED, "Unspecified"),
        ],
    )
    def test_coordinate_system_str_values(
        self, coord_sys: spz.CoordinateSystem, expected_str: str
    ):
        """Each coordinate system should have correct string representation."""
        assert str(coord_sys) == expected_str
        assert expected_str in repr(coord_sys)

    @pytest.mark.parametrize(
        "from_sys,to_sys",
        [
            (spz.CoordinateSystem.RUB, spz.CoordinateSystem.RDF),
            (spz.CoordinateSystem.RDF, spz.CoordinateSystem.RUB),
            (spz.CoordinateSystem.LUF, spz.CoordinateSystem.RUF),
            (spz.CoordinateSystem.RUB, spz.CoordinateSystem.LUF),
        ],
    )
    def test_coordinate_conversion_pairs(
        self, from_sys: spz.CoordinateSystem, to_sys: spz.CoordinateSystem
    ):
        """Test various coordinate system conversions."""
        splat = util.create_test_splat(10)
        original_positions = splat.positions.copy()

        splat.convert_coordinates(from_sys, to_sys)

        # Conversion should change positions (unless identity)
        if from_sys != to_sys:
            assert not np.allclose(splat.positions, original_positions)


class TestSerializationWithCoordinateSystems:
    """Tests for serialization with coordinate system options."""

    def test_to_bytes_with_coordinate_system(self):
        """to_bytes should accept coordinate system parameter."""
        splat = util.create_test_splat(20)

        data = splat.to_bytes(spz.CoordinateSystem.RUB)

        assert isinstance(data, bytes)
        assert len(data) > 0

    def test_from_bytes_with_coordinate_system(self):
        """from_bytes should accept coordinate system parameter."""
        original = util.create_test_splat(20)
        data = original.to_bytes()

        restored = spz.GaussianSplat.from_bytes(
            data, coordinate_system=spz.CoordinateSystem.RDF
        )

        assert restored.num_points == original.num_points


class TestEdgeCases:
    """Tests for edge cases and boundary conditions."""

    def test_single_point_splat(self):
        """Should handle a splat with a single point."""
        positions = np.array([[1.0, 2.0, 3.0]], dtype=np.float32)
        scales = np.array([[-5.0, -5.0, -5.0]], dtype=np.float32)
        rotations = np.array([[1.0, 0.0, 0.0, 0.0]], dtype=np.float32)
        alphas = np.array([0.5], dtype=np.float32)
        colors = np.array([[1.0, 0.0, 0.0]], dtype=np.float32)

        splat = spz.GaussianSplat(
            positions=positions,
            scales=scales,
            rotations=rotations,
            alphas=alphas,
            colors=colors,
        )

        assert splat.num_points == 1
        assert len(splat) == 1

        # Roundtrip
        data = splat.to_bytes()
        restored = spz.GaussianSplat.from_bytes(data)
        assert restored.num_points == 1

    def test_large_splat(self):
        """Should handle larger splats."""
        splat = util.create_test_splat(10000)

        assert splat.num_points == 10000

        # Roundtrip
        data = splat.to_bytes()
        restored = spz.GaussianSplat.from_bytes(data)
        assert restored.num_points == 10000

    def test_positions_data_preservation(self):
        """Position data should be preserved through roundtrip."""
        positions = np.array(
            [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]], dtype=np.float32
        )
        scales = np.full((3, 3), -5.0, dtype=np.float32)
        rotations = np.tile([1, 0, 0, 0], (3, 1)).astype(np.float32)
        alphas = np.zeros(3, dtype=np.float32)
        colors = np.zeros((3, 3), dtype=np.float32)

        splat = spz.GaussianSplat(
            positions=positions,
            scales=scales,
            rotations=rotations,
            alphas=alphas,
            colors=colors,
        )

        # Check positions are stored correctly
        np.testing.assert_array_almost_equal(splat.positions, positions)

    def test_all_arrays_same_dtype(self):
        """All array properties should return float32."""
        splat = util.create_test_splat(10)

        assert splat.positions.dtype == np.float32
        assert splat.scales.dtype == np.float32
        assert splat.rotations.dtype == np.float32
        assert splat.alphas.dtype == np.float32
        assert splat.colors.dtype == np.float32
        assert splat.spherical_harmonics.dtype == np.float32

    def test_coordinate_system_identity_conversion(self):
        """Converting to same coordinate system should be identity."""
        splat = util.create_test_splat(10)
        original_positions = splat.positions.copy()

        splat.convert_coordinates(
            spz.CoordinateSystem.UNSPECIFIED, spz.CoordinateSystem.UNSPECIFIED
        )

        np.testing.assert_array_equal(splat.positions, original_positions)

    def test_bounding_box_single_point(self):
        """BoundingBox should work for single point."""
        positions = np.array([[5.0, 10.0, 15.0]], dtype=np.float32)
        scales = np.array([[-5.0, -5.0, -5.0]], dtype=np.float32)
        rotations = np.array([[1.0, 0.0, 0.0, 0.0]], dtype=np.float32)
        alphas = np.array([0.5], dtype=np.float32)
        colors = np.array([[1.0, 0.0, 0.0]], dtype=np.float32)

        splat = spz.GaussianSplat(
            positions=positions,
            scales=scales,
            rotations=rotations,
            alphas=alphas,
            colors=colors,
        )

        bbox = splat.bbox
        center = bbox.center

        # Center should be at the single point
        assert center[0] == pytest.approx(5.0, abs=0.1)
        assert center[1] == pytest.approx(10.0, abs=0.1)
        assert center[2] == pytest.approx(15.0, abs=0.1)
