use convert_case::Casing;
use syn::{spanned::Spanned, LitStr};

proc_easy::easy_token!(skip);
proc_easy::easy_token!(with);
proc_easy::easy_token!(range);
proc_easy::easy_token!(name);
proc_easy::easy_token!(multiline);
proc_easy::easy_token!(snake_case);
proc_easy::easy_token!(camelCase);
proc_easy::easy_token!(PascalCase);
proc_easy::easy_token!(SCREAMING_SNAKE_CASE);
proc_easy::easy_token!(UPPER_SNAKE_CASE);
proc_easy::easy_token!(Train);
proc_easy::easy_token!(kebab);
proc_easy::easy_token!(case);
proc_easy::easy_token!(Case);
proc_easy::easy_token!(rename_all);
proc_easy::easy_token!(toggle_switch);
proc_easy::easy_token!(transparent);
proc_easy::easy_token!(tags);
proc_easy::easy_token!(inlined);
proc_easy::easy_token!(combobox);

proc_easy::easy_parse! {
    #[derive(Clone, Copy)]
    struct KebabCase {
        kebab: kebab,
        minus: syn::Token![-],
        case: case,
    }
}

proc_easy::easy_parse! {
    #[derive(Clone, Copy)]
    struct TrainCase {
        kebab: Train,
        minus: syn::Token![-],
        case: Case,
    }
}

proc_easy::easy_parse! {
    #[derive(Clone, Copy)]
    enum RenameCase {
        SnakeCase(snake_case),
        CamelCase(camelCase),
        PascalCase(PascalCase),
        ScreamingSnakeCase(SCREAMING_SNAKE_CASE),
        UpperSnakeCase(UPPER_SNAKE_CASE),
        KebabCase(KebabCase),
        TrainCase(TrainCase),
    }
}

impl RenameCase {
    fn rename(&self, ident: &syn::Ident) -> syn::LitStr {
        let ident = ident.to_string();

        let converted = match self {
            RenameCase::SnakeCase(_) => ident.to_case(convert_case::Case::Snake),
            RenameCase::CamelCase(_) => ident.to_case(convert_case::Case::Camel),
            RenameCase::PascalCase(_) => ident.to_case(convert_case::Case::Pascal),
            RenameCase::ScreamingSnakeCase(_) => ident.to_case(convert_case::Case::ScreamingSnake),
            RenameCase::UpperSnakeCase(_) => ident.to_case(convert_case::Case::UpperSnake),
            RenameCase::KebabCase(_) => ident.to_case(convert_case::Case::Kebab),
            RenameCase::TrainCase(_) => ident.to_case(convert_case::Case::Train),
        };

        syn::LitStr::new(&converted, ident.span())
    }
}

proc_easy::easy_argument_value! {
    struct RenameAll {
        rename_all: rename_all,
        case: RenameCase,
    }
}

proc_easy::easy_argument! {
    struct With {
        with: with,

        /// Expression type must implement `FnOnce(&mut FieldType, &mut egui::Ui, &::egui_probe::Style) -> egui::Response`
        expr: syn::Expr,
    }
}

proc_easy::easy_argument! {
    struct ProbeAs {
        probe_as: syn::Token![as],

        /// Expression type must implement `FnOnce(&mut FieldType) -> R`
        /// and R must implement `EguiProbeWrapper`
        expr: syn::Expr,
    }
}

proc_easy::easy_argument_value! {
    struct Range {
        range: range,

        /// `EguiProbeRange<FieldType, ExprType>` must implement `EguiProbeWrapper`.
        expr: syn::Expr,
    }
}

proc_easy::easy_argument_value! {
    struct Name {
        name: name,
        literal: syn::LitStr,
    }
}

proc_easy::easy_argument_group! {
    enum FieldProbeKind {
        Range(Range),
        With(With),
        ProbeAs(ProbeAs),
        Multiline(multiline),
        ToggleSwitch(toggle_switch),
    }
}

proc_easy::easy_attributes! {
    @(egui_probe)
    struct FieldAttributes {
        // If `skip` is present, the field will be skipped.
        // Warning will be generated if other attributes are present with `skip`.
        skip: Option<skip>,
        name: Option<Name>,
        kind : Option<FieldProbeKind>,
    }
}

proc_easy::easy_argument! {
    struct WhereClause {
        where_token: syn::Token![where],
        predicates: proc_easy::EasyTerminated<syn::WherePredicate, syn::Token![,]>,
    }
}

proc_easy::easy_argument_group! {
    enum TagsKind {
        Inlined(inlined),
        ComboBox(combobox),
    }
}

