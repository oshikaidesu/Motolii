import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

class A1Window extends StatefulWidget {
  const A1Window({required this.path});
  final String path;
  @override
  State<A1Window> createState() => _A1WindowState();
}

class _A1WindowState extends State<A1Window> {
  static const _channel = MethodChannel('motolii/a1');
  int? _texture;

  @override
  void initState() {
    super.initState();
    _channel.invokeMethod<int>('start', widget.path).then((id) {
      if (mounted && id != null && id >= 0) setState(() => _texture = id);
    });
  }

  @override
  Widget build(BuildContext context) {
    final texture = _texture;
    return ColoredBox(
      color: const Color(0xFF000000),
      child: texture == null
          ? const SizedBox.expand()
          : Texture(textureId: texture, filterQuality: FilterQuality.none),
    );
  }
}
