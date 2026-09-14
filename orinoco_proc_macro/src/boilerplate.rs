use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::io::Write;
use syn::{
    FnArg, GenericParam, Generics, Ident, ItemTrait, ReturnType, TraitItemFn, Type, TypeParam,
    TypePath,
    parse::{ParseBuffer, Parser},
    parse_macro_input,
    punctuated::Punctuated,
    token::Comma,
};

use crate::debugfile;

#[derive(Debug, Clone)]
pub struct OrcDerivedIDs {
    pub base: Ident,
    pub req: Ident,
    pub rsp: Ident,
    pub tcp_cli: Ident,
    pub rkyv_arc_rsp: Ident,
    pub rkyv_arc_req: Ident,
    pub endpoint: Ident,
}
impl OrcDerivedIDs {
    pub fn new(base: String) -> anyhow::Result<Self> {
        Ok(OrcDerivedIDs {
            base: syn::parse_str(&format!("{base}"))?,
            req: syn::parse_str(&format!("OrcReq{base}"))?,
            rsp: syn::parse_str(&format!("OrcRsp{base}"))?,
            tcp_cli: syn::parse_str(&format!("OrcTcp{base}Client"))?,
            rkyv_arc_rsp: syn::parse_str(&format!("ArchivedOrcRsp{base}"))?,
            rkyv_arc_req: syn::parse_str(&format!("ArchivedOrcReq{base}"))?,
            endpoint: syn::parse_str(&format!("OrcEndpoint{base}"))?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FnDescriptor {
    name: Ident,
    params: Vec<(Ident, TokenStream)>,
    returns: Option<TokenStream>,
}

#[derive(Debug, Clone)]
pub struct InterfaceDescriptor {
    name: Ident,
    ids: OrcDerivedIDs,
    proc: Vec<FnDescriptor>,
    generic_impl: TokenStream,
    generic_ty: TokenStream,
    generic_where: TokenStream,
    generic_idents: Vec<Ident>,
    generics_present: bool,
}

pub fn parse_rpc_description(tree: &ItemTrait) -> InterfaceDescriptor {
    let ids = OrcDerivedIDs::new(tree.ident.to_string()).unwrap();

    let mut descriptors: Vec<FnDescriptor> = Vec::new();

    // let x = tree.generics

    for item in &tree.items {
        // let fun: TraitItemFn = TraitItemFn::parse(item.to_token_stream()).unwrap();
        // item.
        let fun: TraitItemFn = syn::parse(item.to_token_stream().into()).unwrap();
        let rtt: Option<Type> = match &fun.sig.output {
            ReturnType::Default => None,
            ReturnType::Type(_, ty) => Some(ty.as_ref().clone()),
        };

        let mut params = Vec::new();
        for item in fun.sig.inputs {
            match item {
                FnArg::Receiver(_receiver) => (),
                FnArg::Typed(pat_type) => {
                    let name = syn::parse2(pat_type.pat.to_token_stream()).unwrap();
                    let ty = pat_type.ty.to_token_stream().into();
                    params.push((name, ty));
                }
            };
            // params.insert(item., v)
        }

        descriptors.push(FnDescriptor {
            name: fun.sig.ident,
            params: params,
            returns: rtt.map(|rtt| rtt.to_token_stream().into()).unwrap_or(None),
        });

        // let ty: syn::TypePath = syn::parse2(fun.sig.output.to_token_stream()).unwrap();
        // writeln!(debug, "{:#?}", ty.to_token_stream()).unwrap();
        // writeln!(debug, "{:?}", item.into_token_stream()).unwrap();
    }
    let (generic_impl, generic_ty, generic_where) = tree.generics.split_for_impl();
    let mut dbgf = debugfile("__constparams.txt").unwrap();
    writeln!(dbgf, "{}", generic_ty.to_token_stream().to_string()).unwrap();
    dbgf.flush().unwrap();
    // let punct_ids: Punctuated<Ident, Comma> = Punctuated::<Ident, Comma>::parse_separated_nonempty
    //     .parse2(generic_ty.to_token_stream())
    //     .unwrap();
    let mut generic_ids = Vec::new();
    for param in tree.generics.params.iter() {
        let GenericParam::Type(t) = param else {
            continue;
        };
        generic_ids.push(t.ident.clone());
    }

    for ty in &generic_ids {
        writeln!(dbgf, "{}", ty.to_token_stream().to_string()).unwrap();
    }
    let generics_present = !tree.generics.to_token_stream().is_empty();
    InterfaceDescriptor {
        name: tree.ident.clone(),
        ids,
        proc: descriptors,
        generics_present,
        generic_impl: generic_impl.to_token_stream(),
        generic_ty: generic_ty.to_token_stream(),
        generic_where: generic_where.to_token_stream(),
        generic_idents: generic_ids,
    }
}

pub fn req_rep_enums(ids: &OrcDerivedIDs, descriptor: &InterfaceDescriptor) -> TokenStream {
    let (base, req, rsp, tcp_cli, rkyv_arc_rsp) = (
        &ids.base,
        &ids.req,
        &ids.rsp,
        &ids.tcp_cli,
        &ids.rkyv_arc_rsp,
    );
    let mut req_variants = TokenStream::new();
    let mut rsp_variants = TokenStream::new();

    for desc in &descriptor.proc {
        let name = &desc.name;
        let mut variant = TokenStream::new();
        // variant.push_str(&desc.name);
        // variant.push_str("{");
        // variant = quote! { #name { };
        for (name, ty) in &desc.params {
            // variant.push_str(&format!("{name}: {ty},"));
            variant = quote! { #variant #name: #ty, };
        }
        // variant.push_str("},");
        variant = quote! {#name {#variant}};
        // req_variants.push_str(&variant);
        // let returns =
        req_variants = quote! {#req_variants #variant,};

        rsp_variants = match &desc.returns {
            Some(ret) => quote! { #rsp_variants #name(#ret), },
            None => quote! { #rsp_variants #name(()), },
        };

        // rsp_variants.push_str(&format!(
        //     "{}({}),",
        //     desc.name,
        //     desc.returns.replace("Self", "()")
        // ));
    }

    // let total =
    //     format!("enum {req_name} {{ {req_variants} }}\nenum {rsp_name} {{ {rsp_variants} }}\n");

    // let mut debug = debugfile("__emit_enum.txt").unwrap();
    // writeln!(debug, "{total}").unwrap();
    // debug.flush().unwrap();

    // todo!()
    // parse_str(&total).unwrap()
    // let generics = &descriptor.generics;
    let mut phantom_tuple = TokenStream::new();
    for id in &descriptor.generic_idents {
        if phantom_tuple.is_empty() {
            phantom_tuple = quote! {#id}
        } else {
            phantom_tuple = quote! {#phantom_tuple, #id}
        }
    }

    if descriptor.generics_present {
        let (gen_impl, gen_ty, gen_where) = (
            &descriptor.generic_impl,
            &descriptor.generic_ty,
            &descriptor.generic_where,
        );
        quote! {
            #[derive(serde::Serialize, serde::Deserialize,Clone)]
            enum #req #gen_impl { #req_variants __orinoco_phantom { __orinoco_phantom: (#phantom_tuple)}}
            #[derive(serde::Serialize, serde::Deserialize, Clone)]
            enum #rsp #gen_impl {#rsp_variants __orinoco_phantom (#phantom_tuple)}
        }
    } else {
        quote! {
            #[derive(serde::Serialize, serde::Deserialize, Clone)]
            #[allow(non_camel_case_types)]
            enum #req { #req_variants }
            #[derive(serde::Serialize, serde::Deserialize, Clone)]
            #[allow(non_camel_case_types)]
            enum #rsp {#rsp_variants }
        }
    }
}

pub fn tcp_client_boilerplate(
    ids: &OrcDerivedIDs,
    descriptor: &InterfaceDescriptor,
) -> TokenStream {
    let (base, req, rsp, tcp_cli, rkyv_arc_rsp) = (
        &ids.base,
        &ids.req,
        &ids.rsp,
        &ids.tcp_cli,
        &ids.rkyv_arc_rsp,
    );
    let mut rpc_impl = TokenStream::new();

    let generics_ty = &descriptor.generic_ty;
    let generics_impl = &descriptor.generic_impl;

    let mut where_clause = TokenStream::new();
    for id in &descriptor.generic_idents {
        if where_clause.is_empty() {
            where_clause = quote! {where #id: serde::Serialize + serde::de::DeserializeOwned}
        } else {
            where_clause =
                quote! {#where_clause, #id: serde::Serialize + serde::de::DeserializeOwned}
        }
    }

    for desc in &descriptor.proc {
        let mut fn_body = TokenStream::new();
        let mut fn_sig = TokenStream::new();
        let name = &desc.name;
        let ret = &desc.returns;
        for (n, t) in &desc.params {
            if fn_sig.is_empty() {
                fn_sig = quote! { #fn_sig #n : #t};
            } else {
                fn_sig = quote! { #fn_sig , #n : #t};
            }
            fn_body = quote! { #fn_body #n,};
        }
        fn_body = quote! { let #rsp :: #generics_ty :: #name(rsp) = self.request(#req :: #generics_ty :: #name { #fn_body }).unwrap() else {panic!();}; rsp };
        if let Some(ret) = ret {
            fn_sig = quote! { fn #name (&self, #fn_sig) -> #ret {#fn_body} };
        } else {
            fn_sig = quote! { fn #name (&self, #fn_sig) ->  () {#fn_body} };
        }
        rpc_impl = quote! {#rpc_impl
        #fn_sig};
    }

    // let generics = &descriptor.generics;

    quote! {
        #[derive(Clone)]
        pub struct #tcp_cli #generics_impl {
            client: std::sync::Arc<orinoco::OrinocoClient>,
            __orinoco_phantom: std::marker::PhantomData #generics_ty,
        }
        impl #generics_impl #tcp_cli #generics_ty #where_clause  {
            pub fn connect(
                host: impl ToString,
                port: u16,
                endpoint: impl ToString,
            ) -> anyhow::Result<Self> #where_clause {
                Ok(#tcp_cli {
                    client: std::sync::Arc::new(orinoco::OrinocoClient::connect(host, port, endpoint)?),
                    __orinoco_phantom: std::marker::PhantomData,
                })
            }
            pub fn request (&self, msg: #req #generics_ty) -> anyhow::Result<#rsp #generics_ty>  {
                // let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&msg)?.to_vec();
                let bytes = postcard::to_allocvec(&msg)?;
                let rsp_bytes = self.client.request(bytes)?;
                // let rsp = rkyv::access::<#rkyv_arc_rsp #generics_ty, rkyv::rancor::Error>(&rsp_bytes)?;
                // let rsp = rkyv::deserialize::<#rsp #generics_ty, rkyv::rancor::Error>(rsp)?;
                let rsp = postcard::from_bytes(&rsp_bytes)?;
                Ok(rsp)
            }
        }
        impl #generics_impl #base #generics_ty for #tcp_cli #generics_ty #where_clause {
            #rpc_impl
        }
    }
}

/*
pub struct OrcEndpointAnyInterface {}
#[allow(clippy::redundant_closure_call)]
impl OrcEndpointAnyInterface {
    pub fn new(inner: Box<dyn Fn() -> Box<dyn AnyInterface>>) -> OrinocoEndpoint {
        let launch_fn = move |mut socket: TcpStream| {
            let inner: Box<dyn AnyInterface> = inner();
            while let Ok(msg) = OrinocoMsg::recv_on(&mut socket) {
                let inner_result: anyhow::Result<()> = (|| {
                    let msg = rkyv::access::<ArchivedOrcReqAnyInterface, rkyv::rancor::Error>(
                        &msg.bytes,
                    )?;
                    let msg = rkyv::deserialize::<OrcReqAnyInterface, rkyv::rancor::Error>(msg)?;
                    let rsp_bytes = match msg {
                        OrcReqAnyInterface::open { path } => rkyv::to_bytes::<rkyv::rancor::Error>(
                            &OrcRspAnyInterface::open(inner.open(path)),
                        )?
                        .to_vec(),
                        OrcReqAnyInterface::hello { input } => {
                            rkyv::to_bytes::<rkyv::rancor::Error>(&OrcRspAnyInterface::hello(
                                inner.hello(input),
                            ))?
                            .to_vec()
                        }
                        OrcReqAnyInterface::goodbye { other, input } => todo!(),
                    };
                    OrinocoMsg {
                        len: rsp_bytes.len() as u32,
                        uuid: Uuid::new_v4().as_u128(),
                        bytes: rsp_bytes,
                    }
                    .send_on(&mut socket)?;
                    Ok(())
                })();
                let Ok(_) = inner_result else { panic!() };
            }
        };
        OrinocoEndpoint(Box::new(launch_fn))
    }
}
*/

/*

 OrcReqAnyInterface::open { path } => rkyv::to_bytes::<rkyv::rancor::Error>(
                            &OrcRspAnyInterface::open(inner.open(path)),
                        )?
                        .to_vec(),
                        OrcReqAnyInterface::hello { input } => {
                            rkyv::to_bytes::<rkyv::rancor::Error>(&OrcRspAnyInterface::hello(
                                inner.hello(input),
                            ))?
                            .to_vec()
                        }
                        OrcReqAnyInterface::goodbye { other, input } => todo!(),
*/
pub fn tcp_endpoint_boilerplate(
    ids: &OrcDerivedIDs,
    descriptor: &InterfaceDescriptor,
) -> TokenStream {
    let endpoint_name = &ids.endpoint;
    let (base, req, rsp, tcp_cli, rkyv_arc_rsp, rkyv_arc_req) = (
        &ids.base,
        &ids.req,
        &ids.rsp,
        &ids.tcp_cli,
        &ids.rkyv_arc_rsp,
        &ids.rkyv_arc_req,
    );

    let mut match_block = TokenStream::new();

    let mut rpc_impl = TokenStream::new();

    let generics_ty = &descriptor.generic_ty;
    let generics_impl = &descriptor.generic_impl;

    let mut where_clause = TokenStream::new();
    for id in &descriptor.generic_idents {
        if where_clause.is_empty() {
            where_clause = quote! {where #id: serde::Serialize + serde::de::DeserializeOwned}
        } else {
            where_clause =
                quote! {#where_clause, #id: serde::Serialize + serde::de::DeserializeOwned}
        }
    }

    for desc in &descriptor.proc {
        // let mut fn_body = TokenStream::new();
        let mut inner_sig = TokenStream::new();

        let mut match_arm = TokenStream::new();

        let name = &desc.name;
        // let ret = &desc.returns;
        for (n, _) in &desc.params {
            if inner_sig.is_empty() {
                inner_sig = quote! { #inner_sig #n};
            } else {
                inner_sig = quote! { #inner_sig , #n};
            }
            // fn_body = quote! { #fn_body #n,};
        }
        // fn_body = quote! { let #rsp :: #name(rsp) = self.request(#req :: #name { #fn_body }).unwrap() else {panic!();}; rsp };
        // if let Some(ret) = ret {
        //     fn_sig = quote! { fn #name (&self, #fn_sig) -> #ret {#fn_body}};
        // } else {
        //     fn_sig = quote! { fn #name (&self, #fn_sig) -> () {#fn_body}};
        // }
        match_arm = quote! {#match_arm
            #req :: #generics_ty :: #name { #inner_sig } => {
                // rkyv::to_bytes::<rkyv::rancor::Error>(&#rsp :: #name(
                //     inner. #name( #inner_sig ),
                // ))?
                // .to_vec()
                postcard::to_allocvec(&#rsp :: #generics_ty :: #name(
                    inner. #name( #inner_sig ),
                ))?
            }
        ,};

        match_block = quote! {#match_block
        #match_arm};
    }

    quote! {
        pub struct #endpoint_name {}
        impl  #endpoint_name {
            pub fn new #generics_impl (inner: Box<dyn Fn() -> std::sync::Arc<Box<dyn #base #generics_ty>>>) -> orinoco::OrinocoEndpoint #where_clause {
                let launch_fn = move |mut socket: std::net::TcpStream| {
                    let inner: std::sync::Arc<Box<dyn #base #generics_ty>> = inner();
                    while let Ok(msg) = orinoco::OrinocoMsg::recv_on(&mut socket) {
                        let uuid = msg.uuid;
                        // println!("SRV Got: {}", msg.uuid);
                        let inner_result: anyhow::Result<()> = (|| {
                            // let msg = rkyv::access::<#rkyv_arc_req, rkyv::rancor::Error>(
                            //     &msg.bytes,
                            // )?;
                            // let msg = rkyv::deserialize::<#req, rkyv::rancor::Error>(msg)?;
                            let msg = postcard::from_bytes(&msg.bytes)?;
                            let rsp_bytes = match msg {
                                #match_block
                                _ => panic!()
                            };
                            let out_msg = orinoco::OrinocoMsg {
                                len: rsp_bytes.len() as u32,
                                uuid,
                                bytes: rsp_bytes,
                            };
                            // println!("SRV Dispatch: {}",out_msg.uuid);
                            out_msg.send_on(&mut socket)?;
                            Ok(())
                        })();
                        let Ok(_) = inner_result else { panic!() };
                    }
                };
                orinoco::OrinocoEndpoint(std::sync::Arc::new(Box::new(launch_fn)))
            }
        }
    }
}
