use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};
use walkdir::WalkDir;

#[proc_macro]
pub fn get_files_recursive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let dir_path_str = input.value();

    let mut file_paths = Vec::new();
    for entry in WalkDir::new(&dir_path_str).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            file_paths.push(entry.path().to_string_lossy().into_owned());
        }
    }

    let expanded = quote! {
        vec![
            #(#file_paths.into(),)*
        ]
    };

    expanded.into()
}
