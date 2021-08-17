use proc_macro2::TokenStream;
use syn::*;

use crate::config::Config;
use crate::endian::*;
use crate::meta;
use crate::structs::*;

use super::*;

pub struct GenStruct {
    pub name: Ident,
    pub config: Config,

    extra_args: Option<TokenStream>,

    orig_generics: Generics,
    tl_pre: Option<TokenStream>,
    tl_post: Option<TokenStream>,
    parser_tree: StructParserTree,
}

impl Generator for GenStruct {
    fn from_ast(ast: &DeriveInput, endianness: ParserEndianness) -> Result<Self> {
        match &ast.data {
            syn::Data::Struct(datastruct) => GenStruct::from_datastruct(
                &ast.ident,
                datastruct,
                &ast.attrs,
                &ast.generics,
                endianness,
            ),
            _ => panic!("Wrong type for GenEnum::from_ast"),
        }
    }

    #[inline]
    fn name(&self) -> &Ident {
        &self.name
    }

    fn set_debug(&mut self, debug_derive: bool) {
        self.config.debug_derive |= debug_derive;
    }

    #[inline]
    fn extra_args(&self) -> Option<&TokenStream> {
        self.extra_args.as_ref()
    }

    #[inline]
    fn orig_generics(&self) -> &Generics {
        &self.orig_generics
    }

    #[inline]
    fn config(&self) -> &Config {
        &self.config
    }

    fn gen_fn_body(&self, endianness: ParserEndianness) -> Result<TokenStream> {
        let name = &self.name;
        let (tl_pre, tl_post) = (&self.tl_pre, &self.tl_post);
        let input = syn::Ident::new(self.config.input_name(), Span::call_site());
        let orig_input = syn::Ident::new(self.config.orig_input_name(), Span::call_site());

        // prepare tokens
        let (idents, parser_tokens): (Vec<_>, Vec<_>) = self
            .parser_tree
            .parsers
            .iter()
            .map(|sp| {
                let id = syn::Ident::new(&sp.name, Span::call_site());
                // set current endianness for functions that do not specify it
                let item = sp.item.with_endianness(endianness);
                (id, item)
            })
            .unzip();
        let (pre, post): (Vec<_>, Vec<_>) = self
            .parser_tree
            .parsers
            .iter()
            .map(|sp| (sp.pre_exec.as_ref(), sp.post_exec.as_ref()))
            .unzip();
        let idents2 = idents.clone();

        // Code generation
        let struct_def = match (self.parser_tree.empty, self.parser_tree.unnamed) {
            (true, _) => quote! { #name },
            (_, true) => quote! { #name ( #(#idents2),* ) },
            (_, false) => quote! { #name { #(#idents2),* } },
        };

        let fn_body = quote! {
            let #input = #orig_input;
            #tl_pre
            #(#pre let (#input, #idents) = #parser_tokens (#input) ?; #post)*
            let struct_def = #struct_def;
            #tl_post
            Ok((#input, struct_def))
        };
        Ok(fn_body)
    }
}

impl GenStruct {
    pub fn from_datastruct(
        name: &Ident,
        datastruct: &DataStruct,
        attrs: &[Attribute],
        generics: &Generics,
        endianness: ParserEndianness,
    ) -> Result<Self> {
        let name = name.clone();

        // parse top-level attributes and prepare tokens for each field parser
        let meta = meta::parse_nom_top_level_attribute(attrs)?;
        // eprintln!("top-level meta: {:?}", meta);
        let mut config = Config::from_meta_list(name.to_string(), &meta)?;

        // endianness must be set before parsing struct
        set_object_endianness(name.span(), endianness, &meta, &mut config)?;

        let extra_args = get_extra_args(&meta).map(Clone::clone);

        // test endianness validity (not 2 or more)
        validate_endianness(
            endianness,
            config.object_endianness,
            config.global_endianness,
        )?;

        // save global pre/post exec
        let (tl_pre, tl_post) = get_pre_post_exec(&meta, &config);

        let s = parse_struct(datastruct, &mut config)?;

        Ok(GenStruct {
            name,
            config,
            extra_args,
            orig_generics: generics.clone(),
            tl_pre,
            tl_post,
            parser_tree: s,
        })
    }
}
