/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::mem::zeroed;

use mozjs::conversions::{ConversionResult, FromJSValConvertible, jsstr_to_string};
pub use mozjs::conversions::ConversionBehavior;
use mozjs::jsapi::{
	AssertSameCompartment, AssertSameCompartment1, ForOfIterator, ForOfIterator_NonIterableBehavior, JSFunction, JSObject, JSString, RootedObject,
	RootedValue,
};
use mozjs::jsval::{JSVal, UndefinedValue};
use mozjs::rust::{ToBoolean, ToNumber, ToString};

use crate::{Array, Context, Date, Error, ErrorKind, Function, Object, Promise, Result, Value};

pub trait FromValue<'cx>: Sized {
	type Config;
	unsafe fn from_value(cx: &'cx Context, value: &Value<'cx>, strict: bool, config: Self::Config) -> Result<Self>;
}

impl<'cx> FromValue<'cx> for bool {
	type Config = ();

	unsafe fn from_value(_: &'cx Context, value: &Value<'cx>, strict: bool, _: ()) -> Result<bool> {
		if value.is_boolean() {
			return Ok(value.to_boolean());
		}

		if strict {
			Err(Error::new("Expected Boolean in Strict Conversion", ErrorKind::Type))
		} else {
			Ok(ToBoolean(value.handle()))
		}
	}
}

macro_rules! impl_from_value_for_integer {
	($ty:ty) => {
		impl<'cx> FromValue<'cx> for $ty {
			type Config = ConversionBehavior;

			unsafe fn from_value(cx: &'cx Context, value: &Value<'cx>, strict: bool, config: ConversionBehavior) -> Result<$ty> {
				if strict && !value.is_number() {
					return Err(Error::new("Expected Number in Strict Conversion", ErrorKind::Type));
				}

				match <$ty>::from_jsval(**cx, value.handle(), config) {
					Ok(ConversionResult::Success(number)) => Ok(number),
					Err(_) => Err(Error::none()),
					_ => unreachable!(),
				}
			}
		}
	};
}

impl_from_value_for_integer!(u8);
impl_from_value_for_integer!(u16);
impl_from_value_for_integer!(u32);
impl_from_value_for_integer!(u64);

impl_from_value_for_integer!(i8);
impl_from_value_for_integer!(i16);
impl_from_value_for_integer!(i32);
impl_from_value_for_integer!(i64);

impl<'cx> FromValue<'cx> for f32 {
	type Config = ();

	unsafe fn from_value(cx: &'cx Context, value: &Value<'cx>, strict: bool, _: ()) -> Result<f32> {
		f64::from_value(cx, value, strict, ()).map(|float| float as f32)
	}
}

impl<'cx> FromValue<'cx> for f64 {
	type Config = ();

	unsafe fn from_value(cx: &'cx Context, value: &Value<'cx>, strict: bool, _: ()) -> Result<f64> {
		if strict && !value.is_number() {
			return Err(Error::new("Expected Number in Strict Conversion", ErrorKind::Type));
		}

		ToNumber(**cx, value.handle()).map_err(|_| Error::new("Unable to Convert Value to Number", ErrorKind::Type))
	}
}

impl<'cx> FromValue<'cx> for *mut JSString {
	type Config = ();

	unsafe fn from_value(cx: &'cx Context, value: &Value<'cx>, strict: bool, _: ()) -> Result<*mut JSString> {
		if strict && !value.is_string() {
			return Err(Error::new("Expected String in Strict Conversion", ErrorKind::Type));
		}
		Ok(ToString(**cx, value.handle()))
	}
}

impl<'cx> FromValue<'cx> for String {
	type Config = ();

	unsafe fn from_value(cx: &'cx Context, value: &Value<'cx>, strict: bool, _: ()) -> Result<String> {
		if strict && !value.is_string() {
			return Err(Error::new("Expected String in Strict Conversion", ErrorKind::Type));
		}

		Ok(jsstr_to_string(**cx, ToString(**cx, value.handle())))
	}
}

impl<'cx> FromValue<'cx> for *mut JSObject {
	type Config = ();

	unsafe fn from_value(cx: &'cx Context, value: &Value<'cx>, _: bool, _: ()) -> Result<*mut JSObject> {
		if !value.is_object() {
			return Err(Error::new("Expected Object", ErrorKind::Type));
		}
		let object = (**value).to_object();
		AssertSameCompartment(**cx, object);

		Ok(object)
	}
}

impl<'cx> FromValue<'cx> for Object<'cx> {
	type Config = ();

	unsafe fn from_value(cx: &'cx Context, value: &Value<'cx>, _: bool, _: ()) -> Result<Object<'cx>> {
		if !value.is_object() {
			return Err(Error::new("Expected Object", ErrorKind::Type));
		}
		let object = value.to_object(cx);
		AssertSameCompartment(**cx, **object);

		Ok(object)
	}
}

impl<'cx> FromValue<'cx> for Array<'cx> {
	type Config = ();

	unsafe fn from_value(cx: &'cx Context, value: &Value<'cx>, _: bool, _: ()) -> Result<Array<'cx>> {
		if !value.is_object() {
			return Err(Error::new("Expected Array", ErrorKind::Type));
		}

		let object = value.to_object(cx).into_local();
		if let Some(array) = Array::from(cx, object) {
			AssertSameCompartment(**cx, **array);
			Ok(array)
		} else {
			Err(Error::new("Expected Array", ErrorKind::Type))
		}
	}
}

impl<'cx> FromValue<'cx> for Date<'cx> {
	type Config = ();

	unsafe fn from_value(cx: &'cx Context, value: &Value<'cx>, _: bool, _: ()) -> Result<Date<'cx>> {
		if !value.is_object() {
			return Err(Error::new("Expected Date", ErrorKind::Type));
		}

		let object = value.to_object(cx).into_local();
		if let Some(date) = Date::from(cx, object) {
			AssertSameCompartment(**cx, **date);
			Ok(date)
		} else {
			Err(Error::new("Expected Date", ErrorKind::Type))
		}
	}
}

