# Light in form.

1600×1000 material study, rendered through Motolii's real Engine at frame 0.

- `light-in-form.png`: unretouched renderer output.
- `light-in-form.rrd`: editable Motolii document. Asset references are absolute paths in this workspace.
- `create.py`: regenerates the document, smooth OBJ geometry, RGBE studio lighting and 2D assets. Requires Pillow and a built macOS native library.
- The torus uses the Glass surface as chrome; the large sphere uses transmission 0.94 / IOR 1.45. The rounded 2D sheet uses the same surface with transmission 0.9.

```sh
python3 docs/reviews/assets/2026-09-09-glass-gallery/create.py
cargo run -p motolii-render --example surface_frame -- docs/reviews/assets/2026-09-09-glass-gallery/light-in-form.rrd docs/reviews/assets/2026-09-09-glass-gallery/light-in-form.png
```

This is authored scene content. No renderer or application code was changed for the image.

The main PNG now uses 4x MSAA with per-sample mesh shading. `light-in-form-no-aa.png` preserves the original output; `-msaa` and `-aa` preserve the MSAA-only and centroid comparisons; `-sampled` matches the final image. See [the rendering change](../../2026-09-09-surface-antialiasing.md).
