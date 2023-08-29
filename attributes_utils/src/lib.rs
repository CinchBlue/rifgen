use itertools::MultiUnzip;
use quote::{format_ident, quote};
use syn::{ItemImpl, Type, Path};
use syn::GenericArgument;
use syn::PathArguments;

fn extract_type_from_option(ty: &Type, outer_type: &str) -> Option<Type> {
    let path_is_option = |path: &Path| -> bool {
        path.leading_colon.is_none()
            && path.segments.len() == 1
            && path.segments.iter().next().unwrap().ident == outer_type
    };

    match ty {
        Type::Path(typepath) if typepath.qself.is_none() && path_is_option(&typepath.path) => {
            // Get the first segment of the path (there is only one, in fact: "Option"):
            let type_params = &typepath.path.segments.iter().next().unwrap().arguments;
            // It should have only on angle-bracketed param ("<String>"):
            let generic_arg = match type_params {
                PathArguments::AngleBracketed(params) => params.args.iter().next().unwrap(),
                _ => return None,
            };
            // This argument must be a type:
            match generic_arg {
                GenericArgument::Type(ty) => return Some(ty.clone()),
                _ => return None,
            }
        }
        _ => return None,
    }
}


pub fn map_fn_common_arg_return_type(ty: &Type) -> Option<Type> {
    match ty {

        string_type if string_type == &Type::Verbatim(quote!(String)) => {
            Some(Type::Verbatim(quote!(&str)))
        }
        option_type if extract_type_from_option(option_type, "Option").is_some() => {
            let inner_type = map_fn_common_arg_return_type(&extract_type_from_option(option_type, "Option").unwrap());
            println!("inner_type: {:?}", inner_type);
            Some(Type::Verbatim(quote!(Option<#inner_type>)))
        }
        option_type if extract_type_from_option(option_type, "Vec").is_some() => {
            let inner_type = map_fn_common_arg_return_type(&extract_type_from_option(option_type, "Vec").unwrap());
            println!("inner_type: {:?}", inner_type);
            Some(Type::Verbatim(quote!(&[<#inner_type>])))
        }
        Type::Path(path) => {
            let last_segment = path.path.segments.last().unwrap();
            Some(Type::Verbatim(quote!(#last_segment)))
        }
        #[cfg_attr(test, deny(non_exhaustive_omitted_patterns))]
        _ => None,
    }

}

pub fn map_fn_arg_type(ty: Type) -> Type {
    if let Some(ty) = map_fn_common_arg_return_type(&ty) {
        ty
    } else {
        ty
    }
}

pub fn map_fn_return_type(ty: Type) -> Type {
    if let Some(ty) = map_fn_common_arg_return_type(&ty) {
        ty
    } else {
        ty
    }
}

pub fn generate_impl_block(item: &syn::ItemStruct) -> ItemImpl {
    let name = item.clone().ident;
    let vis = item.clone().vis;
    let fields = match item.clone().fields {
        syn::Fields::Named(fields) => fields.named.into_iter().collect::<Vec<_>>(),
        _ => unreachable!(),
    };

    let (f_setter, f_getter, f_ident, f_vis, f_ty): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) =
        fields
            .iter()
            .cloned()
            .filter_map(|f| {
                if let Some(ident) = f.ident {
                    Some((
                        format_ident!("set_{}", ident),
                        format_ident!("get_{}", ident),
                        ident,
                        f.vis,
                        f.ty,
                    ))
                } else {
                    None
                }
            })
            .multiunzip();

    let f_ty_args = f_ty.iter().cloned().map(map_fn_arg_type).collect::<Vec<_>>();
    let f_ty_return = f_ty.iter().cloned().map(map_fn_return_type).collect::<Vec<_>>();


    let impl_block = quote::quote! {
         impl #name {
            #[generate_interface(constructor)]
            #vis fn new(
                #(#f_ident: #f_ty_args),*
            ) -> #name {
                #name {
                    #(#f_ident),*
                }
            }
            #(
                #[generate_interface]
                #f_vis fn #f_setter(&mut self, #f_ident: #f_ty_args) {
                    self.#f_ident = #f_ident;
                }

                #[generate_interface]
                #f_vis fn #f_getter(&self) -> #f_ty_return {
                    (&self.#f_ident).clone()
                }
            )*
        }
    };

    syn::parse2(impl_block).unwrap()
}
