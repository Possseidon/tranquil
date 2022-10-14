use indoc::indoc;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, spanned::Spanned, AttributeArgs, FnArg, Ident, ImplItem, ItemFn, ItemImpl,
    Lit, LitStr, Meta, MetaNameValue, NestedMeta, PatType,
};

enum CommandPath {
    Command {
        name: String,
    },
    Subcommand {
        name: String,
        subcommand: String,
    },
    Grouped {
        name: String,
        group: String,
        subcommand: String,
    },
}

#[derive(Default)]
struct Attributes {
    rename: Option<CommandPath>,
}

trait CommandString: Spanned {
    fn to_command_string(&self) -> String;
}

impl CommandString for LitStr {
    fn to_command_string(&self) -> String {
        self.value()
    }
}

impl CommandString for Ident {
    fn to_command_string(&self) -> String {
        self.to_string()
    }
}

fn parse_command(
    rename: &impl CommandString,
    split_char: char,
) -> Result<CommandPath, TokenStream> {
    let command_string = rename.to_command_string();

    if command_string.is_empty() {
        Err(TokenStream::from(
            syn::Error::new(rename.span(), "commands cannot be empty").into_compile_error(),
        ))?
    }

    let parts = command_string.split(split_char).collect::<Vec<_>>();

    if parts.iter().any(|part| part.is_empty()) {
        Err(TokenStream::from(
            syn::Error::new(
                rename.span(),
                format!(
                    indoc! {r#"
                    invalid command name, valid command names are:
                        `command`
                        `command{}subcommand`
                        `command{}group{}subcommand`
                    "#},
                    split_char, split_char, split_char
                ),
            )
            .into_compile_error(),
        ))?;
    }

    match parts.as_slice() {
        [name, group, subcommand] => Ok(CommandPath::Grouped {
            name: name.to_string(),
            group: group.to_string(),
            subcommand: subcommand.to_string(),
        }),
        [name, subcommand] => Ok(CommandPath::Subcommand {
            name: name.to_string(),
            subcommand: subcommand.to_string(),
        }),
        [name] => Ok(CommandPath::Command {
            name: name.to_string(),
        }),
        _ => Err(TokenStream::from(
            syn::Error::new(
                rename.span(),
                "commands can only have two levels of nesting",
            )
            .into_compile_error(),
        ))?,
    }
}

fn invalid_attribute(span: &impl Spanned) -> TokenStream {
    syn::Error::new(
        span.span(),
        indoc! {r#"
            available attributes are
                `rename = "..."`
        "#},
    )
    .into_compile_error()
    .into()
}

fn invalid_rename_literal(span: &impl Spanned) -> TokenStream {
    syn::Error::new(span.span(), "string expected")
        .into_compile_error()
        .into()
}

fn multiple_renames(span: &impl Spanned) -> TokenStream {
    syn::Error::new(span.span(), "only one rename can be applied")
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
                    if ident.map_or(false, |ident| ident == "rename") {
                        match lit {
                            Lit::Str(lit_str) => {
                                if attributes.rename.is_some() {
                                    errors.push(multiple_renames(&nested_meta));
                                } else {
                                    match parse_command(lit_str, ' ') {
                                        Ok(command) => attributes.rename = Some(command),
                                        Err(error) => errors.push(error),
                                    }
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

    let command_path = attributes
        .rename
        .map_or_else(|| parse_command(&name, '_'), Ok)
        .unwrap_or_else(|error| {
            errors.push(error);
            CommandPath::Command {
                name: name.to_string(),
            }
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

    let make_command_path = |reference| {
        let command_path_or_ref = if reference {
            quote! { l10n::CommandPathRef }
        } else {
            quote! { command::CommandPath }
        };

        let to_string = if reference {
            quote! {}
        } else {
            quote! { .to_string() }
        };

        match &command_path {
            CommandPath::Command { name } => {
                quote! {
                    ::tranquil::#command_path_or_ref::Command {
                        name: #name #to_string
                    }
                }
            }
            CommandPath::Subcommand { name, subcommand } => quote! {
                ::tranquil::#command_path_or_ref::Subcommand {
                    name: #name #to_string,
                    subcommand: #subcommand #to_string,
                }
            },
            CommandPath::Grouped {
                name,
                group,
                subcommand,
            } => quote! {
                ::tranquil::#command_path_or_ref::Grouped {
                    name: #name #to_string,
                    group: #group #to_string,
                    subcommand: #subcommand #to_string,
                }
            },
        }
    };

    let command_path = make_command_path(false);
    let command_path_ref = make_command_path(true);

    let command_options = typed_parameters.clone().map(|PatType { pat, ty, .. }| {
        quote! {
            (|translated_commands: &::tranquil::l10n::TranslatedCommands| {
                let mut option = ::serenity::builder::CreateApplicationCommandOption::default();
                <#ty as ::tranquil::resolve::Resolve>::describe(
                    option
                        .kind(<#ty as ::tranquil::resolve::Resolve>::KIND)
                        .required(<#ty as ::tranquil::resolve::Resolve>::REQUIRED)
                );
                translated_commands.describe_option(#command_path_ref, ::std::stringify!(#pat), &mut option);
                option
            }) as fn(&::tranquil::l10n::TranslatedCommands) -> ::serenity::builder::CreateApplicationCommandOption
        }
    });

    let mut result = TokenStream::from(quote! {
        #item_fn

        fn #name(
            self: ::std::sync::Arc<Self>
        ) -> (::tranquil::command::CommandPath, ::std::boxed::Box<dyn ::tranquil::command::Command>) {
            (
                #command_path,
                ::std::boxed::Box::new(::tranquil::command::ModuleCommand::new(
                    ::std::boxed::Box::new(|ctx, interaction, module: ::std::sync::Arc<Self>| {
                        ::std::boxed::Box::pin(async move {
                            let options = ::tranquil::command::resolve_command_options(&interaction.data).iter();
                            #(#parameter_resolvers)*
                            module.#impl_name(
                                ::tranquil::command::CommandContext{ ctx, interaction },
                                #(#parameters),*
                            ).await
                        })
                    }),
                    ::std::vec![#(#command_options),*],
                    self,
                )),
            )
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
            fn command_map(
                self: ::std::sync::Arc<Self>,
            ) -> ::std::result::Result<::tranquil::command::CommandMap, ::tranquil::command::CommandMapMergeError> {
                ::tranquil::command::CommandMap::new([
                    #(Self::#commands(self.clone())),*
                ])
            }
        }
    })
}
