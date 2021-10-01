use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, parse_str, Expr, Ident, ItemFn};

#[proc_macro_attribute]
pub fn memoize(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);

    // Get function name and swap
    let fn_name = input.sig.ident.clone();
    let real_fn_name = Ident::new(&format!("real_{}", fn_name.to_string()), Span::call_site());
    input.sig.ident = real_fn_name.clone();

    // Build correct function variant for hash map key
    let func_variant_str = fn_name.to_string().to_case(Case::UpperCamel);
    let func_variant = parse_str::<Expr>(&format!("Func::{}", func_variant_str)).unwrap();

    // Finally, build our new function and output our input (with changed function name)
    let output = quote! {
        pub fn #fn_name(&mut self) -> TokenOrNode<TOK, NODE> {
            let pos = self.parser.pos;
            let key = (#func_variant, pos);

            if let Some((node, new_pos)) = self.memos.get(&key) {
                self.parser.pos = *new_pos;
                node.to_owned()
            } else {
                let node = self.#real_fn_name();
                self.memos.insert(key, (node.clone(), pos));
                node
            }
        }

        #input
    };

    TokenStream::from(output)
}
