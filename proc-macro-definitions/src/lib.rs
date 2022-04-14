use call2_for_syn::{call2_allow_incomplete, call2_strict, Incomplete};
use heck::{
	ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutyKebabCase, ToShoutySnakeCase,
	ToShoutySnekCase, ToSnakeCase, ToSnekCase, ToTitleCase, ToUpperCamelCase,
};
use itoa::Buffer;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::quote_spanned;
use syn::{
	parse::{Parse, ParseBuffer, ParseStream},
	parse_quote_spanned,
	spanned::Spanned,
	visit_mut::VisitMut,
	Attribute, Error, Expr, Field, Generics, Item, ItemEnum, ItemStruct, ItemUnion, LitStr, Path,
	Result, Token, Variant, Visibility,
};
use tap::Pipe;
use vec_drain_where::VecDrainWhereExt;

mod kw {
	use syn::custom_keyword;

	custom_keyword!(faible);
	custom_keyword!(name);
	custom_keyword!(names);
	custom_keyword!(no_weak_conversions);
}

#[proc_macro_attribute]
pub fn faible(args: TokenStream1, input: TokenStream1) -> TokenStream1 {
	let mut errors = vec![];

	let args_parser = args_parser(&mut errors);
	let args = syn::parse_macro_input!(args with args_parser);

	let input = syn::parse_macro_input!(input as Item);

	let output = implement(args, input, &mut errors);
	let errors = errors.into_iter().map(Error::into_compile_error);

	quote_spanned! {Span::mixed_site()=>
		#(#errors)*
		#output
	}
	.into()
}

struct Args {
	descriptor: Expr,
	faible: Path,
	names: Expr,
	no_weak_conversions: bool,
}
impl Default for Args {
	fn default() -> Self {
		Self {
			descriptor: parse_quote_spanned! {Span::mixed_site()=> ()},
			faible: parse_quote_spanned! {Span::mixed_site()=> ::faible},
			names: parse_quote_spanned! {Span::mixed_site()=> __faible__name_required},
			no_weak_conversions: false,
		}
	}
}

fn args_parser(errors: &mut Vec<Error>) -> impl '_ + FnOnce(ParseStream) -> Result<Args> {
	move |input: ParseStream| {
		let mut args = Args::default();

		input.insist(errors).then_set(&mut args.descriptor);

		while input
			.parse::<Option<Token![,]>>()
			.expect("infallible")
			.is_some()
		{
			loop {
				let lookahead = input.lookahead1();
				if lookahead.peek(kw::faible) {
					input.parse::<kw::faible>().expect("unreachable");
					input
						.insist::<Token![=]>(errors)
						.and_then(|_| input.insist(errors))
						.then_set(&mut args.faible)
				} else if lookahead.peek(kw::names) {
					input.parse::<kw::names>().expect("unreachable");

					input
						.insist::<Token![=]>(errors)
						.and_then(|_| input.insist(errors))
						.then_set(&mut args.names);
				} else if lookahead.peek(kw::no_weak_conversions) {
					input
						.parse::<kw::no_weak_conversions>()
						.expect("unreachable");
					args.no_weak_conversions = true;
				} else {
					errors.push(lookahead.error());
					input.parse::<TokenTree>().ok();
					continue;
				}
				break;
			}
		}

		if !input.is_empty() {
			errors.push(Error::new_spanned(
				input.parse::<TokenStream>().expect("infallible"),
				"Unexpected tokens.",
			))
		}

		Ok(args)
	}
}

