import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

/// `tabbed_view` gives a tab that does not fit a minimal, off-screen slot (about 13 to 17 px wide) and its label row
/// reports an overflow in debug. Nothing is drawn there; every other error is still reported.
void ignoreSqueezedTabChips() {
  final report = FlutterError.onError;
  FlutterError.onError = (details) {
    final text = details.toString(minLevel: DiagnosticLevel.debug);
    final chip = text.contains('RenderFlex overflowed') &&
        RegExp(r'constraints: BoxConstraints\(w=(\d+\.\d+), h=\d+\.\d+\)').allMatches(text).any((m) => double.parse(m.group(1)!) < 40);
    if (!chip) report?.call(details);
  };
}
