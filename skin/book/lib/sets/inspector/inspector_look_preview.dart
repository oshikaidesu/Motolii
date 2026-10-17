// A look card's preview (Path, Deform, Surface, Glass, the effects...): the thing alone, as the stage draws it, in the
// card's square (the inside's solo picture: Motolii.soloTexture). Its numbers change it as they change the stage.
import 'package:flutter/widgets.dart';
import 'package:lab_book/sets/inspector/inspector_parts.dart';
import 'package:lab_book/tokens.dart';

class LookPreview extends StatefulWidget {
  const LookPreview({super.key});
  static const side = 292; // (points of its picture: twice the square, for a sharp screen)
  @override
  State<LookPreview> createState() => _LookPreviewState();
}

class _LookPreviewState extends State<LookPreview> {
  int _texture = -1;
  Doc? _doc;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final doc = Ctx.of(context).doc;
    if (_doc != null) return;
    _doc = doc;
    doc.onSolo?.call(LookPreview.side, true).then((id) => mounted ? setState(() => _texture = id) : null);
  }

  @override
  void dispose() {
    _doc?.onSolo?.call(LookPreview.side, false); // (none drawn once no card shows it)
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => ClipRRect(
    borderRadius: BorderRadius.circular(8),
    child: ColoredBox(
      color: Grey.g07,
      child: _texture < 0 ? const SizedBox.expand() : Texture(textureId: _texture, filterQuality: FilterQuality.medium),
    ),
  );
}
