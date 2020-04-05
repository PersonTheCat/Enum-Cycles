extern crate proc_macro;
extern crate quote;
extern crate syn;

use self::{AttributeParseError::*, AttributeType::*};
use proc_macro::TokenStream;
use quote::*;
use std::convert::TryFrom;
use syn::export::{Span, TokenStream2};
use syn::spanned::Spanned;
use syn::{Attribute, Data, DataEnum, DeriveInput, Ident, Type, Variant};

/// The main function used to generate an EnumState implementation.
/// Supports four attributes: `default`, `auto`, `first`, and `last`,
/// which have the following indications:
///
/// ### `default`
///
/// When this token is placed at the top level (before the `enum` token),
/// it defines the default variant of the enum as a whole. For example,
/// use `#[default(Numbers::One)]` to indicate that `Numbers::One` is the
/// default state of the supplied enum.
///
/// When this token is placed at the variant level (just before the variant
/// itself), it defines the default input to forward into that variant. For
/// example, use `#[default(x)]` to indicate that `x` is the default value
/// for the variant's fields, separated by commas. `x` must be a constant
/// expression.
///
/// e.g.
/// ```ignore
///     #[default(Numbers::One)]
///     #[derive(Clone, EnumState)
///     enum Numbers {
///         One,
///         #[default(Inner::Left)]
///         Two(Inner)
///     }
///
///     #[derive(Clone, EnumState)]
///     enum Inner {
///         Left,
///         Right
///     }
/// ```
///
/// ### `first`
///
/// When this token is placed at the top level, it informs the compiler
/// to try and retrieve the first value for each field in any possible
/// value in the enum. This requires that all fields in *all variants*
/// implement `EnumState`, unless specifically indicated otherwise.
///
/// When this token is placed at the variant level, it overrides whichever
/// attribute was declared at the top level (excluding `default` and any
/// attribute specified by another macro).
///
/// ### `last`
///
/// This attribute is equivalent to `first`, except it retrieves the last
/// value in the enum instead of the first.
///
/// ### `auto`
///
/// This is another variant of `first` and `last` which informs the compiler
/// to try and use whichever value is specified as the default for any given
/// field in the enum. If anywhere no value is specified as the default value,
/// it will instead use the first value in the enum.
#[proc_macro_derive(EnumState, attributes(default, first, last, auto))]
pub fn derive_enum_cycle(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let ret = if let Data::Enum(ref e) = ast.data {
        if let Err(tokens) = validate_enum(&ast, e) {
            tokens
        } else {
            debug(impl_enum_cycle(&ast, e))
        }
    } else {
        error(&ast.span(), "EnumState can only be derived from enum variants.")
    };
    ret.into()
}

/// Verifies the enum's attributes, ensuring that enough are in place to
/// determine the default values to use for each variant. Can identify
/// some syntax errors, such as whether tokens are missing from a `default`
/// attribute.
fn validate_enum(ast: &DeriveInput, e: &DataEnum) -> Result<(), TokenStream2> {
    for variant in &e.variants {
        if let Err(e) = get_attr_type(ast, &variant) {
            if let NoneFound = e {
                if variant.fields.is_empty() {
                    continue;
                }
            }
            return Err(e.get_message(variant.span()));
        }
    }
    Ok(())
}

/// Attempts to retrieve the type of attribute specified for the given variant.
/// It first searches for any attribute specified at the variant level, and then
/// subsequently at the top level of the enum, if nothing is found. The top level
/// attribute may be ignored if it is a `default` attribute, as these are intended
/// for a different purpose in this position.
fn get_attr_type(ast: &DeriveInput, v: &Variant) -> Result<AttributeType, AttributeParseError> {
    match AttributeType::get_first(&v.attrs) {
        Err(NoneFound) => (),
        r => return r
    }
    for attr in &ast.attrs {
        match AttributeType::try_from(attr) {
            Err(NoneFound) | Ok(Default(_)) => (),
            r => return r
        }
    }
    Err(NoneFound)
}

/// Counterpart to `self::get_attr_type` which can retrieve only a `default` token
/// at the top level of the enum.
fn get_default(ast: &DeriveInput, default: &TokenStream2) -> TokenStream2 {
    for attr in &ast.attrs {
        if let Ok(Default(tokens)) = AttributeType::try_from(attr) {
            return tokens;
        }
    }
    default.clone()
}

fn impl_enum_cycle(ast: &DeriveInput, e: &DataEnum) -> TokenStream2 {
    let (names, values) = get_arrays(ast, e);
    let (first, last) = get_ends(&values);
    let (index_map, name_map) = get_maps(ast, e);
    let default = get_default(ast, &first);
    let name = &ast.ident;
    let size = e.variants.len();

    quote! {
        impl EnumState for #name {
            const _NAMES: &'static [&'static str] = &[#(#names),*];
            const _VALUES: &'static [Self] = &[#(#values),*];
            const _DEFAULT: Self = #default;
            const _FIRST: Self = #first;
            const _LAST: Self = #last;
            const _SIZE: usize = #size;

            fn index(&self) -> usize {
                match *self {
                    #index_map
                }
            }

            fn name(&self) -> &'static str {
                match *self {
                    #name_map
                }
            }
        }
    }
}

