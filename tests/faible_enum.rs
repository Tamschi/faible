use faible::faible;

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
