import '../../session/editor_session.dart';
import 'blend_host.dart';
import 'depth_host.dart';
import 'ease.dart' show LiveEaseHost;
import 'notes.dart' show LiveNotesHost;

/// The desks' hosts (Ease, Notes, Blend, Depth), one each per EditorSession: a desk's work in progress belongs to the
/// document session, not to the widget that shows it, so a desk moved to its own window or rebuilt keeps it.
class DeskSession {
  DeskSession._(this.c);
  static final _all = Expando<DeskSession>();
  static DeskSession of(EditorSession c) => _all[c] ??= DeskSession._(c);
  final EditorSession c;

  late final ease = LiveEaseHost(c);
  late final notes = LiveNotesHost(c);
  late final blend = BlendController(c);
  late final depth = DepthController(c);
}
