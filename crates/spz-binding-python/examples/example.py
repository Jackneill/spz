import numpy as np

from ..pypkg import spz

# Load from file
splat = spz.load("scene.spz")  # -> GaussianSplat
# or
splat = spz.GaussianSplat.load(
    "scene.spz", coordinate_system=spz.CoordinateSystem.RUB
)  # -> GaussianSplat
# or
with spz.SplatReader("scene.spz") as ctx:
    splat2 = ctx.splat  # -> GaussianSplat

with spz.temp_save(splat) as tmp_path:
    import subprocess

    subprocess.run(["viewer", str(tmp_path)])

# Access properties
print(f"{splat.num_points:,} points")
print(f"center: {splat.bbox.center}")
print(f"size: {splat.bbox.size}")

# Access data as numpy arrays
positions = splat.positions  # shape: (num_points, 3)
scales = splat.scales  # shape: (num_points, 3)
rotations = splat.rotations  # shape: (num_points, 4)
alphas = splat.alphas  # shape: (num_points,)
colors = splat.colors  # shape: (num_points, 3)
sh = splat.spherical_harmonics  # shape: (num_points, sh_dim * 3)

# Serialize
data = splat.to_bytes()  # -> bytes
splat2 = spz.GaussianSplat.from_bytes(data)  # -> GaussianSplat

# Create from numpy arrays
new_splat = spz.GaussianSplat(
    positions=np.zeros((2, 3), dtype=np.float32),
    scales=np.full((2, 3), -5.0, dtype=np.float32),
    rotations=np.tile([1.0, 0.0, 0.0, 0.0], (2, 1)).astype(np.float32),
    alphas=np.array([0.5, 0.8], dtype=np.float32),
    colors=np.array([[255.0, 0.0, 0.0], [0.0, 255.0, 0.0]], dtype=np.float32),
)  # -> GaussianSplat

# Save to file
new_splat.save("output.spz")

with spz.SplatWriter("output2.spz") as writer:
    writer.splat = splat2

# Coordinate conversion
with spz.modified_splat("scene.spz", "scene_converted.spz") as splat:
    splat.convert_coordinates(spz.CoordinateSystem.RUB, spz.CoordinateSystem.RDF)
