import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import '../bridge/native_bridge.dart';
import '../bridge/protocol.dart';

part 'session_values.dart';
part 'session_core.dart';
part 'session_native.dart';
part 'session_snapshot.dart';
part 'session_render.dart';
part 'session_commands.dart';
part 'session_files.dart';

/// 一つの窓が持つ編集の座。責任は 5 つ、それぞれが mixin で、共通の状態と
/// 継ぎ目は [SessionCore] にある。ここに残るのは返信を型付ける道具と、
/// 生まれ際・終わり際だけ。
class EditorSession extends SessionCore
    with
        SessionNative,
        SessionSnapshot,
        SessionRender,
        SessionCommands,
        SessionFiles {
  EditorSession({NativeBridge? bridge}) : super(bridge: bridge) {
    document.addListener(_spreadDocument);
    deskWork.addListener(_syncPreferences);
    _listenToHost();
  }

  static const channel = NativeBridge.channel;

  /// The shape a newborn key gets; Easy Ease until the Ease desk says otherwise.
  static const easyEase = {
    'kind': 'Bezier',
    'x1': 0.42,
    'y1': 0.0,
    'x2': 0.58,
    'y2': 1.0,
  };

  /// Take in what actually moved. Keys that arrive unchanged keep the object
  /// they had, so a reply that says nothing new notifies nobody.
  static void take(
    ValueNotifier<Map<String, dynamic>> held,
    Map<String, dynamic> next,
  ) {
    final was = held.value;
    Map<String, dynamic>? merged;
    for (final entry in next.entries) {
      if (was.containsKey(entry.key) && sameValue(was[entry.key], entry.value))
        continue;
      (merged ??= {...was})[entry.key] = entry.value;
    }
    if (merged != null) held.value = merged;
  }

  /// A reply, with every map and every list of maps typed, once. After this
  /// [map] and [maps] hand back the object they are given. "Typed" is the
  /// exact runtime type this makes: a narrower literal (a test's
  /// `List<Map<String, Object>>`) is copied, so a reader's `orElse` fits.
  static Object? typed(Object? v) {
    if (v is Map) {
      if (_typedMap(v)) return v;
      return <String, dynamic>{
        for (final e in v.entries) '${e.key}': typed(e.value),
      };
    }
    if (v is List) {
      if (v.isNotEmpty && v.every((e) => e is Map)) {
        if (v.runtimeType == _mapsType && v.every(_typedMap)) return v;
        return <Map<String, dynamic>>[
          for (final e in v) typed(e) as Map<String, dynamic>,
        ];
      }
      if (v.runtimeType == _listType && v.every(_typedLeaf)) return v;
      return <dynamic>[for (final e in v) typed(e)];
    }
    return v;
  }

  static final _mapType = <String, dynamic>{}.runtimeType;
  static final _mapsType = <Map<String, dynamic>>[].runtimeType;
  static final _listType = <dynamic>[].runtimeType;
  static bool _typedMap(Object? v) =>
      v is Map && v.runtimeType == _mapType && v.values.every(_typedLeaf);
  static bool _typedLeaf(Object? v) {
    if (v is Map) return _typedMap(v);
    if (v is List) {
      if (v.runtimeType == _mapsType) return v.every(_typedMap);
      return v.runtimeType == _listType && v.every(_typedLeaf);
    }
    return true;
  }

  /// Views, not copies: a typed map or list comes back as is. Callers that
  /// change what they get copy first (`.toList()`, `{...}`).
  static Map<String, dynamic> map(dynamic v) {
    if (v is Map<String, dynamic> && v.runtimeType == _mapType) return v;
    return v is Map ? Map<String, dynamic>.from(v) : {};
  }

  static List<Map<String, dynamic>> maps(dynamic v) {
    if (v is List<Map<String, dynamic>> && v.runtimeType == _mapsType) return v;
    if (v is! List) return const [];
    return [for (final e in v) map(e)];
  }

  void dispose() {
    if (_disposed) return;
    final attachmentId = _attachmentId;
    _cancelCadence();
    _disposed = true;
    _bridge.listen(null);
    // Retire this attachment only; the application owns the document and clock.
    _tail.then((_) async {
      try {
        if (attachmentId != null) {
          await _bridge.invoke('detach', {'attachmentId': attachmentId});
        }
      } catch (_) {}
    });
    for (final slice in _slices.values) {
      slice.dispose();
    }
    _slices.clear();
    document.removeListener(_spreadDocument);
    document.dispose();
    _runtimeEpoch.dispose();
    textureId.dispose();
    textureIds.dispose();
    frame.dispose();
    rendered.dispose();
    playing.dispose();
    busy.dispose();
    dragging.dispose();
    error.dispose();
    importedAssets.dispose();
    visibleFrames.dispose();
    deskWork.dispose();
    eyedropper.dispose();
    deskDefault.dispose();
    deskDrawer.dispose();
    panePlaces.dispose();
    anchorPreview.dispose();
    browserTab.dispose();
    textStyleTarget.dispose();
    focusProperty.dispose();
    editingFocus.dispose();
    keyedOnly.dispose();
    viewCommand.dispose();
  }
}
