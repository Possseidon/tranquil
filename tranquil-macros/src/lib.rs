use indoc::indoc;
use itertools::Itertools;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, spanned::Spanned, Expr, ExprLit,
    ExprPath, ExprRange, FnArg, Ident, ImplItem, ItemEnum, ItemFn, ItemImpl, ItemStruct, Lit,
    LitChar, LitStr, Meta, MetaNameValue, PatType, RangeLimits, Token, TypePath,
};

// TODO: Use explicit trait methods in all quote! macros.

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Autocomplete {
    DefaultName,
    CustomName(Ident),
}

// TODO: Consider implementing syn's Parse trait for this

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct SlashAttributes<'a> {
    default: Option<&'a Ident>,
    rename: Option<CommandPath>,
    autocomplete: Option<Autocomplete>,
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
                `default`
                `rename = "..."`
        "#},
    )
    .into_compile_error()
    .into()
}

fn invalid_rename_literal(span: &impl Spanned) -> TokenStream {
    syn::Error::new(span.span(), "expected string")
        .into_compile_error()
        .into()
}

fn multiple_renames(span: &impl Spanned) -> TokenStream {
    syn::Error::new(span.span(), "only one rename can be applied")
        .into_compile_error()
        .into()
}

fn default_on_base_command(span: &impl Spanned) -> TokenStream {
    syn::Error::new(span.span(), "only subcommands can be `default`")
        .into_compile_error()
        .into()
}

fn multiple_autocompletes(span: &impl Spanned) -> TokenStream {
    syn::Error::new(
        span.span(),
        "only one autocomplete function can be specified",
    )
    .into_compile_error()
    .into()
}

fn invalid_autocomplete_ident(span: &impl Spanned) -> TokenStream {
    syn::Error::new(span.span(), "expected identifier")
        .into_compile_error()
        .into()
}

