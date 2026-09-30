import 'package:flutter/widgets.dart';

import 'instrument.dart';
import '../../session/editor_session.dart';
import 'store.dart';
import '../session.dart';
import '../../theme/metrics.dart';

/// The Transform Instrument on the real session, for the Inspector's Transform card.
/// It follows the session's layers and selection; while nothing is selected there is nothing to show.
class NewTransform extends StatefulWidget {
  const NewTransform({super.key, required this.controller, this.showHeader = false});
  final EditorSession controller;

  /// The layer's name and Animate above the instrument; a host that shows them itself hides these.
  final bool showHeader;
  @override
  State<NewTransform> createState() => _NewTransformState();
}

class _NewTransformState extends State<NewTransform> {
  EditorSession get c => widget.controller;
  late final session = InspectorSession.of(c);
  SessionTransformStore? get store => session.transform;

  @override
  void initState() {
    super.initState();
    session.addListener(_changed);
  }

  void _changed() => setState(() {});

  @override
  void dispose() {
    session.removeListener(_changed);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final s = store;
    if (s == null) return const SizedBox.shrink();
    return LayoutBuilder(
      builder: (context, box) {
        final instrument = TransformInstrument(s, showHeader: widget.showHeader);
        // Narrower than the Instrument's wide layout it scrolls inside a fixed height.
        return box.maxWidth < 230 ? SizedBox(height: Surface.px(345), child: instrument) : instrument;
      },
    );
  }
}
