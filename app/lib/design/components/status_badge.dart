import 'package:flutter/material.dart';

import '../../l10n/generated/app_localizations.dart';
import '../status.dart';
import '../tokens/colors.g.dart';
import '../tokens/size.g.dart';
import '../tokens/space.g.dart';
import '../tokens/type.g.dart';

/// A face-button dot with the status icon, then the label.
/// `compact` hides the label visually but keeps it for screen readers.
class StatusBadge extends StatelessWidget {
  const StatusBadge({super.key, required this.status, this.compact = false});

  final LsStatus status;
  final bool compact;

  @override
  Widget build(BuildContext context) {
    final c = Theme.of(context).extension<LsColors>()!;
    final label = status.label(AppLocalizations.of(context));
    final dot = DecoratedBox(
      decoration: BoxDecoration(color: status.color(c), shape: BoxShape.circle),
      child: SizedBox.square(
        dimension: LsSize.iconMd,
        child: Icon(status.icon, size: LsSize.iconSm, color: status.onColor(c)),
      ),
    );
    return Semantics(
      label: label,
      container: true,
      excludeSemantics: true,
      child: compact
          ? dot
          : Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                dot,
                const SizedBox(width: LsSpace.sm),
                Flexible(
                  child: Text(
                    label,
                    style: LsType.label.copyWith(color: c.onSurface),
                  ),
                ),
              ],
            ),
    );
  }
}
