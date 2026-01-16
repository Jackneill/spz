# SPDX-License-Identifier: Apache-2.0 OR MIT

import numpy as np
import spz


def create_test_splat(num_points: int, sh_degree: int = 0) -> spz.GaussianSplat:
    """Create a test GaussianSplat with dummy data."""
    positions = np.random.randn(num_points, 3).astype(np.float32)
    scales = np.full((num_points, 3), -5.0, dtype=np.float32)
    rotations = np.tile([1, 0, 0, 0], (num_points, 1)).astype(np.float32)
    alphas = np.zeros(num_points, dtype=np.float32)
    colors = np.zeros((num_points, 3), dtype=np.float32)

    return spz.GaussianSplat(
        positions=positions,
        scales=scales,
        rotations=rotations,
        alphas=alphas,
        colors=colors,
        sh_degree=sh_degree,
    )
