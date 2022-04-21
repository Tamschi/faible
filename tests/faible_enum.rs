use faible::{faible, Descriptor, VariantFieldAccessMut, VariantFieldAccessRef, VariantFilter};

const STRUCTURED: &str = "structured";

#[faible(
	ValueDescriptor::new(),
	nested_names = "lowerCamelCase",
	no_weak_conversions
)]
pub enum Value {
	#[faible(_, name = "_null")]
	Null,

	#[faible(_, name = "_none")]
	None(),

	#[faible(_, name = "_bool")]
	Bool(#[faible(_, name = index)] bool),

	#[faible(_, name = 0, names = index)]
	Pointer(*mut ()),

	#[faible(_, name = _STRUCTURED)]
	Structured { a: u8, b: u16 },
}

#[faible(ValueDescriptor::new(), no_weak_conversions, names = discriminant)]
pub enum Discriminated {
	A = 1,
	B = 2,
	C = 3,
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

impl<'a, Strong: 'a, E, N> VariantFilter<'a, Strong, E, N> for ValueDescriptor {
	type CommonRef = &'a Strong;
	type CommonMut = Vec<&'a mut ()>;

	fn common(&self, strong: &'a Strong, name: N) -> Result<Option<Self::CommonRef>, E> {
		Ok(Some(strong))
	}

	fn check_mut(&self, strong: &mut Strong, name: N) -> Result<bool, E> {
		unimplemented!()
	}

	fn common_mut(&self, strong: &'a mut Strong, name: N) -> Result<Self::CommonMut, E> {
		unimplemented!()
	}
}

impl<'a, Common: 'a + ?Sized, E, T: ?Sized, N> VariantFieldAccessRef<'a, Common, E, T, N>
	for ValueDescriptor
{
	fn get(&self, common: &Common, name: N) -> Result<&'a T, E> {
		unimplemented!()
	}
}
impl<'a, Common: 'a + ?Sized, E, T: ?Sized, N> VariantFieldAccessMut<'a, Common, E, T, N>
	for ValueDescriptor
{
	fn get_mut(&self, common: &mut Common, name: N) -> Result<&'a mut T, E> {
		unimplemented!()
	}
}

pub struct Error;
impl faible::Error for Error {
	fn no_variant_recognized() -> Self {
		unimplemented!()
	}
}
