import 'package:flutter/widgets.dart';

import 'legacy/app/editor_app.dart';
import 'app/main.dart' as live;

/// The window opens the product UI, lib/app (docs/stage5/product-direction.md). The earlier shells stay reachable
/// as capability migration sources until their capabilities are in the product UI: MOTOLII_SHELL=classic or new.
const shell = String.fromEnvironment('MOTOLII_SHELL', defaultValue: 'live');

void main() => shell == 'live' ? live.main() : runApp(const EditorApp());
