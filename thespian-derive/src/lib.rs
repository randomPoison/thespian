extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn actor(args: TokenStream, input: TokenStream) -> TokenStream {
    dbg!(&args);
    dbg!(&input);

    input
}
