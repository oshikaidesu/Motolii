# anime.js v4.5.0 (MIT, Copyright 2025 Julian Garnier)

`anime.umd.min.js` and `LICENSE.md` exactly as released: https://github.com/juliangarnier/anime/tree/v4.5.0/dist/bundles

Only its easings are used: `anime.eases` (Penner's in / out / inOut / outIn of Power, Quad … Quint, Sine, Circ, Expo,
Bounce, Back(overshoot), Elastic(amplitude, period)), `linear()`, `steps()`, `irregular()`, `cubicBezier()`, `spring()`.
Run in QuickJS-ng by `anime_ease` (filament/script.cpp): a way a key's stretch goes may say `anime('outElastic(1, .3)', u)`.
The bundle looks for a host clock (setImmediate) as it loads; script.cpp gives it a do-nothing one.
