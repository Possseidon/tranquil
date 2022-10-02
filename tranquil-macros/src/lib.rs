use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ItemFn, PatType};

#[proc_macro_attribute]
pub fn slash(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);

    let name = input.sig.ident;
    let impl_name = format_ident!("__{name}");
    input.sig.ident = impl_name.clone();

    let typed_parameters = input
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
                option
                    .kind(<#ty as ::tranquil::resolve::Resolve>::KIND)
                    .name(::std::stringify!(#pat))
                    .required(<#ty as ::tranquil::resolve::Resolve>::REQUIRED);

                <#ty as ::tranquil::resolve::Resolve>::min_int_value().map(|value| option.min_int_value(value));
                <#ty as ::tranquil::resolve::Resolve>::max_int_value().map(|value| option.max_int_value(value));
                <#ty as ::tranquil::resolve::Resolve>::min_number_value().map(|value| option.min_number_value(value));
                <#ty as ::tranquil::resolve::Resolve>::max_number_value().map(|value| option.max_number_value(value));
                <#ty as ::tranquil::resolve::Resolve>::min_length().map(|value| option.min_length(value));
                <#ty as ::tranquil::resolve::Resolve>::max_length().map(|value| option.max_length(value));

                option
            }) as fn() -> ::serenity::builder::CreateApplicationCommandOption
        }
    });

    let expanded = quote! {
        #input

        fn #name(
            self: ::std::sync::Arc<Self>
        ) -> ::std::boxed::Box<dyn ::tranquil::slash_command::SlashCommandImpl> {
            ::std::boxed::Box::new(::tranquil::slash_command::SlashCommand::new(
                stringify!(#name),
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

    TokenStream::from(expanded)
}
