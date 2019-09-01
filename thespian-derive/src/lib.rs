extern crate proc_macro;

use quote::*;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Async, Comma},
    *,
};

#[proc_macro_attribute]
pub fn actor(
    _args: proc_macro::TokenStream,
    tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut result = tokens.clone();

    let input = parse_macro_input!(tokens as Actor);

    let actor_ident = input.ident;
    let proxy_ident = format_ident!("{}Proxy", actor_ident);

    let proxy_methods = input.methods.iter().map(ActorMethod::quote_proxy_method);
    let method_structs = input
        .methods
        .iter()
        .map(|method| method.quote_message_struct(&actor_ident));

    let generated = quote! {
        impl thespian::Actor for #actor_ident {
            type Proxy = #proxy_ident;
        }

        #[allow(bad_style)]
        #[derive(Debug, Clone)]
        pub struct #proxy_ident {
            inner: thespian::ProxyFor<#actor_ident>,
        }

        impl thespian::ActorProxy for #proxy_ident {
            type Actor = #actor_ident;

            fn new(inner: thespian::ProxyFor<#actor_ident>) -> Self {
                Self { inner }
            }
        }

        impl #proxy_ident {
            #( #proxy_methods )*
        }

        #( #method_structs )*
    };
    println!("{}", generated);

    result.extend(proc_macro::TokenStream::from(generated));
    result
}

// TODO: Support generic actor types.
#[derive(Debug)]
struct Actor {
    attrs: Vec<Attribute>,
    ident: Ident,
    methods: Vec<ActorMethod>,
}

impl Parse for Actor {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        input.parse::<Token![impl]>()?;
        let ident = input.parse()?;
        let content;
        braced!(content in input);

        let mut methods = Vec::<ActorMethod>::new();
        while !content.is_empty() {
            methods.push(content.parse()?);
        }

        Ok(Actor {
            attrs,
            ident,
            methods,
        })
    }
}

#[derive(Debug)]
struct ActorMethod {
    attrs: Vec<Attribute>,
    vis: Visibility,
    asyncness: Option<Async>,
    ident: Ident,
    receiver: Receiver,
    args: Punctuated<PatType, Comma>,
    output: ReturnType,
}

impl ActorMethod {
    fn quote_proxy_method(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        let args = &self.args;

        let result_type = match &self.output {
            ReturnType::Default => Box::new(syn::parse_str("()").unwrap()),
            ReturnType::Type(_, ty) => ty.clone(),
        };

        quote! {
            pub async fn #ident(&self, #args) -> Result<#result_type, thespian::MessageError> {
                unimplemented!()
            }
        }
    }

    fn quote_message_struct(&self, actor_ident: &Ident) -> proc_macro2::TokenStream {
        let ident = format_ident!("{}__{}", actor_ident, self.ident);
        let args = self.args.iter().map(|pat| &pat.ty);
        quote! {
            struct #ident(#( #args )*);
        }
    }
}

impl Parse for ActorMethod {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        let asyncness = input.parse::<Token![async]>().ok();
        input.parse::<Token![fn]>()?;
        let ident: Ident = input.parse()?;

        let content;
        parenthesized!(content in input);
        let raw_args: Punctuated<FnArg, Comma> = content.parse_terminated(FnArg::parse)?;

        let mut receiver = None;
        let mut args = Punctuated::new();
        for arg in raw_args {
            match arg {
                FnArg::Receiver(recv) => {
                    // TODO: Validate that reciever is `&self` or `&mut self`.
                    receiver = Some(recv);
                }

                FnArg::Typed(arg) => {
                    args.push(arg);
                }
            }
        }

        let output = input.parse()?;

        // TODO: I guess this will probably break on `where` clauses?

        // NOTE: We must fully parse the body of the method in order to
        let content;
        braced!(content in input);
        let _ = content.call(Block::parse_within)?;

        let receiver = receiver.ok_or_else(|| {
            syn::Error::new(
                ident.span(),
                "Actor method must take `&self` or `&mut self`",
            )
        })?;

        Ok(ActorMethod {
            attrs,
            vis,
            asyncness,
            ident,
            receiver,
            args,
            output,
        })
    }
}
