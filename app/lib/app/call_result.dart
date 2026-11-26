import '../bridge/link/model.dart';

/// What a daemon call came to: a value, or the connection state that stopped
/// it (`failed` carries the daemon's error code). Errors are values: nothing a
/// call does throws to a widget (app-shell spec, Connection).
sealed class CallResult<T> {
  const CallResult();
}

final class CallOk<T> extends CallResult<T> {
  const CallOk(this.value);

  final T value;

  @override
  bool operator ==(Object other) => other is CallOk<T> && other.value == value;

  @override
  int get hashCode => value.hashCode;
}

final class CallFailed<T> extends CallResult<T> {
  const CallFailed(this.state);

  final ConnectionState state;

  @override
  bool operator ==(Object other) =>
      other is CallFailed<T> && other.state == state;

  @override
  int get hashCode => state.hashCode;
}

/// Runs a bridge call. The bridge throws the [ConnectionState] a Rust `Err`
/// carries; anything else thrown (a broken bridge) becomes the code `bridge`.
Future<CallResult<T>> guard<T>(Future<T> Function() call) async {
  try {
    return CallOk(await call());
  } on ConnectionState catch (state) {
    return CallFailed(state);
  } catch (e) {
    return CallFailed(ConnectionState.failed(code: 'bridge', message: '$e'));
  }
}
