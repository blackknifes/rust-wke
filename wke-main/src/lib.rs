use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, ExprLit, ItemFn, Lit, Meta};

#[proc_macro_attribute]
pub fn main(attrs: TokenStream, item: TokenStream) -> TokenStream {
    // 解析输入的代码为一个函数
    let input = parse_macro_input!(item as ItemFn);

    //生成init代码
    let init_block = if !attrs.is_empty() {
        let meta = parse_macro_input!(attrs as Meta);
        if let Meta::NameValue(value) = meta {
            if !value.path.is_ident("dll") {
                panic!("wke::main is only support to dll");
            }

            match value.value {
                Expr::Path(path) => {
                    let fn_get_dll_path = path.path.get_ident().expect("path ident is empty");
                    quote! {
                        let dll_path = #fn_get_dll_path();
                        wke::init(&dll_path)?;
                    }
                }
                Expr::Lit(ExprLit {
                    attrs: _,
                    lit: Lit::Str(path),
                }) => {
                    let dll_path = path.token();
                    quote! {
                        wke::init(#dll_path)?;
                    }
                }
                _ => {
                    panic!("dll path must be function or string path");
                }
            }
        } else {
            panic!("attrs is not name value type");
        }
    } else {
        quote! {}
    };

    // 属性
    let fn_attrs = input.attrs;
    // 函数起那么
    let fn_sig = input.sig;
    // 函数体
    let fn_block = input.block;
    // 是否异步函数
    let is_async = fn_sig.asyncness.is_some();

    // 返回值类型
    let fn_output = match fn_sig.output {
        syn::ReturnType::Default => syn::parse_str("()").unwrap(),
        syn::ReturnType::Type(_, ty) => ty.clone(),
    };
    // 生成新的代码，将异步函数包装在一个 tokio runtime 上下文中执行
    let gen = if is_async {
        quote! {
            #(#fn_attrs)*
            fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()?;
                let localSet = LocalSet::new();
                localSet.block_on(&runtime, async {
                    let _join_handle: tokio::task::JoinHandle<#fn_output> = tokio::task::spawn_local(async move {
                        #init_block

                        #fn_block
                    });

                    loop {
                        match wke::run_once() {
                            wke::RunOnceFlag::Idle => tokio::task::yield_now().await,
                            wke::RunOnceFlag::Exit => break,
                            _ => continue,
                        }
                    }

                    std::result::Result::<(), Box<dyn std::error::Error>>::Ok(())
                })
            }
        }
    } else {
        quote! {
            compile_error!("The function must be async");
        }
    };
    gen.into()
}
