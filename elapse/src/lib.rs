use proc_macro::TokenStream;

mod elapse;

/// ## Simple procedural macro for speed testing
/// 
/// ### Parameters:
/// `#[elapsed(task_name: &str)]`
/// 
/// You can use it for function or expression.
/// - **function**:
/// ```rust
/// #[elapsed("")] // leave taskname empty to auto fill it with function name
/// fn certain_test_func() {
///     println!("this is a testing function");
/// }
/// ```
/// 
/// and the expected output is 
/// ```text
/// this is a testing function
/// task "certain_test_func" consumes 3.202µs
/// ```
/// - **expression**:
/// ```rust
/// fn certain_test_func() {
///     let mut result = 3;
///     #[elapsed("loop")] // since there is no function name behind this macro, so the task_name is needed
///     loop {
///         result += 1;
///         if result > 10 {
///             break;
///         }
///     }
/// }
/// ```
/// 
/// also, the expected output is
/// ```text
/// task "loop" consumes 148ns
/// ```
#[proc_macro_attribute]
pub fn elapsed(args: TokenStream, func: TokenStream) -> TokenStream {
    elapse::elapsed(args, func)
}

#[proc_macro_attribute]
pub fn elapsed_multi_thread(args: TokenStream, target: TokenStream) -> TokenStream {
    elapse::elapsed_multi_thread(args, target)
}

#[proc_macro_attribute]
pub fn get_abstract_syntax_token(_input: TokenStream, attr: TokenStream) -> TokenStream {
    elapse::get_abstract_syntax_token(_input, attr)
}
