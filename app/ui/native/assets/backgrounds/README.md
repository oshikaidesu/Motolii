# Bundled backgrounds

Six 1k HDRIs from [Poly Haven](https://polyhaven.com), licensed CC0 (no attribution, redistribution allowed). Written to `~/.local/share/motolii/builtins/background-<id>-v1.hdr` on first use and offered in the Media tab under HDR. Placing one adds an environment layer: it is drawn as the sky and lights the scene.

| id | file | source |
|---|---|---|
| partly-cloudy-sky | kloofendal_48d_partly_cloudy_puresky.hdr | https://polyhaven.com/a/kloofendal_48d_partly_cloudy_puresky |
| sunset-sky | the_sky_is_on_fire.hdr | https://polyhaven.com/a/the_sky_is_on_fire |
| night-sky | moonless_golf.hdr | https://polyhaven.com/a/moonless_golf |
| meadow | meadow_2.hdr | https://polyhaven.com/a/meadow_2 |
| city-night | shanghai_bund.hdr | https://polyhaven.com/a/shanghai_bund |
| photo-studio | brown_photostudio_02.hdr | https://polyhaven.com/a/brown_photostudio_02 |

To add one: drop the `.hdr` here, add a row to `backgrounds()` in `ui/native/src/editor/create.rs`.
