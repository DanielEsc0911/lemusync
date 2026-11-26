// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'flow_model.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PlayAnswerDto {





@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayAnswerDto);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'PlayAnswerDto()';
}


}

/// @nodoc
class $PlayAnswerDtoCopyWith<$Res>  {
$PlayAnswerDtoCopyWith(PlayAnswerDto _, $Res Function(PlayAnswerDto) __);
}


/// Adds pattern-matching-related methods to [PlayAnswerDto].
extension PlayAnswerDtoPatterns on PlayAnswerDto {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PlayAnswerDto_Wait value)?  wait,TResult Function( PlayAnswerDto_PlayAnyway value)?  playAnyway,TResult Function( PlayAnswerDto_TakeOver value)?  takeOver,TResult Function( PlayAnswerDto_Keep value)?  keep,TResult Function( PlayAnswerDto_Refuse value)?  refuse,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PlayAnswerDto_Wait() when wait != null:
return wait(_that);case PlayAnswerDto_PlayAnyway() when playAnyway != null:
return playAnyway(_that);case PlayAnswerDto_TakeOver() when takeOver != null:
return takeOver(_that);case PlayAnswerDto_Keep() when keep != null:
return keep(_that);case PlayAnswerDto_Refuse() when refuse != null:
return refuse(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PlayAnswerDto_Wait value)  wait,required TResult Function( PlayAnswerDto_PlayAnyway value)  playAnyway,required TResult Function( PlayAnswerDto_TakeOver value)  takeOver,required TResult Function( PlayAnswerDto_Keep value)  keep,required TResult Function( PlayAnswerDto_Refuse value)  refuse,}){
final _that = this;
switch (_that) {
case PlayAnswerDto_Wait():
return wait(_that);case PlayAnswerDto_PlayAnyway():
return playAnyway(_that);case PlayAnswerDto_TakeOver():
return takeOver(_that);case PlayAnswerDto_Keep():
return keep(_that);case PlayAnswerDto_Refuse():
return refuse(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PlayAnswerDto_Wait value)?  wait,TResult? Function( PlayAnswerDto_PlayAnyway value)?  playAnyway,TResult? Function( PlayAnswerDto_TakeOver value)?  takeOver,TResult? Function( PlayAnswerDto_Keep value)?  keep,TResult? Function( PlayAnswerDto_Refuse value)?  refuse,}){
final _that = this;
switch (_that) {
case PlayAnswerDto_Wait() when wait != null:
return wait(_that);case PlayAnswerDto_PlayAnyway() when playAnyway != null:
return playAnyway(_that);case PlayAnswerDto_TakeOver() when takeOver != null:
return takeOver(_that);case PlayAnswerDto_Keep() when keep != null:
return keep(_that);case PlayAnswerDto_Refuse() when refuse != null:
return refuse(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  wait,TResult Function()?  playAnyway,TResult Function()?  takeOver,TResult Function( String? version)?  keep,TResult Function()?  refuse,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PlayAnswerDto_Wait() when wait != null:
return wait();case PlayAnswerDto_PlayAnyway() when playAnyway != null:
return playAnyway();case PlayAnswerDto_TakeOver() when takeOver != null:
return takeOver();case PlayAnswerDto_Keep() when keep != null:
return keep(_that.version);case PlayAnswerDto_Refuse() when refuse != null:
return refuse();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  wait,required TResult Function()  playAnyway,required TResult Function()  takeOver,required TResult Function( String? version)  keep,required TResult Function()  refuse,}) {final _that = this;
switch (_that) {
case PlayAnswerDto_Wait():
return wait();case PlayAnswerDto_PlayAnyway():
return playAnyway();case PlayAnswerDto_TakeOver():
return takeOver();case PlayAnswerDto_Keep():
return keep(_that.version);case PlayAnswerDto_Refuse():
return refuse();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  wait,TResult? Function()?  playAnyway,TResult? Function()?  takeOver,TResult? Function( String? version)?  keep,TResult? Function()?  refuse,}) {final _that = this;
switch (_that) {
case PlayAnswerDto_Wait() when wait != null:
return wait();case PlayAnswerDto_PlayAnyway() when playAnyway != null:
return playAnyway();case PlayAnswerDto_TakeOver() when takeOver != null:
return takeOver();case PlayAnswerDto_Keep() when keep != null:
return keep(_that.version);case PlayAnswerDto_Refuse() when refuse != null:
return refuse();case _:
  return null;

}
}

}

/// @nodoc