// Moving some code outside of `impl_enum_cycle`. Hopefully, this makes it
// easier to read.
fn get_arrays(ast: &DeriveInput, e: &DataEnum) -> (Vec<String>, Vec<TokenStream2>) {
    let names = e.variants.iter()
        .map(|v| v.ident.to_string())
        .collect();
    let values = e.variants.iter()
        .map(|v| get_constructor(ast, v))
        .collect();
    (names, values)
}

/// Produces the necessary tokens for constructing a new variant based on
/// its annotations.
fn get_constructor(ast: &DeriveInput, variant: &Variant) -> TokenStream2 {
    let parent = &ast.ident;
    let name = &variant.ident;

    if variant.fields.is_empty() {
        return quote!(#parent::#name);
    }
    let attr = match get_attr_type(ast, variant).ok().unwrap() {
        Default(tokens) => return quote!(#parent::#name#tokens),
        a => a
    };
    let fields: TokenStream2 = variant.fields.iter()
        .map(|f| get_constant(&f.ty, &attr))
        .collect();
    quote!(#parent::#name(#fields))
}

/// Determines which constant to use for the default value to use in each
/// field in a variant based on its annotations, assuming the constructor
/// has not been explicitly defined.
fn get_constant(f_ty: &Type, attr: &AttributeType) -> TokenStream2 {
    match attr {
        First => quote!(<#f_ty>::_FIRST,),
        Last => quote!(<#f_ty>::_LAST,),
        _ => quote!(<#f_ty>::_DEFAULT,)
    }
}

fn get_ends(vec: &Vec<TokenStream2>) -> (TokenStream2, TokenStream2) {
    (vec.first().unwrap().clone(), vec.last().unwrap().clone())
}

fn get_maps(ast: &DeriveInput, e: &DataEnum) -> (TokenStream2, TokenStream2) {
    (get_index_map(ast, e), get_name_map(ast, e))
}

fn get_index_map(ast: &DeriveInput, e: &DataEnum) -> TokenStream2 {
    e.variants.iter().enumerate()
        .map(|(i, v)| get_map(v, &ast.ident, i))
        .collect()
}

fn get_name_map(ast: &DeriveInput, e: &DataEnum) -> TokenStream2 {
    e.variants.iter()
        .map(|v| get_map(v, &ast.ident, v.ident.to_string()))
        .collect()
}

/// Produces a match arm which will ignore any fields for the given variant,
/// yielding `t` as the branch.
fn get_map(v: &Variant, parent: &Ident, t: impl ToTokens) -> TokenStream2 {
    let name = &v.ident;
    if v.fields.is_empty() {
        quote!(#parent::#name => #t,)
    } else {
        let f = v.fields.iter().map(|_| quote!(_));
        quote!(#parent::#name(#(#f),*) => #t,)
    }
}

/// The list of attributes supported by the macro.
enum AttributeType {
    Default(TokenStream2),
    Auto,
    First,
    Last
}

impl AttributeType {
    /// Looks through each variant in the vector and returns the first supported
    /// attribute found, along with any other tokens it requires or errors
    /// produced in the process.
    fn get_first(attrs: &Vec<Attribute>) -> Result<AttributeType, AttributeParseError> {
        for attr in attrs {
            match Self::try_from(attr) {
                Err(NoneFound) => (),
                r => return r
            }
        }
        Err(NoneFound)
    }
}

impl TryFrom<&Attribute> for AttributeType {
    type Error = AttributeParseError;

    /// Attempts to parse the input attribute as one of the attributes supported
    /// by the macro. May return either a value, a syntax error, or simply
    /// `NoneFound`.
    fn try_from(attr: &Attribute) -> Result<AttributeType, AttributeParseError> {
        let path = match attr.path.get_ident() {
            None => return Err(InvalidPath(attr.span())),
            Some(p) => p,
        };
        return match path.to_string().as_ref() {
            "default" => {
                if attr.tokens.is_empty() {
                    Err(MissingDefault(attr.span()))
                } else {
                    Ok(Default(attr.tokens.clone()))
                }
            }
            "auto" => Ok(Auto),
            "first" => Ok(First),
            "last" => Ok(Last),
            _ => Err(NoneFound)
        };
    }
}

/// The list of errors which the macro is capable of handling when parsing
/// attributes, currently supporting poor path syntax, missing values for
/// `default` types, and simply `NoneFound`.
enum AttributeParseError {
    InvalidPath(Span),
    MissingDefault(Span),
    NoneFound
}

impl AttributeParseError {
    /// Determines the error message to use for each type.
    fn get_message(&self, d: Span) -> TokenStream2 {
        match *self {
            InvalidPath(s) => error(&s, "Invalid path syntax."),
            MissingDefault(s) => error(&s, "Missing argument."),
            NoneFound => error(&d, "Default values must be defined for non-unit types.")
        }
    }
}

fn error(span: &Span, msg: &str) -> TokenStream2 {
    quote_spanned! {
        *span => compile_error!(#msg);
    }
}

/// Reports the entire stream of tokens to the user, provided the library is
/// compiled with the `debug` feature enabled.
fn debug(tokens: TokenStream2) -> TokenStream2 {
    if cfg!(feature = "debug") {
        println!("Generated tokens: \n{}", tokens.to_string());
    }
    tokens
}