use std::collections::{BTreeMap, btree_map::Entry};

use convert_case::{Case, Casing};
use darling::{
    FromDeriveInput, FromField, FromVariant,
    ast::{Data, Fields},
};
use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    Attribute, DeriveInput, Expr, ExprLit, Ident, Lit, Type, parse_macro_input, spanned::Spanned,
};
use unicode_script::{Script, UnicodeScript};

#[proc_macro_derive(Command, attributes(tranquil))]
pub fn derive_command(input: TokenStream) -> TokenStream {
    Command::from_derive_input(&parse_macro_input!(input as DeriveInput)).map_or_else(
        |error| error.write_errors().into(),
        |command_meta| command_meta.into_impl_command().flatten_error().into(),
    )
}

#[derive(FromDeriveInput)]
#[darling(
    attributes(tranquil),
    forward_attrs(doc),
    supports(struct_unit, struct_named, enum_unit, enum_named, enum_newtype)
)]
struct Command {
    ident: Ident,
    data: Data<CommandVariant, CommandOption>,
    attrs: Vec<Attribute>,

    // TODO: kind
    // TODO: integration_types
    // TODO: contexts
    #[darling(default)]
    nsfw: bool,
    // TODO: handler
}

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

#[derive(FromVariant)]
#[darling(attributes(tranquil), forward_attrs(doc))]
struct CommandVariant {
    ident: Ident,
    fields: Fields<CommandOption>,
    attrs: Vec<Attribute>,

    end_group: Option<Ident>,
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

#[derive(FromField)]
#[darling(attributes(tranquil), forward_attrs(doc))]
struct CommandOption {
    /// The name of the option.
    ///
    /// [`None`] indicates that the variant is a tuple, which indicates that this variant is a
    /// command group.
    ident: Option<Ident>,
    ty: Type,
    attrs: Vec<Attribute>,

    #[darling(default)]
    autocomplete: bool,
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

#[derive(Default)]
struct NameDescription {
    name: Localized,
    description: Localized,
}

impl NameDescription {
    /// Parses [`NameDescription`] for doc comments in `attrs`.
    ///
    /// If no command name is specified, `ident` converted to kebab-case is used as a default. If
    /// `group` is provided, it will be stripped from the `ident`.
    ///
    /// The following syntax is parsed:
    ///
    /// ```ignore
    /// /// `command-name` Command description
    /// /// that can span multiple lines
    /// ///
    /// /// - `de` `kommando-name` Kommando Beschreibung
    /// ///   welche ebenfalls mehrere Zeilen haben kann
    /// /// - `fr` Command name translation is optional
    /// ```
    fn parse(attrs: &[Attribute], ident: &Ident, group: Option<&str>) -> syn::Result<Self> {
        let mut lines = attrs
            .iter()
            // doc attributes are technically already filtered out by darling
            .filter(|attr| attr.path().is_ident("doc"))
            .map(|attr| {
                let span = attr.span();
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = &attr.meta.require_name_value()?.value
                {
                    Ok((lit_str.value().trim().to_string(), span))
                } else {
                    Err(syn::Error::new(span, "malformed doc comment"))
                }
            });

        let default_name = || {
            let name = ident.to_string();
            if let Some(group) = group {
                name.strip_prefix(group)
                    .expect("command should have a group prefix")
            } else {
                &name
            }
            .to_case(Case::Kebab)
        };

        let Some(line_span) = lines.next() else {
            return Ok(Self {
                name: Localized {
                    default: default_name(),
                    ..Default::default()
                },
                description: Localized {
                    default: "n/a".to_string(),
                    ..Default::default()
                },
            });
        };

        let (line, span) = line_span?;

        let (name, line) = extract_inline_code(&line, span)?;
        let name = if let Some(name) = name {
            name.to_string()
        } else {
            default_name()
        };

        check_name(&name, span)?;

        let mut description = line.to_string();
        for line_span in &mut lines {
            let (line, _) = line_span?;
            if line.is_empty() {
                break;
            }
            if !description.is_empty() {
                description.push(' ');
            }
            description += &line;
        }

        check_description(&description, span)?;

        let mut name_localizations = BTreeMap::new();
        let mut description_localizations = BTreeMap::new();
        let mut last_locale = None;
        for line_span in &mut lines {
            let (line, span) = line_span?;
            let (has_dash, line) = strip_dash(&line);
            if has_dash {
                let (locale, line) = extract_inline_code(line, span)?;
                let locale = parse_locale(
                    locale.ok_or(syn::Error::new(span, "locale expected"))?,
                    span,
                )?;

                let (name, line) = extract_inline_code(line, span)?;
                match description_localizations.entry(locale) {
                    Entry::Vacant(entry) => entry.insert((line.to_string(), span)),
                    Entry::Occupied(_) => {
                        return Err(syn::Error::new(span, format!("duplicate locale: {locale}")));
                    }
                };

                if let Some(name) = name {
                    match name_localizations.entry(locale) {
                        Entry::Vacant(entry) => entry.insert(name.to_string()),
                        Entry::Occupied(_) => {
                            unreachable!("description should have returned already")
                        }
                    };
                }

                last_locale = Some(locale);
            } else {
                let (description, _) = description_localizations
                    .get_mut(last_locale.ok_or(syn::Error::new(
                        span,
                        "dash expected to start a new localization entry",
                    ))?)
                    .expect("localized description should exist");
                if !description.is_empty() {
                    description.push(' ');
                }
                *description += line;
            }
        }

        for (description, span) in description_localizations.values() {
            check_description(description, *span)?;
        }

        Ok(Self {
            name: Localized {
                default: name,
                localizations: name_localizations,
            },
            description: Localized {
                default: description,
                localizations: description_localizations
                    .into_iter()
                    .map(|(locale, (description, _))| (locale, description))
                    .collect(),
            },
        })
    }

