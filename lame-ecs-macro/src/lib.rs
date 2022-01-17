use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parser, parse_macro_input, spanned::Spanned, Data, DeriveInput, Error, Fields, Ident,
};

#[proc_macro]
pub fn create_component_collection(input: TokenStream) -> TokenStream {
    let parser =
        syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_separated_nonempty;
    let types = match parser.parse(input) {
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
    let expanded = quote! {
        #[derive(Default, lame_ecs::LameEcsComponents)]
        pub struct LameEcsComponentCollection {
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
        impl lame_ecs::ComponentCollection for #ident {
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

        #(impl lame_ecs::Component for #types {
            fn get_vec(components: &mut dyn lame_ecs::ComponentCollection) -> &mut Vec<Option<Self>> {
                &mut lame_ecs::downcast_components_mut::<LameEcsComponentCollection>(components).#field_names
            }
        })*

    };
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn component_iter_mut(input: TokenStream) -> TokenStream {
    let data = match get_component_iter_input(input) {
        Ok(d) => d,
        Err(e) => {
            return e;
        }
    };
    let world = &data.world;
    let fields = &data.fields;
    let captures = &data.captures;
    let expanded = quote! {
        {
            use lame_ecs::itertools;
            let components = lame_ecs::downcast_components_mut::<LameEcsComponentCollection>(#world.components.as_mut());
            itertools::izip!(
                #(&mut components.#fields,)*
                &#world.entities
            )
            .filter_map(|(#(#captures,)* e)| Some((#(#captures.as_mut()?,)* e)))
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn component_iter(input: TokenStream) -> TokenStream {
    let data = match get_component_iter_input(input) {
        Ok(d) => d,
        Err(e) => {
            return e;
        }
    };
    let world = &data.world;
    let fields = &data.fields;
    let captures = &data.captures;
    let expanded = quote! {
        {
            use lame_ecs::itertools;
            let components = lame_ecs::downcast_components::<LameEcsComponentCollection>(#world.components.as_ref());
            itertools::izip!(
                #(&components.#fields,)*
                &#world.entities
            )
            .filter_map(|(#(#captures,)* e)| Some((#(#captures.as_ref()?,)* e)))
        }
    };
    TokenStream::from(expanded)
}

#[proc_macro]
pub fn create_world(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as syn::parse::Nothing);
    let expanded = quote! {
        lame_ecs::World::new(Box::new(LameEcsComponentCollection::default()))
    };
    TokenStream::from(expanded)
}

struct ComponentIterInput {
    world: Ident,
    fields: Vec<Ident>,
    captures: Vec<Ident>,
}

fn get_component_iter_input(input: TokenStream) -> Result<ComponentIterInput, TokenStream> {
    let parser =
        syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_separated_nonempty;
    let ids = parser
        .parse(input)
        .map_err(|e| TokenStream::from(e.to_compile_error()))?;
    if ids.len() <= 1 {
        return Err(TokenStream::from(
            Error::new(
                ids.span(),
                "expected component_iter(world_instance, C0, C1 ...)",
            )
            .to_compile_error(),
        ));
    }
    let mut ids_iter = ids.iter();
    let world = ids_iter.next().unwrap().clone();
    let fields: Vec<Ident> = ids_iter.map(get_field_from_type).collect();
    let captures: Vec<Ident> = fields
        .iter()
        .enumerate()
        .map(|x| Ident::new(&("f".to_owned() + &x.0.to_string()), x.1.span()))
        .collect();
    Ok(ComponentIterInput {
        world,
        fields,
        captures,
    })
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
