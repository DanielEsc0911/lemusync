import 'package:flutter/material.dart';

import '../tokens/colors.g.dart';
import '../tokens/size.g.dart';
import '../tokens/space.g.dart';
import '../tokens/type.g.dart';

/// Nothing here yet: the mark, a title, a hint and one action.
class EmptyState extends StatelessWidget {
  const EmptyState({super.key, required this.title, this.body, this.action});

  final String title;
  final String? body;
  final Widget? action;

  @override
  Widget build(BuildContext context) {
    final c = Theme.of(context).extension<LsColors>()!;
    return Padding(
      padding: const EdgeInsets.all(LsSpace.xl),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        spacing: LsSpace.md,
        children: [
          Image.asset(
            'assets/brand/mark.png',
            width: LsSize.mark,
            height: LsSize.mark,
            excludeFromSemantics: true,
          ),
          Text(
            title,
            textAlign: TextAlign.center,
            style: LsType.title.copyWith(color: c.onSurface),
          ),
          if (body != null)
            Text(
              body!,
              textAlign: TextAlign.center,
              style: LsType.body.copyWith(color: c.onSurfaceMuted),
            ),
          ?action,
        ],
      ),
    );
  }
}
