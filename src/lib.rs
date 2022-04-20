//! A framework for strongly typed manipulation of weakly typed data.
//!
//! [![Zulip Chat](https://img.shields.io/endpoint?label=chat&url=https%3A%2F%2Fiteration-square-automation.schichler.dev%2F.netlify%2Ffunctions%2Fstream_subscribers_shield%3Fstream%3Dproject%252Ffaible)](https://iteration-square.schichler.dev/#narrow/stream/project.2Ffaible)

#![doc(html_root_url = "https://docs.rs/faible/0.0.1")]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![no_std]

use core::mem::{ManuallyDrop, MaybeUninit};

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

pub use faible_proc_macro_definitions::faible;

/// Allows specific errors to be created in generated code.
pub trait Error {
	/// Raised iff no matching enum variant could be determined.
	fn no_variant_recognized() -> Self;
}

pub trait Faible {
	type Descriptor: Descriptor;

	fn as_strong(
		&self,
	) -> Result<&<Self::Descriptor as Descriptor>::Strong, <Self::Descriptor as Descriptor>::Error>;
	fn as_strong_mut(
		&mut self,
	) -> Result<
		&mut <Self::Descriptor as Descriptor>::Strong,
		<Self::Descriptor as Descriptor>::Error,
	>;
}

pub trait Descriptor {
	type Weak;
	type Strong;
	type Error: crate::Error;

	fn strong<'a>(&self, weak: &'a Self::Weak) -> Result<&'a Self::Strong, Self::Error>;
	fn strong_mut<'a>(&self, weak: &'a mut Self::Weak)
		-> Result<&'a mut Self::Strong, Self::Error>;
	fn strong_into_weak(&self, strong: Self::Strong) -> Self::Weak;
	fn try_weak_into_strong(&self, weak: Self::Weak) -> Result<Self::Strong, Self::Error>;
}

/// # Safety
///
/// `Self` must be transmutation-compatible with `T`.
/// `&Self` must be transmutation-compatible with `&T`.
/// `&mut Self` must be transmutation-compatible with `&mut T`.
pub unsafe trait View<T: ?Sized> {
	#[allow(missing_docs)]
	fn from(value: T) -> Self
	where
		Self: Sized,
		T: Sized,
	{
		unsafe {
			(&ManuallyDrop::new(value) as *const ManuallyDrop<T>)
				.cast::<Self>()
				.read()
		}
	}

	#[allow(missing_docs)]
	fn from_ref(value: &T) -> &Self {
		unsafe { *(&value as *const &T).cast() }
	}

	#[allow(missing_docs)]
	fn from_mut(value: &mut T) -> &mut Self {
		unsafe { MaybeUninit::new(value).as_ptr().cast::<&mut Self>().read() }
	}

	#[allow(missing_docs)]
	fn into_inner(self) -> T
	where
		Self: Sized,
		T: Sized,
	{
		unsafe {
			(&ManuallyDrop::new(self) as *const ManuallyDrop<Self>)
				.cast::<T>()
				.read()
		}
	}

	fn from_insertion((slot, prev): (&mut T, Option<T>)) -> (&mut Self, Option<Self>)
	where
		Self: Sized,
		T: Sized,
	{
		(Self::from_mut(slot), prev.map(Self::from))
	}
}

/// # Safety
///
/// This is the identity pun.
unsafe impl<T: ?Sized> View<T> for T {}

pub trait FieldAccess<Strong: ?Sized, E, T: ?Sized, N> {
	fn get<'a>(&self, strong: &'a Strong, name: N) -> Result<&'a T, E>;
	fn get_mut<'a>(&self, strong: &'a mut Strong, name: N) -> Result<&'a mut T, E>;
	fn set(&self, strong: &mut Strong, name: N, value: T) -> Result<(), E>
	where
		T: Sized;
	fn insert<'a>(
		&self,
		strong: &'a mut Strong,
		name: N,
		value: T,
	) -> Result<(&'a mut T, Option<T>), E>
	where
		T: Sized;
}
// impl<Strong: ?Sized, E, T: ?Sized, N, U> FieldAccess<Strong, E, T, N> for &U
// where
// 	U: FieldAccess<Strong, E, T, N>,
// {
// 	fn get<'a>(&self, strong: &'a Strong, name: N) -> Result<&'a T, E> {
// 		U::get(self, strong, name)
// 	}

