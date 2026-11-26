import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/call_result.dart';
import 'package:lemusync/bridge/link/model.dart' as dto;

void main() {
  test('a value becomes CallOk', () async {
    final r = await guard(() async => 7);
    expect(r, const CallOk(7));
  });

  test('a thrown ConnectionState becomes CallFailed with that state', () async {
    const state = dto.ConnectionState.failed(
      code: 'version_not_found',
      message: 'no such version',
    );
    final r = await guard<int>(() async => throw state);
    expect(r, const CallFailed<int>(state));
  });

  test(
    'anything else thrown becomes a bridge failure, never a throw',
    () async {
      final r = await guard<int>(() async => throw StateError('broken'));
      expect(r, isA<CallFailed<int>>());
      final state = (r as CallFailed<int>).state;
      expect(state, isA<dto.ConnectionState_Failed>());
      expect((state as dto.ConnectionState_Failed).code, 'bridge');
    },
  );
}
