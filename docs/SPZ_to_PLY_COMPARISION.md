# Comparision to .PLY

| Element | SPZ Format | PLY Format | Size Reduction (%) |
| ---- | ---- | ---- | ---- |
| Positions |	24-bit fixed point integer with adjustable fractional bits	| 32-bit or 64-bit floating-point	| 25%–62.5% |
| Rotation |	3 components of a quaternion stored as 8-bit signed integers	| 4 components of quaternion as 32-bit floats	| 81.25% |
| Colors (RGB) | 8-bit unsigned integers per channel |	Typically 8-bit or 32-bit floats per channel	| 0%–75% |
| Scales |	8-bit log-encoded integer	| Typically 32-bit or 64-bit floating-point	| 75%–87.5% |
| Alphas (Transparency) |	8-bit unsigned integer | Typically 32-bit float |	75% |
| Spherical | Harmonics	8-bit signed integers for coefficients, with 4-5 bits of precision	| Varies, but usually stored with higher precision (e.g., 32-bit floats)	| 75%–87.5% |

Source: <https://scaniverse.com/news/spz-gaussian-splat-open-source-file-format>
