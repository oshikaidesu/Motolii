// The present inside (motolii/filament/motolii.h), called straight from Dart: dart:ffi on its C calls, as
// sets/glass_live.dart calls the old glass. The skin holds no document: it holds this handle and the numbers of things.
// One call = one change = one step of undo. A call that is not done returns false and [trouble] says why.
// macOS only, Debug build only (the Debug entitlements have the sandbox off so that the library can be loaded).
import 'dart:ffi';

import 'package:ffi/ffi.dart';
import 'package:flutter/services.dart';

/// Where the inside lives. Override with --dart-define=MOTOLII_DIR=/path/to/motolii/filament.
const motoliiDir = String.fromEnvironment('MOTOLII_DIR', defaultValue: '/Users/motolii_maker/motolii/filament');

typedef _P = Pointer<Void>;
typedef _S = Pointer<Utf8>;

class Motolii {
  Motolii._(this._lib, this._m, this.libPath);
  final DynamicLibrary _lib;
  final _P _m;
  final String libPath;

  /// The document at [document] (made if it is not there), on a stage [wide] by [high]. null: it could not be opened.
  static Motolii? open(String document, {int wide = 1920, int high = 1080, String dir = motoliiDir}) {
    final path = '$dir/out/libmotolii.dylib';
    final DynamicLibrary lib;
    try {
      lib = DynamicLibrary.open(path);
    } catch (_) {
      return null;
    }
    final home = '$dir/out/wgpu_kit'.toNativeUtf8(), doc = document.toNativeUtf8();
    lib.lookupFunction<Void Function(_S), void Function(_S)>('motolii_home')(home);
    final m = lib.lookupFunction<_P Function(_S, Int32, Int32), _P Function(_S, int, int)>('motolii_open')(doc, wide, high);
    malloc
      ..free(home)
      ..free(doc);
    return m == nullptr ? null : Motolii._(lib, m, path);
  }

  late final _live = _lib.lookupFunction<Int32 Function(_P), int Function(_P)>('motolii_live');
  late final _play = _lib.lookupFunction<Int32 Function(_P, Int32), int Function(_P, int)>('motolii_play');
  late final _now = _lib.lookupFunction<Double Function(_P), double Function(_P)>('motolii_now');
  late final _at = _lib.lookupFunction<Int32 Function(_P, Double), int Function(_P, double)>('motolii_at');
  late final _seconds = _lib.lookupFunction<Double Function(_P), double Function(_P)>('motolii_seconds');
  late final _add = _lib.lookupFunction<Int32 Function(_P, _S, _S), int Function(_P, _S, _S)>('motolii_add');
  late final _remove = _lib.lookupFunction<Int32 Function(_P, Int32), int Function(_P, int)>('motolii_remove');
  late final _set = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Double), int Function(_P, int, _S, double)>('motolii_set');
  late final _say = _lib.lookupFunction<Int32 Function(_P, Int32, _S, _S), int Function(_P, int, _S, _S)>('motolii_say');
  late final _key = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Double, Double), int Function(_P, int, _S, double, double)>('motolii_key');
  late final _unkey = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Double), int Function(_P, int, _S, double)>('motolii_unkey');
  late final _way = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Double, _S), int Function(_P, int, _S, double, _S)>('motolii_way');
  late final _value = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Double, Pointer<Double>), int Function(_P, int, _S, double, Pointer<Double>)>('motolii_value');
  late final _undo = _lib.lookupFunction<Int32 Function(_P), int Function(_P)>('motolii_undo');
  late final _redo = _lib.lookupFunction<Int32 Function(_P), int Function(_P)>('motolii_redo');
  late final _trouble = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_trouble');
  late final _list = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_list');
  late final _close = _lib.lookupFunction<Void Function(_P), void Function(_P)>('motolii_close');

  T _with<T>(List<String> words, T Function(List<_S>) call) {
    final s = [for (final w in words) w.toNativeUtf8()];
    try {
      return call(s);
    } finally {
      for (final p in s) {
        malloc.free(p);
      }
    }
  }

  static const _channel = MethodChannel('motolii/stage');

  /// The stage's picture as a Flutter texture (the inside draws it by its own clock; nothing is copied). -1: none.
  Future<int> texture() async {
    if (_live(_m) == 0) return -1;
    return await _channel.invokeMethod<int>('start', {'lib': libPath, 'handle': _m.address}) ?? -1;
  }

  String get trouble => _trouble(_m).toDartString();
  String get list => _list(_m).toDartString();
  double get now => _now(_m);
  double get seconds => _seconds(_m);
  bool play(bool on) => _play(_m, on ? 1 : 0) != 0;
  bool at(double seconds) => _at(_m, seconds) != 0;

  /// A new thing (film, picture, text, solid, glass) from a file (a glass: its shape): its number, 0 if it was not made.
  int add(String kind, String file) => _with([kind, file], (s) => _add(_m, s[0], s[1]));
  bool remove(int thing) => _remove(_m, thing) != 0;
  bool set(int thing, String name, double number) => _with([name], (s) => _set(_m, thing, s[0], number)) != 0;
  bool say(int thing, String name, String word) => _with([name, word], (s) => _say(_m, thing, s[0], s[1])) != 0;
  bool key(int thing, String name, double at, double number) => _with([name], (s) => _key(_m, thing, s[0], at, number)) != 0;
  bool unkey(int thing, String name, double at) => _with([name], (s) => _unkey(_m, thing, s[0], at)) != 0;
  bool way(int thing, String name, double at, String way) => _with([name, way], (s) => _way(_m, thing, s[0], at, s[1])) != 0;
  double? value(int thing, String name, double at) => _with([name], (s) {
    final out = malloc<Double>();
    final ok = _value(_m, thing, s[0], at, out) != 0, v = out.value;
    malloc.free(out);
    return ok ? v : null;
  });
  bool undo() => _undo(_m) != 0;
  bool redo() => _redo(_m) != 0;
  void close() => _close(_m);
}
