import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../metrics.dart';
import '../theme.dart';

/// The leaves that come in front of the window and hold it: the modal dialog
/// over its scrim, and the spinner that says to wait.

/// A modal sheet over a black scrim, faded in over 150 ms: [EditorTheme.of(context).panel]
/// with a [EditorTheme.of(context).border] edge, a title, a body and a row of actions at
/// the end — the dialog Material drew under the editor's theme.
Future<T?> showEditorDialog<T>({
  required BuildContext context,
  required WidgetBuilder builder,
}) => showGeneralDialog<T>(
  context: context,
  barrierDismissible: true,
  barrierLabel: 'Dismiss',
  barrierColor: EditorTheme.of(context).scrim,
  transitionDuration: const Duration(milliseconds: 150),
  transitionBuilder: (context, animation, secondary, child) => FadeTransition(
    opacity: CurvedAnimation(parent: animation, curve: Curves.easeOut),
    child: child,
  ),
  pageBuilder: (context, animation, secondary) => Builder(builder: builder),
);

class EditorDialog extends StatelessWidget {
  const EditorDialog({
    super.key,
    this.title,
    this.content,
    this.actions = const [],
  });
  final Widget? title, content;
  final List<Widget> actions;
  @override
  Widget build(BuildContext context) => SafeArea(
    child: Center(
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: EditorMetrics.s44,
          vertical: EditorMetrics.control,
        ),
        child: ConstrainedBox(
          constraints: const BoxConstraints(minWidth: EditorMetrics.s280),
          child: Container(
            decoration: BoxDecoration(
              color: EditorTheme.of(context).panel,
              border: Border.fromBorderSide(
                BorderSide(color: EditorTheme.of(context).border),
              ),
            ),
            child: IntrinsicWidth(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  if (title != null)
                    Padding(
                      padding: const EdgeInsets.fromLTRB(
                        EditorMetrics.control,
                        EditorMetrics.control,
                        EditorMetrics.control,
                        0,
                      ),
                      child: DefaultTextStyle(
                        style: TextStyle(
                          fontSize: EditorMetrics.title,
                          color: EditorTheme.of(context).ink,
                        ),
                        child: Semantics(namesRoute: true, child: title),
                      ),
                    ),
                  if (content != null)
                    Padding(
                      padding: const EdgeInsets.fromLTRB(
                        EditorMetrics.control,
                        EditorMetrics.s16,
                        EditorMetrics.control,
                        EditorMetrics.control,
                      ),
                      child: DefaultTextStyle(
                        style: TextStyle(
                          fontSize: EditorMetrics.font,
                          color: EditorTheme.of(context).ink,
                        ),
                        child: content!,
                      ),
                    ),
                  if (actions.isNotEmpty)
                    Padding(
                      padding: const EdgeInsets.fromLTRB(
                        EditorMetrics.control,
                        0,
                        EditorMetrics.control,
                        EditorMetrics.control,
                      ),
                      child: Row(
                        mainAxisAlignment: MainAxisAlignment.end,
                        children: [
                          for (var i = 0; i < actions.length; i++) ...[
                            if (i > 0) const SizedBox(width: EditorMetrics.s8),
                            actions[i],
                          ],
                        ],
                      ),
                    ),
                ],
              ),
            ),
          ),
        ),
      ),
    ),
  );
}

/// An indeterminate wait: a 36 px ring, a 4 px accent arc sweeping round it.
class EditorSpinner extends StatefulWidget {
  const EditorSpinner({super.key});
  @override
  State<EditorSpinner> createState() => _EditorSpinnerState();
}

class _EditorSpinnerState extends State<EditorSpinner>
    with SingleTickerProviderStateMixin {
  late final _turn = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 1333),
  )..repeat();
  @override
  void dispose() {
    _turn.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => SizedBox.square(
    dimension: EditorMetrics.s36,
    child: CustomPaint(
      painter: _SpinnerPainter(colors: EditorTheme.of(context), _turn),
    ),
  );
}

class _SpinnerPainter extends CustomPainter {
  final EditorTheme colors;

  _SpinnerPainter(this.turn, {this.colors = EditorTheme.chromatic})
    : super(repaint: turn);
  final Animation<double> turn;
  @override
  void paint(Canvas canvas, Size size) {
    final t = turn.value;
    final head = Curves.easeInOut.transform(((t * 2) % 1));
    final start = t * 2 * math.pi * 2 + head * math.pi;
    final sweep = math.pi / 2 + head * math.pi;
    canvas.drawArc(
      (Offset.zero & size).deflate(EditorMetrics.s2),
      start,
      sweep,
      false,
      Paint()
        ..color = colors.accent
        ..style = PaintingStyle.stroke
        ..strokeWidth = EditorMetrics.s4
        ..strokeCap = StrokeCap.butt,
    );
  }

  @override
  bool shouldRepaint(_SpinnerPainter old) =>
      colors != old.colors || old.turn != turn;
}
