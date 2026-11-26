import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/daemon_link.dart';
import 'package:lemusync/bridge/api/daemon.dart';
import 'package:lemusync/bridge/link/flow_model.dart';
import 'package:lemusync/bridge/link/play.dart';

/// A [PlayHandle] without the native library.
class _FakePlayHandle implements PlayHandle {
  final StreamController<PlayUpdate> source = StreamController<PlayUpdate>();
  final List<PlayAnswerDto> answers = [];
  final List<(String, String, String)> started = [];
  int stops = 0;
  bool accept = true;

  @override
  Stream<PlayUpdate> play({
    required String player,
    required String emulator,
    required String game,
  }) {
    started.add((player, emulator, game));
    return source.stream;
  }

  @override
  bool answer({required PlayAnswerDto answer}) {
    answers.add(answer);
    return accept;
  }

  @override
  Future<void> stop() async => stops++;

  @override
  dynamic noSuchMethod(Invocation invocation) => null;
}

void main() {
  late _FakePlayHandle handle;
  late DaemonLink link;

  setUp(() {
    handle = _FakePlayHandle();
    link = FrbDaemonLink(newPlayHandle: () => handle);
  });

  PlaySession start() => link.play(player: 'Ana', emulator: 'cemu', game: 'aa');

  test('nothing starts before someone listens', () {
    start();
    expect(handle.started, isEmpty);
  });

  test(
    'listening starts the session with the ids and forwards updates',
    () async {
      final s = start();
      final got = <PlayUpdate>[];
      final sub = s.updates.listen(got.add);
      handle.source.add(const PlayUpdate.step(PlayStepDto.pulling));
      await pumpEventQueue();
      expect(handle.started, [('Ana', 'cemu', 'aa')]);
      expect(got, [const PlayUpdate.step(PlayStepDto.pulling)]);
      await sub.cancel();
    },
  );

  test(
    'an answer goes to the handle once; none before the session starts',
    () async {
      final s = start();
      expect(s.answer(const PlayAnswerDto.wait()), isFalse);
      expect(handle.answers, isEmpty);
      final sub = s.updates.listen((_) {});
      await pumpEventQueue();
      expect(s.answer(const PlayAnswerDto.takeOver()), isTrue);
      expect(handle.answers, [const PlayAnswerDto.takeOver()]);
      await sub.cancel();
    },
  );

  test('cancelling the updates stops the handle once', () async {
    final sub = start().updates.listen((_) {});
    await pumpEventQueue();
    await sub.cancel();
    expect(handle.stops, 1);
  });

  test('the session ending closes the updates', () async {
    var done = false;
    start().updates.listen((_) {}, onDone: () => done = true);
    await pumpEventQueue();
    await handle.source.close();
    await pumpEventQueue();
    expect(done, isTrue);
  });
}