// 	fn get_mut<'a>(&self, strong: &'a mut Strong, name: N) -> Result<&'a mut T, E> {
// 		U::get_mut(self, strong, name)
// 	}

// 	fn set(&self, strong: &mut Strong, name: N, value: T) -> Result<(), E>
// 	where
// 		T: Sized,
// 	{
// 		U::set(self, strong, name, value)
// 	}

// 	fn insert<'a>(
// 		&self,
// 		strong: &'a mut Strong,
// 		name: N,
// 		value: T,
// 	) -> Result<(&'a mut T, Option<T>), E>
// 	where
// 		T: Sized,
// 	{
// 		U::insert(self, strong, name, value)
// 	}
// }

pub trait UnionFieldAccess<Strong: ?Sized, E, T: ?Sized, N> {
	fn get<'a>(&self, strong: &'a Strong, name: N) -> Result<Option<&'a T>, E>;
	fn get_mut<'a>(&self, strong: &'a mut Strong, name: N) -> Result<Option<&'a mut T>, E>;
	fn set(&self, strong: &mut Strong, name: N, value: T) -> Result<(), E>
	where
		T: Sized;
	fn insert<'a>(
		&self,
		strong: &'a mut Strong,
		name: N,
		value: T,
	) -> Result<(&'a mut T, Option<T>), E>
	where
		T: Sized;
}
// impl<Strong: ?Sized, E, T: ?Sized, N, U> UnionFieldAccess<Strong, E, T, N> for &U
// where
// 	U: UnionFieldAccess<Strong, E, T, N>,
// {
// 	fn get<'a>(&self, strong: &'a Strong, name: N) -> Result<Option<&'a T>, E> {
// 		U::get(self, strong, name)
// 	}

// 	fn get_mut<'a>(&self, strong: &'a mut Strong, name: N) -> Result<Option<&'a mut T>, E> {
// 		U::get_mut(self, strong, name)
// 	}

// 	fn set(&self, strong: &mut Strong, name: N, value: T) -> Result<(), E>
// 	where
// 		T: Sized,
// 	{
// 		U::set(self, strong, name, value)
// 	}

// 	fn insert<'a>(
// 		&self,
// 		strong: &'a mut Strong,
// 		name: N,
// 		value: T,
// 	) -> Result<(&'a mut T, Option<T>), E>
// 	where
// 		T: Sized,
// 	{
// 		U::insert(self, strong, name, value)
// 	}
// }

pub trait VariantFilter<Strong: ?Sized, E, N> {
	fn predicate(&self, strong: &Strong, name: N) -> Result<bool, E>;
}
// impl<Strong: ?Sized, E, N, T> VariantFilter<Strong, E, N> for &T
// where
// 	T: VariantFilter<Strong, E, N>,
// {
// 	fn predicate(&self, strong: &Strong, name: N) -> Result<bool, E> {
// 		T::predicate(self, strong, name)
// 	}
// }
pub trait VariantFieldAccess<Strong: ?Sized, E, T: ?Sized, N> {
	fn get<'a>(&self, strong: &'a Strong, name: N) -> Result<&'a T, E>;
	fn get_mut<'a>(&self, strong: *mut Strong, name: N) -> Result<&'a mut T, E>;
	fn set(&self, strong: &mut Strong, name: N, value: T) -> Result<(), E>
	where
		T: Sized;
	fn insert<'a>(
		&self,
		strong: &'a mut Strong,
		name: N,
		value: T,
	) -> Result<(&'a mut T, Option<T>), E>
	where
		T: Sized;
}
