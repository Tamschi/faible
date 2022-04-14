use faible::{faible, Descriptor};

#[faible(ValueDescriptor::new(), no_weak_conversions)]
pub enum Value {
	#[faible(_, name = "null")]
	Null,

	#[faible(_, name = "bool")]
	Bool(bool),

	#[faible(_, names = "lowerCamelCase")]
	Pointer(*mut ()),

	#[faible(_, names = "lowerCamelCase")]
	Structured { a: u8, b: u16 },
}

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
	type Error = ();

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
