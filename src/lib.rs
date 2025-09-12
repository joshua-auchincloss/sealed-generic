
use convert_case::{Case, Casing};
use darling::*;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Ident, Meta,
    parse_macro_input,
};

#[derive(Debug, FromField)]
#[darling(attributes(define))]
struct Field {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

#[derive(Debug, FromMeta)]
enum Type {
    Ty(syn::Path),
    With {
        ty: syn::Path,

        #[darling(default, multiple)]
        derives: Vec<syn::Path>,

        #[darling(default, multiple)]
        attr: Vec<String>,
    },
}

impl Type {
    fn ident(&self) -> &syn::Path {
        match self {
            Self::Ty(path) => path,
            Self::With { ty, .. } => ty,
        }
    }

    fn derives(&self) -> Vec<syn::Path> {
        match self {
            Self::With { derives, .. } => derives.clone(),
            _ => vec![],
        }
    }

    fn attrs(&self) -> Vec<String> {
        match self {
            Self::With { attr, .. } => attr.clone(),
            _ => vec![],
        }
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(define), supports(struct_any))]
struct Input {
    vis: syn::Visibility,
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), Field>,

    #[darling(default, multiple)]
    types: Vec<Type>,

    #[darling(default, multiple)]
    derives: Vec<syn::Path>,

    #[darling(default, multiple)]
    attr: Vec<String>,
}

impl Input {
    fn merge<T: Clone>(a1: Vec<T>, a2: &Vec<T>) -> Vec<T> {
        let mut a1 = a1;
        let mut a2 = a2.clone();
        a1.append(&mut a2);
        a1
    }

    fn derives(&self, derives: Vec<syn::Path>) -> Vec<syn::Path> {
        Self::merge(derives, &self.derives)
    }

    fn attrs(&self, attrs: Vec<String>) -> Vec<String> {
        Self::merge(attrs, &self.attr)
    }
}

enum TypeOrGeneric {
    Type(proc_macro2::TokenStream),
    Generic(Ident),
}

impl ToTokens for Input {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = self.ident.clone();

        let vis = self.vis.clone();
        let mut tys = vec![];
        for param in &self.generics.params {
            match param {
                syn::GenericParam::Const(..) => {}
                syn::GenericParam::Lifetime(..) => {}
                syn::GenericParam::Type(ty) => tys.push(ty.ident.clone()),
            }
        }

        let mut fields = vec![];
        let mut fields_eq = vec![];

        for f in self.data.as_ref().take_struct().unwrap() {
            let field = f.ident.clone().unwrap();
            let ty = f.ty.clone();
            let ident = Ident::from_string(
                &quote!(
                    #ty
                )
                .to_string(),
            )
            .unwrap();

            fields_eq.push(quote!(
                #field: value.#field,
            ));
            if tys.contains(&ident) {
                fields.push(TypeOrGeneric::Generic(field))
                // is generic
            } else {
                fields.push(TypeOrGeneric::Type(quote!(
                    #field: #ty,
                )))
            }
        }

        let fields_eq: proc_macro2::TokenStream = fields_eq.into_iter().collect();

        for t in &self.types {
            let tt = t.ident();
            let new_ident = Ident::from_string(
                &format!(
                    "{}{}",
                    ident.to_string(),
                    quote!(#tt).to_string().to_case(Case::Pascal)
                )
                .to_case(Case::Pascal),
            )
            .unwrap();

            let new_fields: proc_macro2::TokenStream = fields
                .iter()
                .map(|it| match it {
                    TypeOrGeneric::Generic(field) => {
                        quote!(
                            #field: #tt,
                        )
                    }
                    TypeOrGeneric::Type(tt) => tt.clone(),
                })
                .collect();

            let derives: proc_macro2::TokenStream = self
                .derives(t.derives())
                .iter()
                .map(|it| quote::quote!(#it,))
                .collect();

            let atts: proc_macro2::TokenStream = self
                .attrs(t.attrs())
                .iter()
                .map(|it| {
                    let it: Meta = syn::parse_str(&it).unwrap();
                    quote::quote!(#[#it])
                })
                .collect();

            tokens.extend(quote! {
                #[derive(#derives)]
                #atts
                #vis struct #new_ident {
                    #new_fields
                }
            });

            tokens.extend(quote! {
                impl From<#new_ident> for #ident<#tt> {
                    fn from(value: #new_ident) -> Self {
                        Self {
                            #fields_eq
                        }
                    }
                }
            })
        }
    }
}

#[proc_macro_derive(DefinedGeneric, attributes(define))]
pub fn def_gen(input: TokenStream) -> TokenStream {
    let parsed = Input::from_derive_input(&parse_macro_input!(input)).unwrap();
    quote::quote!(#parsed).into()
}
