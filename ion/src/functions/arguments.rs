/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::fmt::{Debug, Formatter};

use mozjs::jsapi::CallArgs;
use mozjs::jsval::JSVal;
use mozjs::rust::MutableHandle;

use crate::{Context, Value};

/// Function Arguments
pub struct Arguments<'cx> {
	values: Vec<Value<'cx>>,
	this: Value<'cx>,
	rval: MutableHandle<'cx, JSVal>,
	call_args: CallArgs,
}

impl Debug for Arguments<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Arguments")
			.field("values", &self.values)
			.field("this", &self.this)
			.field("rval", &self.rval.get())
			.field("call_args", &self.call_args)
			.finish()
	}
}

impl<'cx> Arguments<'cx> {
	/// Creates new [Arguments] from raw argument,
	pub fn new(cx: &'cx Context, argc: u32, vp: *mut JSVal) -> Arguments<'cx> {
		unsafe {
			let call_args = CallArgs::from_vp(vp, argc);
			let values = (0..argc).map(|i| cx.root_value(call_args.get(i).get()).into()).collect();
			let this = cx.root_value(call_args.thisv().get()).into();
			let rval = MutableHandle::from_raw(call_args.rval());

			Arguments { values, this, rval, call_args }
		}
	}

	/// Returns the number of arguments passed to the function.
	#[allow(clippy::len_without_is_empty)]
	pub fn len(&self) -> usize {
		self.values.len()
	}

	/// Gets the handle of the value at the given index.
	/// Returns [None] if the given index is larger than the number of arguments.
	pub fn value(&self, index: usize) -> Option<&Value<'cx>> {
		if index < self.len() {
			return Some(&self.values[index]);
		}
		None
	}

	/// Returns a range of handles within the arguments.
	pub fn range<'a: 'cx, R: Iterator<Item = usize>>(&'a self, range: R) -> Vec<&'a Value<'cx>> {
		range.filter_map(|index| self.value(index)).collect()
	}

	/// Returns the `this` value of the function.
	pub fn this(&mut self) -> &mut Value<'cx> {
		&mut self.this
	}

	/// Returns the mutable return value of the function.
	pub fn rval(&mut self) -> MutableHandle<JSVal> {
		self.rval
	}

	/// Returns true if the function was called with `new`,
	pub fn is_constructing(&self) -> bool {
		self.call_args.constructing_()
	}

	pub fn call_args(&self) -> CallArgs {
		self.call_args
	}
}
