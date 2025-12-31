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

# Access data (flat arrays, list[float])
positions = splat.positions  # [x1, y1, z1, x2, y2, z2, ...]
scales = splat.scales
rotations = splat.rotations
alphas = splat.alphas
colors = splat.colors
sh = splat.spherical_harmonics

# Serialize
data = splat.to_bytes()  # -> bytes
splat2 = spz.GaussianSplat.from_bytes(data)  # -> GaussianSplat

# Create from data
new_splat = spz.GaussianSplat(
    positions=[0.0, 0.0, 0.0, 1.0, 2.0, 3.0],  # flat array
    scales=[-5.0] * 6,
    rotations=[1.0, 0.0, 0.0, 0.0] * 2,
    alphas=[0.5, 0.8],
    colors=[255.0, 0.0, 0.0, 0.0, 255.0, 0.0],
)  # -> GaussianSplat

# Save to file
new_splat.save("output.spz")

with spz.SplatWriter("output2.spz") as writer:
    writer.splat = splat2

with spz.modified_splat("scene.spz", "scene_rotated.spz") as splat:
    splat.rotate_180_deg_about_x()
