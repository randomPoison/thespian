use proc_macro2::{Literal, TokenStream};
use quote::*;
use syn::{punctuated::Punctuated, *};

#[proc_macro_derive(Actor)]
pub fn derive_actor(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let vis = input.vis;
    let actor_ident = input.ident;
    let proxy_ident = format_ident!("{}Proxy", actor_ident);

    let generated = quote! {
        impl thespian::Actor for #actor_ident {
            type Proxy = #proxy_ident;
        }

        #[derive(Debug, Clone)]
        #vis struct #proxy_ident {
            inner: thespian::ProxyFor<#actor_ident>,
        }

        impl thespian::ActorProxy for #proxy_ident {
            type Actor = #actor_ident;

            fn new(inner: thespian::ProxyFor<#actor_ident>) -> Self {
                Self { inner }
            }
        }
    };

    generated.into()
}

#[proc_macro_attribute]
pub fn actor(
    _args: proc_macro::TokenStream,
    tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut result = TokenStream::from(tokens.clone());

    // Parse the input as an impl block, rejecting any other item types.
    let input = parse_macro_input!(tokens as ItemImpl);

    // Gather all valid method definitions in the impl block, specifically ones with a
    // receiver since associated functions can't be used as messages.
    let methods = input
        .items
        .iter()
        .filter_map(|item| match item {
            ImplItem::Method(item) => Some(item),
            _ => None,
        })
        .filter(|method| method.sig.receiver().is_some())
        .collect::<Vec<_>>();

    let self_ty = input.self_ty;
    let mangled_self_ty = match mangled_type_name(&self_ty) {
        Ok(name) => name,
        Err(err) => return err.to_compile_error().into(),
    };
    let proxy_ty = format_ident!("{}Proxy", mangled_self_ty);

    // Collect the generated items for each message handler defined in the impl block.
    let generated = methods
        .iter()
        .map(|method| {
            let vis = &method.vis;
            let method_name = &method.sig.ident;
            let message_ty = format_ident!("{}__{}", mangled_self_ty, method.sig.ident);

            let inputs = method
                .sig
                .inputs
                .iter()
                .filter_map(|arg| match arg {
                    FnArg::Typed(arg) => Some(arg),
                    FnArg::Receiver(_) => None,
                })
                .collect::<Punctuated<_, Token![,]>>();

            // Extract normalized names and the type for each input for the message.
            let input_name = inputs
                .iter()
                .enumerate()
                .map(|(index, arg)| match &*arg.pat {
                    Pat::Ident(pat) => pat.ident.clone(),
                    _ => format_ident!("arg{}", index),
                })
                .collect::<Vec<_>>();
            let input_ty = inputs.iter().map(|arg| &arg.ty).collect::<Vec<_>>();
            let input_index = inputs.iter().enumerate().map(|(index, _)| Literal::usize_unsuffixed(index));

            let output_ty = match &method.sig.output {
                ReturnType::Default => quote! { () },
                ReturnType::Type(_, output) => output.to_token_stream(),
            };

            let send_fn = match method.sig.output {
                ReturnType::Default => quote! { send_message },
                ReturnType::Type(..) => quote! { send_request },
            };

            // If the message handler is an async fn, we need to append `.await` when we invoke
            // the method in order to ensure we fully execute the handler.
            let dot_await = match &method.sig.asyncness {
                Some(_) => quote! { .await },
                None => quote! {},
            };

            quote! {
                // Generate inherent impl on proxy type.
                impl #proxy_ty {
                    #vis fn #method_name(&self, #( #input_name: #input_ty, )*) -> thespian::Result<impl std::future::Future<Output = #output_ty>> {
                        self.inner.#send_fn(#message_ty( #( #input_name, )* ))
                    }
                }

                // Generate the type for the message.
                #[doc(hidden)]
                #[allow(bad_style)]
                pub struct #message_ty( #( #input_ty, )* );

                // Generate either a `Message` or a `Request` impl for the message type.
                impl thespian::Message for #message_ty {
                    type Actor = #self_ty;
                    type Output = #output_ty;

                    fn handle(self, actor: &mut Self::Actor) -> thespian::futures::future::BoxFuture<'_, Self::Output> {
                        thespian::futures::future::FutureExt::boxed(async {
                            actor.#method_name(#( self.#input_index, )*) #dot_await
                        })
                    }
                }
            }
        })
        .collect::<TokenStream>();

    // Append the generated code to the original code and return the whole thing as the
    // output.
    result.append_all(generated);
    result.into()
}

/// Generates a valid identifier from the given type.
///
/// Returns an error if the type is not a `Type::Path`. Otherwise, the segments of
/// the path are concatenated with `__` to create an identifier from the type
/// reference.
fn mangled_type_name(ty: &Type) -> syn::Result<Ident> {
    let path = match ty {
        Type::Path(path) => path,
        _ => return Err(Error::new_spanned(ty, "Unsupported type expression, only type paths are supported, e.g. `Foo` or `foo::bar::Baz`")),
    };

    let ident_string = path
        .path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("__");
    Ok(format_ident!("{}", ident_string))
}
