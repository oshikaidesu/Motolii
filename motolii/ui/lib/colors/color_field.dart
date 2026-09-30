import 'package:flutter/widgets.dart';

/// Three or six hex digits, with or without `#`, as an opaque colour.
Color? parseHex(String text) {
  var raw = text.trim().replaceFirst('#', '');
  if (raw.length == 3) raw = raw.split('').map((s) => '$s$s').join();
  final n = raw.length == 6 ? int.tryParse(raw, radix: 16) : null;
  return n == null ? null : Color(0xff000000 | n);
}

