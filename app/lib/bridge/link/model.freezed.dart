// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'model.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ConnectionState {





@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is ConnectionState);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'ConnectionState()';
}


}

/// @nodoc
class $ConnectionStateCopyWith<$Res>  {
$ConnectionStateCopyWith(ConnectionState _, $Res Function(ConnectionState) __);
}


/// Adds pattern-matching-related methods to [ConnectionState].
extension ConnectionStatePatterns on ConnectionState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ConnectionState_Connecting value)?  connecting,TResult Function( ConnectionState_Online value)?  online,TResult Function( ConnectionState_NotRunning value)?  notRunning,TResult Function( ConnectionState_Unsupported value)?  unsupported,TResult Function( ConnectionState_Unsafe value)?  unsafe,TResult Function( ConnectionState_Refused value)?  refused,TResult Function( ConnectionState_Failed value)?  failed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ConnectionState_Connecting() when connecting != null:
return connecting(_that);case ConnectionState_Online() when online != null:
return online(_that);case ConnectionState_NotRunning() when notRunning != null:
return notRunning(_that);case ConnectionState_Unsupported() when unsupported != null:
return unsupported(_that);case ConnectionState_Unsafe() when unsafe != null:
return unsafe(_that);case ConnectionState_Refused() when refused != null:
return refused(_that);case ConnectionState_Failed() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ConnectionState_Connecting value)  connecting,required TResult Function( ConnectionState_Online value)  online,required TResult Function( ConnectionState_NotRunning value)  notRunning,required TResult Function( ConnectionState_Unsupported value)  unsupported,required TResult Function( ConnectionState_Unsafe value)  unsafe,required TResult Function( ConnectionState_Refused value)  refused,required TResult Function( ConnectionState_Failed value)  failed,}){
final _that = this;
switch (_that) {
case ConnectionState_Connecting():
return connecting(_that);case ConnectionState_Online():
return online(_that);case ConnectionState_NotRunning():
return notRunning(_that);case ConnectionState_Unsupported():
return unsupported(_that);case ConnectionState_Unsafe():
return unsafe(_that);case ConnectionState_Refused():
return refused(_that);case ConnectionState_Failed():
return failed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ConnectionState_Connecting value)?  connecting,TResult? Function( ConnectionState_Online value)?  online,TResult? Function( ConnectionState_NotRunning value)?  notRunning,TResult? Function( ConnectionState_Unsupported value)?  unsupported,TResult? Function( ConnectionState_Unsafe value)?  unsafe,TResult? Function( ConnectionState_Refused value)?  refused,TResult? Function( ConnectionState_Failed value)?  failed,}){
final _that = this;
switch (_that) {
case ConnectionState_Connecting() when connecting != null:
return connecting(_that);case ConnectionState_Online() when online != null:
return online(_that);case ConnectionState_NotRunning() when notRunning != null:
return notRunning(_that);case ConnectionState_Unsupported() when unsupported != null:
return unsupported(_that);case ConnectionState_Unsafe() when unsafe != null:
return unsafe(_that);case ConnectionState_Refused() when refused != null:
return refused(_that);case ConnectionState_Failed() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  connecting,TResult Function( String daemonVersion)?  online,TResult Function()?  notRunning,TResult Function()?  unsupported,TResult Function()?  unsafe,TResult Function( String message)?  refused,TResult Function( String code,  String message)?  failed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ConnectionState_Connecting() when connecting != null:
return connecting();case ConnectionState_Online() when online != null:
return online(_that.daemonVersion);case ConnectionState_NotRunning() when notRunning != null:
return notRunning();case ConnectionState_Unsupported() when unsupported != null:
return unsupported();case ConnectionState_Unsafe() when unsafe != null:
return unsafe();case ConnectionState_Refused() when refused != null:
return refused(_that.message);case ConnectionState_Failed() when failed != null:
return failed(_that.code,_that.message);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  connecting,required TResult Function( String daemonVersion)  online,required TResult Function()  notRunning,required TResult Function()  unsupported,required TResult Function()  unsafe,required TResult Function( String message)  refused,required TResult Function( String code,  String message)  failed,}) {final _that = this;
switch (_that) {
case ConnectionState_Connecting():
return connecting();case ConnectionState_Online():
return online(_that.daemonVersion);case ConnectionState_NotRunning():
return notRunning();case ConnectionState_Unsupported():
return unsupported();case ConnectionState_Unsafe():
return unsafe();case ConnectionState_Refused():
return refused(_that.message);case ConnectionState_Failed():
return failed(_that.code,_that.message);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  connecting,TResult? Function( String daemonVersion)?  online,TResult? Function()?  notRunning,TResult? Function()?  unsupported,TResult? Function()?  unsafe,TResult? Function( String message)?  refused,TResult? Function( String code,  String message)?  failed,}) {final _that = this;
switch (_that) {
case ConnectionState_Connecting() when connecting != null:
return connecting();case ConnectionState_Online() when online != null:
return online(_that.daemonVersion);case ConnectionState_NotRunning() when notRunning != null:
return notRunning();case ConnectionState_Unsupported() when unsupported != null:
return unsupported();case ConnectionState_Unsafe() when unsafe != null:
return unsafe();case ConnectionState_Refused() when refused != null:
return refused(_that.message);case ConnectionState_Failed() when failed != null:
return failed(_that.code,_that.message);case _:
  return null;

}
}

}