class PlayAnswerDto_Wait extends PlayAnswerDto {
  const PlayAnswerDto_Wait(): super._();
  






@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayAnswerDto_Wait);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'PlayAnswerDto.wait()';
}


}




/// @nodoc


class PlayAnswerDto_PlayAnyway extends PlayAnswerDto {
  const PlayAnswerDto_PlayAnyway(): super._();
  






@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayAnswerDto_PlayAnyway);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'PlayAnswerDto.playAnyway()';
}


}




/// @nodoc


class PlayAnswerDto_TakeOver extends PlayAnswerDto {
  const PlayAnswerDto_TakeOver(): super._();
  






@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayAnswerDto_TakeOver);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'PlayAnswerDto.takeOver()';
}


}




/// @nodoc


class PlayAnswerDto_Keep extends PlayAnswerDto {
  const PlayAnswerDto_Keep({this.version}): super._();
  

 final  String? version;

/// Create a copy of PlayAnswerDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayAnswerDto_KeepCopyWith<PlayAnswerDto_Keep> get copyWith => _$PlayAnswerDto_KeepCopyWithImpl<PlayAnswerDto_Keep>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayAnswerDto_Keep&&(identical(other.version, version) || other.version == version));
}


@override
int get hashCode {
    return Object.hash(runtimeType,version);
}

@override
String toString() {
    return 'PlayAnswerDto.keep(version: $version)';
}


}

/// @nodoc
abstract mixin class $PlayAnswerDto_KeepCopyWith<$Res> implements $PlayAnswerDtoCopyWith<$Res> {
  factory $PlayAnswerDto_KeepCopyWith(PlayAnswerDto_Keep value, $Res Function(PlayAnswerDto_Keep) _then) = _$PlayAnswerDto_KeepCopyWithImpl;
@useResult
$Res call({
 String? version
});




}
/// @nodoc
class _$PlayAnswerDto_KeepCopyWithImpl<$Res>
    implements $PlayAnswerDto_KeepCopyWith<$Res> {
  _$PlayAnswerDto_KeepCopyWithImpl(this._self, this._then);

  final PlayAnswerDto_Keep _self;
  final $Res Function(PlayAnswerDto_Keep) _then;

/// Create a copy of PlayAnswerDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? version = freezed,}) {
  return _then(PlayAnswerDto_Keep(
version: freezed == version ? _self.version : version // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class PlayAnswerDto_Refuse extends PlayAnswerDto {
  const PlayAnswerDto_Refuse(): super._();
  






@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayAnswerDto_Refuse);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'PlayAnswerDto.refuse()';
}


}




/// @nodoc
mixin _$PlayQuestionDto {





@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayQuestionDto);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'PlayQuestionDto()';
}


}

/// @nodoc
class $PlayQuestionDtoCopyWith<$Res>  {
$PlayQuestionDtoCopyWith(PlayQuestionDto _, $Res Function(PlayQuestionDto) __);
}


/// Adds pattern-matching-related methods to [PlayQuestionDto].
extension PlayQuestionDtoPatterns on PlayQuestionDto {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PlayQuestionDto_Lease value)?  lease,TResult Function( PlayQuestionDto_Conflict value)?  conflict,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PlayQuestionDto_Lease() when lease != null:
return lease(_that);case PlayQuestionDto_Conflict() when conflict != null:
return conflict(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PlayQuestionDto_Lease value)  lease,required TResult Function( PlayQuestionDto_Conflict value)  conflict,}){
final _that = this;
switch (_that) {
case PlayQuestionDto_Lease():
return lease(_that);case PlayQuestionDto_Conflict():
return conflict(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PlayQuestionDto_Lease value)?  lease,TResult? Function( PlayQuestionDto_Conflict value)?  conflict,}){
final _that = this;
switch (_that) {
case PlayQuestionDto_Lease() when lease != null:
return lease(_that);case PlayQuestionDto_Conflict() when conflict != null:
return conflict(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String by,  BigInt agoS,  bool stale)?  lease,TResult Function( ConflictDto conflict)?  conflict,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PlayQuestionDto_Lease() when lease != null:
return lease(_that.by,_that.agoS,_that.stale);case PlayQuestionDto_Conflict() when conflict != null:
return conflict(_that.conflict);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String by,  BigInt agoS,  bool stale)  lease,required TResult Function( ConflictDto conflict)  conflict,}) {final _that = this;
switch (_that) {
case PlayQuestionDto_Lease():
return lease(_that.by,_that.agoS,_that.stale);case PlayQuestionDto_Conflict():
return conflict(_that.conflict);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String by,  BigInt agoS,  bool stale)?  lease,TResult? Function( ConflictDto conflict)?  conflict,}) {final _that = this;
switch (_that) {
case PlayQuestionDto_Lease() when lease != null:
return lease(_that.by,_that.agoS,_that.stale);case PlayQuestionDto_Conflict() when conflict != null:
return conflict(_that.conflict);case _:
  return null;

}
}

}

