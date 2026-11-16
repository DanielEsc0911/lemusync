import 'package:flutter/material.dart';

import '../../l10n/generated/app_localizations.dart';
import '../icons.dart';
import '../tokens/colors.g.dart';
import '../tokens/size.g.dart';
import '../tokens/space.g.dart';
import '../tokens/type.g.dart';

/// Something failed: icon, what happened, and a way to try again.
class ErrorState extends StatelessWidget {
  const ErrorState({
    super.key,
    required this.title,
    this.body,
    this.retryLabel,
    this.onRetry,
  });

  final String title;
  final String? body;
  final String? retryLabel;
  final VoidCallback? onRetry;

  @override
  Widget build(BuildContext context) {
    final c = Theme.of(context).extension<LsColors>()!;
    return Padding(
      padding: const EdgeInsets.all(LsSpace.xl),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        spacing: LsSpace.md,
        children: [
          Icon(LsIcons.error, size: LsSize.mark, color: c.danger),
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
          if (onRetry != null)
            FilledButton(
              onPressed: onRetry,
              child: Text(
                retryLabel ?? AppLocalizations.of(context).actionRetry,
              ),
            ),
        ],
      ),
    );
  }
}
