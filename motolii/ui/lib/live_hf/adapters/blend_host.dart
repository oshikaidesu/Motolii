import 'dart:async';
import 'dart:convert';

import 'package:flutter/widgets.dart';

import '../../hf/desk/blend.dart' show BlendHost;
import '../../session/editor_session.dart';
import '../../session/read_model.dart';

/// What the Blend desk does with the document, apart from how it looks: the layers a mode applies to, the mode they
/// wear, the specimens the runtime draws for it, the hover that previews on Stage without touching history, and the
/// one click that is one undo step. The operations are the ones the Classic desk sends (`previewBlend`,
/// `cancelPreview`, `applyBlend`).
class BlendController extends ChangeNotifier implements BlendHost {
  BlendController(this.c) {
    _selection = _mark;
    _slice = c.slice('blend', const ['path', 'capabilities'], derived: () => blendReading(c))..addListener(_sync);
    _read();
  }

  final EditorSession c;
  late final DocumentSlice _slice;
  String _selection = '';
  Map<String, dynamic> _samples = const {};
  String? _previewing, _wanted, hover;
  Timer? _hoverTimer;
  Future<void>? _flight;
  bool applying = false, _gone = false;

  /// The mode every target wears (null when they differ or there is none), the layers' names, and whether a click may act.
  @override
  String? current;
  @override
  List<String> names = const [];
  bool get idle => names.isEmpty;
  @override
  bool get live => names.isNotEmpty && !applying;

  List<Map<String, dynamic>> get targets => blendTargets(c);
  String get _mark => jsonEncode([c.state['path'], c.selectedIds]);

  /// The specimen colours the runtime drew for one mode on the layer the hover previews on.
  @override
  List<Color> beds(String mode) => [
        for (final rgb in _samples[mode] as List? ?? const [])
          Color.fromARGB(255, ((rgb[0] as num).clamp(0, 1) * 255).round(), ((rgb[1] as num).clamp(0, 1) * 255).round(), ((rgb[2] as num).clamp(0, 1) * 255).round()),
      ];

  void _read() {
    final t = targets;
    final fresh = panelMap((t.isEmpty ? c.activeLayer : t.last)?['blendPreviews']);
    if (fresh.isNotEmpty) _samples = fresh;
    final values = t.map((l) => '${l['blendMode'] ?? 'Normal'}').toSet();
    current = values.length == 1 ? values.single : null;
    names = [for (final l in t) '${l['name'] ?? 'Layer ${l['id']}'}'];
  }

  void _sync() {
    if (_gone) return;
    if (_selection != _mark) {
      _selection = _mark;
      _hoverTimer?.cancel();
      hover = null;
      _want(null);
    }
    _read();
    notifyListeners();
  }

  /// Latest wish wins: sweeping across the grid must not queue one round trip per tile.
  void _want(String? mode) {
    _wanted = mode;
    _flight ??= _pump();
  }

  Future<void> _pump() async {
    await Future<void>.value();
    try {
      while (!_gone && _wanted != _previewing) {
        if (targets.isEmpty) _wanted = null;
        final mode = _wanted;
        if (mode == null) {
          _previewing = null;
          await c.command('cancelPreview');
        } else {
          _previewing = mode;
          await c.command('previewBlend', {'mode': mode});
        }
      }
    } finally {
      _flight = null;
    }
  }

  /// Aim at a mode, or at nothing; both ends wait one beat so a sweep sends one preview.
  @override
  void aim(String? mode) {
    if (mode != null && (applying || targets.isEmpty || !(c.state['capabilities'] as List? ?? const []).contains('previewBlend'))) return;
    hover = mode;
    notifyListeners();
    _hoverTimer?.cancel();
    _hoverTimer = Timer(const Duration(milliseconds: 90), () {
      if (!_gone && hover == mode) _want(mode);
    });
  }

  /// Give the document back now: the pointer is gone, not merely moving.
  @override
  void leave() {
    _hoverTimer?.cancel();
    if (_gone) return;
    hover = null;
    notifyListeners();
    _want(null);
  }

  bool get previewing => _previewing != null;

  /// One click, one undo step: `applyBlend` folds the running preview away before it applies.
  @override
  Future<void> apply(String mode) async {
    _hoverTimer?.cancel();
    if (applying || !(c.state['capabilities'] as List? ?? const []).contains('applyBlend')) return;
    final mark = _mark;
    final ids = [for (final l in targets) if ('${l['blendMode'] ?? 'Normal'}' != mode) l['id']];
    applying = true;
    hover = null;
    notifyListeners();
    try {
      if (ids.isEmpty) {
        _want(null);
        await _flight;
      } else {
        _wanted = null;
        _previewing = null;
        await _flight;
        if (!_gone && mark == _mark) await c.command('applyBlend', {'mode': mode});
      }
    } finally {
      if (!_gone) {
        applying = false;
        notifyListeners();
      }
    }
  }

  @override
  void dispose() {
    _gone = true;
    _hoverTimer?.cancel();
    _slice.removeListener(_sync);
    if (_previewing != null) c.command('cancelPreview');
    super.dispose();
  }
}
