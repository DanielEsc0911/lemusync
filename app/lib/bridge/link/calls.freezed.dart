// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'calls.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$SyncNow {





@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncNow);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'SyncNow()';
}


}

/// @nodoc
class $SyncNowCopyWith<$Res>  {
$SyncNowCopyWith(SyncNow _, $Res Function(SyncNow) __);
}


/// Adds pattern-matching-related methods to [SyncNow].
extension SyncNowPatterns on SyncNow {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SyncNow_Done value)?  done,TResult Function( SyncNow_NotRunning value)?  notRunning,TResult Function( SyncNow_Failed value)?  failed,TResult Function( SyncNow_TimedOut value)?  timedOut,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SyncNow_Done() when done != null:
return done(_that);case SyncNow_NotRunning() when notRunning != null:
return notRunning(_that);case SyncNow_Failed() when failed != null:
return failed(_that);case SyncNow_TimedOut() when timedOut != null:
return timedOut(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SyncNow_Done value)  done,required TResult Function( SyncNow_NotRunning value)  notRunning,required TResult Function( SyncNow_Failed value)  failed,required TResult Function( SyncNow_TimedOut value)  timedOut,}){
final _that = this;
switch (_that) {
case SyncNow_Done():
return done(_that);case SyncNow_NotRunning():
return notRunning(_that);case SyncNow_Failed():
return failed(_that);case SyncNow_TimedOut():
return timedOut(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SyncNow_Done value)?  done,TResult? Function( SyncNow_NotRunning value)?  notRunning,TResult? Function( SyncNow_Failed value)?  failed,TResult? Function( SyncNow_TimedOut value)?  timedOut,}){
final _that = this;
switch (_that) {
case SyncNow_Done() when done != null:
return done(_that);case SyncNow_NotRunning() when notRunning != null:
return notRunning(_that);case SyncNow_Failed() when failed != null:
return failed(_that);case SyncNow_TimedOut() when timedOut != null:
return timedOut(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  done,TResult Function()?  notRunning,TResult Function( String code,  String message)?  failed,TResult Function()?  timedOut,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SyncNow_Done() when done != null:
return done();case SyncNow_NotRunning() when notRunning != null:
return notRunning();case SyncNow_Failed() when failed != null:
return failed(_that.code,_that.message);case SyncNow_TimedOut() when timedOut != null:
return timedOut();case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  done,required TResult Function()  notRunning,required TResult Function( String code,  String message)  failed,required TResult Function()  timedOut,}) {final _that = this;
switch (_that) {
case SyncNow_Done():
return done();case SyncNow_NotRunning():
return notRunning();case SyncNow_Failed():
return failed(_that.code,_that.message);case SyncNow_TimedOut():
return timedOut();}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  done,TResult? Function()?  notRunning,TResult? Function( String code,  String message)?  failed,TResult? Function()?  timedOut,}) {final _that = this;
switch (_that) {
case SyncNow_Done() when done != null:
return done();case SyncNow_NotRunning() when notRunning != null:
return notRunning();case SyncNow_Failed() when failed != null:
return failed(_that.code,_that.message);case SyncNow_TimedOut() when timedOut != null:
return timedOut();case _:
  return null;

}
}

}

/// @nodoc


class SyncNow_Done extends SyncNow {
  const SyncNow_Done(): super._();
  






@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncNow_Done);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'SyncNow.done()';
}


}




/// @nodoc


class SyncNow_NotRunning extends SyncNow {
  const SyncNow_NotRunning(): super._();
  






@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncNow_NotRunning);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'SyncNow.notRunning()';
}


}




/// @nodoc


class SyncNow_Failed extends SyncNow {
  const SyncNow_Failed({required this.code, required this.message}): super._();
  

 final  String code;
 final  String message;

/// Create a copy of SyncNow
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SyncNow_FailedCopyWith<SyncNow_Failed> get copyWith => _$SyncNow_FailedCopyWithImpl<SyncNow_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncNow_Failed&&(identical(other.code, code) || other.code == code)&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode {
    return Object.hash(runtimeType,code,message);
}

@override
String toString() {
    return 'SyncNow.failed(code: $code, message: $message)';
}


}

/// @nodoc
abstract mixin class $SyncNow_FailedCopyWith<$Res> implements $SyncNowCopyWith<$Res> {
  factory $SyncNow_FailedCopyWith(SyncNow_Failed value, $Res Function(SyncNow_Failed) _then) = _$SyncNow_FailedCopyWithImpl;
@useResult
$Res call({
 String code, String message
});




}
/// @nodoc
class _$SyncNow_FailedCopyWithImpl<$Res>
    implements $SyncNow_FailedCopyWith<$Res> {
  _$SyncNow_FailedCopyWithImpl(this._self, this._then);

  final SyncNow_Failed _self;
  final $Res Function(SyncNow_Failed) _then;

/// Create a copy of SyncNow
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? code = null,Object? message = null,}) {
  return _then(SyncNow_Failed(
code: null == code ? _self.code : code // ignore: cast_nullable_to_non_nullable
as String,message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SyncNow_TimedOut extends SyncNow {
  const SyncNow_TimedOut(): super._();
  






@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is SyncNow_TimedOut);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'SyncNow.timedOut()';
}


}




// dart format on
