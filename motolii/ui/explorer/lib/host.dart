// The real Motolii host, loaded straight from its native library (the same `motolii_probe_*` entry points the macOS
// runner calls). Stories open real documents and send real operations: nothing of the product's meaning lives here.
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

typedef _OpenC = Pointer<Void> Function(Pointer<Utf8>);
typedef _RequestC = Pointer<Utf8> Function(Pointer<Void>, Pointer<Utf8>);
typedef _CloseC = Void Function(Pointer<Void>);
typedef _CatalogC = Pointer<Utf8> Function(Pointer<Utf8>);

class RealHost {
  RealHost._(this._context);

  static final DynamicLibrary _lib = DynamicLibrary.open(Platform.environment['MOTOLII_NATIVE_LIBRARY'] ?? _defaultLibrary());
  static final _open = _lib.lookupFunction<_OpenC, _OpenC>('motolii_probe_open');
  static final _request = _lib.lookupFunction<_RequestC, _RequestC>('motolii_probe_request');
  static final _close = _lib.lookupFunction<_CloseC, void Function(Pointer<Void>)>('motolii_probe_close');
  static final _catalog = _lib.lookupFunction<_CatalogC, _CatalogC>('motolii_catalog_request');

  /// The media catalog (one per process, not per document): a JSON request, the JSON reply.
  static Map<String, dynamic> catalog(String command) {
    final p = command.toNativeUtf8();
    try {
      final reply = _catalog(p);
      if (reply == nullptr) throw StateError('The catalog gave no reply');
      return (jsonDecode(reply.toDartString()) as Map).cast<String, dynamic>();
    } finally {
      malloc.free(p);
    }
  }

  /// explorer/ sits in motolii/ui; the workspace builds the library into motolii/target/debug.
  static String _defaultLibrary() {
    var dir = Directory.current;
    for (var i = 0; i < 6; i++) {
      final lib = File('${dir.path}/motolii/target/debug/libmotolii_ui.dylib');
      if (lib.existsSync()) return lib.path;
      final here = File('${dir.path}/target/debug/libmotolii_ui.dylib');
      if (here.existsSync()) return here.path;
      dir = dir.parent;
    }
    return 'libmotolii_ui.dylib';
  }

  Pointer<Void> _context;

  /// A document from disk, or a new one ('').
  static RealHost open(String path) {
    final p = path.toNativeUtf8();
    try {
      final ctx = _open(p);
      if (ctx == nullptr) throw StateError('The host could not open "$path"');
      return RealHost._(ctx);
    } finally {
      malloc.free(p);
    }
  }

  Map<String, dynamic> request(Map<String, dynamic> command) => requestText(jsonEncode(command));

  Map<String, dynamic> requestText(String command) {
    final p = command.toNativeUtf8();
    try {
      final reply = _request(_context, p);
      if (reply == nullptr) throw StateError('The host gave no reply');
      return (jsonDecode(reply.toDartString()) as Map).cast<String, dynamic>();
    } finally {
      malloc.free(p);
    }
  }

  void close() {
    if (_context == nullptr) return;
    _close(_context);
    _context = nullptr;
  }
}