/// @nodoc


class ConnectionState_Connecting extends ConnectionState {
  const ConnectionState_Connecting(): super._();
  






@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is ConnectionState_Connecting);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'ConnectionState.connecting()';
}


}




/// @nodoc


class ConnectionState_Online extends ConnectionState {
  const ConnectionState_Online({required this.daemonVersion}): super._();
  

 final  String daemonVersion;

/// Create a copy of ConnectionState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ConnectionState_OnlineCopyWith<ConnectionState_Online> get copyWith => _$ConnectionState_OnlineCopyWithImpl<ConnectionState_Online>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is ConnectionState_Online&&(identical(other.daemonVersion, daemonVersion) || other.daemonVersion == daemonVersion));
}


@override
int get hashCode {
    return Object.hash(runtimeType,daemonVersion);
}

@override
String toString() {
    return 'ConnectionState.online(daemonVersion: $daemonVersion)';
}


}

/// @nodoc
abstract mixin class $ConnectionState_OnlineCopyWith<$Res> implements $ConnectionStateCopyWith<$Res> {
  factory $ConnectionState_OnlineCopyWith(ConnectionState_Online value, $Res Function(ConnectionState_Online) _then) = _$ConnectionState_OnlineCopyWithImpl;
@useResult
$Res call({
 String daemonVersion
});




}
/// @nodoc
class _$ConnectionState_OnlineCopyWithImpl<$Res>
    implements $ConnectionState_OnlineCopyWith<$Res> {
  _$ConnectionState_OnlineCopyWithImpl(this._self, this._then);

  final ConnectionState_Online _self;
  final $Res Function(ConnectionState_Online) _then;

/// Create a copy of ConnectionState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? daemonVersion = null,}) {
  return _then(ConnectionState_Online(
daemonVersion: null == daemonVersion ? _self.daemonVersion : daemonVersion // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ConnectionState_NotRunning extends ConnectionState {
  const ConnectionState_NotRunning(): super._();
  






@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is ConnectionState_NotRunning);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'ConnectionState.notRunning()';
}


}




/// @nodoc


class ConnectionState_Unsupported extends ConnectionState {
  const ConnectionState_Unsupported(): super._();
  






@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is ConnectionState_Unsupported);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'ConnectionState.unsupported()';
}


}




/// @nodoc


class ConnectionState_Unsafe extends ConnectionState {
  const ConnectionState_Unsafe(): super._();
  






@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is ConnectionState_Unsafe);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'ConnectionState.unsafe()';
}


}




/// @nodoc


class ConnectionState_Refused extends ConnectionState {
  const ConnectionState_Refused({required this.message}): super._();
  

