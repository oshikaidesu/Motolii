abstract class NativeFrames {
  String request(String json);
  int render(int surfaceId, String view);
}

class FfiFrames implements NativeFrames {
  FfiFrames(String library, this.context);

  int context;

  @override
  String request(String json) =>
      throw UnsupportedError('Native frames are unavailable on this platform');

  @override
  int render(int surfaceId, String view) =>
      throw UnsupportedError('Native frames are unavailable on this platform');

  void dispose() {}
}
