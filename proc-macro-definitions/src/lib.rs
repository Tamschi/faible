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
	Attribute, Error, Expr, Field, Generics, Item, ItemStruct, LitStr, Path, Result, Token,
	Visibility,
};

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
}
impl Default for Args {
	fn default() -> Self {
		Self {
			descriptor: parse_quote_spanned! {Span::mixed_site()=> ()},
			faible: parse_quote_spanned! {Span::mixed_site()=> ::faible},
			names: parse_quote_spanned! {Span::mixed_site()=> "verbatim"},
		}
	}
}

fn args_parser(errors: &mut Vec<Error>) -> impl '_ + FnOnce(ParseStream) -> Result<Args> {
	mod kw {
		use syn::custom_keyword;

		custom_keyword!(faible);
		custom_keyword!(names);
	}

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
	} = match input {
		Item::Enum(_) => todo!(),
		Item::Struct(struct_) => process_struct(struct_, &args),
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
	} = args;

	let fields = quote_spanned! {fields_span.resolved_at(Span::mixed_site())=>
		(pub <#descriptor as #faible::Descriptor>::Weak)
	};

	let where_ = generics.where_clause.as_ref();
	let (impl_generics, type_generics, impl_where) = generics.split_for_impl();
	quote_spanned! {Span::mixed_site()=>
		#(#attrs)*
		#[repr(transparent)]
		#vis #struct_token #ident #generics #fields #where_ #semicolon

		/// # Safety
		///
		/// Automatically implemented by [faible](https://github.com/Tamschi/faible#readme).
		#[automatically_derived]
		unsafe impl #impl_generics #faible::View<<#descriptor as #faible::Descriptor>::Weak> for #ident #type_generics #impl_where {}

		#[automatically_derived]
		impl #impl_generics #ident #type_generics #impl_where {
			#(#methods)*
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
}

fn process_struct(struct_: ItemStruct, args: &Args) -> Processed {
	let Args {
		descriptor,
		faible,
		names,
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
					colon_token,
					ty,
				},
			)| {
				let mut buffer = Buffer::new();
				let ident = ident.unwrap_or_else(|| Ident::new(buffer.format(i), ty.span()));
				let colon_token = colon_token.unwrap_or_else(|| Token![:](ty.span()));

				let ident_string = ident.to_string();
				let get = if ident_string.starts_with(|c: char| c.is_ascii_digit()) {
					Ident::new(&format!("get_{ident_string}"), ident.span())
				} else {
					ident.clone()
				};
				let get_mut = Ident::new(&format!("{get}_mut"), ident.span());
				let set = Ident::new(&format!("set_{ident_string}"), ident.span());

				let name = make_name(&ident, names);

				quote_spanned! {ty.span().resolved_at(Span::mixed_site())=>
					#vis fn #get(&self) -> #faible::Result<&#ty> {
						let descriptor = #descriptor;
						let strong = #faible::Descriptor::strong(&descriptor, &self.0)?;
						#faible::FieldAccess::get(&descriptor, strong, #name)
					}

					#vis fn #get_mut(&mut self) -> #faible::Result<&mut #ty> {
						let descriptor = #descriptor;
						let strong = #faible::Descriptor::strong_mut(&descriptor, &mut self.0)?;
						#faible::FieldAccess::get_mut(&descriptor, strong, #name)
					}

					#vis fn #set(&mut self, value: #ty) -> #faible::Result<()> {
						let descriptor = #descriptor;
						let strong = #faible::Descriptor::strong_mut(&descriptor, &mut self.0)?;
						#faible::FieldAccess::set(&descriptor, strong, #name, value)
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
	}
}

fn make_name(ident: &Ident, names: &Expr) -> Expr {
	struct NameVisitor<'a>(&'a Ident);
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
					name if name.starts_with("literally") => name
						.strip_prefix("literally")
						.expect("unreachable")
						.to_string(),
					_ => return,
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
					name if name.starts_with("literally") => name
						.strip_prefix("literally")
						.expect("unreachable")
						.to_string(),
					_ => return,
				},
				self.0.span(),
			);
		}
	}

	let mut name = names.clone();
	NameVisitor(ident).visit_expr_mut(&mut name);
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
