from ..py import spz

# Load from file
splat = spz.load("scene.spz")
# or
splat = spz.GaussianSplat.load("scene.spz", coordinate_system=spz.CoordinateSystem.RUB)

# Access properties
print(f"{splat.num_points:,} points")
print(f"center: {splat.bbox.center}")
print(f"size: {splat.bbox.size}")

# Access data (flat arrays)
positions = splat.positions  # [x1, y1, z1, x2, y2, z2, ...]
scales = splat.scales
rotations = splat.rotations
alphas = splat.alphas
colors = splat.colors
sh = splat.spherical_harmonics

# Serialize
data = splat.to_bytes()
splat2 = spz.GaussianSplat.from_bytes(data)

# Save to file
splat.save("output.spz")

# Create from data
new_splat = spz.GaussianSplat(
    positions=[0.0, 0.0, 0.0, 1.0, 2.0, 3.0],  # flat array
    scales=[-5.0] * 6,
    rotations=[1.0, 0.0, 0.0, 0.0] * 2,
    alphas=[0.5, 0.8],
    colors=[255.0, 0.0, 0.0, 0.0, 255.0, 0.0],
)