#[proc_macro_attribute]
pub fn slash(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut errors = vec![];

    let nested_metas =
        parse_macro_input!(attr with Punctuated::<Meta, Token![,]>::parse_terminated);

    let mut item_fn = parse_macro_input!(item as ItemFn);
    let name = item_fn.sig.ident;
    let impl_name = format_ident!("__{name}");
    item_fn.sig.ident = impl_name.clone();

    let attributes = {
        let mut attributes = SlashAttributes::default();
        for nested_meta in &nested_metas {
            match nested_meta {
                Meta::NameValue(MetaNameValue { path, value, .. }) => {
                    let ident = path.get_ident();
                    if ident.map_or(false, |ident| ident == "rename") {
                        match value {
                            Expr::Lit(ExprLit {
                                lit: Lit::Str(lit_str),
                                ..
                            }) => {
                                if attributes.rename.is_some() {
                                    errors.push(multiple_renames(&nested_meta));
                                } else {
                                    match parse_command(lit_str, ' ') {
                                        Ok(command) => attributes.rename = Some(command),
                                        Err(error) => errors.push(error),
                                    }
                                }
                            }
                            _ => errors.push(invalid_rename_literal(&value)),
                        }
                    } else if ident.map_or(false, |ident| ident == "autocomplete") {
                        match value {
                            Expr::Lit(ExprLit {
                                lit: Lit::Str(lit_str),
                                ..
                            }) => {
                                if attributes.autocomplete.is_some() {
                                    errors.push(multiple_autocompletes(&nested_meta));
                                } else {
                                    match lit_str.parse_with(syn::Ident::parse) {
                                        Ok(ident) => {
                                            attributes.autocomplete =
                                                Some(Autocomplete::CustomName(ident))
                                        }
                                        Err(_) => errors.push(invalid_autocomplete_ident(&value)),
                                    }
                                }
                            }
                            _ => errors.push(invalid_autocomplete_ident(&value)),
                        }
                    } else {
                        errors.push(invalid_attribute(&nested_meta));
                    }
                }
                Meta::Path(path) => {
                    let ident = path.get_ident();
                    if ident.map_or(false, |ident| ident == "default") {
                        attributes.default = ident;
                    } else if ident.map_or(false, |ident| ident == "autocomplete") {
                        attributes.autocomplete = Some(Autocomplete::DefaultName);
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

    if let (Some(ident), CommandPath::Command { .. }) = (attributes.default, &command_path) {
        errors.push(default_on_base_command(ident));
    }

    let typed_parameters = item_fn
        .sig
        .inputs
        .iter()
        .skip(2) // TODO: Don't just skip &self and CommandCtx.
        .filter_map(|input| match input {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_type) => Some(pat_type),
        });

    let parameters = typed_parameters
        .clone()
        .map(|PatType { pat, .. }| pat)
        .collect::<Vec<_>>();

    let parameter_names = parameters
        .iter()
        .map(|parameter| quote! { ::std::stringify!(#parameter) });

    let parameter_resolvers = typed_parameters.clone().map(|PatType { ty, .. }| {
        quote! {
            <#ty as ::tranquil::resolve::Resolve>::resolve(
                ::tranquil::resolve::ResolveContext {
                    // Technically unwrap instead of flatten would also work, but better safe than sorry.
                    option: options.next().flatten(),
                    http: ctx.bot.http.clone(),
                },
            )
        }
    });

    let join_futures = if parameters.is_empty() {
        quote! {}
    } else {
        quote! {
            let (#(#parameters),*,) = ::tranquil::serenity::futures::try_join!(#(#parameter_resolvers),*)?;
        }
    };

    let autocompleter = if let Some(autocomplete) = attributes.autocomplete {
        let autocompleter_name = match autocomplete {
            Autocomplete::DefaultName => format_ident!("autocomplete_{name}"),
            Autocomplete::CustomName(name) => format_ident!("{name}"),
        };
        quote! {
            ::std::option::Option::Some(
                ::std::boxed::Box::new(|module, ctx| {
                    ::std::boxed::Box::pin(async move {
                        module.#autocompleter_name(ctx).await
                    })
                })
            )
        }
    } else {
        quote! { ::std::option::Option::None }
    };

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

    let command_options = typed_parameters.map(|PatType { pat, ty, .. }| {
        quote! {
            (
                ::std::convert::From::from(::std::stringify!(#pat)),
                (|l10n: &::tranquil::l10n::L10n| {
                    let mut option = ::tranquil::serenity::builder::CreateApplicationCommandOption::default();
                    <#ty as ::tranquil::resolve::Resolve>::describe(
                        option
                            .kind(<#ty as ::tranquil::resolve::Resolve>::KIND)
                            .required(<#ty as ::tranquil::resolve::Resolve>::REQUIRED),
                        l10n,
                    );
                    // TODO: This can technically be done outside of the macro, now that the name is accessible there.
                    l10n.describe_command_option(#command_path_ref, ::std::stringify!(#pat), &mut option);
                    option
                }) as fn(&::tranquil::l10n::L10n) -> ::tranquil::serenity::builder::CreateApplicationCommandOption,
            )
        }
    });

    let is_default_option = attributes.default.is_some();

    let mut result = TokenStream::from(quote! {
        #item_fn

        fn #name(
            self: ::std::sync::Arc<Self>
        ) -> (::tranquil::command::CommandPath, ::std::boxed::Box<dyn ::tranquil::command::Command>) {
            (
                #command_path,
                ::std::boxed::Box::new(::tranquil::command::ModuleCommand::new(
                    self,
                    ::std::boxed::Box::new(|module, mut ctx| {
                        ::std::boxed::Box::pin(async move {
                            let mut options = ::tranquil::resolve::find_options(
                                [#(#parameter_names),*],
                                ::tranquil::resolve::resolve_command_options(
                                    ctx.interaction.data.options.clone() // TODO: avoid clone?
                                ),
                            ).into_iter();
                            #join_futures
                            module.#impl_name(ctx, #(#parameters),*).await
                        })
                    }),
                    #autocompleter,
                    ::std::vec![#(#command_options),*],
                    #is_default_option,
                )),
            )
        }
    });
    result.extend(errors);
    result
}

#[proc_macro_attribute]
pub fn autocompleter(attr: TokenStream, item: TokenStream) -> TokenStream {
    // TODO: Deduplicate code

    let mut errors = vec![];

    let nested_metas =
        parse_macro_input!(attr with Punctuated::<Meta, Token![,]>::parse_terminated);

    if let Some(meta) = nested_metas.first() {
        errors.push(TokenStream::from(
            syn::Error::new(meta.span(), "autocomplete does not support any parameters")
                .to_compile_error(),
        ))
    }

    let mut item_fn = parse_macro_input!(item as ItemFn);
    let name = item_fn.sig.ident;
    let impl_name = format_ident!("__{name}");
    item_fn.sig.ident = impl_name.clone();

    let typed_parameters = item_fn
        .sig
        .inputs
        .iter()
        .skip(2) // TODO: Don't just skip &self and AutocompleteCtx.
        .filter_map(|input| match input {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_type) => Some(pat_type),
        });

    let parameters = typed_parameters
        .clone()
        .map(|PatType { pat, .. }| pat)
        .collect::<Vec<_>>();

    let parameter_names = parameters
        .iter()
        .map(|parameter| quote! { ::std::stringify!(#parameter) });

    let parameter_resolvers = typed_parameters.map(|PatType { ty, .. }| {
        quote! {
            <#ty as ::tranquil::resolve::Resolve>::resolve(
                ::tranquil::resolve::ResolveContext {
                    // Technically unwrap instead of flatten would also work, but better safe than sorry.
                    option: options.next().flatten(),
                    http: ctx.bot.http.clone(),
                },
            )
        }
    });

    let join_futures = if parameters.is_empty() {
        quote! {}
    } else {
        quote! {
            let (#(#parameters),*,) = ::tranquil::serenity::futures::try_join!(#(#parameter_resolvers),*)?;
        }
    };

    let mut result = TokenStream::from(quote! {
        #item_fn

        async fn #name(
            &self,
            mut ctx: ::tranquil::context::AutocompleteCtx,
        ) -> ::tranquil::anyhow::Result<()> {
            let mut options = ::tranquil::resolve::find_options(
                [#(#parameter_names),*],
                ::tranquil::resolve::resolve_command_options(
                    ctx.interaction.data.options.clone() // TODO: avoid clone?
                ),
            ).into_iter();
            #join_futures
            self.#impl_name(ctx, #(#parameters),*).await
        }
    });
    result.extend(errors);
    result
}

#[proc_macro_attribute]
pub fn command_provider(attr: TokenStream, item: TokenStream) -> TokenStream {
    // TODO: Deduplicate code

    let nested_metas =
        parse_macro_input!(attr with Punctuated::<Meta, Token![,]>::parse_terminated);

    let mut errors = vec![];

    if let Some(meta) = nested_metas.first() {
        errors.push(TokenStream::from(
            syn::Error::new(
                meta.span(),
                "command_provider does not support any parameters",
            )
            .to_compile_error(),
        ))
    }

    let impl_item = parse_macro_input!(item as ItemImpl);
    let type_name = &impl_item.self_ty;

    let commands = impl_item.items.iter().filter_map(|item| match item {
        ImplItem::Fn(impl_item_method) => Some(&impl_item_method.sig.ident),
        _ => None,
    });

    let mut result = TokenStream::from(quote! {
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
    });
    result.extend(errors);
    result
}

#[proc_macro_derive(Choices)]
pub fn derive_choices(item: TokenStream) -> TokenStream {
    // TODO: Better error messages for unsupported enums.

    let enum_item = parse_macro_input!(item as ItemEnum);
    let name = enum_item.ident;
    let variants = enum_item.variants;

    let choices = variants.iter().map(|variant| {
        let name = &variant.ident;
        quote! {
            ::tranquil::resolve::Choice {
                name: ::std::convert::From::from(::std::stringify!(#name)),
                value: ::std::convert::From::from(::std::stringify!(#name)),
            }
        }
    });

    let resolvers = variants.iter().map(|variant| {
        let name = &variant.ident;
        quote! {
            ::std::stringify!(#name) => ::std::option::Option::Some(Self::#name),
        }
    });

    quote! {
        impl ::tranquil::resolve::Choices for #name {
            fn name() -> ::std::string::String {
                ::std::convert::From::from(::std::stringify!(#name))
            }

            fn choices() -> ::std::vec::Vec<::tranquil::resolve::Choice> {
                ::std::vec![#(#choices),*]
            }

            fn resolve(option: ::std::string::String) -> ::std::option::Option<Self> {
                match ::std::convert::AsRef::as_ref(&option) {
                    #(#resolvers)*
                    _ => ::std::option::Option::None,
                }
            }
        }
    }
    .into()
}

#[proc_macro_derive(Module)]
pub fn derive_module(item: TokenStream) -> TokenStream {
    let struct_item = parse_macro_input!(item as ItemStruct);
    let name = struct_item.ident;

    quote! { impl ::tranquil::module::Module for #name {} }.into()
}

fn missing_uuid(span: &impl Spanned, uuid_attr_name: &str) -> TokenStream {
    syn::Error::new(
        span.span(),
        format!(r#"missing uuid: `#[{uuid_attr_name}("00000000-0000-0000-0000-000000000000")]`"#),
    )
    .into_compile_error()
    .into()
}

fn multiple_uuids(span: &impl Spanned) -> TokenStream {
    syn::Error::new(span.span(), "only one uuid must be specified")
        .into_compile_error()
        .into()
}

fn invalid_uuid(span: &impl Spanned, uuid_attr_name: &str) -> TokenStream {
    syn::Error::new(
        span.span(),
        format!(r#"uuid must be a string: `#[{uuid_attr_name}("00000000-0000-0000-0000-000000000000")]`"#),
    )
    .into_compile_error()
    .into()
}

fn missing_module(span: &impl Spanned) -> TokenStream {
    syn::Error::new(span.span(), "missing module: `#[module(MyModule)]`")
        .into_compile_error()
        .into()
}

fn multiple_modules(span: &impl Spanned) -> TokenStream {
    syn::Error::new(span.span(), "only one module must be specified")
        .into_compile_error()
        .into()
}

fn invalid_module(span: &impl Spanned) -> TokenStream {
    syn::Error::new(
        span.span(),
        r#"module must be a type: `#[module(MyModule)]`"#,
    )
    .into_compile_error()
    .into()
}

fn missing_option(span: &impl Spanned) -> TokenStream {
    syn::Error::new(span.span(), r#"missing option: `#[option("My Option")]`"#)
        .into_compile_error()
        .into()
}

fn multiple_options(span: &impl Spanned) -> TokenStream {
    syn::Error::new(span.span(), "only one option must be specified per variant")
        .into_compile_error()
        .into()
}

fn invalid_option(span: &impl Spanned) -> TokenStream {
    syn::Error::new(
        span.span(),
        indoc! {r#"
            valid options:
                `#[option("label")]`
                `#[option("label", default)]`
                `#[option("label", "description")]`
                `#[option("label", "description", default)]`
                `#[option('🟥', "label")]`
                `#[option('🟥', "label", default)]`
                `#[option('🟥', "label", "description")]`
                `#[option('🟥', "label", "description", default)]`
        "#},
    )
    .into_compile_error()
    .into()
}

fn multiple_default_options(span: &impl Spanned) -> TokenStream {
    syn::Error::new(span.span(), "only one option can be the default")
        .into_compile_error()
        .into()
}

fn multiple_options_count(span: &impl Spanned) -> TokenStream {
    syn::Error::new(span.span(), "only one option count must be specified")
        .into_compile_error()
        .into()
}

fn invalid_options_count(span: &impl Spanned) -> TokenStream {
    syn::Error::new(
        span.span(),
        indoc! {r#"
            options count must be an integer or range:
                `#[options_count(4)]`
                `#[options_count(2..=5)]`
                `#[options_count(..=3)]`
                `#[options_count(3..=)]`
        "#},
    )
    .into_compile_error()
    .into()
}

#[proc_macro_derive(SelectMenuChoice, attributes(choice_uuid, module, option))]
pub fn derive_select_menu_choice(item: TokenStream) -> TokenStream {
    let mut errors: Vec<TokenStream> = vec![];

    let enum_item = parse_macro_input!(item as ItemEnum);

    let name = &enum_item.ident;

    let SelectMenu {
        module,
        uuid,
        default_option,
        options,
    } = match parse_select_menu_enum(&enum_item, "choice_uuid", &mut errors) {
        Ok(menu) => menu,
        Err(error) => return error,
    };

    let choices = options
        .iter()
        .map(|SelectMenuOption { name, .. }| quote! { Self::#name, });

    let default_choice = if let Some(name) = default_option {
        quote! { ::std::option::Option::Some(Self::#name) }
    } else {
        quote! { ::std::option::Option::None }
    };

    let map_to_select_menu_option = options.iter().map(
        |SelectMenuOption {
             name,
             emoji,
             label,
             description,
             default,
         }| {
            let emoji = if let Some(emoji) = emoji {
                quote! { ::std::option::Option::Some(#emoji.into()) }
            } else {
                quote! { ::std::option::Option::None }
            };

            let description = if let Some(description) = description {
                quote! { #description }
            } else {
                quote! { "" }
            };

            quote! {
                Self::#name => SelectMenuOption {
                    emoji: #emoji,
                    label: #label.to_string(),
                    description: #description.to_string(),
                    default: #default,
                },
            }
        },
    );

    let custom_id_map = options
        .iter()
        .enumerate()
        .map(|(i, SelectMenuOption { name, .. })| {
            let i = i.to_string();
            quote! { #i => ::std::result::Result::Ok(Self::#name), }
        });

    let mut result = TokenStream::from(quote! {
        impl ::tranquil::select_menu::SelectMenuChoice for #name {
            type Module = #module;

            const UUID: Uuid = uuid!(#uuid);
            const CHOICES: &'static [Self] = &[ #( #choices )* ];
            const DEFAULT_CHOICE: ::std::option::Option<Self> = #default_choice;

            fn create() -> ::tranquil::serenity::builder::CreateSelectMenu {
                ::tranquil::select_menu::SelectMenu::new(
                    Self::CHOICES.iter().map(|choice| match choice {
                        #( #map_to_select_menu_option )*
                    })
                )
                .create(&::tranquil::select_menu::SelectHandler::<Self>::default())
            }

            fn from_value(value: &str) -> ::tranquil::anyhow::Result<Self> {
                match value {
                    #( #custom_id_map )*
                    _ => ::tranquil::anyhow::bail!("invalid select menu value"),
                }
            }
        }
    });
    result.extend(errors);
    result
}

#[proc_macro_derive(SelectMenuOptions, attributes(options_uuid, options_count, option))]
pub fn derive_select_menu_options(item: TokenStream) -> TokenStream {
    let mut errors: Vec<TokenStream> = vec![];

    let enum_item = parse_macro_input!(item as ItemEnum);

    let name = &enum_item.ident;

    let SelectMenu {
        module,
        uuid,
        default_option,
        options,
    } = match parse_select_menu_enum(&enum_item, "options_uuid", &mut errors) {
        Ok(menu) => menu,
        Err(error) => return error,
    };

    let mut options_count_attr = None;
    for attr in &enum_item.attrs {
        if attr.meta.path().is_ident("options_count") {
            if options_count_attr.is_none() {
                options_count_attr = Some(attr);
            } else {
                errors.push(multiple_options_count(attr));
            }
        }
    }

    let (min_count, max_count) = match options_count_attr {
        Some(options_range_attr) => {
            let Ok(options_count_expr) = options_range_attr.parse_args::<Expr>() else {
                return invalid_options_count(options_range_attr);
            };

            match options_count_expr {
                Expr::Lit(ExprLit {
                    lit: Lit::Int(int), ..
                }) => (Some(int.clone()), Some(int)),
                Expr::Range(ExprRange {
                    start,
                    limits: RangeLimits::Closed(_),
                    end,
                    ..
                }) => (
                    match start {
                        Some(value) => match *value {
                            Expr::Lit(ExprLit {
                                lit: Lit::Int(int), ..
                            }) => Some(int),
                            _ => return invalid_options_count(&options_count_attr),
                        },
                        None => None,
                    },
                    match end {
                        Some(value) => match *value {
                            Expr::Lit(ExprLit {
                                lit: Lit::Int(int), ..
                            }) => Some(int),
                            _ => return invalid_options_count(&options_count_attr),
                        },
                        None => None,
                    },
                ),
                _ => return invalid_options_count(&options_count_attr),
            }
        }
        None => (None, None),
    };

    let default_choice = if let Some(name) = default_option {
        quote! { ::std::option::Option::Some(Self::#name) }
    } else {
        quote! { ::std::option::Option::None }
    };

    let map_to_select_menu_option = options.iter().map(
        |SelectMenuOption {
             name,
             emoji,
             label,
             description,
             default,
         }| {
            let emoji = if let Some(emoji) = emoji {
                quote! { ::std::option::Option::Some(#emoji.into()) }
            } else {
                quote! { ::std::option::Option::None }
            };

            let description = if let Some(description) = description {
                quote! { #description }
            } else {
                quote! { "" }
            };

            quote! {
                Self::#name => SelectMenuOption {
                    emoji: #emoji,
                    label: #label.to_string(),
                    description: #description.to_string(),
                    default: #default,
                },
            }
        },
    );

    let custom_id_map = options
        .iter()
        .enumerate()
        .map(|(i, SelectMenuOption { name, .. })| {
            let i = i.to_string();
            quote! { #i => ::std::result::Result::Ok(Self::#name), }
        });

    let min_count = if let Some(min_count) = min_count {
        quote! { #min_count }
    } else {
        quote! { 0 }
    };
    let max_count = if let Some(max_count) = max_count {
        quote! { #max_count }
    } else {
        quote! { EnumSet::<Self>::variant_count().into() }
    };

    let mut result = TokenStream::from(quote! {
        impl ::tranquil::select_menu::SelectMenuOptions for #name {
            type Module = #module;

            const UUID: Uuid = uuid!(#uuid);
            const DEFAULT_OPTION: ::std::option::Option<Self> = #default_choice;

            fn create_multi() -> ::tranquil::serenity::builder::CreateSelectMenu {
                ::tranquil::select_menu::SelectMenu::new(
                    ::tranquil::enumset::EnumSet::<Self>::all().iter().map(|choice| match choice {
                        #( #map_to_select_menu_option )*
                    })
                )
                .min_values(#min_count)
                .max_values(#max_count)
                .create(&::tranquil::select_menu::MultiSelectHandler::<Self>::default())
            }

            fn from_values(values: &[String]) -> ::tranquil::anyhow::Result<::tranquil::enumset::EnumSet<Self>> {
                values.iter()
                    .map(|option| match option.as_str() {
                        #( #custom_id_map )*
                        _ => ::tranquil::anyhow::bail!("invalid select menu value"),
                    })
                    .collect()
            }
        }
    });
    result.extend(errors);
    result
}

struct SelectMenu<'a> {
    module: TypePath,
    uuid: LitStr,
    default_option: Option<&'a Ident>,
    options: Vec<SelectMenuOption<'a>>,
}

struct SelectMenuOption<'a> {
    name: &'a Ident,
    emoji: Option<LitChar>,
    label: LitStr,
    description: Option<LitStr>,
    default: bool,
}

fn parse_select_menu_enum<'a>(
    enum_item: &'a ItemEnum,
    uuid_attr_name: &str,
    errors: &mut Vec<TokenStream>,
) -> Result<SelectMenu<'a>, TokenStream> {
    let mut uuid_attr = None;
    for attr in &enum_item.attrs {
        if attr.meta.path().is_ident(uuid_attr_name) {
            if uuid_attr.is_none() {
                uuid_attr = Some(attr);
            } else {
                errors.push(multiple_uuids(attr));
            }
        }
    }

    let Some(uuid_attr) = uuid_attr else {
        return Err(missing_uuid(&enum_item.ident, uuid_attr_name));
    };

    let Ok(uuid) = uuid_attr.parse_args::<LitStr>() else {
        return Err(invalid_uuid(uuid_attr.meta.path(), uuid_attr_name));
    };

    let mut module_attr = None;
    for attr in &enum_item.attrs {
        if attr.meta.path().is_ident("module") {
            if module_attr.is_none() {
                module_attr = Some(attr);
            } else {
                errors.push(multiple_modules(attr));
            }
        }
    }

    let Some(module_attr) = module_attr else {
        return Err(missing_module(&enum_item.ident));
    };

    let Ok(module) = module_attr.parse_args::<TypePath>() else {
        return Err(invalid_module(module_attr.meta.path()));
    };

    let mut default_option = None;
    let mut options = vec![];

    for variant in &enum_item.variants {
        let mut option_attr = None;

        for attr in &variant.attrs {
            if attr.meta.path().is_ident("option") {
                if option_attr.is_none() {
                    option_attr = Some(attr);
                } else {
                    errors.push(multiple_options(attr));
                }
            }
        }

        let Some(option_attr) = option_attr else {
            return Err(missing_option(variant));
        };

        let Ok(lit_list) =
            option_attr.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)
        else {
            return Err(invalid_option(option_attr));
        };

        let lits = lit_list.iter().collect_vec();
        let (emoji, label, description, default) = match &lits[..] {
            [Expr::Lit(ExprLit {
                lit: Lit::Str(label),
                ..
            })] => (None, label, None, false),

            [Expr::Lit(ExprLit {
                lit: Lit::Str(label),
                ..
            }), Expr::Path(ExprPath {
                path: default_path, ..
            })] if default_path.is_ident("default") => (None, label, None, true),

            [Expr::Lit(ExprLit {
                lit: Lit::Str(label),
                ..
            }), Expr::Lit(ExprLit {
                lit: Lit::Str(description),
                ..
            })] => (None, label, Some(description), false),

            [Expr::Lit(ExprLit {
                lit: Lit::Str(label),
                ..
            }), Expr::Lit(ExprLit {
                lit: Lit::Str(description),
                ..
            }), Expr::Path(ExprPath {
                path: default_path, ..
            })] if default_path.is_ident("default") => (None, label, Some(description), true),

            [Expr::Lit(ExprLit {
                lit: Lit::Char(emoji),
                ..
            }), Expr::Lit(ExprLit {
                lit: Lit::Str(label),
                ..
            })] => (Some(emoji), label, None, false),

            [Expr::Lit(ExprLit {
                lit: Lit::Char(emoji),
                ..
            }), Expr::Lit(ExprLit {
                lit: Lit::Str(label),
                ..
            }), Expr::Path(ExprPath {
                path: default_path, ..
            })] if default_path.is_ident("default") => (Some(emoji), label, None, true),

            [Expr::Lit(ExprLit {
                lit: Lit::Char(emoji),
                ..
            }), Expr::Lit(ExprLit {
                lit: Lit::Str(label),
                ..
            }), Expr::Lit(ExprLit {
                lit: Lit::Str(description),
                ..
            })] => (Some(emoji), label, Some(description), false),

            [Expr::Lit(ExprLit {
                lit: Lit::Char(emoji),
                ..
            }), Expr::Lit(ExprLit {
                lit: Lit::Str(label),
                ..
            }), Expr::Lit(ExprLit {
                lit: Lit::Str(description),
                ..
            }), Expr::Path(ExprPath {
                path: default_path, ..
            })] if default_path.is_ident("default") => {
                (Some(emoji), label, Some(description), true)
            }

            _ => return Err(invalid_option(option_attr)),
        };

        if default {
            if default_option.is_none() {
                default_option = Some(&variant.ident);
            } else {
                return Err(multiple_default_options(&lit_list));
            }
        }

        options.push(SelectMenuOption {
            name: &variant.ident,
            emoji: emoji.cloned(),
            label: label.clone(),
            description: description.cloned(),
            default,
        });
    }

    Ok(SelectMenu {
        module,
        uuid,
        default_option,
        options,
    })
}