fn implement(args: Args, input: Item, errors: &mut Vec<Error>) -> TokenStream {
	let Processed {
		attrs,
		vis,
		struct_token,
		ident,
		generics,
		fields_span,
		methods,
		semicolon,
		items,
	} = match input {
		Item::Enum(enum_) => process_enum(enum_, &args, errors),
		Item::Struct(struct_) => process_struct(struct_, &args, errors),
		Item::Union(union) => process_union(union, &args, errors),
		_ => {
			errors.push(Error::new(
				Span::mixed_site(),
				"This attribute is only valid on `struct` items.",
			));
			return TokenStream::new();
		}
	};

	let Args {
		descriptor,
		faible,
		names: _,
		no_weak_conversions,
	} = args;

	let descriptor_type = descriptor_type(&descriptor, errors);

	let fields = quote_spanned! {fields_span.resolved_at(Span::mixed_site())=>
		(pub <#descriptor_type as #faible::Descriptor>::Weak)
	};

	let where_ = generics.where_clause.as_ref();
	let (impl_generics, type_generics, impl_where) = generics.split_for_impl();

	let weak_conversions = (!no_weak_conversions).then(|| {
		quote_spanned! {Span::mixed_site()=>
			#[automatically_derived]
			impl #impl_generics core::convert::From<<#descriptor_type as #faible::Descriptor>::Weak> for #ident #type_generics #impl_where {
				fn from(value: <#descriptor_type as #faible::Descriptor>::Weak) -> Self {
					Self(value)
				}
			}

			#[automatically_derived]
			impl #impl_generics core::convert::From<#ident #type_generics> for <#descriptor_type as #faible::Descriptor>::Weak #impl_where {
				fn from(value: #ident #type_generics) -> Self {
					value.0
				}
			}
		}
	});

	quote_spanned! {Span::mixed_site()=>
		#(#attrs)*
		#[repr(transparent)]
		#vis #struct_token #ident #generics #fields #where_ #semicolon

		#[automatically_derived]
		impl #impl_generics #faible::Faible for #ident #type_generics #impl_where {
			type Descriptor = #descriptor_type;

			fn as_strong(&self) -> ::core::result::Result<
				&<Self::Descriptor as #faible::Descriptor>::Strong,
				<Self::Descriptor as #faible::Descriptor>::Error
			> {
				#faible::Descriptor::strong(&#descriptor, &self.0)
			}

			fn as_strong_mut(&mut self) -> ::core::result::Result<
				&mut <Self::Descriptor as #faible::Descriptor>::Strong,
				<Self::Descriptor as #faible::Descriptor>::Error
			> {
				#faible::Descriptor::strong_mut(&#descriptor, &mut self.0)
			}
		}

		#[automatically_derived]
		impl #impl_generics core::convert::From<<#descriptor_type as #faible::Descriptor>::Strong> for #ident #type_generics #impl_where {
			fn from(value: <#descriptor_type as #faible::Descriptor>::Strong) -> Self {
				Self(#faible::Descriptor::strong_into_weak(&#descriptor, value))
			}
		}

		#[automatically_derived]
		impl #impl_generics core::convert::TryFrom<#ident #type_generics> for <#descriptor_type as #faible::Descriptor>::Strong #impl_where {
			type Error = <#descriptor_type as #faible::Descriptor>::Error;

			fn try_from(value: #ident #type_generics) -> ::core::result::Result<Self, Self::Error> {
				#faible::Descriptor::try_weak_into_strong(&#descriptor, value.0)
			}
		}

		#weak_conversions

		/// # Safety
		///
		/// Automatically implemented by [faible](https://github.com/Tamschi/faible#readme).
		#[automatically_derived]
		unsafe impl #impl_generics #faible::View<<#descriptor_type as #faible::Descriptor>::Weak> for #ident #type_generics #impl_where {}

		#[automatically_derived]
		impl #impl_generics #ident #type_generics #impl_where {
			#(#methods)*
		}

		#(#items)*
	}
}

fn descriptor_type(descriptor: &Expr, errors: &mut Vec<Error>) -> Path {
	match call2_allow_incomplete(quote_spanned!(Span::mixed_site()=> #descriptor), |input| {
		input.parse::<Path>()
	}) {
		Ok(mut path) => {
			if path.segments.len() > 1 {
				let last_ident_string =
					path.segments.last().expect("unreachable").ident.to_string();
				if last_ident_string
					.strip_prefix("r#")
					.unwrap_or(&last_ident_string)
					.chars()
					.next()
					.expect("This *should* be non-empty.")
					.is_ascii_lowercase()
				{
					path.segments.pop().expect("unreachable");
					path.segments
						.pop()
						.expect("unreachable")
						.into_value()
						.pipe(|segment| path.segments.push(segment));
				}
			}
			path
		}
		Err(error) => {
			errors.push(error);
			parse_quote_spanned!(descriptor.span()=> __faible__UnknownType)
		}
	}
}

struct Processed {
	attrs: Vec<Attribute>,
	vis: Visibility,
	struct_token: Token![struct],
	ident: Ident,
	generics: Generics,
	fields_span: Span,
	methods: Vec<TokenStream>,
	semicolon: Token![;],
	items: Vec<TokenStream>,
}

