# How Motolii is built

Motolii has three parts: `app/document` stores a project's meaning, `app/renderer` draws and exports frames, and `app/ui` is the Flutter app users see.

Run it with `tools/motolii-ui.sh dev`. Build its Rust bridge with `tools/motolii-ui.sh native`, then check the repo with `tools/motolii-ui.sh check`.
