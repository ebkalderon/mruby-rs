use darling::{ast, Error, FromDeriveInput, FromMeta, FromVariant};
use heck::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Visibility};

#[derive(Copy, Clone, Debug, FromMeta)]
#[darling(default)]
enum Case {
    #[darling(rename = "camelCase")]
    CamelCase,
    #[darling(rename = "kebab-case")]
    KebabCase,
    #[darling(rename = "lowercase")]
    Lowercase,
    #[darling(rename = "lowercase spaced")]
    LowercaseSpaced,
    #[darling(rename = "PascalCase")]
    PascalCase,
    #[darling(rename = "SCREAMING_SNAKE_CASE")]
    ScreamingSnakeCase,
    #[darling(rename = "snake_case")]
    SnakeCase,
    #[darling(rename = "UPPERCASE")]
    Uppercase,
    #[darling(rename = "UPPERCASE SPACED")]
    UppercaseSpaced,
}

impl Case {
    fn apply(&self, text: &str) -> String {
        match *self {
            Case::CamelCase => text.to_camel_case(),
            Case::KebabCase => text.to_kebab_case(),
            Case::Lowercase => text.to_lowercase(),
            Case::LowercaseSpaced => text.to_title_case().to_lowercase(),
            Case::PascalCase => text.to_mixed_case(),
            Case::ScreamingSnakeCase => text.to_shouty_snake_case(),
            Case::SnakeCase => text.to_snake_case(),
            Case::Uppercase => text.to_uppercase(),
            Case::UppercaseSpaced => text.to_title_case().to_uppercase(),
        }
    }
}

impl Default for Case {
    fn default() -> Self {
        Case::SnakeCase
    }
}

#[derive(Debug, FromVariant)]
#[darling(supports(unit))]
struct Variant {
    ident: Ident,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(symbol), supports(enum_any))]
struct SymbolInput {
    vis: Visibility,
    ident: Ident,
    #[darling(default)]
    rename_all: Case,
    data: ast::Data<Variant, ()>,
}

pub fn derive_symbol(ast: &DeriveInput) -> Result<TokenStream, Error> {
    let input = SymbolInput::from_derive_input(ast).expect("blah");
    let ty = input.ident.clone();
    let impls = gen_trait_impls(input)?;
    let output = wrap_in_dummy_const(ty, impls);
    Ok(output)
}

fn gen_trait_impls(input: SymbolInput) -> Result<TokenStream, Error> {
    let ty_name = input.ident;
    let str_case = input.rename_all;
    let variants = input
        .data
        .take_enum()
        .ok_or(Error::custom("symbol enum must have at least one variant"))?;

    let idents1 = variants.iter().map(|var| &var.ident);
    let idents2 = idents1.clone();

    let strs1 = idents1
        .clone()
        .map(|var| var.to_string())
        .map(|s| str_case.apply(&s));
    let strs2 = strs1.clone();

    let tokens = quote! {
        impl<'a> _mruby::symbol::FromSymbol<'a> for #ty_name {
            fn from_name(s: &'a str) -> ::std::result::Result<Self, _mruby::symbol::InvalidSymbolError> {
                use #ty_name::*;
                match s {
                    #( #strs2 => Ok(#idents2), )*
                    sym => Err(_mruby::symbol::InvalidSymbolError::new(sym)),
                }
            }
        }

        impl _mruby::symbol::ToSymbol for #ty_name {
            fn as_str(&self) -> &str {
                use #ty_name::*;
                match *self {
                    #( #idents1 => #strs1, )*
                }
            }
        }

        impl _mruby::de::FromValue for #ty_name {
            fn from_value(de: _mruby::de::Deserializer) -> ::std::result::Result<Self, _mruby::de::CastError> {
                de.deserialize_symbol()
            }
        }

        impl _mruby::ser::ToValue for #ty_name {
            fn to_value(&self, ser: _mruby::ser::Serializer) -> _mruby::Value {
                ser.serialize_symbol(self)
            }
        }
    };

    Ok(tokens)
}

fn wrap_in_dummy_const(ident: Ident, impls: TokenStream) -> TokenStream {
    let name = format!("_IMPL_SYMBOL_ENUM_FOR_{}", ident.to_string().to_uppercase());
    let dummy_const = Ident::new(&name, ident.span());
    quote! {
        const #dummy_const: () = {
            #[allow(unknown_lints)]
            #[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
            #[allow(rust_2018_idioms)]
            extern crate self as _mruby;

            #impls
        };
    }
}
