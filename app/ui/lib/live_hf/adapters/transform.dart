import 'package:flutter/widgets.dart';

import '../../hf/insp/transform.dart';
import '../../session/editor_session.dart';
import 'transform_store.dart';

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
  static const _watched = ['layers', 'selectedId', 'selectedIds', 'animate', 'capabilities', 'documentRevision'];
  EditorSession get c => widget.controller;
  SessionTransformStore? store;

  @override
  void initState() {
    super.initState();
    c.slice('newTransform', _watched).addListener(_absorb);
    c.rendered.addListener(_absorb);
    c.focusProperty.addListener(_focus);
    _absorb();
  }

  void _absorb() {
    if (c.layers.isEmpty || c.activeLayer == null) {
      if (store != null) setState(() => store = null);
      return;
    }
    if (store == null) {
      setState(() => store = SessionTransformStore(c));
    } else {
      store!.absorb();
    }
  }

  void _focus() {
    final id = c.focusProperty.value;
    if (id != null) store?.focusProperty(id);
  }

  @override
  void dispose() {
    c.slice('newTransform', _watched).removeListener(_absorb);
    c.rendered.removeListener(_absorb);
    c.focusProperty.removeListener(_focus);
    store?.dispose();
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
        return box.maxWidth < 230 ? SizedBox(height: 460, child: instrument) : instrument;
      },
    );
  }
}
