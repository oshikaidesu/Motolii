import 'package:flutter/widgets.dart';

import '../../../theme/metrics.dart';
import '../../../theme/neutral.dart';

/// A thin node in Explore's graph that is not an asset: a Source, a Folder, the project, a type. It stands for a relation
/// the owner can prove (these assets are in this folder), never for a made-up similarity.
class GraphHub {
  const GraphHub({required this.id, required this.label, required this.relation, required this.rect});
  final String id, label;

  /// The relation kind it belongs to (folder, source, type, project): its colour and its filter.
  final String relation;
  final Rect rect;
}

/// A proven relation between two nodes (an asset id or a hub id).
class GraphEdge {
  const GraphEdge(this.a, this.b, this.relation);
  final String a, b, relation;
}

/// What Explore's graph adds to a [Frame]: hubs, the edges between nodes, and what to hold back while one node is in focus.
class GraphOverlay {
  const GraphOverlay({required this.hubs, required this.edges, this.initialScale, this.tints = const {}, this.blobs = const []});

  /// Metadata that is not topology: a thin colour on a face (its type), and soft regions behind faces that share a place.
  final Map<String, Color> tints;
  final List<({Offset at, double radius, Color color})> blobs;

  /// The camera's first zoom when the caller wants one (a comparison at one magnification); else the whole map is fitted.
  final double? initialScale;
  final List<GraphHub> hubs;
  final List<GraphEdge> edges;

  /// What stays emphasised when [id] is in focus: what it is joined to, and what shares a hub with it (two files in one
  /// folder are related through the folder).
  Set<String> neighbours(String id) {
    final direct = <String>{
      for (final e in edges)
        if (e.a == id) e.b else if (e.b == id) e.a,
    };
    final hubIds = {for (final h in hubs) h.id};
    return {
      ...direct,
      for (final e in edges)
        if (hubIds.contains(e.a) && direct.contains(e.a)) e.b else if (hubIds.contains(e.b) && direct.contains(e.b)) e.a,
    };
  }
}

/// One colour per relation kind (the kind's identity, not a position): also the filter chips' dots.
const relationColors = {
  'similar': Color(0xFF8A94A8),
  'folder': Color(0xFF7FB2E5),
  'source': Color(0xFFE5B27F),
  'type': Color(0xFFA48FE0),
  'project': Color(0xFF8FD6A0),
  'duplicate': Color(0xFFE58F8F),
};

class GraphEdges extends CustomPainter {
  GraphEdges({required this.edges, required this.at, required this.focus, required this.near});
  final List<GraphEdge> edges;

  /// A node's centre by id (assets and hubs alike); an edge with an unplaced end is not drawn.
  final Offset? Function(String id) at;
  final String? focus;
  final Set<String> near;

  @override
  void paint(Canvas canvas, Size size) {
    for (final e in edges) {
      final a = at(e.a), b = at(e.b);
      if (a == null || b == null) continue;
      final lit = focus == null || e.a == focus || e.b == focus;
      final color = (relationColors[e.relation] ?? N.g51).withValues(alpha: lit ? (focus == null ? .55 : .95) : .12);
      canvas.drawLine(a, b, Paint()
        ..color = color
        ..strokeWidth = lit && focus != null ? 1.6 : 1);
    }
  }

  @override
  bool shouldRepaint(GraphEdges o) => true;
}

/// A hub as a small labelled pill in its relation's colour.
class HubPill extends StatelessWidget {
  const HubPill({super.key, required this.hub, this.dim = false});
  final GraphHub hub;
  final bool dim;
  @override
  Widget build(BuildContext context) {
    final color = relationColors[hub.relation] ?? N.g51;
    return Opacity(
      opacity: dim ? .3 : 1,
      child: Container(
        alignment: Alignment.center,
        padding: const EdgeInsets.symmetric(horizontal: 6),
        decoration: BoxDecoration(color: N.g10, borderRadius: BorderRadius.circular(9), border: Border.all(color: color, width: 1)),
        child: Text(hub.label, maxLines: 1, softWrap: false, overflow: TextOverflow.ellipsis, style: Dn.micro(N.g91)),
      ),
    );
  }
}

/// Soft regions behind the faces that share a place (a folder): drawn under the lines, never a line themselves.
class GraphBlobs extends CustomPainter {
  GraphBlobs(this.blobs);
  final List<({Offset at, double radius, Color color})> blobs;
  @override
  void paint(Canvas canvas, Size size) {
    for (final b in blobs) {
      canvas.drawCircle(b.at, b.radius, Paint()..color = b.color);
    }
  }

  @override
  bool shouldRepaint(GraphBlobs o) => true;
}
