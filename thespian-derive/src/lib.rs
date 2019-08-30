extern crate proc_macro;

use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::*,
    *,
};

#[proc_macro_attribute]
pub fn actor(
    _args: proc_macro::TokenStream,
    tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut result = tokens.clone();

    let input = parse_macro_input!(tokens as Actor);
    dbg!(&input);

    let actor_ident = input.ident;
    let module_ident = Ident::new(
        &format!("thespian_generated__{}", actor_ident),
        actor_ident.span(),
    );
    let context_ident = Ident::new(&format!("{}__Context", actor_ident), actor_ident.span());
    let proxy_ident = Ident::new(&format!("{}__Proxy", actor_ident), actor_ident.span());

    let generated = quote! {
        #[doc(hidden)]
        #[allow(bad_style)]
        pub mod #module_ident {
            impl thespian::Actor for super::#actor_ident {
                type Context = #context_ident;
                type Proxy = #proxy_ident;
            }

            #[derive(Debug)]
            pub struct #context_ident;

            impl thespian::ActorContext for #context_ident {
                type Actor = super::#actor_ident;

                fn into_future(self) -> Box<dyn std::future::Future<Output = ()>> {
                    unimplemented!()
                }
            }

            #[derive(Debug)]
            pub struct #proxy_ident;

            impl thespian::ActorProxy for #proxy_ident {
                type Actor = super::#actor_ident;
            }
        }
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

        for method in &methods {
            if method.ident == "new" {
                return Err(syn::Error::new(
                    method.ident.span(),
                    format!(
                        "method name conflicts with generated fn `{}Client::new`",
                        ident
                    ),
                ));
            }

            if method.ident == "serve" {
                return Err(syn::Error::new(
                    method.ident.span(),
                    format!("method name conflicts with generated fn `{}::serve`", ident),
                ));
            }
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
            Error::new(
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