impl<'cx> FromValue<'cx> for Promise<'cx> {
	type Config = ();

	unsafe fn from_value(cx: &'cx Context, value: &Value<'cx>, _: bool, _: ()) -> Result<Promise<'cx>> {
		if !value.is_object() {
			return Err(Error::new("Expected Promise", ErrorKind::Type));
		}

		let object = value.to_object(cx).into_local();
		if let Some(promise) = Promise::from(object) {
			AssertSameCompartment(**cx, **promise);
			Ok(promise)
		} else {
			Err(Error::new("Expected Promise", ErrorKind::Type))
		}
	}
}

impl<'cx> FromValue<'cx> for *mut JSFunction {
	type Config = ();

	unsafe fn from_value(cx: &'cx Context, value: &Value<'cx>, _: bool, _: ()) -> Result<*mut JSFunction> {
		if !value.is_object() {
			return Err(Error::new("Expected Function", ErrorKind::Type));
		}

		let function_obj = value.to_object(cx);
		if let Some(function) = Function::from_object(cx, &*function_obj) {
			AssertSameCompartment(**cx, **function_obj);
			Ok(**function)
		} else {
			Err(Error::new("Expected Function", ErrorKind::Type))
		}
	}
}

impl<'cx> FromValue<'cx> for Function<'cx> {
	type Config = ();

	unsafe fn from_value(cx: &'cx Context, value: &Value<'cx>, _: bool, _: ()) -> Result<Function<'cx>> {
		if !value.is_object() {
			return Err(Error::new("Expected Function", ErrorKind::Type));
		}

		let function_obj = value.to_object(cx);
		if let Some(function) = Function::from_object(cx, &*function_obj) {
			AssertSameCompartment(**cx, **function_obj);
			Ok(function)
		} else {
			Err(Error::new("Expected Function", ErrorKind::Type))
		}
	}
}

impl<'cx> FromValue<'cx> for JSVal {
	type Config = ();

	unsafe fn from_value(cx: &'cx Context, value: &Value<'cx>, _: bool, _: ()) -> Result<JSVal> {
		AssertSameCompartment1(**cx, value.handle().into());
		Ok(***value)
	}
}

impl<'cx> FromValue<'cx> for Value<'cx> {
	type Config = ();

	unsafe fn from_value(cx: &'cx Context, value: &Value<'cx>, _: bool, _: ()) -> Result<Value<'cx>> {
		AssertSameCompartment1(**cx, value.handle().into());
		Ok(cx.root_value(***value).into())
	}
}

impl<'cx, T: FromValue<'cx>> FromValue<'cx> for Option<T> {
	type Config = T::Config;

	unsafe fn from_value(cx: &'cx Context, value: &Value<'cx>, strict: bool, config: T::Config) -> Result<Option<T>> {
		if value.is_null_or_undefined() {
			Ok(None)
		} else {
			Ok(Some(T::from_value(cx, value, strict, config)?))
		}
	}
}

// Copied from [rust-mozjs](https://github.com/servo/rust-mozjs/blob/master/src/conversions.rs#L619-L642)
struct ForOfIteratorGuard<'a> {
	root: &'a mut ForOfIterator,
}

impl<'a> ForOfIteratorGuard<'a> {
	fn new(cx: &Context, root: &'a mut ForOfIterator) -> Self {
		unsafe {
			root.iterator.add_to_root_stack(**cx);
		}
		ForOfIteratorGuard { root }
	}
}

impl<'a> Drop for ForOfIteratorGuard<'a> {
	fn drop(&mut self) {
		unsafe {
			self.root.iterator.remove_from_root_stack();
		}
	}
}

impl<'cx, T: FromValue<'cx>> FromValue<'cx> for Vec<T>
where
	T::Config: Clone,
{
	type Config = T::Config;

	// Adapted from [rust-mozjs](https://github.com/servo/rust-mozjs/blob/master/src/conversions.rs#L644-L707)
	unsafe fn from_value(cx: &'cx Context, value: &Value<'cx>, strict: bool, config: T::Config) -> Result<Vec<T>> {
		if !value.is_object() {
			return Err(Error::new("Expected Object", ErrorKind::Type));
		}
		let object = value.to_object(cx);
		if strict && !Array::is_array(cx, &*object) {
			return Err(Error::new("Expected Array", ErrorKind::Type));
		}

		let zero = zeroed();
		let mut iterator = ForOfIterator {
			cx_: **cx,
			iterator: RootedObject::new_unrooted(),
			nextMethod: RootedValue::new_unrooted(),
			index: u32::MAX, // NOT_ARRAY
			..zero
		};
		let iterator = ForOfIteratorGuard::new(cx, &mut iterator);
		let iterator = &mut *iterator.root;

		if !iterator.init(value.handle().into(), ForOfIterator_NonIterableBehavior::AllowNonIterable) {
			return Err(Error::new("Failed to Initialise Iterator", ErrorKind::Type));
		}

		if iterator.iterator.ptr.is_null() {
			return Err(Error::new("Expected Iterable", ErrorKind::Type));
		}

		let mut ret = vec![];

		let mut value = Value::from(cx.root_value(UndefinedValue()));
		loop {
			let mut done = false;
			if !iterator.next(value.handle_mut().into(), &mut done) {
				return Err(Error::new("Failed to Execute Next on Iterator", ErrorKind::Type));
			}

			if done {
				break;
			}
			ret.push(T::from_value(cx, &value, strict, config.clone())?);
		}
		Ok(ret)
	}
}
