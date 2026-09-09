import 'dart:async';
import 'dart:collection';
import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';

import '../session/editor_session.dart';

/// Samples, render and status all queue behind one worker on the native side,
/// so the shelf asks only for the specimens still on screen, waits for the
/// scroll to settle, and lets go of the worker between small bursts.
class _SampleShelf {
  _SampleShelf(this.controller);
  final EditorSession controller;
  static final _shelves = Expando<_SampleShelf>();
  static _SampleShelf of(EditorSession controller) =>
      _shelves[controller] ??= _SampleShelf(controller);

  /// More than any font shelf holds, so one format edit cannot evict the rest.
  static const _kept = 2048;
  static const _burst = 3;
  static const _settled = Duration(milliseconds: 120);
  static const _breath = Duration(milliseconds: 16);

  final _drawn = LinkedHashMap<String, Uint8List?>();
  final _wanted = <String, Map<String, dynamic>>{};
  final _watchers = <String, Set<VoidCallback>>{};
  Timer? _settle;
  bool _drawing = false;

  bool drawn(String key) => _drawn.containsKey(key);
  Uint8List? image(String key) {
    if (!_drawn.containsKey(key)) return null;
    final image = _drawn.remove(key);
    return _drawn[key] = image;
  }

  void want(String key, Map<String, dynamic> request, VoidCallback arrived) {
    _watchers.putIfAbsent(key, () => {}).add(arrived);
    _wanted[key] = request;
    _settle?.cancel();
    _settle = Timer(_settled, _draw);
  }

  void drop(String key, VoidCallback arrived) {
    final watching = _watchers[key];
    if (watching == null) return;
    watching.remove(arrived);
    if (watching.isEmpty) {
      _watchers.remove(key);
      _wanted.remove(key);
    }
  }

  Future<void> _draw() async {
    if (_drawing) return;
    _drawing = true;
    try {
      var served = 0;
      while (_wanted.isNotEmpty) {
        // Newest wish first: what the reader just scrolled to.
        final key = _wanted.keys.last;
        final request = _wanted.remove(key)!;
        if (!_watchers.containsKey(key)) continue;
        Uint8List? drawn;
        try {
          final reply = await controller.native('request', {
            'command': jsonEncode({'op': 'visualSample', ...request}),
          });
          final data = EditorSession.map(reply)['image'];
          drawn = data is String ? base64Decode(data) : null;
        } catch (_) {
          drawn = null;
        }
        _drawn[key] = drawn;
        while (_drawn.length > _kept) {
          _drawn.remove(_drawn.keys.first);
        }
        for (final arrived in {...?_watchers[key]}) {
          arrived();
        }
        if (++served % _burst == 0) await Future<void>.delayed(_breath);
      }
    } finally {
      _drawing = false;
    }
  }
}

/// A small specimen the document renderer draws. The name stands alone until
/// the picture arrives.
class NativeVisualSample extends StatefulWidget {
  const NativeVisualSample({
    super.key,
    required this.controller,
    required this.request,
    this.fit = BoxFit.contain,
  });
  final EditorSession controller;
  final Map<String, dynamic> request;
  final BoxFit fit;
  @override
  State<NativeVisualSample> createState() => _NativeVisualSampleState();
}

class _NativeVisualSampleState extends State<NativeVisualSample> {
  late _SampleShelf _shelf = _SampleShelf.of(widget.controller);
  String _key = '';
  Uint8List? _image;
  bool _watching = false;

  @override
  void initState() {
    super.initState();
    _ask();
  }

  @override
  void didUpdateWidget(NativeVisualSample old) {
    super.didUpdateWidget(old);
    final key = jsonEncode(widget.request);
    if (key == _key && (_watching || _image != null)) return;
    _let();
    _ask();
  }

  @override
  void dispose() {
    _let();
    super.dispose();
  }

  void _ask() {
    _shelf = _SampleShelf.of(widget.controller);
    _key = jsonEncode(widget.request);
    _image = null;
    if (widget.controller.state['visualSamples'] != true) return;
    if (_shelf.drawn(_key)) {
      _image = _shelf.image(_key);
      return;
    }
    _watching = true;
    _shelf.want(_key, widget.request, _arrived);
  }

  void _let() {
    if (!_watching) return;
    _watching = false;
    _shelf.drop(_key, _arrived);
  }

  void _arrived() {
    if (!mounted) return;
    setState(() => _image = _shelf.image(_key));
  }

  @override
  Widget build(BuildContext context) => _image == null
      ? const SizedBox.shrink()
      : Image.memory(
          _image!,
          fit: widget.fit,
          gaplessPlayback: false,
          excludeFromSemantics: true,
        );
}
