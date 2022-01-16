use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parser, parse_macro_input, spanned::Spanned, Data, DeriveInput, Error, Fields, Ident,
};

#[proc_macro_attribute]
pub fn component_collection(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let parser =
        syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_separated_nonempty;
    let types = match parser.parse(metadata) {
        Ok(t) => t,
        Err(e) => {
            return TokenStream::from(e.to_compile_error());
        }
    };
    let types: Vec<syn::Ident> = types.into_iter().collect();
    let field_names: Vec<syn::Ident> = types
        .iter()
        .map(|x| syn::Ident::new(&(x.to_string().to_lowercase() + "s"), x.span()))
        .collect();
    let struct_name = parse_macro_input!(input as DeriveInput).ident;
    let expanded = quote! {
        #[derive(Default, LameEcsComponents)]
        pub struct #struct_name{
            #(pub #field_names: Vec<Option<#types>>,)*
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro_derive(LameEcsComponents)]
pub fn derive_lame_ecs_components_fn(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_data = match &input.data {
        Data::Struct(s) => s,
        _ => {
            return TokenStream::from(
                Error::new(input.span(), "expected struct").to_compile_error(),
            );
        }
    };
    let field_names = match get_field_names(struct_data) {
        Some(f) => f,
        None => {
            return TokenStream::from(
                Error::new(input.span(), "expected named fields").to_compile_error(),
            );
        }
    };
    let types = match get_component_types(struct_data) {
        Some(f) => f,
        None => {
            return TokenStream::from(
                Error::new(input.span(), "expected Vec<Optional<Type>>").to_compile_error(),
            );
        }
    };
    let ident = &input.ident;
    let expanded = quote! {
        impl ComponentCollection for #ident {
            fn push_none(&mut self) {
                #(self.#field_names.push(None);)*
            }
            fn remove(&mut self, index: usize) {
                #(self.#field_names.remove(index);)*
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }

        #(impl Component for #types {
            fn get_vec(components: &mut dyn ComponentCollection) -> &mut Vec<Option<Self>> {
                &mut downcast_components_mut::<#ident>(components).#field_names
            }
        })*

    };
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn get_component_collection(input: TokenStream) -> TokenStream {
    let parser =
        syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_separated_nonempty;
    let ids = match parser.parse(input) {
        Ok(t) => t,
        Err(e) => {
            return TokenStream::from(e.to_compile_error());
        }
    };
    if ids.len() != 2 {
        return TokenStream::from(
            Error::new(
                ids.span(),
                "expected get_component_collection(world_instance, CollectionType",
            )
            .to_compile_error(),
        );
    }
    let mut iter = ids.iter();
    let world_instance = iter.next().unwrap();
    let collection_type = iter.next().unwrap();
    let expanded = quote! {
        downcast_components_mut::<#collection_type>(#world_instance.components.as_mut())
    };
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn component_iter(input: TokenStream) -> TokenStream {
    let parser =
        syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_separated_nonempty;
    let ids = match parser.parse(input) {
        Ok(t) => t,
        Err(e) => {
            return TokenStream::from(e.to_compile_error());
        }
    };
    if ids.len() <= 2 {
        return TokenStream::from(
            Error::new(
                ids.span(),
                "expected component_iter(world_instance, CollectionType, C0, C1 ...)",
            )
            .to_compile_error(),
        );
    }
    let mut ids_iter = ids.iter();
    let world = ids_iter.next().unwrap();
    let collection = ids_iter.next().unwrap();
    let fields: Vec<Ident> = ids_iter.map(get_field_from_type).collect();
    let captures: Vec<Ident> = fields
        .iter()
        .enumerate()
        .map(|x| Ident::new(&("f".to_owned() + &x.0.to_string()), x.1.span()))
        .collect();
    let expanded = quote! {
        itertools::izip!(
            #(&mut #collection.#fields,)*
            &#world.entities
        )
        .filter_map(|(#(#captures,)* e)| Some((#(#captures.as_mut()?,)* e)))
    };
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn create_world(input: TokenStream) -> TokenStream {
    let collection_ty = parse_macro_input!(input as Ident);
    let expanded = quote! {
        World::new(Box::new(#collection_ty::default()))
    };
    TokenStream::from(expanded)
}

fn get_field_from_type(ty: &Ident) -> Ident {
    let name = ty.to_string().to_lowercase() + "s";
    Ident::new(&name, ty.span())
}

fn get_field_names(data: &syn::DataStruct) -> Option<Vec<&syn::Ident>> {
    match &data.fields {
        Fields::Named(fs) => Some(fs.named.iter().filter_map(|f| f.ident.as_ref()).collect()),
        _ => None,
    }
}

fn is_type(ty: &syn::Type, name: &str) -> bool {
    match ty {
        syn::Type::Path(p)
            if p.qself.is_none()
                && p.path.segments.len() == 1
                && p.path.segments.first().unwrap().ident == name =>
        {
            true
        }
        _ => false,
    }
}

fn get_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    let argument = match ty {
        syn::Type::Path(p) => match &p.path.segments.first()?.arguments {
            syn::PathArguments::AngleBracketed(a) => a.args.first()?,
            _ => {
                return None;
            }
        },
        _ => {
            return None;
        }
    };
    match argument {
        syn::GenericArgument::Type(ty) => Some(ty),
        _ => None,
    }
}

fn get_component_type(ty: &syn::Type) -> Option<&syn::Type> {
    if !is_type(ty, "Vec") {
        return None;
    }
    let ty = get_inner_type(ty)?;
    if !is_type(ty, "Option") {
        return None;
    }
    get_inner_type(ty)
}

fn get_component_types(data: &syn::DataStruct) -> Option<Vec<&syn::Type>> {
    let field_types: Vec<&syn::Type> = match &data.fields {
        Fields::Named(fs) => Some(fs.named.iter().map(|f| &f.ty).collect()),
        _ => None,
    }?;
    let mut component_types = Vec::new();
    component_types.reserve(field_types.len());
    for ty in &field_types {
        component_types.push(get_component_type(ty)?);
    }
    Some(component_types)
}
