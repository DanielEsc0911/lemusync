import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/app_controller.dart';
import 'package:lemusync/bridge/link/model.dart' as dto;
import 'package:lemusync/design/components/ls_banner.dart';
import 'package:lemusync/screens/command_line.dart';
import 'package:lemusync/screens/connection_gate.dart';

import '../support/fake_link.dart';
import '../support/fonts.dart';
import 'harness.dart';

void main() {
  setUpAll(loadBrandFonts);

  late FakeDaemonLink link;
  late AppController c;

  setUp(() {
    link = FakeDaemonLink();
    c = AppController(link)..start();
  });
  tearDown(() => c.dispose());

  Future<void> pumpGate(WidgetTester t) =>
      t.pumpWidget(harness(c, const ConnectionGate(child: Text('CHILD'))));

  Future<void> connection(WidgetTester t, dto.ConnectionState s) async {
    link.emit(dto.LinkEvent.connection(s));
    await t.pump();
  }

  testWidgets('connecting with nothing cached', (t) async {
    await pumpGate(t);
    expect(find.text('Connecting…'), findsOneWidget);
    expect(find.text('CHILD'), findsNothing);
  });

  testWidgets('not running without cache: retry and start command', (t) async {
    await pumpGate(t);
    await connection(t, const dto.ConnectionState.notRunning());
    expect(find.text("LemuSync isn't running"), findsOneWidget);
    expect(find.text('lemusync service start'), findsOneWidget);
    expect(find.text('Try again'), findsOneWidget);
    expect(find.text('CHILD'), findsNothing);
    await t.tap(find.text('Try again'));
    await t.pump();
    expect(link.snapshotCalls, 1);
  });

  testWidgets('not running with cached status: child under stale banner', (
    t,
  ) async {
    await pumpGate(t);
    await feed(t, link, st: status(dto.StateDto.synced));
    await connection(t, const dto.ConnectionState.notRunning());
    expect(find.text('CHILD'), findsOneWidget);
    await t.pump();
    expect(find.byType(LsBanner), findsOneWidget);
    expect(find.text('Showing the last known state'), findsOneWidget);
    expect(find.text('lemusync service start'), findsNothing);
  });

  testWidgets('unsupported', (t) async {
    await pumpGate(t);
    await connection(t, const dto.ConnectionState.unsupported());
    expect(find.text('Not available on this system yet'), findsOneWidget);
    expect(find.text('CHILD'), findsNothing);
    expect(find.byType(CommandLine), findsNothing);
  });

  testWidgets('refused: no retry, no command', (t) async {
    await pumpGate(t);
    await connection(t, const dto.ConnectionState.refused(message: 'v2 vs v1'));
    expect(find.text("This app and the service don't match"), findsOneWidget);
    expect(find.text('Try again'), findsNothing);
    expect(find.byType(CommandLine), findsNothing);
  });

  testWidgets('failed: code first, raw message secondary, retry, logs', (
    t,
  ) async {
    await pumpGate(t);
    await connection(
      t,
      const dto.ConnectionState.failed(code: 'boom', message: 'raw detail'),
    );
    expect(find.textContaining('Error code: boom'), findsOneWidget);
    expect(find.text('raw detail'), findsOneWidget);
    expect(find.text('Try again'), findsOneWidget);
    expect(find.text('lemusync logs'), findsOneWidget);
  });

  testWidgets('unsafe: no retry', (t) async {
    await pumpGate(t);
    await connection(t, const dto.ConnectionState.unsafe());
    expect(
      find.text("LemuSync won't connect to an unsafe socket"),
      findsOneWidget,
    );
    expect(find.text('Try again'), findsNothing);
  });

  testWidgets('online: the child only', (t) async {
    await pumpGate(t);
    await feed(t, link, st: status(dto.StateDto.synced));
    expect(find.text('CHILD'), findsOneWidget);
    expect(find.byType(LsBanner), findsNothing);
  });

  for (final MapEntry(key: name, value: colors) in themes.entries) {
    testWidgets('200% text, 320 wide, no overflow in $name', (t) async {
      await t.pumpWidget(
        harness(
          c,
          const ConnectionGate(child: Text('CHILD')),
          colors: colors,
          scale: 2,
          width: 320,
        ),
      );
      await connection(
        t,
        const dto.ConnectionState.failed(code: 'boom', message: 'raw detail'),
      );
      expect(t.takeException(), isNull);
    });
  }
}
