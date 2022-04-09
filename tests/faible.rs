use faible::{faible, Descriptor, FieldAccess, View};
use serde_json::{map::Entry, Map, Number, Value};
use std::mem;
use tap::Pipe;

#[faible(JsonObjectDescriptor, faible = ::faible, names = "lowerCamelCase")]
pub struct MapInfo {
	pub id: MapId,
	pub expanded: BoolValue,
	pub name: StringValue,
	pub order: NumberValue,
	pub parent_id: MapId,
	pub scroll_x: NumberValue,
	pub scroll_y: NumberValue,
}

pub struct JsonObjectDescriptor;
impl Descriptor for JsonObjectDescriptor {
	type Weak = Value;
	type Strong = Map<String, Value>;

	fn strong<'a>(&self, this: &'a Self::Weak) -> faible::Result<&'a Self::Strong> {
		match this {
			Value::Object(strong) => Ok(strong),
			_ => todo!(),
		}
	}

	fn strong_mut<'a>(&self, this: &'a mut Self::Weak) -> faible::Result<&'a mut Self::Strong> {
		match this {
			Value::Object(strong) => Ok(strong),
			_ => todo!(),
		}
	}
}

impl<T: View<Value>> FieldAccess<<JsonObjectDescriptor as Descriptor>::Strong, T, &'static str>
	for JsonObjectDescriptor
{
	fn get<'a>(
		&self,
		this: &'a <JsonObjectDescriptor as Descriptor>::Strong,
		name: &'static str,
	) -> faible::Result<&'a T> {
		this.get(name).map(T::from_ref).ok_or_else(|| todo!())
	}

	fn get_mut<'a>(
		&self,
		this: &'a mut <JsonObjectDescriptor as Descriptor>::Strong,
		name: &'static str,
	) -> faible::Result<&'a mut T> {
		this.get_mut(name).map(T::from_mut).ok_or_else(|| todo!())
	}

	fn set(
		&self,
		this: &mut <JsonObjectDescriptor as Descriptor>::Strong,
		name: &'static str,
		value: T,
	) -> faible::Result<()>
	where
		T: Sized,
	{
		let value = value.into_inner();
		match this.entry(name) {
			Entry::Vacant(vacant) => vacant.insert(value).pipe(drop),
			Entry::Occupied(occupied) => *occupied.into_mut() = value,
		}
		Ok(())
	}

	fn insert<'a>(
		&self,
		this: &'a mut <JsonObjectDescriptor as Descriptor>::Strong,
		name: &'static str,
		value: T,
	) -> faible::Result<(&'a mut T, Option<T>)>
	where
		T: Sized,
	{
		let value = value.into_inner();
		match this.entry(name) {
			Entry::Vacant(vacant) => (vacant.insert(value), None),
			Entry::Occupied(occupied) => {
				let slot = occupied.into_mut();
				let prev = mem::replace(slot, value);
				(slot, Some(prev))
			}
		}
		.pipe(T::from_insertion)
		.pipe(Ok)
	}
}

#[faible(JsonNumberDescriptor)]
pub struct MapId;
#[faible(JsonBoolDescriptor)]
pub struct BoolValue;
#[faible(JsonStringDescriptor)]
pub struct StringValue;
#[faible(JsonNumberDescriptor)]
pub struct NumberValue;

pub struct JsonNumberDescriptor;
impl Descriptor for JsonNumberDescriptor {
	type Weak = Value;
	type Strong = Number;

	fn strong<'a>(&self, this: &'a Self::Weak) -> faible::Result<&'a Self::Strong> {
		match this {
			Value::Number(number) => Ok(number),
			_ => todo!(),
		}
	}

	fn strong_mut<'a>(&self, this: &'a mut Self::Weak) -> faible::Result<&'a mut Self::Strong> {
		match this {
			Value::Number(number) => Ok(number),
			_ => todo!(),
		}
	}
}

pub struct JsonBoolDescriptor;
impl Descriptor for JsonBoolDescriptor {
	type Weak = Value;
	type Strong = bool;

	fn strong<'a>(&self, this: &'a Self::Weak) -> faible::Result<&'a Self::Strong> {
		match this {
			Value::Bool(bool) => Ok(bool),
			_ => todo!(),
		}
	}

	fn strong_mut<'a>(&self, this: &'a mut Self::Weak) -> faible::Result<&'a mut Self::Strong> {
		match this {
			Value::Bool(bool) => Ok(bool),
			_ => todo!(),
		}
	}
}

pub struct JsonStringDescriptor;
impl Descriptor for JsonStringDescriptor {
	type Weak = Value;
	type Strong = String;

	fn strong<'a>(&self, this: &'a Self::Weak) -> faible::Result<&'a Self::Strong> {
		match this {
			Value::String(string) => Ok(string),
			_ => todo!(),
		}
	}

	fn strong_mut<'a>(&self, this: &'a mut Self::Weak) -> faible::Result<&'a mut Self::Strong> {
		match this {
			Value::String(string) => Ok(string),
			_ => todo!(),
		}
	}
}
