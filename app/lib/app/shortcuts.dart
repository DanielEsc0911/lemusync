import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import 'destination.dart';

class DestinationIntent extends Intent {
  const DestinationIntent(this.destination);
  final Destination destination;
}

const _digits = [
  LogicalKeyboardKey.digit1,
  LogicalKeyboardKey.digit2,
  LogicalKeyboardKey.digit3,
  LogicalKeyboardKey.digit4,
  LogicalKeyboardKey.digit5,
];

/// Ctrl+digit and Cmd+digit for every destination.
final Map<ShortcutActivator, Intent> destinationShortcuts = {
  for (final d in Destination.values) ...{
    SingleActivator(_digits[d.index], control: true): DestinationIntent(d),
    SingleActivator(_digits[d.index], meta: true): DestinationIntent(d),
  },
};