    fn build(self) -> NameDescriptionBuilt {
        let name_localized = self
            .name
            .localizations
            .iter()
            .map(|(locale, name)| quote! { .name_localized(#locale, #name) });

        let description_localized =
            self.description.localizations.iter().map(
                |(locale, description)| quote! { .description_localized(#locale, #description) },
            );

        NameDescriptionBuilt {
            name: self.name.default,
            description: self.description.default,
            localizations: quote! {
                #( #name_localized )*
                #( #description_localized )*
            },
        }
    }
}

struct NameDescriptionBuilt {
    name: String,
    description: String,
    localizations: proc_macro2::TokenStream,
}

/// Strips the `-` from `- foo` and returns `true` if a `-` was stripped.
fn strip_dash(line: &str) -> (bool, &str) {
    if let Some(stripped) = line.strip_prefix('-') {
        (true, stripped.trim_start())
    } else {
        (false, line)
    }
}

/// Extracts the contents of an inline code block at the start of `line`.
///
/// E.g. `` `foo` bar`` is turned into `foo` and `bar`, trimming any space between the two. `line`
/// itself is not trimmed; the `` ` `` has to be the very first character.
///
/// Returns [`Err`] if `line` starts with a `` ` `` but doesn't contain a closing one.
///
/// Returns [`None`] along with the entire original `line` if it doesn't start with a `` ` ``.
fn extract_inline_code(line: &str, span: Span) -> syn::Result<(Option<&str>, &str)> {
    if let Some(line) = line.strip_prefix('`') {
        if let Some((code, rest)) = line.split_once('`') {
            Ok((Some(code), rest.trim_start()))
        } else {
            Err(syn::Error::new(span, "unclosed backtick"))
        }
    } else {
        Ok((None, line))
    }
}

#[derive(Default)]
struct Localized {
    default: String,
    localizations: BTreeMap<&'static str, String>,
}

fn check_name(name: &str, span: Span) -> syn::Result<()> {
    check_string(32, "name", name, span)?;

    let invalid_chars = name
        .chars()
        .filter(|c| !is_valid_command_char(*c))
        .sorted()
        .dedup()
        .collect::<String>();

    if !invalid_chars.is_empty() {
        return Err(syn::Error::new(
            span,
            format!("characters {invalid_chars:?} cannot be used in command names"),
        ));
    }

    Ok(())
}

fn is_valid_command_char(c: char) -> bool {
    match c {
        '-' | '_' | '\'' => true,
        _ if !lowercase_is_same(c) => false,
        _ if c.is_alphanumeric() => true,
        _ if matches!(c.script(), Script::Devanagari | Script::Thai) => true,
        _ => false,
    }
}

fn lowercase_is_same(c: char) -> bool {
    c.to_lowercase().exactly_one().is_ok_and(|lower| lower == c)
}

fn check_description(description: &str, span: Span) -> syn::Result<()> {
    check_string(100, "description", description, span)
}

fn check_string(max_len: usize, what: &str, string: &str, span: Span) -> syn::Result<()> {
    let len = string.chars().count(); // characters, not bytes
    if len == 0 {
        Err(syn::Error::new(span, format!("{what} cannot be empty")))
    } else if len > max_len {
        Err(syn::Error::new(
            span,
            format!("{what} must be at most {max_len} characters ({len} > {max_len})"),
        ))
    } else {
        Ok(())
    }
}

/// Parses a `locale` from a `&str` into a `&'static str`.
///
/// See https://discord.com/developers/docs/reference#locales
fn parse_locale(locale: &str, span: Span) -> syn::Result<&'static str> {
    match locale {
        "id" => Ok("id"),         // Indonesian
        "da" => Ok("da"),         // Danish
        "de" => Ok("de"),         // German
        "en-GB" => Ok("en-GB"),   // English, UK
        "en-US" => Ok("en-US"),   // English, US
        "es-ES" => Ok("es-ES"),   // Spanish
        "es-419" => Ok("es-419"), // Spanish, LATAM
        "fr" => Ok("fr"),         // French
        "hr" => Ok("hr"),         // Croatian
        "it" => Ok("it"),         // Italian
        "lt" => Ok("lt"),         // Lithuanian
        "hu" => Ok("hu"),         // Hungarian
        "nl" => Ok("nl"),         // Dutch
        "no" => Ok("no"),         // Norwegian
        "pl" => Ok("pl"),         // Polish
        "pt-BR" => Ok("pt-BR"),   // Portuguese, Brazilian
        "ro" => Ok("ro"),         // Romanian, Romania
        "fi" => Ok("fi"),         // Finnish
        "sv-SE" => Ok("sv-SE"),   // Swedish
        "vi" => Ok("vi"),         // Vietnamese
        "tr" => Ok("tr"),         // Turkish
        "cs" => Ok("cs"),         // Czech
        "el" => Ok("el"),         // Greek
        "bg" => Ok("bg"),         // Bulgarian
        "ru" => Ok("ru"),         // Russian
        "uk" => Ok("uk"),         // Ukrainian
        "hi" => Ok("hi"),         // Hindi
        "th" => Ok("th"),         // Thai
        "zh-CN" => Ok("zh-CN"),   // Chinese, China
        "ja" => Ok("ja"),         // Japanese
        "zh-TW" => Ok("zh-TW"),   // Chinese, Taiwan
        "ko" => Ok("ko"),         // Korean
        _ => Err(syn::Error::new(
            span,
            format!(
                "invalid locale: {locale}; see https://discord.com/developers/docs/reference#locales"
            ),
        )),
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
