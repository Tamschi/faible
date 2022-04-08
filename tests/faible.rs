use faible::{Descriptor, FieldAccess, View};
use serde_json::{map::Entry, Map, Number, Value};
use tap::Pipe;

// #[faible(JsonObjectDescriptor, names = "camelCase")]
pub struct MapInfo {
	id: MapId,
}

struct JsonObjectDescriptor;
impl Descriptor for JsonObjectDescriptor {
	type Weak = Value;
	type Strong = Map<String, Value>;

	fn strong(this: &Self::Weak) -> &Self::Strong {
		match this {
			Value::Object(strong) => strong,
			_ => todo!(),
		}
	}

	fn strong_mut(this: &mut Self::Weak) -> &mut Self::Strong {
		match this {
			Value::Object(strong) => strong,
			_ => todo!(),
		}
	}
}

impl<T: View<Value>> FieldAccess<<JsonObjectDescriptor as Descriptor>::Strong, T, &'static str>
	for JsonObjectDescriptor
{
	fn get<'a>(
		this: &'a <JsonObjectDescriptor as Descriptor>::Strong,
		name: &'static str,
	) -> faible::Result<&'a T> {
		this.get(name).map(T::from_ref).ok_or_else(|| todo!())
	}

	fn get_mut<'a>(
		this: &'a mut <JsonObjectDescriptor as Descriptor>::Strong,
		name: &'static str,
	) -> faible::Result<&'a mut T> {
		this.get_mut(name).map(T::from_mut).ok_or_else(|| todo!())
	}

	fn set(
		this: &mut <JsonObjectDescriptor as Descriptor>::Strong,
		name: &'static str,
		value: T,
	) -> faible::Result<()>
	where
		T: Sized,
	{
		this[name] = value.into_inner();
		Ok(())
	}

	fn insert<'a>(
		this: &'a mut <JsonObjectDescriptor as Descriptor>::Strong,
		name: &'static str,
		value: T,
	) -> faible::Result<&'a mut T>
	where
		T: Sized,
	{
		let value = value.into_inner();
		match this.entry(name) {
			Entry::Vacant(vacant) => vacant.insert(value),
			Entry::Occupied(occupied) => {
				let slot = occupied.into_mut();
				*slot = value;
				slot
			}
		}
		.pipe(T::from_mut)
		.pipe(Ok)
	}

	fn replace(
		this: &mut <JsonObjectDescriptor as Descriptor>::Strong,
		name: &'static str,
		value: T,
	) -> faible::Result<Option<T>>
	where
		T: Sized,
	{
		let value = value.into_inner();
		match this.entry(name) {
			Entry::Vacant(vacant) => {
				vacant.insert(value);
				None
			}
			Entry::Occupied(mut occupied) => occupied.insert(value).pipe(Some),
		}
		.map(T::from)
		.pipe(Ok)
	}
}

// #[faible(MapIdDescriptor)]
pub struct MapId;
