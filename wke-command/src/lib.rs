use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn command(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    // 解析输入的代码为一个函数
    let input = parse_macro_input!(item as ItemFn);
    let block = &input.block;
    let sig = &input.sig;
    let fn_name = &sig.ident;
    let inputs = &sig.inputs;
    let output = &sig.output;

    let async_gen = if !sig.asyncness.is_some() {
        quote! {
            compile_error!("command function must be async")
        }
    } else {
        quote! {
            async
        }
    };

    let args = inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Receiver(_) => {
                quote! {compile_error!("command function cannot include self argument")}
            }
            syn::FnArg::Typed(arg) => {
                let name = &arg.pat;
                let ty = &arg.ty;
                quote! {
                    #name: #ty,
                }
            }
        })
        .collect::<Vec<_>>();

    let vars = inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Receiver(_) => {
                quote! {compile_error!("command function cannot include self argument")}
            }
            syn::FnArg::Typed(arg) => {
                let name = &arg.pat;
                quote! {
                    #name,
                }
            }
        })
        .collect::<Vec<_>>();

    let gen = quote! {
        #async_gen fn #fn_name(value: serde_json::Value) #output {
            #[derive(Deserialize)]
            struct TempRequestParam {
                #(#args)*
            }

            let TempRequestParam {
                #(#vars)*
            } = serde_json::from_value(value)?;

            {
                #block
            }
        }
    };

    gen.into()
}
