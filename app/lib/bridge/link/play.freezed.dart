// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'play.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PlayUpdate {

 Object get field0;



@override
bool operator ==(Object other) {
  final _this = this as PlayUpdate;
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayUpdate&&const DeepCollectionEquality().equals(other.field0, _this.field0));
}


@override
int get hashCode {
  final _this = this as PlayUpdate;
  return Object.hash(runtimeType,const DeepCollectionEquality().hash(_this.field0));
}

@override
String toString() {
  final _this = this as PlayUpdate;
  return 'PlayUpdate(field0: ${_this.field0})';
}


}

/// @nodoc
class $PlayUpdateCopyWith<$Res>  {
$PlayUpdateCopyWith(PlayUpdate _, $Res Function(PlayUpdate) __);
}


/// Adds pattern-matching-related methods to [PlayUpdate].
extension PlayUpdatePatterns on PlayUpdate {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PlayUpdate_Step value)?  step,TResult Function( PlayUpdate_Question value)?  question,TResult Function( PlayUpdate_Done value)?  done,TResult Function( PlayUpdate_Failed value)?  failed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PlayUpdate_Step() when step != null:
return step(_that);case PlayUpdate_Question() when question != null:
return question(_that);case PlayUpdate_Done() when done != null:
return done(_that);case PlayUpdate_Failed() when failed != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PlayUpdate_Step value)  step,required TResult Function( PlayUpdate_Question value)  question,required TResult Function( PlayUpdate_Done value)  done,required TResult Function( PlayUpdate_Failed value)  failed,}){
final _that = this;
switch (_that) {
case PlayUpdate_Step():
return step(_that);case PlayUpdate_Question():
return question(_that);case PlayUpdate_Done():
return done(_that);case PlayUpdate_Failed():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PlayUpdate_Step value)?  step,TResult? Function( PlayUpdate_Question value)?  question,TResult? Function( PlayUpdate_Done value)?  done,TResult? Function( PlayUpdate_Failed value)?  failed,}){
final _that = this;
switch (_that) {
case PlayUpdate_Step() when step != null:
return step(_that);case PlayUpdate_Question() when question != null:
return question(_that);case PlayUpdate_Done() when done != null:
return done(_that);case PlayUpdate_Failed() when failed != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( PlayStepDto field0)?  step,TResult Function( PlayQuestionDto field0)?  question,TResult Function( PlayResultDto field0)?  done,TResult Function( ConnectionState field0)?  failed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PlayUpdate_Step() when step != null:
return step(_that.field0);case PlayUpdate_Question() when question != null:
return question(_that.field0);case PlayUpdate_Done() when done != null:
return done(_that.field0);case PlayUpdate_Failed() when failed != null:
return failed(_that.field0);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( PlayStepDto field0)  step,required TResult Function( PlayQuestionDto field0)  question,required TResult Function( PlayResultDto field0)  done,required TResult Function( ConnectionState field0)  failed,}) {final _that = this;
switch (_that) {
case PlayUpdate_Step():
return step(_that.field0);case PlayUpdate_Question():
return question(_that.field0);case PlayUpdate_Done():
return done(_that.field0);case PlayUpdate_Failed():
return failed(_that.field0);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( PlayStepDto field0)?  step,TResult? Function( PlayQuestionDto field0)?  question,TResult? Function( PlayResultDto field0)?  done,TResult? Function( ConnectionState field0)?  failed,}) {final _that = this;
switch (_that) {
case PlayUpdate_Step() when step != null:
return step(_that.field0);case PlayUpdate_Question() when question != null:
return question(_that.field0);case PlayUpdate_Done() when done != null:
return done(_that.field0);case PlayUpdate_Failed() when failed != null:
return failed(_that.field0);case _:
  return null;

}
}

}

/// @nodoc


class PlayUpdate_Step extends PlayUpdate {
  const PlayUpdate_Step(this.field0): super._();
  

@override final  PlayStepDto field0;

/// Create a copy of PlayUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayUpdate_StepCopyWith<PlayUpdate_Step> get copyWith => _$PlayUpdate_StepCopyWithImpl<PlayUpdate_Step>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayUpdate_Step&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode {
    return Object.hash(runtimeType,field0);
}

@override
String toString() {
    return 'PlayUpdate.step(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $PlayUpdate_StepCopyWith<$Res> implements $PlayUpdateCopyWith<$Res> {
  factory $PlayUpdate_StepCopyWith(PlayUpdate_Step value, $Res Function(PlayUpdate_Step) _then) = _$PlayUpdate_StepCopyWithImpl;
@useResult
$Res call({
 PlayStepDto field0
});




}
/// @nodoc
class _$PlayUpdate_StepCopyWithImpl<$Res>
    implements $PlayUpdate_StepCopyWith<$Res> {
  _$PlayUpdate_StepCopyWithImpl(this._self, this._then);

  final PlayUpdate_Step _self;
  final $Res Function(PlayUpdate_Step) _then;

/// Create a copy of PlayUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(PlayUpdate_Step(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as PlayStepDto,
  ));
}


}

