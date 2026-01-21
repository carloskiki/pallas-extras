use proc_macro::TokenStream;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn duplicate(input: TokenStream) -> TokenStream {
    // 1. Parse the input as a single string literal (e.g., "src/foo.rs")
    let input_lit = parse_macro_input!(input as LitStr);
    let relative_path = input_lit.value();

    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR env var is not set");
    
    let mut file_path = PathBuf::from(manifest_dir);
    file_path.push(&relative_path);

    // 3. Read the file content
    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            return syn::Error::new(
                input_lit.span(),
                format!("Could not read file {:?}: {}", file_path, e),
            )
            .to_compile_error()
            .into();
        }
    };

    // 4. Convert the string content into a TokenStream
    let file_tokens = match proc_macro2::TokenStream::from_str(&content) {
        Ok(t) => t,
        Err(e) => {
            return syn::Error::new(
                input_lit.span(),
                format!("Could not parse file content: {}", e),
            )
            .to_compile_error()
            .into();
        }
    };

    // // 5. CRITICAL STEP: Rebuild Tracking
    // // We append a dummy include_str! so Cargo knows to rebuild if the external file changes.
    // // Without this, changing the external file won't trigger a recompile of the main file.
    // let path_str = file_path.to_string_lossy();
    // let output = quote! {
    //     #file_tokens
    //     const _: &str = include_str!(#path_str); 
    // };

    file_tokens.into()
}
