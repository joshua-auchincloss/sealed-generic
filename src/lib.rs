use convert_case::{Case, Casing};
use darling::*;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{Ident, Meta, TypePath, parse_macro_input};

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

    #[darling(default)]
    sealed: bool,
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
    Type { field: Ident, ty: syn::Type },
    Generic(Ident),
    GenericWithNested { field: Ident, path: TypePath },
}

impl ToTokens for Input {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = self.ident.clone();

        let vis = self.vis.clone();

        let generic: Ident;

        let tys = self.generics.type_params().collect::<Vec<_>>();
        let lts = self.generics.lifetimes().collect::<Vec<_>>();

        if tys.len() > 1 {
            panic!("only one generic type may be specified")
        }

        generic = tys.get(0).unwrap().ident.clone();

        let mut fields = vec![];
        let mut fields_eq = vec![];
        let mut field_lts = HashMap::new();

        for f in self.data.as_ref().take_struct().expect("struct") {
            let field = f.ident.clone().expect("ident");
            let ty = f.ty.clone();

            fields_eq.push(quote!(
                #field: value.#field,
            ));

            let mut with_path = |path: &syn::TypePath| {
                let segment = path.path.segments.first().expect("path segment");
                let ident = segment.ident.clone();
                let field = field.clone();
                let ty = ty.clone();
                if generic == ident {
                    fields.push(TypeOrGeneric::Generic(field))
                } else if !segment.arguments.is_empty() {
                    fields.push(TypeOrGeneric::GenericWithNested {
                        field: field,
                        path: path.clone(),
                    })
                } else {
                    fields.push(TypeOrGeneric::Type { field, ty })
                }
            };

            match ty.clone() {
                syn::Type::Reference(typeref) => {
                    if let Some(lt) = typeref.lifetime {
                        field_lts.insert(field.clone(), lt);
                    }
                    match *typeref.elem {
                        syn::Type::Path(ref path) => with_path(path),
                        ty => panic!("unknown type: {ty:#?}"),
                    }
                }
                syn::Type::Path(path) => {
                    with_path(&path);
                }
                _ => panic!("unknown type"),
            }
        }

        let fields_eq: proc_macro2::TokenStream = fields_eq.into_iter().collect();

        let seal_mod =
            Ident::from_string(&("Sealed".to_string() + &ident.to_string()).to_case(Case::Pascal))
                .expect("sealed trait ident to parse");

        if self.sealed {
            tokens.extend(quote!(
                pub(crate) trait #seal_mod {}
            ));
        }

        let get_field_lt = |field: &Ident, ty: proc_macro2::TokenStream| {
            let field_pre = if let Some(lt) = field_lts.get(&field) {
                let ex: proc_macro2::TokenStream = format!("&{} {}", quote!(#lt), ty.to_string())
                    .parse()
                    .expect("type with lifetime");

                quote!(#field: #ex,)
            } else {
                quote!(
                    #field: #ty,
                )
                .into()
            };

            field_pre
        };

        let has_lts = !lts.is_empty();
        let lts: proc_macro2::TokenStream = lts.iter().map(|lt| quote!(#lt,)).collect();
        let lt_quo = if has_lts { quote!(<#lts>) } else { quote!() };

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
            .expect("type ident to parse");

            let new_fields: proc_macro2::TokenStream = fields
                .iter()
                .map(|it| match it {
                    TypeOrGeneric::Generic(field) => get_field_lt(field, quote!(#tt)),
                    TypeOrGeneric::GenericWithNested { field, path } => {
                        let new_ty = quote!(#path).to_string().replace(" ", "").replace(
                            &format!("<{}>", generic.to_string()),
                            &quote!(<#tt>).to_string(),
                        );
                        let as_path = syn::Path::from_string(&new_ty).unwrap();
                        get_field_lt(field, quote!(#as_path))
                    }
                    TypeOrGeneric::Type { field, ty } => get_field_lt(field, quote!(#ty)),
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
                    let it: Meta = syn::parse_str(&it).expect("meta to parse");
                    quote::quote!(#[#it])
                })
                .collect();

            tokens.extend(quote! {
                #[derive(#derives)]
                #atts
                #vis struct #new_ident #lt_quo {
                    #new_fields
                }
            });

            if self.sealed {
                tokens.extend(quote!(
                    impl #lt_quo #seal_mod for #tt #lt_quo {}
                ));
            }

            tokens.extend(quote! {
                impl #lt_quo From<#new_ident #lt_quo> for #ident<#lts #tt> {
                    fn from(value: #new_ident #lt_quo) -> Self {
                        Self {
                            #fields_eq
                        }
                    }
                }

                impl #lt_quo From<#ident<#lts #tt>> for #new_ident #lt_quo {
                    fn from(value: #ident<#lts #tt>) -> Self {
                        Self {
                            #fields_eq
                        }
                    }
                }
            })
        }
    }
}

#[proc_macro_derive(SealedGeneric, attributes(define))]
pub fn def_gen(input: TokenStream) -> TokenStream {
    let parsed = Input::from_derive_input(&parse_macro_input!(input)).expect("parse");
    quote::quote!(#parsed).into()
}
