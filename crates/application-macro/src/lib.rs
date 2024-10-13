use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, ItemStatic};

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemStatic);
    let ident = &item.ident;

    quote! {
        #item

        fn main() {
            use clap::Parser;
            let args = Parser::parse();
            if let Err(err) = #ident.main(args) {
                eprintln!("{err}");
            }
        }
    }
    .into()
}
