import 'dart:convert';
import 'dart:ffi';
import 'dart:math' as math;

/// The same-frame path to the runtime: a request and a render return before
/// the frame that asked for them is built. The channel bridge stays for what
/// is not per frame (windows, pickers, settings) and for tests.
abstract class NativeFrames {
  /// One JSON op in, the reply JSON out.
  String request(String json);

  /// Draw `view` into the IOSurface `surfaceId`; 0 on success.
  int render(int surfaceId, String view);
}

typedef _RequestC = Pointer<Uint8> Function(Pointer<Void>, Pointer<Uint8>);
typedef _RenderC = Int32 Function(Pointer<Void>, Uint32, Pointer<Uint8>);
typedef _RenderD = int Function(Pointer<Void>, int, Pointer<Uint8>);
typedef _MallocC = Pointer<Uint8> Function(Size);
typedef _MallocD = Pointer<Uint8> Function(int);
typedef _FreeC = Void Function(Pointer<Uint8>);
typedef _FreeD = void Function(Pointer<Uint8>);
typedef _StrlenC = Size Function(Pointer<Uint8>);
typedef _StrlenD = int Function(Pointer<Uint8>);

/// `dart:ffi` over the dylib the host already opened; the runtime context is
/// the host's, handed over once per document.
class FfiFrames implements NativeFrames {
  FfiFrames(String library, int context)
    : _library = DynamicLibrary.open(library),
      _context = Pointer.fromAddress(context) {
    _request = _library.lookupFunction<_RequestC, _RequestC>(
      'motolii_probe_request',
    );
    _render = _library.lookupFunction<_RenderC, _RenderD>(
      'motolii_probe_render',
    );
    final process = DynamicLibrary.process();
    _malloc = process.lookupFunction<_MallocC, _MallocD>('malloc');
    _free = process.lookupFunction<_FreeC, _FreeD>('free');
    _strlen = process.lookupFunction<_StrlenC, _StrlenD>('strlen');
  }

  final DynamicLibrary _library;
  Pointer<Void> _context;
  late final _RequestC _request;
  late final _RenderD _render;
  late final _MallocD _malloc;
  late final _FreeD _free;
  late final _StrlenD _strlen;
  Pointer<Uint8> _scratch = nullptr;
  int _scratchSize = 0;

  int get context => _context.address;
  set context(int address) => _context = Pointer.fromAddress(address);

  Pointer<Uint8> _cString(String text) {
    final bytes = utf8.encode(text);
    if (bytes.length + 1 > _scratchSize) {
      if (_scratch != nullptr) _free(_scratch);
      _scratchSize = math.max(bytes.length + 1, 4096);
      _scratch = _malloc(_scratchSize);
    }
    final view = _scratch.asTypedList(_scratchSize);
    view.setAll(0, bytes);
    view[bytes.length] = 0;
    return _scratch;
  }

  @override
  String request(String json) {
    if (_context == nullptr) throw StateError('Open a document first');
    final reply = _request(_context, _cString(json));
    if (reply == nullptr) throw StateError('Rust request returned no reply');
    return utf8.decode(reply.asTypedList(_strlen(reply)));
  }

  @override
  int render(int surfaceId, String view) {
    if (_context == nullptr) return -1;
    return _render(_context, surfaceId, _cString(view));
  }

  void dispose() {
    if (_scratch != nullptr) _free(_scratch);
    _scratch = nullptr;
    _scratchSize = 0;
    _context = nullptr;
  }
}
