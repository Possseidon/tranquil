use darling::{
    FromDeriveInput, FromField, FromVariant,
    ast::{Data, Fields},
};

#[derive(FromDeriveInput)]
#[darling(
    attributes(tranquil),
    forward_attrs(doc),
    supports(struct_unit, struct_named, enum_unit, enum_named, enum_newtype)
)]
struct Command {
    ident: syn::Ident,
    data: Data<CommandVariant, CommandOption>,
    attrs: Vec<syn::Attribute>,

    // TODO: kind
    // TODO: integration_types
    // TODO: contexts
    #[darling(default)]
    nsfw: bool,
    // TODO: handler
}

#[derive(FromVariant)]
#[darling(attributes(tranquil), forward_attrs(doc))]
struct CommandVariant {
    ident: syn::Ident,
    fields: Fields<CommandOption>,
    attrs: Vec<syn::Attribute>,

    end_group: Option<syn::Ident>,
}

#[derive(FromField)]
#[darling(attributes(tranquil), forward_attrs(doc))]
struct CommandOption {
    /// The name of the option.
    ///
    /// [`None`] indicates that the variant is a tuple, which indicates that this variant is a
    /// command group.
    ident: Option<syn::Ident>,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,

    #[darling(default)]
    autocomplete: bool,
}