 final  String message;

/// Create a copy of ConnectionState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ConnectionState_RefusedCopyWith<ConnectionState_Refused> get copyWith => _$ConnectionState_RefusedCopyWithImpl<ConnectionState_Refused>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is ConnectionState_Refused&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode {
    return Object.hash(runtimeType,message);
}

@override
String toString() {
    return 'ConnectionState.refused(message: $message)';
}


}

/// @nodoc
abstract mixin class $ConnectionState_RefusedCopyWith<$Res> implements $ConnectionStateCopyWith<$Res> {
  factory $ConnectionState_RefusedCopyWith(ConnectionState_Refused value, $Res Function(ConnectionState_Refused) _then) = _$ConnectionState_RefusedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$ConnectionState_RefusedCopyWithImpl<$Res>
    implements $ConnectionState_RefusedCopyWith<$Res> {
  _$ConnectionState_RefusedCopyWithImpl(this._self, this._then);

  final ConnectionState_Refused _self;
  final $Res Function(ConnectionState_Refused) _then;

/// Create a copy of ConnectionState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(ConnectionState_Refused(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ConnectionState_Failed extends ConnectionState {
  const ConnectionState_Failed({required this.code, required this.message}): super._();
  

 final  String code;
 final  String message;

/// Create a copy of ConnectionState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ConnectionState_FailedCopyWith<ConnectionState_Failed> get copyWith => _$ConnectionState_FailedCopyWithImpl<ConnectionState_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is ConnectionState_Failed&&(identical(other.code, code) || other.code == code)&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode {
    return Object.hash(runtimeType,code,message);
}

@override
String toString() {
    return 'ConnectionState.failed(code: $code, message: $message)';
}


}

