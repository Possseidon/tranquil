use std::collections::{BTreeMap, HashMap};

use convert_case::{Case, Casing};
use darling::{
    FromDeriveInput, FromField, FromMeta, FromVariant,
    ast::{Data, NestedMeta},
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Ident, Lit, parse_macro_input};

#[proc_macro_derive(Command, attributes(tranquil))]
pub fn derive_command(input: TokenStream) -> TokenStream {
    CommandMeta::from_derive_input(&parse_macro_input!(input as DeriveInput)).map_or_else(
        |error| error.write_errors().into(),
        CommandMeta::into_impl_command,
    )
}

type Name = Localized<32>;

type Description = Localized<100>;

#[derive(FromDeriveInput)]
#[darling(
    attributes(tranquil),
    supports(struct_unit, struct_named, enum_unit, enum_named)
)]
struct CommandMeta {
    ident: Ident,
    name: Option<Name>,
    description: Description,
    #[darling(default)]
    nsfw: bool,
    data: Data<Subcommand, Options>,
}

#[derive(FromVariant)]
struct Subcommand {
    name: Option<Name>,
    description: Description,
}

#[derive(FromField)]
struct Options {
    name: Option<Name>,
    description: Description,
}

impl CommandMeta {
    fn into_impl_command(self) -> TokenStream {
        let type_name = &self.ident;

        let name = self
            .name
            .as_ref()
            .map(|name| name.default.clone())
            .unwrap_or_else(|| type_name.to_string().to_case(Case::Kebab));

        let name_localized = self
            .name
            .into_iter()
            .flat_map(|name| name.localizations)
            .map(|(locale, name)| quote! { .name_localized(#locale, #name) });

        let description = self.description.default;

        let description_localized =
            self.description.localizations.into_iter().map(
                |(locale, description)| quote! { .description_localized(#locale, #description) },
            );

        let nsfw = self.nsfw.then(|| quote! { .nsfw(true) }).into_iter();

        // match self.data {
        //     Data::Enum(items) => panic!("enum"),
        //     Data::Struct(fields) => panic!("struct"),
        // }

        let serenity = quote! { ::tranquil::serenity::all };
        let anyhow = quote! { ::tranquil::anyhow };
        quote! {
            impl Command for #type_name {
                const NAME: &str = #name;

                type Autocomplete = ::tranquil::interaction::command::NoAutocomplete; // TODO

                fn create_command() -> #serenity::CreateCommand {
                    #serenity::CreateCommand::new(#name)
                        #( #name_localized )*
                        // TODO: .kind(#kind)
                        // TODO: .default_member_permissions(#permissions)
                        .description(#description)
                        #( #description_localized )*
                        // TODO: .set_options(#options)
                        // TODO: .integration_types(#integration_types)
                        // TODO: .contexts(#contexts)
                        #( #nsfw )*
                        // TODO: .handler(#handler)
                }

                fn resolve_autocomplete(
                    data: &mut #serenity::CommandInteraction,
                ) -> #anyhow::Result<Self::Autocomplete> {
                    todo!()
                }

                fn resolve_command(data: &mut CommandInteraction) -> #anyhow::Result<Self> {
                    todo!()
                }
            }
        }
        .into()
    }
}

struct Localized<const MAX_LEN: usize> {
    default: String,
    localizations: BTreeMap<&'static str, String>,
}

impl<const MAX_LEN: usize> FromMeta for Localized<MAX_LEN> {
    fn from_string(value: &str) -> darling::Result<Self> {
        check_string::<MAX_LEN>(value)?;
        Ok(Self {
            default: value.to_string(),
            localizations: Default::default(),
        })
    }

    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let (default, rest) = if let [NestedMeta::Lit(Lit::Str(lit_str)), ..] = items {
            let value = lit_str.value();
            check_string::<MAX_LEN>(&value).map_err(|error| error.with_span(&lit_str))?;
            (Some(value), &items[1..])
        } else {
            (None, items)
        };

        let localizations = HashMap::<Ident, String>::from_list(rest)?
            .into_iter()
            .map(|(locale, value)| -> darling::Result<_> {
                // parse locale first, since it comes first in the input
                let locale = parse_locale_ident(&locale)?;
                check_string::<MAX_LEN>(&value)?;
                Ok((locale, value))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;

        Ok(Self {
            default: default
                .or_else(|| localizations.get("en-US").cloned())
                .ok_or_else(|| darling::Error::custom("specify default or en_US"))?,
            localizations,
        })
    }
}

/// Returns an appropdiate error if `value` is not between 1 and `MAX_LEN` characters long.
fn check_string<const MAX_LEN: usize>(value: &str) -> darling::Result<()> {
    let len = value.len();
    if len == 0 {
        Err(darling::Error::custom("cannot be empty"))
    } else if len > MAX_LEN {
        Err(darling::Error::custom(format!(
            "must be at most {MAX_LEN} characters ({len} > {MAX_LEN})"
        )))
    } else {
        Ok(())
    }
}

/// Parses a `locale` from an [`Ident`] into a proper locale code (e.g. `en_US` into `en-US`).
fn parse_locale_ident(locale: &Ident) -> darling::Result<&'static str> {
    match locale.to_string().as_str() {
        "id" => Ok("id"),
        "da" => Ok("da"),
        "de" => Ok("de"),
        "en_GB" => Ok("en-GB"),
        "en_US" => Ok("en-US"),
        "es_ES" => Ok("es-ES"),
        "es_419" => Ok("es-419"),
        "fr" => Ok("fr"),
        "hr" => Ok("hr"),
        "it" => Ok("it"),
        "lt" => Ok("lt"),
        "hu" => Ok("hu"),
        "nl" => Ok("nl"),
        "no" => Ok("no"),
        "pl" => Ok("pl"),
        "pt_BR" => Ok("pt-BR"),
        "ro" => Ok("ro"),
        "fi" => Ok("fi"),
        "sv_SE" => Ok("sv-SE"),
        "vi" => Ok("vi"),
        "tr" => Ok("tr"),
        "cs" => Ok("cs"),
        "el" => Ok("el"),
        "bg" => Ok("bg"),
        "ru" => Ok("ru"),
        "uk" => Ok("uk"),
        "hi" => Ok("hi"),
        "th" => Ok("th"),
        "zh_CN" => Ok("zh-CN"),
        "ja" => Ok("ja"),
        "zh_TW" => Ok("zh-TW"),
        "ko" => Ok("ko"),
        _ => Err(darling::Error::custom("invalid locale").with_span(&locale)),
    }
}
