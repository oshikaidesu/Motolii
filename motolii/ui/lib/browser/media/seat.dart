import 'package:flutter/widgets.dart';

import '../../session/editor_session.dart';
import 'catalog_controls.dart';
import 'catalog_session.dart';

/// The Media seat: the catalog Browser over the work's own assets (This project, as the old shelf showed them) and every
/// registered Source. It owns the catalog session for as long as the seat is open.
class MediaSeat extends StatefulWidget {
  const MediaSeat({super.key, required this.c});
  final EditorSession c;
  @override
  State<MediaSeat> createState() => _MediaSeatState();
}

class _MediaSeatState extends State<MediaSeat> {
  late final CatalogSession session = CatalogSession(widget.c);

  @override
  void initState() {
    super.initState();
    session.load();
  }

  @override
  void dispose() {
    session.dispose();
    super.dispose();
  }

  @override
  // A work that holds nothing has nothing to show under "This project": the first look is at everything the person has.
  Widget build(BuildContext context) => CatalogMedia(session: session, startProject: EditorSession.maps(widget.c.state['assets']).isNotEmpty);
}
