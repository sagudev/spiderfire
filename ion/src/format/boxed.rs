/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use mozjs::jsapi::{ESClass, Unbox};

use crate::{Context, Object, Value};
use crate::format::Config;
use crate::format::primitive::format_primitive;

/// Formats a boxed primitive ([Object]) as a [String] using the given [Config].
/// The supported boxed types are `Boolean`, `Number`, `String` and `BigInt`.
///
/// ### Unimplemented
/// - Support for `BigInt`
pub fn format_boxed(cx: &Context, cfg: Config, object: &Object, class: ESClass) -> String {
	let mut unboxed = Value::object(cx, &Object::new(cx));

	unsafe {
		if Unbox(**cx, object.handle().into(), unboxed.handle_mut().into()) {
			use ESClass::*;
			match class {
				Boolean | Number | String => format_primitive(cx, cfg, &unboxed),
				BigInt => format!("Unimplemented Formatting: {}", "BigInt"),
				_ => unreachable!(),
			}
		} else {
			String::from("Boxed Value")
		}
	}
}
