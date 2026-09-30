import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

/// One numeric parameter of the curve in force and how it is edited: scrubbed or typed as previews, one commit, or put back.
class EaseParam {
  const EaseParam(this.name, this.value, {required this.preview, required this.commit, required this.finish, required this.cancel});
  final String name;
  final double value;
  final Future<void> Function(double) preview, commit;
  final Future<void> Function() finish, cancel;
}

/// One curve on the shelf: what the runtime calls it, the samples it draws, and whether it is the one in force.
class EasePreset {
  const EasePreset(this.kind, this.shape, {required this.selected});
  final String kind;
  final Map<String, dynamic> shape;
  final bool selected;
}

/// One interval between selected keys, on the timeline rail.
class EaseSegment {
  const EaseSegment(this.frame, this.end, {required this.active});
  final int frame, end;
  final bool active;
}

/// What the Ease desk knows, apart from how it is drawn: the curve in force, the intervals or the layers it is for, the
/// shelf of curves, the parameters and the handles the runtime's model gives it, and every action the desk can take.
/// The Classic desk and the New desk are two skins on this one view; the logic is written once, in the desk.
class EaseView {
  const EaseView({
    required this.target,
    required this.title,
    required this.hasInterval,
    required this.canApply,
    required this.sequence,
    required this.mixed,
    required this.free,
    required this.keysSelected,
    required this.segments,
    required this.frame,
    required this.playhead,
    required this.shape,
    required this.shown,
    required this.presets,
    required this.hovered,
    required this.params,
    required this.handles,
    required this.notice,
    required this.newKeyKind,
    required this.hasSaved,
    required this.leading,
    required this.motion,
    required this.lo,
    required this.hi,
    required this.peek,
    required this.endPeek,
    required this.choose,
    required this.grab,
    required this.drag,
    required this.release,
    required this.cancel,
    required this.setFree,
    required this.apply,
    required this.copy,
    required this.save,
    required this.useForNew,
    required this.clearSaved,
    required this.play,
  });

  /// The long line of who this is for and why it may not apply; and the short title over the curve.
  final String target, title;
  final bool hasInterval, canApply, mixed, free;

  /// The layers a Sequence spreads its delay over (0 when the desk is on keys).
  final int sequence, keysSelected;
  final List<EaseSegment> segments;
  final ValueListenable<int> frame;
  final double? playhead;
  final Map<String, dynamic> shape, shown;
  final List<EasePreset> presets;
  final int? hovered;
  final List<EaseParam> params;

  /// The handles of the shape in force, in curve coordinates, and the vertical range the plot shows.
  final List<Offset> handles;
  final double lo, hi;
  final String? notice;
  final String newKeyKind;
  final bool hasSaved;
  final Widget? leading;
  final Animation<double> motion;

  final void Function(int index) peek, choose;
  final VoidCallback endPeek, release, cancel, apply, copy, save, useForNew, clearSaved, play;
  final void Function(int handle) grab;
  final void Function(int handle, Offset curvePoint) drag;
  final ValueChanged<bool> setFree;
}
