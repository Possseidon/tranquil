use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use darling::{
    FromDeriveInput, FromField, FromVariant,
    ast::{Data, Fields},
};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    Attribute, DeriveInput, Expr, ExprLit, Ident, Lit, LitStr, Type, parse_macro_input,
    spanned::Spanned,
};
use unicode_script::{Script, UnicodeScript};

#[proc_macro_derive(Command, attributes(tranquil))]
pub fn derive_command(input: TokenStream) -> TokenStream {
    CommandMeta::from_derive_input(&parse_macro_input!(input as DeriveInput)).map_or_else(
        |error| error.write_errors().into(),
        CommandMeta::into_impl_command,
    )
}

#[derive(FromDeriveInput)]
#[darling(
    attributes(tranquil),
    forward_attrs(doc),
    supports(struct_unit, struct_named, enum_unit, enum_named)
)]
struct CommandMeta {
    ident: Ident,
    data: Data<SubCommand, CommandOption>,
    attrs: Vec<Attribute>,

    // TODO: kind
    // TODO: integration_types
    // TODO: contexts
    #[darling(default)]
    nsfw: bool,
    // TODO: handler
}

impl CommandMeta {
    fn into_impl_command(self) -> TokenStream {
        let type_name = &self.ident;

        let name_description = match NameDescription::parse(&self.attrs, type_name) {
            Ok(name_description) => name_description,
            Err(error) => return error.into_compile_error().into(),
        };

        let name = name_description.name.default;
        let description = name_description.description.default;

        let nsfw = self.nsfw.then(|| quote! { .nsfw(true) }).into_iter();

        // let options: &mut dyn Iterator<Item = _> = match self.data {
        //     Data::Enum(subcommands) => &mut subcommands
        //         .into_iter()
        //         .map(|subcommand| subcommand.create()),
        //     Data::Struct(command_options) => &mut command_options
        //         .fields
        //         .into_iter()
        //         .map(|command_option| command_option.create()),
        // };

        let result = quote! { ::tranquil::interaction::error::Result };
        let serenity = quote! { ::tranquil::serenity::all };
        quote! {
            impl ::tranquil::interaction::command::Command for #type_name {
                const NAME: &str = #name;

                type Autocomplete = ::tranquil::interaction::command::NoAutocomplete; // TODO

                fn create() -> #serenity::CreateCommand {
                    #serenity::CreateCommand::new(#name)
                        // #( #name_localized )*
                        // TODO: .kind(#kind)
                        // TODO: .default_member_permissions(#permissions)
                        .description(#description)
                        // #( #description_localized )*
                        // .set_options(::std::vec![ #( #options ),* ])
                        // TODO: .integration_types(#integration_types)
                        // TODO: .contexts(#contexts)
                        #( #nsfw )*
                        // TODO: .handler(#handler)
                }

                fn resolve_autocomplete(
                    data: &mut #serenity::CommandInteraction,
                ) -> #result<Self::Autocomplete> {
                    todo!()
                }

                fn resolve_command(data: &mut CommandInteraction) -> #result<Self> {
                    todo!()
                }
            }
        }
        .into()
    }
}

#[derive(FromVariant)]
#[darling(attributes(tranquil), forward_attrs(doc))]
struct SubCommand {
    ident: Ident,
    fields: Fields<CommandOption>,
    attrs: Vec<Attribute>,
}

#[derive(FromField)]
#[darling(attributes(tranquil), forward_attrs(doc))]
struct CommandOption {
    ident: Option<Ident>,
    ty: Type,
    attrs: Vec<Attribute>,

    #[darling(default)]
    autocomplete: bool,
}

#[derive(Default)]
struct NameDescription {
    name: Localized,
    sub_name: Option<Localized>,
    description: Localized,
}