fn process_enum(enum_: ItemEnum, args: &Args, errors: &mut Vec<Error>) -> Processed {
	let Args {
		descriptor,
		faible,
		names,
		no_weak_conversions,
	} = args;
	let ItemEnum {
		attrs,
		vis,
		enum_token,
		ident,
		generics,
		brace_token,
		variants,
	} = enum_;

	let ref_ty = Ident::new(&(ident.to_string() + "VariantRef"), Span::call_site());
	let mut_ty = Ident::new(&(ident.to_string() + "VariantMut"), Span::call_site());
	let owned_ty = Ident::new(&(ident.to_string() + "VariantOwned"), Span::call_site());

	let descriptor_type = descriptor_type(descriptor, errors);

	let has_fields = variants.iter().any(|variant| !variant.fields.is_empty());

	let mut owned_variants = vec![];
	let mut ref_variants = vec![];
	let mut mut_variants = vec![];

	for (
		i,
		Variant {
			mut attrs,
			ident,
			fields,
			discriminant,
		},
	) in variants.into_iter().enumerate()
	{
		let args = take_args_from_attrs(&args.into(), &mut attrs, errors);

		owned_variants.push(Variant {
			attrs: attrs.clone(),
			ident: ident.clone(),
			fields: fields.clone(),
			discriminant: discriminant.clone(),
		});

		ref_variants.push(Variant {
			attrs: attrs.clone(),
			ident: ident.clone(),
			fields: {
				let mut fields = fields.clone();
				for field in fields.iter_mut() {
					let ty = &field.ty;
					field.ty = parse_quote_spanned! {ty.span().resolved_at(Span::mixed_site())=>
						&'access #ty
					};
				}
				fields
			},
			discriminant: discriminant.clone(),
		});

		mut_variants.push(Variant {
			attrs: attrs.clone(),
			ident: ident.clone(),
			fields: {
				let mut fields = fields.clone();
				for field in fields.iter_mut() {
					let ty = &field.ty;
					field.ty = parse_quote_spanned! {ty.span().resolved_at(Span::mixed_site())=>
						&'access mut #ty
					};
				}
				fields
			},
			discriminant: discriminant.clone(),
		});
	}

	let borrow_generics = {
		let mut generics = generics.clone();
		if has_fields {
			generics
				.params
				.insert(0, parse_quote_spanned!(Span::mixed_site()=> 'access));
		}
		generics
	};
	let where_ = generics.where_clause.as_ref();

	let items = vec![
		quote_spanned! {Span::mixed_site()=>
			#(#attrs)*
			#[automatically_derived]
			#vis #enum_token #owned_ty #generics #where_ {
				#(#owned_variants,)*
			}
		},
		quote_spanned! {Span::mixed_site()=>
			#(#attrs)*
			#[automatically_derived]
			#vis #enum_token #ref_ty #borrow_generics #where_ {
				#(#ref_variants,)*
			}
		},
		quote_spanned! {Span::mixed_site()=>
			#(#attrs)*
			#[automatically_derived]
			#vis #enum_token #mut_ty #borrow_generics #where_ {
				#(#mut_variants,)*
			}
		},
	];

	let methods = vec![
		quote_spanned! {Span::mixed_site()=>
			#vis fn as_variant<'access>(&'access self) -> ::core::result::Result<
				#ref_ty #borrow_generics,
				<#descriptor_type as #faible::Descriptor>::Error,
			> {
				//TODO
				{
					Err(<<#descriptor_type as #faible::Descriptor>::Error as #faible::Error>::no_variant_recognized())
				}
			}
		},
		quote_spanned! {Span::mixed_site()=>
			#vis fn as_variant_mut<'access>(&'access mut self) -> ::core::result::Result<
				#mut_ty #borrow_generics,
				<#descriptor_type as #faible::Descriptor>::Error,
			> {
				//TODO
				{
					Err(<<#descriptor_type as #faible::Descriptor>::Error as #faible::Error>::no_variant_recognized())
				}
			}
		},
	];

	Processed {
		attrs,
		vis,
		struct_token: Token![struct](enum_token.span),
		ident,
		generics,
		fields_span: brace_token.span,
		methods,
		semicolon: Token![;](brace_token.span),
		items,
	}
}
#[derive(Clone)]
struct InnerArgs {
	descriptor: Expr,
	name: Option<Expr>,
	names: Expr,
}
impl From<Args> for InnerArgs {
	fn from(args: Args) -> Self {
		Self {
			descriptor: args.descriptor,
			name: None,
			names: args.names,
		}
	}
}
impl From<&Args> for InnerArgs {
	fn from(args: &Args) -> Self {
		Self {
			descriptor: args.descriptor.clone(),
			name: None,
			names: args.names.clone(),
		}
	}
}

fn take_args_from_attrs(
	args: &InnerArgs,
	attrs: &mut Vec<Attribute>,
	errors: &mut Vec<Error>,
) -> InnerArgs {
	let mut inner_args = args.clone();

	for attr in attrs.e_drain_where(|attr| attr.path.is_ident("faible")) {
		let Attribute { tokens, .. } = attr;

		call2_strict(tokens, |input| {
			if input.peek(Token![_]) {
				input.parse::<Token![_]>().expect("unreachable");
			} else {
				input.insist(errors).then_set(&mut inner_args.descriptor);
			}

			while input
				.parse::<Option<Token![,]>>()
				.expect("infallible")
				.is_some()
			{
				let lookahead = input.lookahead1();

				if lookahead.peek(kw::name) {
					let name = input.parse::<kw::name>().expect("unreachable");
					if inner_args.name.is_some() {
						errors.push(Error::new(name.span, "Duplicate name definition."))
					}
					input.insist::<Token![=]>(errors);
					inner_args.name = input.insist(errors);
				} else if lookahead.peek(kw::names) {
					input.parse::<kw::names>().expect("unreachable");
					input.insist::<Token![=]>(errors);
					input.insist(errors).then_set(&mut inner_args.names);
				} else {
					errors.push(lookahead.error())
				}
			}
		})
		.unwrap_or_else(
			|Incomplete {
			     parsed: (),
			     syn_error,
			 }| errors.push(syn_error),
		);
	}

	inner_args
}

fn process_struct(struct_: ItemStruct, args: &Args, errors: &mut Vec<Error>) -> Processed {
	let Args {
		descriptor,
		faible,
		names,
		no_weak_conversions: _,
	} = args;
	let ItemStruct {
		attrs,
		vis,
		struct_token,
		ident,
		generics,
		fields,
		semi_token,
	} = struct_;

	let descriptor_type = descriptor_type(descriptor, errors);

	let fields_span = fields.span();
	let methods = fields
		.into_iter()
		.enumerate()
		.map(
			|(
				i,
				Field {
					attrs,
					vis,
					ident,
					colon_token: _,
					ty,
				},
			)| {
				let ident = ident.unwrap_or_else(|| Ident::new(Buffer::new().format(i), ty.span()));
				let ident_string = ident.to_string();

				let get = if ident_string.starts_with(|c: char| c.is_ascii_digit()) {
					Ident::new(&format!("get_{ident_string}"), ident.span())
				} else {
					ident.clone()
				};
				let get_mut = Ident::new(&format!("{get}_mut"), ident.span());
				let set = Ident::new(&format!("set_{ident_string}"), ident.span());
				let insert = Ident::new(&format!("insert_{ident_string}"), ident.span());

				let name = make_name(&ident, names, errors);

				quote_spanned! {ty.span().resolved_at(Span::mixed_site())=>
					#(#attrs)*
					#vis fn #get(&self) -> ::core::result::Result<&#ty, <#descriptor_type as #faible::Descriptor>::Error> {
						let descriptor = #descriptor;
						let strong = #faible::Descriptor::strong(&descriptor, &self.0)?;
						#faible::FieldAccess::get(&descriptor, strong, #name)
					}

					#(#attrs)*
					#vis fn #get_mut(&mut self) -> ::core::result::Result<&mut #ty, <#descriptor_type as #faible::Descriptor>::Error> {
						let descriptor = #descriptor;
						let strong = #faible::Descriptor::strong_mut(&descriptor, &mut self.0)?;
						#faible::FieldAccess::get_mut(&descriptor, strong, #name)
					}

					#(#attrs)*
					#vis fn #set(&mut self, value: #ty) -> ::core::result::Result<(), <#descriptor_type as #faible::Descriptor>::Error> {
						let descriptor = #descriptor;
						let strong = #faible::Descriptor::strong_mut(&descriptor, &mut self.0)?;
						#faible::FieldAccess::set(&descriptor, strong, #name, value)
					}

					#(#attrs)*
					#vis fn #insert(&mut self, value: #ty) -> ::core::result::Result<(&mut #ty, ::core::option::Option<#ty>), <#descriptor_type as #faible::Descriptor>::Error> {
						let descriptor = #descriptor;
						let strong = #faible::Descriptor::strong_mut(&descriptor, &mut self.0)?;
						#faible::FieldAccess::insert(&descriptor, strong, #name, value)
					}
				}
			},
		)
		.collect();

	Processed {
		attrs,
		vis,
		struct_token,
		ident,
		generics,
		fields_span,
		methods,
		semicolon: semi_token.unwrap_or_else(|| Token![;](fields_span)),
		items: vec![],
	}
}

