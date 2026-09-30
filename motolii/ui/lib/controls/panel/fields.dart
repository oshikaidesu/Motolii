import 'package:flutter/widgets.dart';

import '../../theme/metrics.dart';
import '../../theme/editor_theme.dart';

/// The box typing happens in, and the field that holds a draft until it is
/// committed or refused.

/// The one box a field sits in, at rest and while typing. The theme draws no
/// frame around a TextField, so a field has this border and no other: line at
/// rest, accent while it owns the keys, error while it refuses a draft. A tap
/// anywhere in the box hands the keys to [focus].
class EditorFieldFrame extends StatelessWidget {
  const EditorFieldFrame({
    super.key,
    required this.child,
    this.focus,
    this.error = false,
    this.height,
    this.minHeight,
    this.maxHeight,
    this.padding,
    this.color,
  });
  final Widget child;
  final FocusNode? focus;
  final bool error;
  final double? height, minHeight, maxHeight;
  final EdgeInsets? padding;
  final Color? color;

  Widget _box(BuildContext context, bool focused) => GestureDetector(
    behavior: HitTestBehavior.translucent,
    onTap: focus?.requestFocus,
    child: Container(
      height: height ?? Surface.workRow,
      constraints: minHeight == null && maxHeight == null
          ? null
          : BoxConstraints(
              minHeight: minHeight ?? 0,
              maxHeight: maxHeight ?? double.infinity,
            ),
      padding: padding ?? EdgeInsets.symmetric(horizontal: Surface.px(5)),
      decoration: BoxDecoration(
        color: color ?? EditorTheme.of(context).app,
        border: Border.all(
          color: error
              ? EditorTheme.of(context).error
              : focused
              ? EditorTheme.of(context).accent
              : EditorTheme.of(context).line,
        ),
      ),
      child: child,
    ),
  );

  @override
  Widget build(BuildContext context) => focus == null
      ? _box(context, false)
      : ListenableBuilder(
          listenable: focus!,
          builder: (_, __) => _box(context, focus!.hasFocus),
        );
}


