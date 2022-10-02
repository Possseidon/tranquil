use convert_case::{Case, Casing};
use indoc::indoc;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, spanned::Spanned, AttributeArgs, FnArg, ImplItem, ItemFn, ItemImpl, Lit,
    Meta, MetaNameValue, NestedMeta, PatType,
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
    let mut errors = vec![];

    let nested_metas = parse_macro_input!(attr as AttributeArgs);

    let mut item_fn = parse_macro_input!(item as ItemFn);
    let name = item_fn.sig.ident;
    let impl_name = format_ident!("__{name}");
    item_fn.sig.ident = impl_name.clone();

    let attributes = {
        let mut attributes = Attributes::default();
        for nested_meta in nested_metas.iter() {
            match nested_meta {
                NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. })) => {
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

    let command_name = attributes
        .rename
        .map_or(name.to_string(), |rename| match rename {
            Rename::ConvertCase(case) => name.to_string().to_case(case),
            Rename::To(name) => name,
        });

    let typed_parameters = item_fn
        .sig
        .inputs
        .iter()
        .skip(2) // TODO: Don't just skip self, CommandInteraction.
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
            (|translated_commands: &::tranquil::l10n::TranslatedCommands| {
                let mut option = ::serenity::builder::CreateApplicationCommandOption::default();
                <#ty as ::tranquil::resolve::Resolve>::describe(
                    option
                        .kind(<#ty as ::tranquil::resolve::Resolve>::KIND)
                        .name(::std::stringify!(#pat))
                        .required(<#ty as ::tranquil::resolve::Resolve>::REQUIRED)
                );
                translated_commands.describe_option(stringify!(#name), stringify!(#pat), &mut option);
                option
            }) as fn(&::tranquil::l10n::TranslatedCommands) -> ::serenity::builder::CreateApplicationCommandOption
        }
    });

    let mut result = TokenStream::from(quote! {
        #item_fn

        fn #name(
            self: ::std::sync::Arc<Self>
        ) -> ::std::boxed::Box<dyn ::tranquil::command::CommandImpl> {
            ::std::boxed::Box::new(::tranquil::command::Command::new(
                #command_name,
                ::std::boxed::Box::new(|ctx, interaction, module: ::std::sync::Arc<Self>| {
                    ::std::boxed::Box::pin(async move {
                        let options = interaction.data.options.iter();
                        #(#parameter_resolvers)*
                        module.#impl_name(
                            ::tranquil::command::CommandContext{ ctx, interaction },
                            #(#parameters),*,
                        ).await
                    })
                }),
                ::std::vec![#(#command_options),*],
                self,
            ))
        }
    });
    result.extend(errors);
    result
}

#[proc_macro_attribute]
pub fn command_provider(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_item = parse_macro_input!(item as ItemImpl);
    let type_name = &impl_item.self_ty;

    let commands = impl_item.items.iter().filter_map(|item| match item {
        ImplItem::Method(impl_item_method) => Some(&impl_item_method.sig.ident),
        _ => None,
    });

    TokenStream::from(quote! {
        #impl_item

        impl ::tranquil::command::CommandProvider for #type_name {
            fn commands(
                self: ::std::sync::Arc<Self>,
            ) -> ::tranquil::command::Commands {
                ::std::vec![#(Self::#commands(self.clone())),*]
            }
        }
    })
}
