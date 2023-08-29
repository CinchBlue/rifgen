use itertools::MultiUnzip;
use quote::{format_ident, quote};
use syn::{ItemImpl, Type};

pub fn map_fn_arg_type(ty: Type) -> Type {
    match ty {
        string_type if string_type == Type::Verbatim(quote!(String)) => {
            Type::Verbatim(quote!(&str))
        }
        #[cfg_attr(test, deny(non_exhaustive_omitted_patterns))]
        other_type => other_type,
    }
}

pub fn map_fn_return_type(ty: Type) -> Type {
    match ty {
        string_type if string_type == Type::Verbatim(quote!(String)) => {
            Type::Verbatim(quote!(&str))
        }
        #[cfg_attr(test, deny(non_exhaustive_omitted_patterns))]
        other_type => other_type,
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
