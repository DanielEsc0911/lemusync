// SP3b probe (not part of the app): runs the generated bridge against the
// socket under $LEMUSYNC_TEST_ROOT/run, never the user's own daemon.
//
//   cargo build -p lemusync-bridge   (from the repo root)
//   LEMUSYNC_TEST_ROOT=$(mktemp -d) dart run tool/sp3b_probe.dart
//
// Prints `snapshot: <connection>`. With a test daemon already serving that
// socket, also listens for events, cancels, and prints what it saw.
import 'dart:async';
import 'dart:io';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:lemusync/app/daemon_link.dart';
import 'package:lemusync/bridge/frb_generated.dart';

Future<void> main() async {
  if ((Platform.environment['LEMUSYNC_TEST_ROOT'] ?? '').isEmpty) {
    stderr.writeln('set LEMUSYNC_TEST_ROOT to an empty temp dir');
    exit(2);
  }
  // `dart run` does not bundle the native asset: load the cargo build directly.
  final lib =
      Platform.environment['PROBE_LIB'] ??
      '../target/debug/liblemusync_bridge.so';
  await RustLib.init(externalLibrary: ExternalLibrary.open(lib));
  final link = FrbDaemonLink();
  final snap = await link.snapshot();
  stdout.writeln('snapshot: ${snap.connection}');
  if (Platform.environment['PROBE_EVENTS'] == null) {
    exit(0);
  }
  stdout.writeln('games: ${(await link.games())?.games.length}');
  var seen = 0;
  final done = Completer<void>();
  final sub = link.events().listen((e) {
    stdout.writeln('event: $e');
    if (++seen == 4 && !done.isCompleted) done.complete();
  });
  await done.future.timeout(const Duration(seconds: 15));
  await sub.cancel();
  stdout.writeln('cancelled after $seen events');
  await Future<void>.delayed(const Duration(seconds: 2));
  exit(0);
}
