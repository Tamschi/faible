use faible::{faible, Descriptor, Error, FieldAccess, View};
use serde_json::{map::Entry, Map, Number, Value};
use std::mem;
use tap::Pipe;

#[faible(JsonObjectDescriptor("MapInfo"), faible = ::faible, names = "lowerCamelCase")]
pub struct MapInfo {
	pub id: MapId,
	pub expanded: BoolValue,
	pub name: StringValue,
	pub order: NumberValue,
	pub parent_id: MapId,
	pub scroll_x: NumberValue,
	pub scroll_y: NumberValue,
}

pub struct JsonObjectDescriptor(&'static str);
impl Descriptor for JsonObjectDescriptor {
	type Weak = Value;
	type Strong = Map<String, Value>;
	type Error = Error;

	fn strong<'a>(&self, this: &'a Self::Weak) -> Result<&'a Self::Strong, Self::Error> {
		match this {
			Value::Object(strong) => Ok(strong),
			this => Err(Error::new(format!(
				"{} is expected to be an object, but was {:?}.",
				self.0, &this
			))),
		}
	}

	fn strong_mut<'a>(
		&self,
		this: &'a mut Self::Weak,
	) -> Result<&'a mut Self::Strong, Self::Error> {
		match this {
			Value::Object(strong) => Ok(strong),
			this => Err(Error::new(format!(
				"{} is expected to be an object, but was {:?}.",
				self.0, &this
			))),
		}
	}

	fn strong_into_weak(&self, strong: Self::Strong) -> Self::Weak {
		Value::Object(strong)
	}

	fn try_weak_into_strong(&self, weak: Self::Weak) -> Result<Self::Strong, Self::Error> {
		match weak {
			Value::Object(strong) => Ok(strong),
			weak => Err(Error::new(format!(
				"{} is expected to be an object, but was {:?}.",
				self.0, weak
			))),
		}
	}
}

impl<T: View<Value>>
	FieldAccess<<JsonObjectDescriptor as Descriptor>::Strong, Error, T, &'static str>
	for JsonObjectDescriptor
{
	fn get<'a>(
		&self,
		this: &'a <JsonObjectDescriptor as Descriptor>::Strong,
		name: &'static str,
	) -> Result<&'a T, Error> {
		this.get(name)
			.map(T::from_ref)
			.ok_or_else(|| Error::new(format!("Expected field {name} in {}.", self.0)))
	}

	fn get_mut<'a>(
		&self,
		this: &'a mut <JsonObjectDescriptor as Descriptor>::Strong,
		name: &'static str,
	) -> Result<&'a mut T, Error> {
		this.get_mut(name)
			.map(T::from_mut)
			.ok_or_else(|| Error::new(format!("Expected field {name} in {}.", self.0)))
	}

	fn set(
		&self,
		this: &mut <JsonObjectDescriptor as Descriptor>::Strong,
		name: &'static str,
		value: T,
	) -> Result<(), Error>
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
	) -> Result<(&'a mut T, Option<T>), Error>
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

#[faible(JsonNumberDescriptor("MapId"))]
pub struct MapId;
#[faible(JsonBoolDescriptor("BoolValue"))]
pub struct BoolValue;
#[faible(JsonStringDescriptor("StringValue"))]
pub struct StringValue;
#[faible(JsonNumberDescriptor("NumberValue"))]
pub struct NumberValue;

pub struct JsonNumberDescriptor(&'static str);
impl Descriptor for JsonNumberDescriptor {
	type Weak = Value;
	type Strong = Number;
	type Error = Error;

	fn strong<'a>(&self, weak: &'a Self::Weak) -> Result<&'a Self::Strong, Self::Error> {
		match weak {
			Value::Number(number) => Ok(number),
			weak => Err(Error::new(format!(
				"{} is expected to be a number, but was {:?}.",
				self.0, &weak
			))),
		}
	}

	fn strong_mut<'a>(
		&self,
		weak: &'a mut Self::Weak,
	) -> Result<&'a mut Self::Strong, Self::Error> {
		match weak {
			Value::Number(number) => Ok(number),
			weak => Err(Error::new(format!(
				"{} is expected to be a number, but was {:?}.",
				self.0, &weak
			))),
		}
	}

	fn strong_into_weak(&self, strong: Self::Strong) -> Self::Weak {
		Value::Number(strong)
	}

	fn try_weak_into_strong(&self, weak: Self::Weak) -> Result<Self::Strong, Self::Error> {
		match weak {
			Value::Number(strong) => Ok(strong),
			weak => Err(Error::new(format!(
				"{} is expected to be a number, but was {:?}.",
				self.0, weak
			))),
		}
	}
}

pub struct JsonBoolDescriptor(&'static str);
impl Descriptor for JsonBoolDescriptor {
	type Weak = Value;
	type Strong = bool;
	type Error = Error;

	fn strong<'a>(&self, weak: &'a Self::Weak) -> Result<&'a Self::Strong, Self::Error> {
		match weak {
			Value::Bool(bool) => Ok(bool),
			weak => Err(Error::new(format!(
				"{} is expected to be a bool, but was {:?}.",
				self.0, &weak
			))),
		}
	}

	fn strong_mut<'a>(
		&self,
		weak: &'a mut Self::Weak,
	) -> Result<&'a mut Self::Strong, Self::Error> {
		match weak {
			Value::Bool(bool) => Ok(bool),
			weak => Err(Error::new(format!(
				"{} is expected to be an object, but was {:?}.",
				self.0, &weak
			))),
		}
	}

	fn strong_into_weak(&self, strong: Self::Strong) -> Self::Weak {
		Value::Bool(strong)
	}

	fn try_weak_into_strong(&self, weak: Self::Weak) -> Result<Self::Strong, Self::Error> {
		match weak {
			Value::Bool(strong) => Ok(strong),
			weak => Err(Error::new(format!(
				"{} is expected to be a bool, but was {:?}.",
				self.0, weak
			))),
		}
	}
}

pub struct JsonStringDescriptor(&'static str);
impl Descriptor for JsonStringDescriptor {
	type Weak = Value;
	type Strong = String;
	type Error = Error;

	fn strong<'a>(&self, weak: &'a Self::Weak) -> Result<&'a Self::Strong, Self::Error> {
		match weak {
			Value::String(string) => Ok(string),
			weak => Err(Error::new(format!(
				"{} is expected to be a string, but was {:?}.",
				self.0, &weak
			))),
		}
	}

	fn strong_mut<'a>(
		&self,
		weak: &'a mut Self::Weak,
	) -> Result<&'a mut Self::Strong, Self::Error> {
		match weak {
			Value::String(string) => Ok(string),
			weak => Err(Error::new(format!(
				"{} is expected to be a string, but was {:?}.",
				self.0, &weak
			))),
		}
	}

	fn strong_into_weak(&self, strong: Self::Strong) -> Self::Weak {
		Value::String(strong)
	}

	fn try_weak_into_strong(&self, weak: Self::Weak) -> Result<Self::Strong, Self::Error> {
		match weak {
			Value::String(strong) => Ok(strong),
			weak => Err(Error::new(format!(
				"{} is expected to be a string, but was {:?}.",
				self.0, weak
			))),
		}
	}
}
