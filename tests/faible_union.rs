use faible::{faible, Descriptor, UnionFieldAccess};
use std::{convert::Infallible, marker::PhantomData, mem, ptr::NonNull};

#[faible(NullableDescriptor::<T>::new(), names = "_unused", no_weak_conversions)]
pub union Nullable<T: 'static + ?Sized> {
	pub non_null: NonNull<T>,
	pub raw: *mut T,
}

pub struct NullableDescriptor<T: ?Sized>(PhantomData<T>);
impl<T: ?Sized> NullableDescriptor<T> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<T: 'static + ?Sized> Descriptor for NullableDescriptor<T> {
	type Weak = *mut T;
	type Strong = *mut T;
	type Error = Infallible;

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

impl<T: ?Sized, E, N> UnionFieldAccess<*mut T, E, NonNull<T>, N> for NullableDescriptor<T> {
	fn get<'a>(&self, strong: &'a *mut T, name: N) -> Result<Option<&'a NonNull<T>>, E> {
		unimplemented!()
	}

	fn get_mut<'a>(
		&self,
		strong: &'a mut *mut T,
		name: N,
	) -> Result<Option<&'a mut NonNull<T>>, E> {
		unimplemented!()
	}

	fn set(&self, strong: &mut *mut T, name: N, value: NonNull<T>) -> Result<(), E>
	where
		NonNull<T>: Sized,
	{
		unimplemented!()
	}

	fn insert<'a>(
		&self,
		strong: &'a mut *mut T,
		name: N,
		value: NonNull<T>,
	) -> Result<(&'a mut NonNull<T>, Option<NonNull<T>>), E>
	where
		NonNull<T>: Sized,
	{
		unimplemented!()
	}
}

impl<T: ?Sized, E, N> UnionFieldAccess<*mut T, E, *mut T, N> for NullableDescriptor<T> {
	fn get<'a>(&self, strong: &'a *mut T, name: N) -> Result<Option<&'a *mut T>, E> {
		Ok(Some(strong))
	}

	fn get_mut<'a>(&self, strong: &'a mut *mut T, name: N) -> Result<Option<&'a mut *mut T>, E> {
		Ok(Some(strong))
	}

	fn set(&self, strong: &mut *mut T, name: N, value: *mut T) -> Result<(), E>
	where
		*mut T: Sized,
	{
		*strong = value;
		Ok(())
	}

	fn insert<'a>(
		&self,
		strong: &'a mut *mut T,
		name: N,
		value: *mut T,
	) -> Result<(&'a mut *mut T, Option<*mut T>), E>
	where
		*mut T: Sized,
	{
		let previous = mem::replace(strong, value);
		Ok((strong, Some(previous)))
	}
}
