import 'dart:collection';
import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';

import '../session/editor_session.dart';

/// Small specimens produced by the document renderer, with session-local reuse.
class NativeVisualSample extends StatelessWidget {
  const NativeVisualSample({
    super.key,
    required this.controller,
    required this.request,
    this.fit = BoxFit.contain,
  });
  final EditorSession controller;
  final Map<String, dynamic> request;
  final BoxFit fit;
  static final _sessions = Expando<LinkedHashMap<String, Future<Uint8List?>>>();
  Future<Uint8List?> _image() {
    final cache = _sessions[controller] ??= LinkedHashMap();
    final key = jsonEncode(request);
    final hit = cache.remove(key);
    if (hit != null) {
      cache[key] = hit;
      return hit;
    }
    final future = controller
        .native('request', {
          'command': jsonEncode({'op': 'visualSample', ...request}),
        })
        .then((value) {
          final data = EditorSession.map(value)['image'];
          return data is String ? base64Decode(data) : null;
        });
    cache[key] = future;
    while (cache.length > 128) {
      cache.remove(cache.keys.first);
    }
    return future;
  }

  @override
  Widget build(BuildContext context) =>
      controller.state['visualSamples'] != true
      ? const SizedBox.shrink()
      : FutureBuilder<Uint8List?>(
          key: ValueKey(jsonEncode(request)),
          future: _image(),
          builder: (context, snapshot) => snapshot.data == null
              ? const SizedBox.shrink()
              : Image.memory(
                  snapshot.data!,
                  fit: fit,
                  gaplessPlayback: false,
                  excludeFromSemantics: true,
                ),
        );
}