proc_easy::easy_argument! {
    struct EnumTags {
        tags: tags,
        kind: TagsKind,
    }
}

proc_easy::easy_attributes! {
    @(egui_probe)
    struct TypeAttributes {
        rename_all: Option<RenameAll>,
        where_clause: Option<WhereClause>,
        transparent: Option<transparent>,
        tags: Option<EnumTags>,
    }
}

proc_easy::easy_attributes! {
    @(egui_probe)
    struct VariantAttributes {
        name: Option<Name>,
        transparent: Option<transparent>,
    }
}

fn make_name(
    name: Option<Name>,
    ident: Option<&syn::Ident>,
    rename_case: Option<RenameCase>,
) -> syn::LitStr {
    match name {
        Some(name) => name.literal,
        None => match (ident, rename_case) {
            (None, _) => LitStr::new("", proc_macro2::Span::call_site()),
            (Some(ident), None) => LitStr::new(&ident.to_string(), ident.span()),
            (Some(ident), Some(rename_case)) => rename_case.rename(ident),
        },
    }
}

fn field_probe(
    idx: usize,
    field: &syn::Field,
    rename_case: Option<RenameCase>,
) -> syn::Result<Option<proc_macro2::TokenStream>> {
    let attributes: FieldAttributes = proc_easy::EasyAttributes::parse(&field.attrs, field.span())?;

    if attributes.skip.is_some() {
        match attributes.name {
            Some(name) => {
                return Err(syn::Error::new_spanned(
                    name.name,
                    "Cannot name skipped field",
                ))
            }
            None => {}
        }
        match attributes.kind {
            Some(FieldProbeKind::With(with)) => {
                return Err(syn::Error::new_spanned(
                    with.with,
                    "Cannot use `with` attribute for skipped field",
                ))
            }
            Some(FieldProbeKind::ProbeAs(probe_as)) => {
                return Err(syn::Error::new_spanned(
                    probe_as.probe_as,
                    "Cannot use `as` attribute for skipped field",
                ))
            }
            Some(FieldProbeKind::Range(range)) => {
                return Err(syn::Error::new_spanned(
                    range.range,
                    "Cannot use `range` attribute for skipped field",
                ))
            }
            Some(FieldProbeKind::Multiline(multiline)) => {
                return Err(syn::Error::new_spanned(
                    multiline,
                    "Cannot use `multiline` attribute for skipped field",
                ))
            }
            Some(FieldProbeKind::ToggleSwitch(toggle_switch)) => {
                return Err(syn::Error::new_spanned(
                    toggle_switch,
                    "Cannot use `toggle_switch` attribute for skipped field",
                ))
            }
            None => {}
        }

        return Ok(None);
    }

    let name = make_name(attributes.name, field.ident.as_ref(), rename_case);

    let binding = quote::format_ident!("___{}", idx);

    let tokens = match attributes.kind {
        None => {
            quote::quote_spanned! {field.span() =>
                _f(#name, #binding)
            }
        }
        Some(FieldProbeKind::With(with)) => {
            let expr = with.expr;
            quote::quote_spanned! {field.span() =>
                _f(#name, &mut probe_with(#expr, #binding))
            }
        }
        Some(FieldProbeKind::ProbeAs(probe_as)) => {
            let expr = probe_as.expr;
            quote::quote_spanned! {field.span() =>
                _f(#name, &mut probe_as(#expr, #binding))
            }
        }
        Some(FieldProbeKind::Range(range)) => {
            let expr = range.expr;
            quote::quote_spanned! {field.span() =>
                _f(#name, &mut probe_range(#expr, #binding))
            }
        }
        Some(FieldProbeKind::Multiline(_)) => {
            quote::quote_spanned! {field.span() =>
                _f(#name, &mut probe_multiline(#binding))
            }
        }
        Some(FieldProbeKind::ToggleSwitch(_)) => {
            quote::quote_spanned! {field.span() =>
                _f(#name, &mut probe_toggle_switch(#binding))
            }
        }
    };

    Ok(Some(tokens))
}

fn variant_selected(
    variant: &syn::Variant,
    rename_case: Option<RenameCase>,
) -> syn::Result<proc_macro2::TokenStream> {
    let attributes: VariantAttributes =
        proc_easy::EasyAttributes::parse(&variant.attrs, variant.span())?;

    let ident: &proc_macro2::Ident = &variant.ident;

    let name = make_name(attributes.name, Some(ident), rename_case);

    let pattern = match variant.fields {
        syn::Fields::Unit => quote::quote!(Self::#ident),
        syn::Fields::Unnamed(_) => quote::quote! {Self::#ident (..)},
        syn::Fields::Named(_) => quote::quote! {Self::#ident {..}},
    };

    let tokens = quote::quote_spanned! {variant.ident.span() =>
        #pattern => #name
    };

    Ok(tokens)
}

fn variant_probe(
    variant: &syn::Variant,
    rename_case: Option<RenameCase>,
) -> syn::Result<proc_macro2::TokenStream> {
    let attributes: VariantAttributes =
        proc_easy::EasyAttributes::parse(&variant.attrs, variant.span())?;

    let ident = &variant.ident;

    let construct = match variant.fields {
        syn::Fields::Unit => quote::quote!(Self::#ident),
        syn::Fields::Unnamed(ref fields) => {
            let defaults = fields.unnamed.iter().map(|field| {
                let ty = &field.ty;
                quote::quote!(<#ty as ::core::default::Default>::default())
            });
            quote::quote! {Self::#ident ( #(#defaults,)* )}
        }
        syn::Fields::Named(ref fields) => {
            let defaults = fields.named.iter().map(|field| {
                let ident = field.ident.as_ref().unwrap();
                let ty = &field.ty;
                quote::quote!(#ident: <#ty as ::core::default::Default>::default())
            });
            quote::quote! {Self::#ident { #(#defaults,)* }}
        }
    };

    let name = make_name(attributes.name, Some(ident), rename_case);

    let pattern = match variant.fields {
        syn::Fields::Unit => quote::quote!(Self::#ident),
        syn::Fields::Unnamed(_) => quote::quote! {Self::#ident (..)},
        syn::Fields::Named(_) => quote::quote! {Self::#ident {..}},
    };

    let tokens = quote::quote_spanned! {variant.ident.span() =>
        let mut checked = match self { #pattern => true, _ => false };
        if _ui.selectable_label(checked, #name).clicked() {
            if !checked {
                *self = #construct;
            }
        }
    };

    Ok(tokens)
}

fn variant_inline_probe(
    variant: &syn::Variant,
    rename_case: Option<RenameCase>,
) -> syn::Result<proc_macro2::TokenStream> {
    let attributes: VariantAttributes =
        proc_easy::EasyAttributes::parse(&variant.attrs, variant.span())?;

    let ident = &variant.ident;

    if attributes.transparent.is_some() {
        let pattern = match variant.fields {
            syn::Fields::Unit => quote::quote!(Self::#ident),
            syn::Fields::Unnamed(ref fields) => {
                let fields = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| quote::format_ident!("___{}", idx));
                quote::quote! {Self::#ident ( #(#fields,)* )}
            }
            syn::Fields::Named(ref fields) => {
                let fields = fields.named.iter().enumerate().map(|(idx, field)| {
                    let binding = quote::format_ident!("___{}", idx);
                    let ident = field.ident.as_ref().unwrap();
                    quote::quote!(#ident: #binding)
                });
                quote::quote! {Self::#ident { #(#fields,)* }}
            }
        };

        let fields_probe: Vec<_> = variant
            .fields
            .iter()
            .enumerate()
            .filter_map(|(idx, field)| field_probe(idx, field, rename_case).transpose())
            .collect::<syn::Result<_>>()?;

        if fields_probe.len() != 1 {
            return Err(syn::Error::new_spanned(
                attributes.transparent.unwrap(),
                "Transparent variant must have exactly one non-skipped field",
            ));
        }

        let field_probe = &fields_probe[0];

        let tokens = quote::quote_spanned! {variant.ident.span() =>
            #pattern => {
                let mut _f = move |_label, field| {
                    ::egui_probe::EguiProbe::probe(field, _ui, _style)
                };

                #field_probe;
            },
        };

        Ok(tokens)
    } else {
        let pattern = match variant.fields {
            syn::Fields::Unit => quote::quote!(Self::#ident),
            syn::Fields::Unnamed(_) => quote::quote! {Self::#ident (..)},
            syn::Fields::Named(_) => quote::quote! {Self::#ident {..}},
        };

        Ok(quote::quote! { #pattern => {} })
    }
}

fn variant_has_inner(variant: &syn::Variant) -> syn::Result<proc_macro2::TokenStream> {
    let attributes: VariantAttributes =
        proc_easy::EasyAttributes::parse(&variant.attrs, variant.span())?;

    let ident = &variant.ident;

    let pattern = match variant.fields {
        syn::Fields::Unit => quote::quote!(Self::#ident),
        syn::Fields::Unnamed(_) => quote::quote! {Self::#ident (..)},
        syn::Fields::Named(_) => quote::quote! {Self::#ident {..}},
    };

    let has_inner = attributes.transparent.is_none()
        && match variant.fields {
            syn::Fields::Unit => false,
            syn::Fields::Unnamed(ref fields) => !fields.unnamed.is_empty(),
            syn::Fields::Named(ref fields) => !fields.named.is_empty(),
        };

    let tokens = quote::quote_spanned! {variant.ident.span() =>
        #pattern => #has_inner,
    };

    Ok(tokens)
}

fn variant_iterate_inner(
    variant: &syn::Variant,
    rename_case: Option<RenameCase>,
) -> syn::Result<proc_macro2::TokenStream> {
    let attributes: VariantAttributes =
        proc_easy::EasyAttributes::parse(&variant.attrs, variant.span())?;

    let ident = &variant.ident;

    if attributes.transparent.is_some() {
        let pattern = match variant.fields {
            syn::Fields::Unit => quote::quote!(Self::#ident),
            syn::Fields::Unnamed(_) => quote::quote! {Self::#ident (..)},
            syn::Fields::Named(_) => quote::quote! {Self::#ident {..}},
        };

        Ok(quote::quote! { #pattern => {} })
    } else {
        let pattern = match variant.fields {
            syn::Fields::Unit => quote::quote!(Self::#ident),
            syn::Fields::Unnamed(ref fields) => {
                let fields = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(idx, _)| quote::format_ident!("___{}", idx));
                quote::quote! {Self::#ident ( #(#fields,)* )}
            }
            syn::Fields::Named(ref fields) => {
                let fields = fields.named.iter().enumerate().map(|(idx, field)| {
                    let binding = quote::format_ident!("___{}", idx);
                    let ident = field.ident.as_ref().unwrap();
                    quote::quote!(#ident: #binding)
                });
                quote::quote! {Self::#ident { #(#fields,)* }}
            }
        };

        let fields_probe: Vec<_> = variant
            .fields
            .iter()
            .enumerate()
            .filter_map(|(idx, field)| field_probe(idx, field, rename_case).transpose())
            .collect::<syn::Result<_>>()?;

        let tokens = quote::quote_spanned! {variant.ident.span() =>
            #pattern => {
                #(#fields_probe)*
            },
        };

        Ok(tokens)
    }
}

pub fn derive(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let ident = &input.ident;
    let generics = &input.generics;

    let attributes: TypeAttributes = proc_easy::EasyAttributes::parse(&input.attrs, ident.span())?;
    let rename_case = attributes.rename_all.map(|rename_all| rename_all.case);

    let (impl_generics, ty_generics, mut where_clause) = generics.split_for_impl();

    let mut extended_where_clause;
    if let Some(derive_where_clause) = attributes.where_clause {
        extended_where_clause = where_clause.cloned().unwrap_or_else(|| syn::WhereClause {
            where_token: derive_where_clause.where_token,
            predicates: syn::punctuated::Punctuated::new(),
        });
        for predicate in derive_where_clause.predicates.iter() {
            extended_where_clause.predicates.push(predicate.clone());
        }
        where_clause = Some(&extended_where_clause);
    }

    match input.data {
        syn::Data::Struct(data) => {
            if attributes.tags.is_some() {
                return Err(syn::Error::new_spanned(
                    attributes.tags.unwrap().tags,
                    "Tags may be specified only for enums",
                ));
            }

            let pattern = match data.fields {
                syn::Fields::Unit => quote::quote!(Self),
                syn::Fields::Unnamed(ref fields) => {
                    let fields = fields
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(idx, _)| quote::format_ident!("___{}", idx));
                    quote::quote! {Self ( #(#fields,)* )}
                }
                syn::Fields::Named(ref fields) => {
                    let fields = fields.named.iter().enumerate().map(|(idx, field)| {
                        let binding = quote::format_ident!("___{}", idx);
                        let ident = field.ident.as_ref().unwrap();
                        quote::quote!(#ident: #binding)
                    });
                    quote::quote! {Self { #(#fields,)* }}
                }
            };

            let fields_probe: Vec<_> = data
                .fields
                .iter()
                .enumerate()
                .filter_map(|(idx, field)| field_probe(idx, field, rename_case).transpose())
                .collect::<syn::Result<_>>()?;

            if attributes.transparent.is_some() {
                if fields_probe.len() != 1 {
                    return Err(syn::Error::new_spanned(
                        attributes.transparent.unwrap(),
                        "Transparent struct must have exactly one non-skipped field",
                    ));
                }

                let field_probe = &fields_probe[0];

                let tokens = quote::quote! {
                    impl #impl_generics ::egui_probe::EguiProbe for #ident #ty_generics
                    #where_clause
                    {
                        fn probe(&mut self, ui: &mut ::egui_probe::egui::Ui, style: &::egui_probe::Style) -> ::egui_probe::egui::Response {
                            use ::egui_probe::private::{probe_with, probe_as, probe_range, probe_multiline, probe_toggle_switch};

                            let #pattern = self;

                            let mut _f = move |_label, field| {
                                ::egui_probe::EguiProbe::probe(field, ui, style)
                            };

                            #field_probe
                        }

                        fn has_inner(&self) -> bool {
                            false
                        }

                        fn iterate_inner(&mut self, _f: &mut dyn FnMut(&str, &mut dyn ::egui_probe::EguiProbe)) {}
                    }
                };
                Ok(tokens)
            } else {
                let tokens = quote::quote! {
                    impl #impl_generics ::egui_probe::EguiProbe for #ident #ty_generics
                    #where_clause
                    {
                        fn probe(&mut self, ui: &mut ::egui_probe::egui::Ui, _style: &::egui_probe::Style) -> ::egui_probe::egui::Response {
                            ui.weak(::egui_probe::private::stringify!(#ident))
                        }

                        fn has_inner(&self) -> bool {
                            true
                        }

                        fn iterate_inner(&mut self, _f: &mut dyn FnMut(&str, &mut dyn ::egui_probe::EguiProbe)) {
                            use ::egui_probe::private::{probe_with, probe_as, probe_range, probe_multiline, probe_toggle_switch};

                            let #pattern = self;

                            #(
                                #fields_probe;
                            )*
                        }
                    }
                };
                Ok(tokens)
            }
        }

        syn::Data::Enum(data) => {
            let variants_selected = data
                .variants
                .iter()
                .map(|variant| variant_selected(variant, rename_case))
                .collect::<syn::Result<Vec<_>>>()?;

            let variants_probe = data
                .variants
                .iter()
                .map(|variant| variant_probe(variant, rename_case))
                .collect::<syn::Result<Vec<_>>>()?;

            let variants_inline_probe = data
                .variants
                .iter()
                .map(|variant| variant_inline_probe(variant, rename_case))
                .collect::<syn::Result<Vec<_>>>()?;

            let variants_has_inner = data
                .variants
                .iter()
                .map(|variant| variant_has_inner(variant))
                .collect::<syn::Result<Vec<_>>>()?;

            let variants_iterate_inner = data
                .variants
                .iter()
                .map(|variant| variant_iterate_inner(variant, rename_case))
                .collect::<syn::Result<Vec<_>>>()?;

            let variants_style = match attributes.tags {
                None => quote::quote!(_style.variants),
                Some(EnumTags {
                    kind: TagsKind::Inlined(_),
                    ..
                }) => quote::quote!(::egui_probe::VariantsStyle::Inlined),
                Some(EnumTags {
                    kind: TagsKind::ComboBox(_),
                    ..
                }) => quote::quote!(::egui_probe::VariantsStyle::ComboBox),
            };

            let tokens = quote::quote! {
                impl #impl_generics ::egui_probe::EguiProbe for #ident #ty_generics
                    #where_clause
                    {
                        fn probe(&mut self, ui: &mut ::egui_probe::egui::Ui, _style: &::egui_probe::Style) -> ::egui_probe::egui::Response {
                            use ::egui_probe::private::{probe_with, probe_as, probe_range, probe_multiline, probe_toggle_switch};

                            ui.horizontal(|_ui| {
                                match #variants_style {
                                    ::egui_probe::VariantsStyle::Inlined => {
                                        #(
                                            #variants_probe
                                        )*
                                    }
                                    ::egui_probe::VariantsStyle::ComboBox => {
                                        let selected_variant = match self { #(#variants_selected,)* };
                                        let cbox = ::egui_probe::egui::ComboBox::from_id_source(_ui.id()).selected_text(selected_variant);

                                        cbox.show_ui(_ui, |_ui| {
                                            #(
                                                #variants_probe
                                            )*
                                        });
                                    }
                                }

                                match self {#(
                                    #variants_inline_probe
                                )*}
                            }).response
                        }

                        fn has_inner(&self) -> bool {
                            match self {#(
                                #variants_has_inner
                            )*}
                        }

                        fn iterate_inner(&mut self, _f: &mut dyn FnMut(&str, &mut dyn ::egui_probe::EguiProbe)) {
                            use ::egui_probe::private::{probe_with, probe_as, probe_range, probe_multiline, probe_toggle_switch};

                            match self {#(
                                #variants_iterate_inner
                            )*}
                        }
                    }
            };

            Ok(tokens)
        }
        syn::Data::Union(_) => Err(syn::Error::new_spanned(
            input,
            "EguiProbe can only be derived for structs and enums",
        )),
    }
}