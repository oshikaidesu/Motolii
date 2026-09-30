import 'package:flutter/widgets.dart';

import '../../theme/metrics.dart';
import '../../theme/editor_theme.dart';

/// What the controls sit in: the panel bar.

/// A panel bar. Its Row keeps Spacer alignment while the content fits and
/// slides sideways when it does not, instead of overflowing the pane.
class EditorBar extends StatelessWidget {
  const EditorBar({
    super.key,
    required this.children,
    this.height,
    this.padding = EdgeInsets.zero,
    this.decoration,
  });
  final List<Widget> children;
  final double? height;
  final EdgeInsetsGeometry padding;
  final BoxDecoration? decoration;
  @override
  Widget build(BuildContext context) => Container(
    height: height ?? Surface.px(22),
    decoration:
        decoration ?? BoxDecoration(color: EditorTheme.of(context).panel),
    child: LayoutBuilder(
      builder: (context, box) => SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: ConstrainedBox(
          constraints: BoxConstraints(minWidth: box.maxWidth),
          child: IntrinsicWidth(
            child: Padding(
              padding: padding,
              child: Row(children: children),
            ),
          ),
        ),
      ),
    ),
  );
}


