//! A framework for strongly typed manipulation of weakly typed data.
//!
//! [![Zulip Chat](https://img.shields.io/endpoint?label=chat&url=https%3A%2F%2Fiteration-square-automation.schichler.dev%2F.netlify%2Ffunctions%2Fstream_subscribers_shield%3Fstream%3Dproject%252Ffaible)](https://iteration-square.schichler.dev/#narrow/stream/project.2Ffaible)

#![doc(html_root_url = "https://docs.rs/faible/0.0.1")]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::semicolon_if_nothing_returned)]

use std::{
	fmt::{self, Display, Formatter},
	mem::{ManuallyDrop, MaybeUninit},
};

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

pub use faible_proc_macro_definitions::faible;

#[derive(Debug)]
pub struct Error(String);
pub type Result<T> = core::result::Result<T, Error>;

impl Error {
	#[must_use]
	pub fn new(message: String) -> Error {
		Self(message)
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", &self.0)
	}
}
impl std::error::Error for Error {}

pub trait Faible {
	type Descriptor: Descriptor;

	fn as_strong(&self) -> Result<&<Self::Descriptor as Descriptor>::Strong>;
	fn as_strong_mut(&mut self) -> Result<&mut <Self::Descriptor as Descriptor>::Strong>;
}

pub trait Descriptor {
	type Weak;
	type Strong;
	fn strong<'a>(&self, this: &'a Self::Weak) -> Result<&'a Self::Strong>;
	fn strong_mut<'a>(&self, this: &'a mut Self::Weak) -> Result<&'a mut Self::Strong>;
	fn strong_into_weak(&self, strong: Self::Strong) -> Self::Weak;
	fn try_weak_into_strong(&self, weak: Self::Weak) -> Result<Self::Strong>;
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

pub trait FieldAccess<Strong: ?Sized, T: ?Sized, N> {
	fn get<'a>(&self, strong: &'a Strong, name: N) -> Result<&'a T>;
	fn get_mut<'a>(&self, strong: &'a mut Strong, name: N) -> Result<&'a mut T>;
	fn set(&self, strong: &mut Strong, name: N, value: T) -> Result<()>
	where
		T: Sized;
	fn insert<'a>(&self, strong: &'a mut Strong, name: N, value: T) -> Result<(&'a mut T, Option<T>)>
	where
		T: Sized;
}

pub trait UnionFieldAccess<Strong: ?Sized, T: ?Sized, N> {
	fn get<'a>(&self, strong: &'a Strong, name: N) -> Result<Option<&'a T>>;
	fn get_mut<'a>(&self, strong: &'a mut Strong, name: N) -> Result<Option<&'a mut T>>;
	fn set(&self, strong: &mut Strong, name: N, value: T) -> Result<()>
	where
		T: Sized;
	fn insert<'a>(&self, strong: &'a mut Strong, name: N, value: T) -> Result<(&'a mut T, Option<T>)>
	where
		T: Sized;
}

pub trait VariantFilter<Strong: ?Sized, N> {
	fn predicate(&self, strong: &Strong, name: N) -> Result<bool>;
}