fn process_union(union: ItemUnion, args: &Args, errors: &mut Vec<Error>) -> Processed {
	let Args {
		descriptor,
		faible,
		names,
		no_weak_conversions: _,
	} = args;
	let ItemUnion {
		attrs,
		vis,
		union_token,
		ident,
		generics,
		fields,
	} = union;

	let descriptor_type = descriptor_type(descriptor, errors);

	let methods = fields
		.named
		.into_iter()
		.enumerate()
		.map(
			|(
				i,
				Field {
					attrs,
					vis,
					ident,
					colon_token: _,
					ty,
				},
			)| {
				let ident = ident.unwrap_or_else(|| Ident::new(Buffer::new().format(i), ty.span()));
				let ident_string = ident.to_string();

				let get = if ident_string.starts_with(|c: char| c.is_ascii_digit()) {
					Ident::new(&format!("get_{ident_string}"), ident.span())
				} else {
					ident.clone()
				};
				let get_mut = Ident::new(&format!("{get}_mut"), ident.span());
				let set = Ident::new(&format!("set_{ident_string}"), ident.span());
				let insert = Ident::new(&format!("insert_{ident_string}"), ident.span());

				let name = make_name(&ident, names, errors);

				quote_spanned! {ty.span().resolved_at(Span::mixed_site())=>
					#(#attrs)*
					#vis fn #get(&self) -> ::core::result::Result<Option<&#ty>, <#descriptor_type as #faible::Descriptor>::Error> {
						let descriptor = #descriptor;
						let strong = #faible::Descriptor::strong(&descriptor, &self.0)?;
						#faible::UnionFieldAccess::get(&descriptor, strong, #name)
					}

					#(#attrs)*
					#vis fn #get_mut(&mut self) -> ::core::result::Result<Option<&mut #ty>, <#descriptor_type as #faible::Descriptor>::Error> {
						let descriptor = #descriptor;
						let strong = #faible::Descriptor::strong_mut(&descriptor, &mut self.0)?;
						#faible::UnionFieldAccess::get_mut(&descriptor, strong, #name)
					}

					#(#attrs)*
					#vis fn #set(&mut self, value: #ty) -> ::core::result::Result<(), <#descriptor_type as #faible::Descriptor>::Error> {
						let descriptor = #descriptor;
						let strong = #faible::Descriptor::strong_mut(&descriptor, &mut self.0)?;
						#faible::UnionFieldAccess::set(&descriptor, strong, #name, value)
					}

					#(#attrs)*
					#vis fn #insert(&mut self, value: #ty) -> ::core::result::Result<(&mut #ty, ::core::option::Option<#ty>), <#descriptor_type as #faible::Descriptor>::Error> {
						let descriptor = #descriptor;
						let strong = #faible::Descriptor::strong_mut(&descriptor, &mut self.0)?;
						#faible::UnionFieldAccess::insert(&descriptor, strong, #name, value)
					}
				}
			},
		)
		.collect();

	Processed {
		attrs,
		vis,
		struct_token: Token![struct](union_token.span),
		ident,
		generics,
		fields_span: fields.brace_token.span,
		methods,
		semicolon: Token![;](fields.brace_token.span),
		items: vec![],
	}
}

