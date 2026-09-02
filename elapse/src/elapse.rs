use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, Expr::{self}, ItemFn, Lit, Stmt, Token, parse::Parse, parse_macro_input, punctuated::Punctuated};

fn get_type_info(literal: &Lit) -> &'static str {
    match literal {
        Lit::Bool(_) => "boolean",
        Lit::Byte(_) => "byte",
        Lit::ByteStr(_) => "byte str",
        Lit::Char(_) => "char",
        Lit::Float(_) => "float",
        Lit::Int(_) => "integer",
        Lit::Str(_) => "string",
        Lit::Verbatim(_) => "verbatim",
    }
}

enum ElapseTarget {
    Fn(ItemFn),
    Expr(Expr)
}

impl Parse for ElapseTarget {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if let Ok(fn_item) = input.parse::<ItemFn>() {
            return Ok(ElapseTarget::Fn(fn_item));
        }

        match input.parse::<Expr>() {
            Ok(expr) => Ok(ElapseTarget::Expr(expr)),
            Err(_) => {
                Err(Error::new(
                    input.span(),
                    "expected a function definition or a block"
                ))
            }
        }
    }
}

struct ElapseArgs {
    name: String, 
    iterations: usize,
}

impl Parse for ElapseArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args = Punctuated::<Lit, Token![,]>::parse_terminated(input)?;
        if args.is_empty() {
            return Err(Error::new(input.span(), "expected as least a task name"));
        }

        let task_name = match &args[0] {
            Lit::Str(s) => s.value(),
            _ => return Err(Error::new(args[0].span(), format!("expected a string, exactly {}", get_type_info(&args[0]))))
        };

        let iterations = if args.len() >= 2 {
            match &args[1] {
                Lit::Int(iter) => 
                    iter.base10_parse::<usize>()
                        .map_err(|_| Error::new(iter.span(), "invalid integer"))?,
                _ => return Err(Error::new(args[1].span(), format!("expected a integer, exactly {}", get_type_info(&args[1]))))
            }
        } else {
            1
        };

        Ok(ElapseArgs { name: task_name, iterations })
    }
}

pub(crate) fn elapsed(attr: TokenStream, target: TokenStream) -> TokenStream {
    // process attr (as Lit) --- which just defines the task name as TimeGuard.name: &'static
    let mut task_name = match syn::parse::<Lit>(attr.clone()) {
        Ok(Lit::Str(task_name)) => {
            task_name.value()
        },
        Ok(invalid_case) => {
            return Error::new_spanned(invalid_case.clone(), format!("expected a string, in fact a `{}`", get_type_info(&invalid_case)))
                .to_compile_error().into()
        },
        Err(e) => {
            return e.to_compile_error().into()
        }
    };

    let caller = match syn::parse::<ElapseTarget>(target) {
        Ok(inner_item) => inner_item,
        Err(e) => return e.to_compile_error().into()
    };

    // process target TokenStream
    match caller {
        ElapseTarget::Fn(func) => {
            let func_vis = &func.vis;
            let func_blk = &func.block;

            let func_decl = &func.sig;
            let func_name = &func_decl.ident;
            let func_generic = &func_decl.generics;
            let func_input = &func_decl.inputs;
            let func_ret = &func_decl.output;

            if task_name.is_empty() {
                task_name = func_name.to_string();
            }

            quote! {
                #func_vis fn #func_name #func_generic(#func_input) #func_ret {
                    // use runtime::TimeGuard;

                    let time_counter = ::runtime::TimeGuard::new(#task_name.into());

                    #func_blk
                }
            }
        },
        ElapseTarget::Expr(expr) => {
            quote! {
                {
                    let time_counter = ::runtime::TimeGuard::new(#task_name.into());

                    #expr
                }
            }
        }
    }.into()
}

// #[elapsed_milti_thread(taskname: &'static str, times: usize)]
pub(crate) fn elapsed_multi_thread(attr: TokenStream, target: TokenStream) -> TokenStream {
    let elapse_args = match syn::parse::<ElapseArgs>(attr) {
        Ok(args) => args,
        Err(e) => return e.to_compile_error().into(),
    };

    let (mut task_name, routine) = (elapse_args.name, &elapse_args.iterations);

    // process the target TokenStream
    let content = match syn::parse::<ElapseTarget>(target) {
        Ok(content) => content,
        Err(e) => return e.to_compile_error().into(),
    };

    let main_task = match content {
        ElapseTarget::Fn(func) => {
            let func_vis = &func.vis;
            let func_blk = &func.block;

            let func_decl = &func.sig;
            let func_name = &func_decl.ident;
            let func_generic = &func_decl.generics;
            let func_input = &func_decl.inputs;
            let func_ret = &func_decl.output;

            if task_name == "" {
                task_name = func_name.to_string();
            }

            // only those functions without input args are supported
            quote! {
                #func_vis fn #func_name #func_generic(#func_input) #func_ret {
                    use std::sync::{Arc, Mutex};

                    // init basic runtime env
                    let counter = Arc::new(Mutex::new(Duration::ZERO));
                    let mut receivers = vec![];

                    if let Some(pool) = ::runtime::ThreadPool::new(#routine) {
                        for i in 0..#routine {
                            let shared = Arc::clone(&counter);
                            let rx = pool.execute_with_reply(move || {
                                let _guard = ::runtime::ArcTimeGuard::new(shared);

                                #func_blk
                            });
                            receivers.push(rx);
                        }

                        pool.join();
                    }

                    let total = counter.lock().expect("Mutex is posioned");
                    let average = *total / #routine as u32;
                    println!("average time usage of task \"{}\" is {:?}", #task_name, average);

                    match receivers[0].recv() {
                        Ok(inner) => inner,
                        Err(_) => panic!()
                    }
                }
            }
        },
        ElapseTarget::Expr(expr) => {
            quote! {
                {
                    use std::sync::{Arc, Mutex};

                    let counter = Arc::new(Mutex::new(Duration::ZERO));
                    let mut receivers = vec![];

                    if let Some(pool) = ::runtime::ThreadPool::new(#routine) {
                        for i in 0..#routine {
                            let shared = Arc::clone(&counter);
                            let rx = pool.execute_with_reply(move || {
                                let _guard = ::runtime::ArcTimeGuard::new(shared);

                                #expr
                            });
                            receivers.push(rx);
                        }
                        pool.join();
                    }

                    let total = counter.lock().expect("Mutex is posioned");
                    let average = *total / #routine as u32;
                    println!("average time usage of task \"{}\" is {:?}", #task_name, average);

                    // handle return value
                    match receivers[0].recv() {
                        Ok(inner) => inner,
                        Err(_) => panic!()
                    }
                }
            }
        }
    };
    
    main_task.into()
}

pub(crate) fn get_abstract_syntax_token(_input: TokenStream, attr: TokenStream) -> TokenStream {
    let block = parse_macro_input!(attr as Stmt);
    
    match &block {
        Stmt::Local(local) => {
            eprintln!("type Local with value {:#?}", local);
        },
        Stmt::Item(syntax_item) => {
            eprintln!("type Item with value {:#?}", syntax_item);
        },
        Stmt::Expr(expr) => {
            eprintln!("type Expr with value{:#?}", expr);
        },
        Stmt::Semi(semi, _) => {
            eprintln!("type Semi with value {:#?}", semi);
        }
    }

    quote! {#block}.into()
}