impl NameDescription {
    /// Parses [`NameDescription`] for doc comments in `attrs` using `ident` as a fallback name.
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
    fn parse(attrs: &[Attribute], ident: &Ident) -> syn::Result<Self> {
        let mut lines = attrs
            .iter()
            // doc attributes are technically already filtered out by darling
            .filter(|attr| attr.path().is_ident("doc"))
            .map(|attr| {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }) = &attr.meta.require_name_value()?.value
                {
                    Ok((lit_str.value().trim().to_string(), attr.span()))
                } else {
                    Err(syn::Error::new(attr.span(), "malformed doc comment"))
                }
            });

        // the name of the struct/variant/field is used as kebab case by default
        let default_name = || ident.to_string().to_case(Case::Kebab);

        let Some(line) = lines.next() else {
            return Ok(Self {
                name: Localized {
                    default: default_name(),
                    ..Default::default()
                },
                sub_name: None,
                description: Localized {
                    default: "n/a".to_string(),
                    ..Default::default()
                },
            });
        };

        let (line, mut span) = line?;

        let (name, line) = extract_inline_code(&line, span)?;
        let (name, sub_name) = if let Some(name) = name {
            let (name, sub_name) = parse_command_name(name);
            if let Some(sub_name) = sub_name {
                check_name(sub_name, span)?;
            }
            check_name(name, span)?;
            (name.to_string(), sub_name)
        } else {
            (default_name(), None)
        };

        let mut description = line.to_string();
        for line in &mut lines {
            let (line, next_span) = line?;
            if line.is_empty() {
                break;
            }
            description.push(' ');
            description += &line;
            span = span.join(next_span).expect("should be in the same file");
        }

        check_description(&description, span);

        // TODO: localizations
        for line in &mut lines {}

        Ok(Self {
            name: Localized {
                default: name,
                ..Default::default()
            },
            sub_name: sub_name.map(|sub_name| Localized {
                default: sub_name.to_string(),
                ..Default::default()
            }),
            description: Localized {
                default: description,
                ..Default::default()
            },
        })
    }
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

fn parse_command_name(full_name: &str) -> (&str, Option<&str>) {
    if let Some((name, sub_name)) = full_name.split_once(' ') {
        (name, Some(sub_name))
    } else {
        (full_name, None)
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
        .filter(|&c| {
            c != '-'
                && c != '_'
                && c != '\''
                && !c.is_alphanumeric()
                && !matches!(c.script(), Script::Devanagari | Script::Thai)
        })
        .collect::<Vec<_>>();

    if !invalid_chars.is_empty() {
        const REGEX: &str = r"[-_'\p{L}\p{N}\p{sc=Deva}\p{sc=Thai}]";
        return Err(syn::Error::new(
            span,
            format!("{invalid_chars:?} are not a valid for command names; they must match {REGEX}"),
        ));
    }

    Ok(())
}

fn check_description(description: &str, span: Span) -> syn::Result<()> {
    check_string(100, "description", description, span)
}

fn check_string(max_len: usize, what: &str, string: &str, span: Span) -> syn::Result<()> {
    let len = string.len();
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
fn parse_locale(locale: &str, span: Span) -> syn::Result<&'static str> {
    match locale {
        "id" => Ok("id"),
        "da" => Ok("da"),
        "de" => Ok("de"),
        "en-GB" => Ok("en-GB"),
        "en-US" => Ok("en-US"),
        "es-ES" => Ok("es-ES"),
        "es-419" => Ok("es-419"),
        "fr" => Ok("fr"),
        "hr" => Ok("hr"),
        "it" => Ok("it"),
        "lt" => Ok("lt"),
        "hu" => Ok("hu"),
        "nl" => Ok("nl"),
        "no" => Ok("no"),
        "pl" => Ok("pl"),
        "pt-BR" => Ok("pt-BR"),
        "ro" => Ok("ro"),
        "fi" => Ok("fi"),
        "sv-SE" => Ok("sv-SE"),
        "vi" => Ok("vi"),
        "tr" => Ok("tr"),
        "cs" => Ok("cs"),
        "el" => Ok("el"),
        "bg" => Ok("bg"),
        "ru" => Ok("ru"),
        "uk" => Ok("uk"),
        "hi" => Ok("hi"),
        "th" => Ok("th"),
        "zh-CN" => Ok("zh-CN"),
        "ja" => Ok("ja"),
        "zh-TW" => Ok("zh-TW"),
        "ko" => Ok("ko"),
        _ => Err(syn::Error::new(span, format!("invalid locale: {locale}"))),
    }
}
