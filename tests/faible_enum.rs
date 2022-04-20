use faible::{faible, Descriptor, VariantFilter};

const STRUCTURED: &str = "structured";

#[faible(ValueDescriptor::new(), no_weak_conversions)]
pub enum Value {
	#[faible(_, name = "_null")]
	Null,

	#[faible(_, name = "_bool")]
	Bool(bool),

	#[faible(_, name = 0, names = "lowerCamelCase")]
	Pointer(*mut ()),

	#[faible(_, name = _STRUCTURED, names = "lowerCamelCase")]
	Structured { a: u8, b: u16 },
}

#[faible(ValueDescriptor::new(), no_weak_conversions)]
pub enum Empty {}

pub struct ValueDescriptor;
impl ValueDescriptor {
	pub fn new() -> Self {
		Self
	}
}

impl Default for ValueDescriptor {
	fn default() -> Self {
		Self::new()
	}
}

impl Descriptor for ValueDescriptor {
	type Weak = ();
	type Strong = ();
	type Error = Error;

	fn strong<'a>(&self, weak: &'a Self::Weak) -> Result<&'a Self::Strong, Self::Error> {
		Ok(weak)
	}

	fn strong_mut<'a>(
		&self,
		weak: &'a mut Self::Weak,
	) -> Result<&'a mut Self::Strong, Self::Error> {
		Ok(weak)
	}

	fn strong_into_weak(&self, strong: Self::Strong) -> Self::Weak {
		strong
	}

	fn try_weak_into_strong(&self, weak: Self::Weak) -> Result<Self::Strong, Self::Error> {
		Ok(weak)
	}
}

impl<Strong, E, N> VariantFilter<Strong, E, N> for ValueDescriptor {
	fn predicate(&self, _strong: &Strong, _name: N) -> Result<bool, E> {
		unimplemented!()
	}
}

pub struct Error;
impl faible::Error for Error {
	fn no_variant_recognized() -> Self {
		unimplemented!()
	}
}