/// @nodoc
abstract mixin class $ConnectionState_FailedCopyWith<$Res> implements $ConnectionStateCopyWith<$Res> {
  factory $ConnectionState_FailedCopyWith(ConnectionState_Failed value, $Res Function(ConnectionState_Failed) _then) = _$ConnectionState_FailedCopyWithImpl;
@useResult
$Res call({
 String code, String message
});




}
/// @nodoc
class _$ConnectionState_FailedCopyWithImpl<$Res>
    implements $ConnectionState_FailedCopyWith<$Res> {
  _$ConnectionState_FailedCopyWithImpl(this._self, this._then);

  final ConnectionState_Failed _self;
  final $Res Function(ConnectionState_Failed) _then;

/// Create a copy of ConnectionState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? code = null,Object? message = null,}) {
  return _then(ConnectionState_Failed(
code: null == code ? _self.code : code // ignore: cast_nullable_to_non_nullable
as String,message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$LinkEvent {

 Object? get field0;



@override
bool operator ==(Object other) {
  final _this = this as LinkEvent;
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LinkEvent&&const DeepCollectionEquality().equals(other.field0, _this.field0));
}


@override
int get hashCode {
  final _this = this as LinkEvent;
  return Object.hash(runtimeType,const DeepCollectionEquality().hash(_this.field0));
}

@override
String toString() {
  final _this = this as LinkEvent;
  return 'LinkEvent(field0: ${_this.field0})';
}


}

/// @nodoc
class $LinkEventCopyWith<$Res>  {
$LinkEventCopyWith(LinkEvent _, $Res Function(LinkEvent) __);
}


/// Adds pattern-matching-related methods to [LinkEvent].
extension LinkEventPatterns on LinkEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( LinkEvent_Connection value)?  connection,TResult Function( LinkEvent_Status value)?  status,TResult Function( LinkEvent_Session value)?  session,TResult Function( LinkEvent_Progress value)?  progress,TResult Function( LinkEvent_Snapshot value)?  snapshot,TResult Function( LinkEvent_Games value)?  games,required TResult orElse(),}){
final _that = this;
switch (_that) {
case LinkEvent_Connection() when connection != null:
return connection(_that);case LinkEvent_Status() when status != null:
return status(_that);case LinkEvent_Session() when session != null:
return session(_that);case LinkEvent_Progress() when progress != null:
return progress(_that);case LinkEvent_Snapshot() when snapshot != null:
return snapshot(_that);case LinkEvent_Games() when games != null:
return games(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( LinkEvent_Connection value)  connection,required TResult Function( LinkEvent_Status value)  status,required TResult Function( LinkEvent_Session value)  session,required TResult Function( LinkEvent_Progress value)  progress,required TResult Function( LinkEvent_Snapshot value)  snapshot,required TResult Function( LinkEvent_Games value)  games,}){
final _that = this;
switch (_that) {
case LinkEvent_Connection():
return connection(_that);case LinkEvent_Status():
return status(_that);case LinkEvent_Session():
return session(_that);case LinkEvent_Progress():
return progress(_that);case LinkEvent_Snapshot():
return snapshot(_that);case LinkEvent_Games():
return games(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( LinkEvent_Connection value)?  connection,TResult? Function( LinkEvent_Status value)?  status,TResult? Function( LinkEvent_Session value)?  session,TResult? Function( LinkEvent_Progress value)?  progress,TResult? Function( LinkEvent_Snapshot value)?  snapshot,TResult? Function( LinkEvent_Games value)?  games,}){
final _that = this;
switch (_that) {
case LinkEvent_Connection() when connection != null:
return connection(_that);case LinkEvent_Status() when status != null:
return status(_that);case LinkEvent_Session() when session != null:
return session(_that);case LinkEvent_Progress() when progress != null:
return progress(_that);case LinkEvent_Snapshot() when snapshot != null:
return snapshot(_that);case LinkEvent_Games() when games != null:
return games(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( ConnectionState field0)?  connection,TResult Function( StatusDto field0)?  status,TResult Function( SessionDto? field0)?  session,TResult Function( ProgressDto field0)?  progress,TResult Function( Snapshot field0)?  snapshot,TResult Function( GamesDto field0)?  games,required TResult orElse(),}) {final _that = this;
switch (_that) {
case LinkEvent_Connection() when connection != null:
return connection(_that.field0);case LinkEvent_Status() when status != null:
return status(_that.field0);case LinkEvent_Session() when session != null:
return session(_that.field0);case LinkEvent_Progress() when progress != null:
return progress(_that.field0);case LinkEvent_Snapshot() when snapshot != null:
return snapshot(_that.field0);case LinkEvent_Games() when games != null:
return games(_that.field0);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( ConnectionState field0)  connection,required TResult Function( StatusDto field0)  status,required TResult Function( SessionDto? field0)  session,required TResult Function( ProgressDto field0)  progress,required TResult Function( Snapshot field0)  snapshot,required TResult Function( GamesDto field0)  games,}) {final _that = this;
switch (_that) {
case LinkEvent_Connection():
return connection(_that.field0);case LinkEvent_Status():
return status(_that.field0);case LinkEvent_Session():
return session(_that.field0);case LinkEvent_Progress():
return progress(_that.field0);case LinkEvent_Snapshot():
return snapshot(_that.field0);case LinkEvent_Games():
return games(_that.field0);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( ConnectionState field0)?  connection,TResult? Function( StatusDto field0)?  status,TResult? Function( SessionDto? field0)?  session,TResult? Function( ProgressDto field0)?  progress,TResult? Function( Snapshot field0)?  snapshot,TResult? Function( GamesDto field0)?  games,}) {final _that = this;
switch (_that) {
case LinkEvent_Connection() when connection != null:
return connection(_that.field0);case LinkEvent_Status() when status != null:
return status(_that.field0);case LinkEvent_Session() when session != null:
return session(_that.field0);case LinkEvent_Progress() when progress != null:
return progress(_that.field0);case LinkEvent_Snapshot() when snapshot != null:
return snapshot(_that.field0);case LinkEvent_Games() when games != null:
return games(_that.field0);case _:
  return null;

}
}

}

/// @nodoc


class LinkEvent_Connection extends LinkEvent {
  const LinkEvent_Connection(this.field0): super._();
  

@override final  ConnectionState field0;

/// Create a copy of LinkEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LinkEvent_ConnectionCopyWith<LinkEvent_Connection> get copyWith => _$LinkEvent_ConnectionCopyWithImpl<LinkEvent_Connection>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is LinkEvent_Connection&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode {
    return Object.hash(runtimeType,field0);
}

@override
String toString() {
    return 'LinkEvent.connection(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $LinkEvent_ConnectionCopyWith<$Res> implements $LinkEventCopyWith<$Res> {
  factory $LinkEvent_ConnectionCopyWith(LinkEvent_Connection value, $Res Function(LinkEvent_Connection) _then) = _$LinkEvent_ConnectionCopyWithImpl;
@useResult
$Res call({
 ConnectionState field0
});


$ConnectionStateCopyWith<$Res> get field0;

}
/// @nodoc
class _$LinkEvent_ConnectionCopyWithImpl<$Res>
    implements $LinkEvent_ConnectionCopyWith<$Res> {
  _$LinkEvent_ConnectionCopyWithImpl(this._self, this._then);

  final LinkEvent_Connection _self;
  final $Res Function(LinkEvent_Connection) _then;

/// Create a copy of LinkEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(LinkEvent_Connection(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as ConnectionState,
  ));
}

