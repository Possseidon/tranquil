use convert_case::{Case, Casing};
use indoc::indoc;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, spanned::Spanned, AttributeArgs, FnArg, ItemFn, Lit, MetaNameValue,
    NestedMeta, PatType,
};

enum Rename {
    ConvertCase(Case),
    To(String),
}

#[derive(Default)]
struct Attributes {
    rename: Option<Rename>,
}

fn invalid_attribute(span: impl Spanned) -> TokenStream {
    syn::Error::new(
        span.span(),
        indoc! {r#"
            available attributes are
                `case = "..."`
                `rename = "..."`
        "#},
    )
    .into_compile_error()
    .into()
}

fn parse_case(case: &str) -> Option<Case> {
    // Discord does not allow uppercase names.
    match case {
        // "camel" => Some(Case::Camel),
        // "pascal" => Some(Case::Pascal),
        // "upper-camel" => Some(Case::UpperCamel),
        "snake" => Some(Case::Snake),
        // "upper-snake" => Some(Case::UpperSnake),
        // "screaming-snake" => Some(Case::ScreamingSnake),
        "kebab" => Some(Case::Kebab),
        // "cobol" => Some(Case::Cobol),
        // "upper-kebab" => Some(Case::UpperKebab),
        // "train" => Some(Case::Train),
        "flat" => Some(Case::Flat),
        // "upper-flat" => Some(Case::UpperFlat),
        _ => None,
    }
}

fn invalid_case(name: &str, span: impl Spanned) -> TokenStream {
    syn::Error::new(
        span.span(),
        format!(
            indoc! {r#"
                available case transformations are
                    `case = "snake"`           -> {}
                    `case = "kebab"`           -> {}
                    `case = "flat"`            -> {}
            "#},
            // name.to_case(Case::Camel),
            // name.to_case(Case::Pascal),
            // name.to_case(Case::UpperCamel),
            name.to_case(Case::Snake),
            // name.to_case(Case::UpperSnake),
            // name.to_case(Case::ScreamingSnake),
            name.to_case(Case::Kebab),
            // name.to_case(Case::Cobol),
            // name.to_case(Case::UpperKebab),
            // name.to_case(Case::Train),
            name.to_case(Case::Flat),
            // name.to_case(Case::UpperFlat),
        ),
    )
    .into_compile_error()
    .into()
}

fn invalid_rename_literal(span: impl Spanned) -> TokenStream {
    syn::Error::new(
        span.span(),
        indoc! {r#"
            string expected
        "#},
    )
    .into_compile_error()
    .into()
}

fn multiple_renames(span: impl Spanned) -> TokenStream {
    syn::Error::new(
        span.span(),
        indoc! {r#"
            only one rename or case transformation can be applied
        "#},
    )
    .into_compile_error()
    .into()
}

#[proc_macro_attribute]
pub fn slash(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut errors = Vec::new();

    let attr = parse_macro_input!(attr as AttributeArgs);

    let mut item = parse_macro_input!(item as ItemFn);
    let name = item.sig.ident;
    let impl_name = format_ident!("__{name}");
    item.sig.ident = impl_name.clone();

    let attributes = {
        let mut attributes = Attributes::default();
        for nested_meta in attr.iter() {
            match nested_meta {
                NestedMeta::Meta(syn::Meta::NameValue(MetaNameValue { path, lit, .. })) => {
                    let ident = path.get_ident();
                    if ident.map_or(false, |ident| ident == "case") {
                        match lit {
                            Lit::Str(lit_str) => {
                                if let Some(case) = parse_case(&lit_str.value()) {
                                    if attributes.rename.is_some() {
                                        errors.push(multiple_renames(&nested_meta));
                                    } else {
                                        attributes.rename = Some(Rename::ConvertCase(case));
                                    }
                                } else {
                                    errors.push(invalid_case(&name.to_string(), &lit_str));
                                }
                            }
                            _ => errors.push(invalid_case(&name.to_string(), &lit)),
                        }
                    } else if ident.map_or(false, |ident| ident == "rename") {
                        match lit {
                            Lit::Str(lit_str) => {
                                if attributes.rename.is_some() {
                                    errors.push(multiple_renames(&nested_meta));
                                } else {
                                    attributes.rename = Some(Rename::To(lit_str.value()));
                                }
                            }
                            _ => errors.push(invalid_rename_literal(&lit)),
                        }
                    } else {
                        errors.push(invalid_attribute(&nested_meta));
                    }
                }
                _ => {
                    errors.push(invalid_attribute(&nested_meta));
                }
            }
        }
        attributes
    };

    let slash_command_name = attributes
        .rename
        .map_or(name.to_string(), |rename| match rename {
            Rename::ConvertCase(case) => name.to_string().to_case(case),
            Rename::To(name) => name,
        });

    let typed_parameters = item
        .sig
        .inputs
        .iter()
        .skip(3) // TODO: Don't just skip self, ctx and interaction.
        .filter_map(|input| match input {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_type) => Some(pat_type),
        });

    let parameters = typed_parameters.clone().map(|PatType { pat, .. }| pat);

    let parameter_resolvers = typed_parameters.clone().map(|PatType { pat, ty, .. }| {
        quote! {
            let #pat = <#ty as ::tranquil::resolve::Resolve>::resolve(
                ::std::stringify!(#pat),
                options.clone(),
            )?;
        }
    });

    let command_options = typed_parameters.clone().map(|PatType { pat, ty, .. }| {
        quote! {
            (|| {
                let mut option = ::serenity::builder::CreateApplicationCommandOption::default();
                <#ty as ::tranquil::resolve::Resolve>::describe(
                    option
                        .kind(<#ty as ::tranquil::resolve::Resolve>::KIND)
                        .name(::std::stringify!(#pat))
                        .required(<#ty as ::tranquil::resolve::Resolve>::REQUIRED)
                );
                option
            }) as fn() -> ::serenity::builder::CreateApplicationCommandOption
        }
    });

    let expanded = quote! {
        #item

        fn #name(
            self: ::std::sync::Arc<Self>
        ) -> ::std::boxed::Box<dyn ::tranquil::slash_command::SlashCommandImpl> {
            ::std::boxed::Box::new(::tranquil::slash_command::SlashCommand::new(
                #slash_command_name,
                ::std::boxed::Box::new(|ctx, interaction, module: ::std::sync::Arc<Self>| {
                    ::std::boxed::Box::pin(async move {
                        let options = interaction.data.options.iter();
                        #(#parameter_resolvers)*
                        module.#impl_name(ctx, interaction, #(#parameters),*).await
                    })
                }),
                vec![#(#command_options),*],
                self,
            ))
        }
    };

    let mut result = TokenStream::from(expanded);
    result.extend(errors);
    result
}
