extern crate proc_macro;

use quote::*;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
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

    let proxy_methods = input
        .methods
        .iter()
        .map(|method| method.quote_proxy_method(&actor_ident));
    let method_structs = input
        .methods
        .iter()
        .map(|method| method.quote_message_struct(&actor_ident));

    let generated = quote! {
        impl thespian::Actor for #actor_ident {
            type Proxy = #proxy_ident;
        }

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
    fn quote_proxy_method(&self, actor_ident: &Ident) -> proc_macro2::TokenStream {
        let vis = &self.vis;
        let ident = &self.ident;

        // TODO: We can't use `args` directly as the parameters for the function since
        // the user might have used a pattern for one of the parameters instead of
        // binding it to a variable name. In these cases, we need to generate an
        // alternate variable name to use instead.
        let args = &self.args;

        // Determine which send method on `ProxyFor<A>` should be used to send the method
        // based on whether or not the message handler is async.
        let send_method = if self.asyncness.is_some() {
            quote! { send_async }
        } else {
            quote! { send_sync }
        };

        // Generate the expression for initializing the message object.
        let struct_ident = self.message_struct_ident(actor_ident);
        let struct_params: Punctuated<_, Comma> = self.args.iter().map(|pat| &pat.pat).collect();

        let result_type = self.result_type();

        quote! {
            #vis async fn #ident(&mut self, #args) -> Result<#result_type, thespian::MessageError> {
                self.inner.#send_method(#struct_ident(#struct_params)).await
            }
        }
    }

    fn quote_message_struct(&self, actor_ident: &Ident) -> proc_macro2::TokenStream {
        let ident = self.message_struct_ident(actor_ident);
        let args: Punctuated<_, Comma> = self.args.iter().map(|pat| &pat.ty).collect();
        let message_impl = if self.asyncness.is_some() {
            self.impl_async_message(actor_ident)
        } else {
            self.impl_sync_message(actor_ident)
        };

        quote! {
            #[allow(bad_style)]
            struct #ident(#args);

            #message_impl
        }
    }

    fn impl_sync_message(&self, actor_ident: &Ident) -> proc_macro2::TokenStream {
        let method_ident = &self.ident;
        let struct_ident = self.message_struct_ident(actor_ident);
        let result_type = self.result_type();
        let forward_params = self
            .args
            .iter()
            .enumerate()
            .map(|(index, pat)| LitInt::new(&format!("{}", index), pat.span()));

        quote! {
            impl thespian::SyncMessage for #struct_ident {
                type Actor = #actor_ident;
                type Result = #result_type;

                fn handle(self, actor: &mut Self::Actor) -> Self::Result {
                    actor.#method_ident(#( self.#forward_params, )*)
                }
            }
        }
    }

    fn impl_async_message(&self, actor_ident: &Ident) -> proc_macro2::TokenStream {
        let method_ident = &self.ident;
        let struct_ident = self.message_struct_ident(actor_ident);
        let result_type = self.result_type();
        let forward_params = self
            .args
            .iter()
            .enumerate()
            .map(|(index, pat)| LitInt::new(&format!("{}", index), pat.span()));

        quote! {
            impl thespian::AsyncMessage for #struct_ident {
                type Actor = #actor_ident;
                type Result = #result_type;

                fn handle(self, actor: &mut Self::Actor) -> futures::future::BoxFuture<'_, Self::Result> {
                    futures::future::FutureExt::boxed(actor.#method_ident(#( self.#forward_params, )*))
                }
            }
        }
    }

    fn message_struct_ident(&self, actor_ident: &Ident) -> Ident {
        format_ident!("{}__{}", actor_ident, self.ident)
    }

    fn result_type(&self) -> Box<Type> {
        match &self.output {
            ReturnType::Default => Box::new(syn::parse_str("()").unwrap()),
            ReturnType::Type(_, ty) => ty.clone(),
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
