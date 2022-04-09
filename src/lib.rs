//! A framework for strongly typed manipulation of weakly typed data.
//!
//! [![Zulip Chat](https://img.shields.io/endpoint?label=chat&url=https%3A%2F%2Fiteration-square-automation.schichler.dev%2F.netlify%2Ffunctions%2Fstream_subscribers_shield%3Fstream%3Dproject%252Ffaible)](https://iteration-square.schichler.dev/#narrow/stream/project.2Ffaible)

#![doc(html_root_url = "https://docs.rs/faible/0.0.1")]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::semicolon_if_nothing_returned)]

use std::mem::{ManuallyDrop, MaybeUninit};

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

pub use faible_proc_macro_definitions::faible;

pub struct Error(String);
pub type Result<T> = core::result::Result<T, Error>;

impl Error {
	pub fn new(message: String) -> Error {
		Self(message)
	}
}

pub trait Faible {
	fn validate(&self) -> Result<()>;
}

pub trait Descriptor {
	type Weak;
	type Strong;
	fn strong<'a>(&self, this: &'a Self::Weak) -> Result<&'a Self::Strong>;
	fn strong_mut<'a>(&self, this: &'a mut Self::Weak) -> Result<&'a mut Self::Strong>;
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

pub trait FieldAccess<This: ?Sized, T: ?Sized, N> {
	fn get<'a>(&self, this: &'a This, name: N) -> Result<&'a T>;
	fn get_mut<'a>(&self, this: &'a mut This, name: N) -> Result<&'a mut T>;
	fn set(&self, this: &mut This, name: N, value: T) -> Result<()>
	where
		T: Sized;
	fn insert<'a>(&self, this: &'a mut This, name: N, value: T) -> Result<(&'a mut T, Option<T>)>
	where
		T: Sized;
}
