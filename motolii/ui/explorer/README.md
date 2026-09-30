# Motolii component explorer

Production panels (`motolii_ui/lib`) over real documents in the real host, many states at once, hot reload.
Built on [Widgetbook](https://pub.dev/packages/widgetbook). Production does not depend on it; deleting `explorer/`
changes nothing.

    scripts/motolii-ui.sh native          # once, from the repository root
    FLUTTER_BIN=… explorer/run.sh         # Widgetbook: pick a story, change size and UI scale, r / R to reload

    MOTOLII_EXPLORER_SHOTS=/tmp/shots MOTOLII_EXPLORER_ONLY='dense|narrow' explorer/run.sh
                                          # write each story (or the named ones) as a PNG; R shoots them again

- `lib/host.dart`, `lib/binding.dart`: the production session's `motolii/probe` channel answered by the real native
  host through `dart:ffi` (the same `motolii_probe_*` entry points the macOS runner uses). No behaviour is faked.
- `fixtures/`: the documents and host scripts stories open. A story may set inputs (choose, seek, open lanes) through
  the product's own operations; it never decides what anything means.
- Stories are for looking and touching. They are not tests: nothing is compared, and no size or position is frozen.
  When a fixture falls behind the product, update the fixture.