/// @nodoc


class PlayUpdate_Question extends PlayUpdate {
  const PlayUpdate_Question(this.field0): super._();
  

@override final  PlayQuestionDto field0;

/// Create a copy of PlayUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayUpdate_QuestionCopyWith<PlayUpdate_Question> get copyWith => _$PlayUpdate_QuestionCopyWithImpl<PlayUpdate_Question>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayUpdate_Question&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode {
    return Object.hash(runtimeType,field0);
}

@override
String toString() {
    return 'PlayUpdate.question(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $PlayUpdate_QuestionCopyWith<$Res> implements $PlayUpdateCopyWith<$Res> {
  factory $PlayUpdate_QuestionCopyWith(PlayUpdate_Question value, $Res Function(PlayUpdate_Question) _then) = _$PlayUpdate_QuestionCopyWithImpl;
@useResult
$Res call({
 PlayQuestionDto field0
});


$PlayQuestionDtoCopyWith<$Res> get field0;

}
/// @nodoc
class _$PlayUpdate_QuestionCopyWithImpl<$Res>
    implements $PlayUpdate_QuestionCopyWith<$Res> {
  _$PlayUpdate_QuestionCopyWithImpl(this._self, this._then);

  final PlayUpdate_Question _self;
  final $Res Function(PlayUpdate_Question) _then;

/// Create a copy of PlayUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(PlayUpdate_Question(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as PlayQuestionDto,
  ));
}

/// Create a copy of PlayUpdate
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$PlayQuestionDtoCopyWith<$Res> get field0 {
  
  return $PlayQuestionDtoCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

/// @nodoc


class PlayUpdate_Done extends PlayUpdate {
  const PlayUpdate_Done(this.field0): super._();
  

@override final  PlayResultDto field0;

/// Create a copy of PlayUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayUpdate_DoneCopyWith<PlayUpdate_Done> get copyWith => _$PlayUpdate_DoneCopyWithImpl<PlayUpdate_Done>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayUpdate_Done&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode {
    return Object.hash(runtimeType,field0);
}

@override
String toString() {
    return 'PlayUpdate.done(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $PlayUpdate_DoneCopyWith<$Res> implements $PlayUpdateCopyWith<$Res> {
  factory $PlayUpdate_DoneCopyWith(PlayUpdate_Done value, $Res Function(PlayUpdate_Done) _then) = _$PlayUpdate_DoneCopyWithImpl;
@useResult
$Res call({
 PlayResultDto field0
});




}
/// @nodoc
class _$PlayUpdate_DoneCopyWithImpl<$Res>
    implements $PlayUpdate_DoneCopyWith<$Res> {
  _$PlayUpdate_DoneCopyWithImpl(this._self, this._then);

  final PlayUpdate_Done _self;
  final $Res Function(PlayUpdate_Done) _then;

/// Create a copy of PlayUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(PlayUpdate_Done(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as PlayResultDto,
  ));
}


}

/// @nodoc


class PlayUpdate_Failed extends PlayUpdate {
  const PlayUpdate_Failed(this.field0): super._();
  

@override final  ConnectionState field0;

/// Create a copy of PlayUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayUpdate_FailedCopyWith<PlayUpdate_Failed> get copyWith => _$PlayUpdate_FailedCopyWithImpl<PlayUpdate_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayUpdate_Failed&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode {
    return Object.hash(runtimeType,field0);
}

@override
String toString() {
    return 'PlayUpdate.failed(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $PlayUpdate_FailedCopyWith<$Res> implements $PlayUpdateCopyWith<$Res> {
  factory $PlayUpdate_FailedCopyWith(PlayUpdate_Failed value, $Res Function(PlayUpdate_Failed) _then) = _$PlayUpdate_FailedCopyWithImpl;
@useResult
$Res call({
 ConnectionState field0
});


$ConnectionStateCopyWith<$Res> get field0;

}
/// @nodoc
class _$PlayUpdate_FailedCopyWithImpl<$Res>
    implements $PlayUpdate_FailedCopyWith<$Res> {
  _$PlayUpdate_FailedCopyWithImpl(this._self, this._then);

  final PlayUpdate_Failed _self;
  final $Res Function(PlayUpdate_Failed) _then;

/// Create a copy of PlayUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(PlayUpdate_Failed(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as ConnectionState,
  ));
}

/// Create a copy of PlayUpdate
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ConnectionStateCopyWith<$Res> get field0 {
  
  return $ConnectionStateCopyWith<$Res>(_self.field0, (value) {
    return _then(_self.copyWith(field0: value));
  });
}
}

// dart format on
