use std::{any::Any, marker::PhantomData, mem, ptr::NonNull};

use faible::{faible, Descriptor, UnionFieldAccess};

#[faible(NullableDescriptor::<T>::new(), names = "_unused")]
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
	type Weak = Box<dyn Any>;
	type Strong = *mut T;

	fn strong<'a>(&self, this: &'a Self::Weak) -> faible::Result<&'a Self::Strong> {
		this.downcast_ref().ok_or_else(|| unimplemented!())
	}

	fn strong_mut<'a>(&self, this: &'a mut Self::Weak) -> faible::Result<&'a mut Self::Strong> {
		this.downcast_mut().ok_or_else(|| unimplemented!())
	}

	fn strong_into_weak(&self, strong: Self::Strong) -> Self::Weak {
		Box::new(strong)
	}

	fn try_weak_into_strong(&self, weak: Self::Weak) -> faible::Result<Self::Strong> {
		*weak.downcast().unwrap_or_else(|_| unimplemented!())
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
