use faible::{faible, Descriptor, UnionFieldAccess};
use std::{marker::PhantomData, mem, ptr::NonNull};

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

	fn strong<'a>(&self, weak: &'a Self::Weak) -> faible::Result<&'a Self::Strong> {
		Ok(weak)
	}

	fn strong_mut<'a>(&self, weak: &'a mut Self::Weak) -> faible::Result<&'a mut Self::Strong> {
		Ok(weak)
	}

	fn strong_into_weak(&self, strong: Self::Strong) -> Self::Weak {
		strong
	}

	fn try_weak_into_strong(&self, weak: Self::Weak) -> faible::Result<Self::Strong> {
		Ok(weak)
	}
}

impl<T: ?Sized, N> UnionFieldAccess<*mut T, NonNull<T>, N> for NullableDescriptor<T> {
	fn get<'a>(&self, strong: &'a *mut T, name: N) -> faible::Result<Option<&'a NonNull<T>>> {
		unimplemented!()
	}

	fn get_mut<'a>(
		&self,
		strong: &'a mut *mut T,
		name: N,
	) -> faible::Result<Option<&'a mut NonNull<T>>> {
		unimplemented!()
	}

	fn set(&self, strong: &mut *mut T, name: N, value: NonNull<T>) -> faible::Result<()>
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
	) -> faible::Result<(&'a mut NonNull<T>, Option<NonNull<T>>)>
	where
		NonNull<T>: Sized,
	{
		unimplemented!()
	}
}

impl<T: ?Sized, N> UnionFieldAccess<*mut T, *mut T, N> for NullableDescriptor<T> {
	fn get<'a>(&self, strong: &'a *mut T, name: N) -> faible::Result<Option<&'a *mut T>> {
		Ok(Some(strong))
	}

	fn get_mut<'a>(
		&self,
		strong: &'a mut *mut T,
		name: N,
	) -> faible::Result<Option<&'a mut *mut T>> {
		Ok(Some(strong))
	}

	fn set(&self, strong: &mut *mut T, name: N, value: *mut T) -> faible::Result<()>
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
	) -> faible::Result<(&'a mut *mut T, Option<*mut T>)>
	where
		*mut T: Sized,
	{
		let previous = mem::replace(strong, value);
		Ok((strong, Some(previous)))
	}
}
