use proc_macro::TokenStream;
use quote::quote;
use uuid::Uuid;

#[proc_macro]
pub fn print_unique(_input: TokenStream) -> TokenStream {
    let ident = Uuid::new_v4().as_u128();

    quote! {
        (#ident as u128)
    }.into()
}