/// Create a copy of LinkEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ConnectionStateCopyWith<$Res> get field0 {
  
  return $ConnectionStateCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class LinkEvent_Status extends LinkEvent {
  const LinkEvent_Status(this.field0): super._();
  

@override final  StatusDto field0;

/// Create a copy of LinkEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LinkEvent_StatusCopyWith<LinkEvent_Status> get copyWith => _$LinkEvent_StatusCopyWithImpl<LinkEvent_Status>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is LinkEvent_Status&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode {
    return Object.hash(runtimeType,field0);
}

@override
String toString() {
    return 'LinkEvent.status(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $LinkEvent_StatusCopyWith<$Res> implements $LinkEventCopyWith<$Res> {
  factory $LinkEvent_StatusCopyWith(LinkEvent_Status value, $Res Function(LinkEvent_Status) _then) = _$LinkEvent_StatusCopyWithImpl;
@useResult
$Res call({
 StatusDto field0
});




}
/// @nodoc
class _$LinkEvent_StatusCopyWithImpl<$Res>
    implements $LinkEvent_StatusCopyWith<$Res> {
  _$LinkEvent_StatusCopyWithImpl(this._self, this._then);

  final LinkEvent_Status _self;
  final $Res Function(LinkEvent_Status) _then;

/// Create a copy of LinkEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(LinkEvent_Status(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as StatusDto,
  ));
}


}

/// @nodoc


class LinkEvent_Session extends LinkEvent {
  const LinkEvent_Session([this.field0]): super._();
  

@override final  SessionDto? field0;

/// Create a copy of LinkEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LinkEvent_SessionCopyWith<LinkEvent_Session> get copyWith => _$LinkEvent_SessionCopyWithImpl<LinkEvent_Session>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is LinkEvent_Session&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode {
    return Object.hash(runtimeType,field0);
}

@override
String toString() {
    return 'LinkEvent.session(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $LinkEvent_SessionCopyWith<$Res> implements $LinkEventCopyWith<$Res> {
  factory $LinkEvent_SessionCopyWith(LinkEvent_Session value, $Res Function(LinkEvent_Session) _then) = _$LinkEvent_SessionCopyWithImpl;
@useResult
$Res call({
 SessionDto? field0
});




}
/// @nodoc
class _$LinkEvent_SessionCopyWithImpl<$Res>
    implements $LinkEvent_SessionCopyWith<$Res> {
  _$LinkEvent_SessionCopyWithImpl(this._self, this._then);

  final LinkEvent_Session _self;
  final $Res Function(LinkEvent_Session) _then;

/// Create a copy of LinkEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = freezed,}) {
  return _then(LinkEvent_Session(
freezed == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as SessionDto?,
  ));
}


}

/// @nodoc