/// @nodoc


class PlayQuestionDto_Lease extends PlayQuestionDto {
  const PlayQuestionDto_Lease({required this.by, required this.agoS, required this.stale}): super._();
  

 final  String by;
 final  BigInt agoS;
/// Expired: only then may the player take it over.
 final  bool stale;

/// Create a copy of PlayQuestionDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayQuestionDto_LeaseCopyWith<PlayQuestionDto_Lease> get copyWith => _$PlayQuestionDto_LeaseCopyWithImpl<PlayQuestionDto_Lease>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayQuestionDto_Lease&&(identical(other.by, by) || other.by == by)&&(identical(other.agoS, agoS) || other.agoS == agoS)&&(identical(other.stale, stale) || other.stale == stale));
}


@override
int get hashCode {
    return Object.hash(runtimeType,by,agoS,stale);
}

@override
String toString() {
    return 'PlayQuestionDto.lease(by: $by, agoS: $agoS, stale: $stale)';
}


}

/// @nodoc
abstract mixin class $PlayQuestionDto_LeaseCopyWith<$Res> implements $PlayQuestionDtoCopyWith<$Res> {
  factory $PlayQuestionDto_LeaseCopyWith(PlayQuestionDto_Lease value, $Res Function(PlayQuestionDto_Lease) _then) = _$PlayQuestionDto_LeaseCopyWithImpl;
@useResult
$Res call({
 String by, BigInt agoS, bool stale
});




}
/// @nodoc
class _$PlayQuestionDto_LeaseCopyWithImpl<$Res>
    implements $PlayQuestionDto_LeaseCopyWith<$Res> {
  _$PlayQuestionDto_LeaseCopyWithImpl(this._self, this._then);

  final PlayQuestionDto_Lease _self;
  final $Res Function(PlayQuestionDto_Lease) _then;

/// Create a copy of PlayQuestionDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? by = null,Object? agoS = null,Object? stale = null,}) {
  return _then(PlayQuestionDto_Lease(
by: null == by ? _self.by : by // ignore: cast_nullable_to_non_nullable
as String,agoS: null == agoS ? _self.agoS : agoS // ignore: cast_nullable_to_non_nullable
as BigInt,stale: null == stale ? _self.stale : stale // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class PlayQuestionDto_Conflict extends PlayQuestionDto {
  const PlayQuestionDto_Conflict({required this.conflict}): super._();
  

 final  ConflictDto conflict;

/// Create a copy of PlayQuestionDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayQuestionDto_ConflictCopyWith<PlayQuestionDto_Conflict> get copyWith => _$PlayQuestionDto_ConflictCopyWithImpl<PlayQuestionDto_Conflict>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayQuestionDto_Conflict&&(identical(other.conflict, conflict) || other.conflict == conflict));
}


@override
int get hashCode {
    return Object.hash(runtimeType,conflict);
}

@override
String toString() {
    return 'PlayQuestionDto.conflict(conflict: $conflict)';
}


}

/// @nodoc
abstract mixin class $PlayQuestionDto_ConflictCopyWith<$Res> implements $PlayQuestionDtoCopyWith<$Res> {
  factory $PlayQuestionDto_ConflictCopyWith(PlayQuestionDto_Conflict value, $Res Function(PlayQuestionDto_Conflict) _then) = _$PlayQuestionDto_ConflictCopyWithImpl;
@useResult
$Res call({
 ConflictDto conflict
});




}
/// @nodoc
class _$PlayQuestionDto_ConflictCopyWithImpl<$Res>
    implements $PlayQuestionDto_ConflictCopyWith<$Res> {
  _$PlayQuestionDto_ConflictCopyWithImpl(this._self, this._then);

  final PlayQuestionDto_Conflict _self;
  final $Res Function(PlayQuestionDto_Conflict) _then;

/// Create a copy of PlayQuestionDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? conflict = null,}) {
  return _then(PlayQuestionDto_Conflict(
conflict: null == conflict ? _self.conflict : conflict // ignore: cast_nullable_to_non_nullable
as ConflictDto,
  ));
}


}

// dart format on