fn make_name(ident: &Ident, names: &Expr, errors: &mut Vec<Error>) -> Expr {
	struct NameVisitor<'a>(&'a Ident, &'a mut Vec<Error>);
	impl VisitMut for NameVisitor<'_> {
		fn visit_lit_str_mut(&mut self, i: &mut syn::LitStr) {
			let name = self.0.to_string();
			*i = LitStr::new(
				&match i.value().as_str() {
					"kebab-case" => name.to_kebab_case(),
					"lowerCamelCase" => name.to_lower_camel_case(),
					"PascalCase" => name.to_pascal_case(),
					"SHOUTY-KEBAB-CASE" => name.to_shouty_kebab_case(),
					"SHOUTY_SNAKE_CASE" => name.to_shouty_snake_case(),
					"SHOUTY_SNEK_CASE" => name.TO_SHOUTY_SNEK_CASE(),
					"snake_case" => name.to_snake_case(),
					"snek_case" => name.to_snek_case(),
					"Title Case" => name.to_title_case(),
					"UpperCamelCase" => name.to_upper_camel_case(),
					"verbatim" => name,
					name if name.starts_with('_') => {
						name.strip_prefix('_').expect("unreachable").to_string()
					}
					_ => {
						self.1.push(Error::new(i.span(), r#"Unrecognised name string literal. (Prefix its value with `_` to use it literally.)
Replaced literals are: "kebab_case", "lowerCamelCase", "PascalCase", "SHOUTY_KEBAB_CASE", "SHOUTY_SNAKE_CASE", "SHOUTY_SNEK_CASE", "snake_case", "snek_case", "Title_Case", "UpperCamelCase", "verbatim"."#));
						return;
					}
				},
				self.0.span(),
			);
		}

		fn visit_ident_mut(&mut self, i: &mut Ident) {
			let name = self.0.to_string();
			*i = Ident::new(
				&match i.to_string().as_str() {
					"kebab_case" => name.to_kebab_case(),
					"lowerCamelCase" => name.to_lower_camel_case(),
					"PascalCase" => name.to_pascal_case(),
					"SHOUTY_KEBAB_CASE" => name.to_shouty_kebab_case(),
					"SHOUTY_SNAKE_CASE" => name.to_shouty_snake_case(),
					"SHOUTY_SNEK_CASE" => name.TO_SHOUTY_SNEK_CASE(),
					"snake_case" => name.to_snake_case(),
					"snek_case" => name.to_snek_case(),
					"Title_Case" => name.to_title_case(),
					"UpperCamelCase" => name.to_upper_camel_case(),
					"verbatim" => name,
					"__faible__name_required" => {
						self.1.push(Error::new(self.0.span(), "A field name expression is required. (`#[faible(…, names = <expr>)]`, try identifiers and string literals for more information.)"));
						*i = Ident::new(
							"__faible__name_required",
							self.0.span().resolved_at(Span::mixed_site()),
						);
						return;
					}
					name if name.starts_with('_') => {
						name.strip_prefix('_').expect("unreachable").to_string()
					}
					_ => {
						self.1.push(Error::new(i.span(), "Unrecognised name identifier. (Prefix it with `_` to use it literally.)
Replaced identifiers are: `kebab_case`, `lowerCamelCase`, `PascalCase`, `SHOUTY_KEBAB_CASE`, `SHOUTY_SNAKE_CASE`, `SHOUTY_SNEK_CASE`, `snake_case`, `snek_case`, `Title_Case`, `UpperCamelCase`, `verbatim`."));
						return;
					}
				},
				self.0.span(),
			);
		}
	}

	let mut name = names.clone();
	NameVisitor(ident, errors).visit_expr_mut(&mut name);
	name
}

trait Insist {
	fn insist<T: Parse>(&self, errors: &mut Vec<Error>) -> Option<T>;
}
impl Insist for ParseBuffer<'_> {
	fn insist<T: Parse>(&self, errors: &mut Vec<Error>) -> Option<T> {
		while !self.is_empty() {
			match self.parse() {
				Ok(ok) => return Some(ok),
				Err(error) => errors.push(error),
			}
			self.parse::<TokenTree>().expect("infallible");
		}
		None
	}
}

trait ThenSet<T> {
	fn then_set(self, slot: &mut T);
}
impl<T> ThenSet<T> for Option<T> {
	fn then_set(self, slot: &mut T) {
		if let Some(value) = self {
			*slot = value;
		}
	}
}
