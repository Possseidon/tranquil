// mod attributes;
// mod command;
// mod localization;

use proc_macro::TokenStream;

#[proc_macro_derive(Command, attributes(tranquil))]
pub fn derive_command(input: TokenStream) -> TokenStream {
    Default::default()

    // Command::from_derive_input(&parse_macro_input!(input as DeriveInput)).map_or_else(
    //     |error| error.write_errors().into(),
    //     |command_meta| command_meta.into_impl_command().flatten_error().into(),
    // )
}

/*

impl Command {
    fn into_impl_command(self) -> syn::Result<proc_macro2::TokenStream> {
        let type_name = &self.ident;

        let NameDescriptionBuilt {
            name,
            description,
            localizations,
        } = NameDescription::parse(&self.attrs, type_name, None)?.build();

        let nsfw = self.nsfw.then(|| quote! { .nsfw(true) }).into_iter();

        let (options, resolve_command, autocomplete, resolve_autocomplete) = match &self.data {
            Data::Enum(command_variants) => {
                let mut options = Vec::<proc_macro2::TokenStream>::new(); // TODO: remove type hint
                let mut group = None;
                for command_variant in command_variants {
                    command_variant.validate()?;

                    if command_variant.is_group() {
                        group = Some(command_variant.ident.to_string());
                        continue;
                    }

                    let implicit_end_group = group
                        .as_ref()
                        .is_some_and(|group| !command_variant.is_in_group(group));
                    if let Some(end_group) = &command_variant.end_group {
                        if implicit_end_group {
                            return Err(syn::Error::new_spanned(end_group, "invalid end_group"));
                        }
                        group = None;
                    } else if implicit_end_group {
                        group = None;
                    }

                    // command_variant.
                }

                (quote! { #( #options ),* }, quote! {}, quote! {}, quote! {})
            }
            Data::Struct(command_options) => (
                CommandOption::create_list(command_options),
                quote! {},
                quote! {},
                quote! {},
            ),
        };

        let result = quote! { ::tranquil::interaction::error::Result };
        let serenity = serenity();
        Ok(quote! {
            impl ::tranquil::interaction::command::Command for #type_name {
                const NAME: &str = #name;

                type Autocomplete = #autocomplete;

                fn create() -> #serenity::CreateCommand {
                    #serenity::CreateCommand::new(#name)
                        .description(#description)
                        #localizations
                        // TODO: .kind(#kind)
                        // TODO: .default_member_permissions(#permissions)
                        .set_options(::std::vec![#options])
                        // TODO: .integration_types(#integration_types)
                        // TODO: .contexts(#contexts)
                        #( #nsfw )*
                        // TODO: .handler(#handler)
                }

                fn resolve_command(data: &mut CommandInteraction) -> #result<Self> {
                    #resolve_command
                }

                fn resolve_autocomplete(
                    data: &mut #serenity::CommandInteraction,
                ) -> #result<Self::Autocomplete> {
                    #resolve_autocomplete
                }
            }
        })
    }
}

impl CommandVariant {
    fn validate(&self) -> syn::Result<()> {
        if self.is_group() {
            if self.fields.len() > 1 {
                return Err(syn::Error::new_spanned(
                    &self.ident,
                    "invalid command group",
                ));
            }
            if let Some(end_group) = &self.end_group {
                return Err(syn::Error::new_spanned(end_group, "invalid end_group"));
            }
        }
        Ok(())
    }

    /// Whether this [`CommandVariant`] is a command group marker.
    fn is_group(&self) -> bool {
        self.fields
            .iter()
            .next()
            .is_some_and(|command_option| command_option.ident.is_none())
    }

    /// Whether this [`CommandVariant`] is in the given `group`.
    ///
    /// Returns `true` if the command is the group itself, which would require two variants with the
    /// same name.
    fn is_in_group(&self, group: &str) -> bool {
        self.ident.to_string().starts_with(group)
    }

    // fn create(&self, group: Option<&Ident>) -> syn::Result<proc_macro2::TokenStream> {
    //     let NameDescriptionBuilt {
    //         name,
    //         description,
    //         localizations,
    //     } = NameDescription::parse(&self.attrs, &self.ident, group)?.build();

    //     // let options = todo!();

    //     if self.is_group() {}

    //     let serenity = serenity();
    //     Ok(quote! {
    //         #serenity::CreateCommandOption::new(
    //             #serenity::CommandOptionType::SubCommand,
    //             #name,
    //             #description,
    //         )
    //             #localizations
    //             // .set_sub_options(::std::vec![ #( #options ),* ])
    //     })
    // }
}

impl CommandOption {
    fn create(&self) -> syn::Result<proc_macro2::TokenStream> {
        let ty = &self.ty;

        let NameDescriptionBuilt {
            name,
            description,
            localizations,
        } = NameDescription::parse(
            &self.attrs,
            self.ident.as_ref().expect("should not be called on groups"),
            None,
        )?
        .build();

        let autocomplete = self
            .autocomplete
            .then(|| quote! { .autocomplete(true) })
            .into_iter();

        Ok(quote! {
            <#ty as ::tranquil::interaction::command::option::CommandOption>::create(
                #name.into(),
                #description.into(),
            )
                #localizations
                #( #autocomplete )*
        })
    }

    fn create_list(command_options: &Fields<CommandOption>) -> proc_macro2::TokenStream {
        let options = command_options
            .fields
            .iter()
            .map(|command_option| command_option.create().flatten_error());
        quote! { #( #options ),* }
    }
}

fn serenity() -> proc_macro2::TokenStream {
    quote! { ::tranquil::serenity::all }
}

trait FlattenErrorExt {
    fn flatten_error(self) -> proc_macro2::TokenStream;
}

impl FlattenErrorExt for syn::Result<proc_macro2::TokenStream> {
    fn flatten_error(self) -> proc_macro2::TokenStream {
        self.unwrap_or_else(|error| error.to_compile_error())
    }
}

*/