class LinkEvent_Progress extends LinkEvent {
  const LinkEvent_Progress(this.field0): super._();
  

@override final  ProgressDto field0;

/// Create a copy of LinkEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LinkEvent_ProgressCopyWith<LinkEvent_Progress> get copyWith => _$LinkEvent_ProgressCopyWithImpl<LinkEvent_Progress>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is LinkEvent_Progress&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode {
    return Object.hash(runtimeType,field0);
}

@override
String toString() {
    return 'LinkEvent.progress(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $LinkEvent_ProgressCopyWith<$Res> implements $LinkEventCopyWith<$Res> {
  factory $LinkEvent_ProgressCopyWith(LinkEvent_Progress value, $Res Function(LinkEvent_Progress) _then) = _$LinkEvent_ProgressCopyWithImpl;
@useResult
$Res call({
 ProgressDto field0
});




}
/// @nodoc
class _$LinkEvent_ProgressCopyWithImpl<$Res>
    implements $LinkEvent_ProgressCopyWith<$Res> {
  _$LinkEvent_ProgressCopyWithImpl(this._self, this._then);

  final LinkEvent_Progress _self;
  final $Res Function(LinkEvent_Progress) _then;

/// Create a copy of LinkEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(LinkEvent_Progress(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as ProgressDto,
  ));
}


}

/// @nodoc


class LinkEvent_Snapshot extends LinkEvent {
  const LinkEvent_Snapshot(this.field0): super._();
  

@override final  Snapshot field0;

/// Create a copy of LinkEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LinkEvent_SnapshotCopyWith<LinkEvent_Snapshot> get copyWith => _$LinkEvent_SnapshotCopyWithImpl<LinkEvent_Snapshot>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is LinkEvent_Snapshot&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode {
    return Object.hash(runtimeType,field0);
}

@override
String toString() {
    return 'LinkEvent.snapshot(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $LinkEvent_SnapshotCopyWith<$Res> implements $LinkEventCopyWith<$Res> {
  factory $LinkEvent_SnapshotCopyWith(LinkEvent_Snapshot value, $Res Function(LinkEvent_Snapshot) _then) = _$LinkEvent_SnapshotCopyWithImpl;
@useResult
$Res call({
 Snapshot field0
});




}
/// @nodoc
class _$LinkEvent_SnapshotCopyWithImpl<$Res>
    implements $LinkEvent_SnapshotCopyWith<$Res> {
  _$LinkEvent_SnapshotCopyWithImpl(this._self, this._then);

  final LinkEvent_Snapshot _self;
  final $Res Function(LinkEvent_Snapshot) _then;

/// Create a copy of LinkEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(LinkEvent_Snapshot(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as Snapshot,
  ));
}


}

/// @nodoc


class LinkEvent_Games extends LinkEvent {
  const LinkEvent_Games(this.field0): super._();
  

@override final  GamesDto field0;

/// Create a copy of LinkEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LinkEvent_GamesCopyWith<LinkEvent_Games> get copyWith => _$LinkEvent_GamesCopyWithImpl<LinkEvent_Games>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is LinkEvent_Games&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode {
    return Object.hash(runtimeType,field0);
}

@override
String toString() {
    return 'LinkEvent.games(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $LinkEvent_GamesCopyWith<$Res> implements $LinkEventCopyWith<$Res> {
  factory $LinkEvent_GamesCopyWith(LinkEvent_Games value, $Res Function(LinkEvent_Games) _then) = _$LinkEvent_GamesCopyWithImpl;
@useResult
$Res call({
 GamesDto field0
});




}
/// @nodoc
class _$LinkEvent_GamesCopyWithImpl<$Res>
    implements $LinkEvent_GamesCopyWith<$Res> {
  _$LinkEvent_GamesCopyWithImpl(this._self, this._then);

  final LinkEvent_Games _self;
  final $Res Function(LinkEvent_Games) _then;

/// Create a copy of LinkEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(LinkEvent_Games(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as GamesDto,
  ));
}


}

// dart format on
