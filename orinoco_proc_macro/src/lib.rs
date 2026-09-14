use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

use proc_macro::TokenTree;
use quote::ToTokens;
use syn::parse::Parse;
use syn::*;

// use proc_macro2::TokenStream;
use syn::parse_macro_input;

mod boilerplate;
mod thread_boilerplate;

fn dump(path: impl ToString, txt: impl std::fmt::Debug) -> anyhow::Result<()> {
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path.to_string())?;
    writeln!(f, "{txt:#?}")?;
    f.flush()?;
    Ok(())
}

fn debugfile(path: impl ToString) -> anyhow::Result<std::fs::File> {
    Ok(OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path.to_string())?)
}

// pub trait OrinocoRpc {}

#[derive(Debug)]
struct ProcDescriptor {
    name: String,
    params: HashMap<String, String>,
    returns: String,
}

fn emit_enum(name: String, description: &Vec<ProcDescriptor>) -> proc_macro2::TokenStream {
    let (req_name, rsp_name) = (format!("{name}Req"), format!("{name}Rsp"));
    let mut req_variants = String::new();
    let mut rsp_variants = String::new();

    for desc in description {
        let mut variant = String::new();
        variant.push_str(&desc.name);
        variant.push_str("{");
        for (name, ty) in &desc.params {
            variant.push_str(&format!("{name}: {ty},"));
        }
        variant.push_str("},");
        req_variants.push_str(&variant);
        // let returns =
        rsp_variants.push_str(&format!(
            "{}({}),",
            desc.name,
            desc.returns.replace("Self", "()")
        ));
    }

    let total =
        format!("enum {req_name} {{ {req_variants} }}\nenum {rsp_name} {{ {rsp_variants} }}\n");

    let mut debug = debugfile("__emit_enum.txt").unwrap();
    writeln!(debug, "{total}").unwrap();
    debug.flush().unwrap();

    // todo!()
    parse_str(&total).unwrap()
}

#[proc_macro_attribute]
pub fn orinoco_thread_rpc(
    attr: proc_macro::TokenStream,
    mut item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_tree = item.clone();
    dump("__proc_macro_dump_1_rawtt.txt", attr.clone()).unwrap();
    let tree = parse_macro_input!(item_tree as ItemTrait);

    let mut debug = debugfile("__orc_debug.rs").unwrap();

    let cmd_name = attr.to_string();

    let ids =
        thread_boilerplate::OrcDerivedIDs::new(cmd_name.clone(), tree.ident.to_string()).unwrap();
    // let temp_tt = boilerplate::tcp_client_boilerplate(&ids);

    let desc = thread_boilerplate::parse_rpc_description(cmd_name.clone(), &tree);
    let enum_shit: proc_macro::TokenStream = thread_boilerplate::req_rep_enums(&ids, &desc).into();
    let tcp_shit: proc_macro::TokenStream =
        thread_boilerplate::tcp_client_boilerplate(&ids, &desc).into();
    let tcp_endpoint_shit: proc_macro::TokenStream =
        thread_boilerplate::tcp_endpoint_boilerplate(&ids, &desc).into();

    item.extend(enum_shit);
    item.extend(tcp_shit);
    item.extend(tcp_endpoint_shit);

    writeln!(debug, "{}", item.to_string()).unwrap();
    item

    // writeln!(debug, "{}", enum_shit.to_string()).unwrap();

    // // dump("__proc_macro_dump_x_rawtt.txt", temp_tt.to_string()).unwrap();
    // let mut debug = debugfile("__pm_debug.txt").unwrap();

    // writeln!(debug, "{:?}", tree.ident.to_string()).unwrap();
    // let mut descriptors: Vec<ProcDescriptor> = Vec::new();

    // for item in tree.items {
    //     // let fun: TraitItemFn = TraitItemFn::parse(item.to_token_stream()).unwrap();
    //     // item.
    //     let fun: TraitItemFn = syn::parse(item.to_token_stream().into()).unwrap();
    //     let rtt: Option<Type> = match &fun.sig.output {
    //         ReturnType::Default => None,
    //         ReturnType::Type(_, ty) => Some(ty.as_ref().clone()),
    //     };
    //     writeln!(debug, "{:#?}", rtt.to_token_stream().to_string()).unwrap();
    //     writeln!(debug, "{:#?}", fun.sig.to_token_stream().to_string()).unwrap();

    //     let mut params = HashMap::new();
    //     for item in fun.sig.inputs {
    //         match item {
    //             FnArg::Receiver(_receiver) => (),
    //             FnArg::Typed(pat_type) => {
    //                 let name = pat_type.pat.to_token_stream().to_string();
    //                 let ty = pat_type.ty.to_token_stream().to_string();
    //                 params.insert(name, ty);
    //             }
    //         };
    //         // params.insert(item., v)
    //     }

    //     descriptors.push(ProcDescriptor {
    //         name: fun.sig.ident.to_string(),
    //         params: params,
    //         returns: rtt.to_token_stream().to_string(),
    //     });

    //     // let ty: syn::TypePath = syn::parse2(fun.sig.output.to_token_stream()).unwrap();
    //     // writeln!(debug, "{:#?}", ty.to_token_stream()).unwrap();
    //     // writeln!(debug, "{:?}", item.into_token_stream()).unwrap();
    // }
    // writeln!(debug, "{:#?}", descriptors).unwrap();
    // debug.flush().unwrap();

    // let enum_tok: proc_macro::TokenStream = emit_enum(tree.ident.to_string(), &descriptors).into();

    // // dump("__proc_macro_dump_2_parsed.txt", tree.clone()).unwrap();
    // item.extend(enum_tok);
    // item
}

#[proc_macro_attribute]
pub fn orinoco_rpc(
    attr: proc_macro::TokenStream,
    mut item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_tree = item.clone();
    dump("__proc_macro_dump_1_rawtt.txt", item.clone()).unwrap();
    let tree = parse_macro_input!(item_tree as ItemTrait);

    let mut debug = debugfile("__orc_debug.rs").unwrap();

    let ids = boilerplate::OrcDerivedIDs::new(tree.ident.to_string()).unwrap();
    // let temp_tt = boilerplate::tcp_client_boilerplate(&ids);

    let desc = boilerplate::parse_rpc_description(&tree);
    let enum_shit: proc_macro::TokenStream = boilerplate::req_rep_enums(&ids, &desc).into();
    let tcp_shit: proc_macro::TokenStream = boilerplate::tcp_client_boilerplate(&ids, &desc).into();
    let tcp_endpoint_shit: proc_macro::TokenStream =
        boilerplate::tcp_endpoint_boilerplate(&ids, &desc).into();

    item.extend(enum_shit);
    item.extend(tcp_shit);
    item.extend(tcp_endpoint_shit);

    writeln!(debug, "{}", item.to_string()).unwrap();
    item

    // writeln!(debug, "{}", enum_shit.to_string()).unwrap();

    // // dump("__proc_macro_dump_x_rawtt.txt", temp_tt.to_string()).unwrap();
    // let mut debug = debugfile("__pm_debug.txt").unwrap();

    // writeln!(debug, "{:?}", tree.ident.to_string()).unwrap();
    // let mut descriptors: Vec<ProcDescriptor> = Vec::new();

    // for item in tree.items {
    //     // let fun: TraitItemFn = TraitItemFn::parse(item.to_token_stream()).unwrap();
    //     // item.
    //     let fun: TraitItemFn = syn::parse(item.to_token_stream().into()).unwrap();
    //     let rtt: Option<Type> = match &fun.sig.output {
    //         ReturnType::Default => None,
    //         ReturnType::Type(_, ty) => Some(ty.as_ref().clone()),
    //     };
    //     writeln!(debug, "{:#?}", rtt.to_token_stream().to_string()).unwrap();
    //     writeln!(debug, "{:#?}", fun.sig.to_token_stream().to_string()).unwrap();

    //     let mut params = HashMap::new();
    //     for item in fun.sig.inputs {
    //         match item {
    //             FnArg::Receiver(_receiver) => (),
    //             FnArg::Typed(pat_type) => {
    //                 let name = pat_type.pat.to_token_stream().to_string();
    //                 let ty = pat_type.ty.to_token_stream().to_string();
    //                 params.insert(name, ty);
    //             }
    //         };
    //         // params.insert(item., v)
    //     }

    //     descriptors.push(ProcDescriptor {
    //         name: fun.sig.ident.to_string(),
    //         params: params,
    //         returns: rtt.to_token_stream().to_string(),
    //     });

    //     // let ty: syn::TypePath = syn::parse2(fun.sig.output.to_token_stream()).unwrap();
    //     // writeln!(debug, "{:#?}", ty.to_token_stream()).unwrap();
    //     // writeln!(debug, "{:?}", item.into_token_stream()).unwrap();
    // }
    // writeln!(debug, "{:#?}", descriptors).unwrap();
    // debug.flush().unwrap();

    // let enum_tok: proc_macro::TokenStream = emit_enum(tree.ident.to_string(), &descriptors).into();

    // // dump("__proc_macro_dump_2_parsed.txt", tree.clone()).unwrap();
    // item.extend(enum_tok);
    // item
}

// #[proc_macro_attribute]
// pub fn orinoco_rpc(
//     attr: proc_macro::TokenStream,
//     mut item: proc_macro::TokenStream,
// ) -> proc_macro::TokenStream {
//     let item_tree = item.clone();
//     dump("__proc_macro_dump_1_rawtt.txt", item.clone()).unwrap();
//     let tree = parse_macro_input!(item_tree as ItemTrait);

//     let mut debug = debugfile("__orc_debug.rs").unwrap();

//     let ids = boilerplate::OrcDerivedIDs::new(tree.ident.to_string()).unwrap();
//     let temp_tt = boilerplate::tcp_client_boilerplate(&ids);

//     writeln!(debug, "{}", temp_tt.to_string()).unwrap();

//     let desc = boilerplate::parse_rpc_description(&tree);
//     let enum_shit = boilerplate::req_rep_enums(&ids, &desc);

//     writeln!(debug, "{}", enum_shit.to_string()).unwrap();

//     // dump("__proc_macro_dump_x_rawtt.txt", temp_tt.to_string()).unwrap();
//     let mut debug = debugfile("__pm_debug.txt").unwrap();

//     writeln!(debug, "{:?}", tree.ident.to_string()).unwrap();
//     let mut descriptors: Vec<ProcDescriptor> = Vec::new();

//     for item in tree.items {
//         // let fun: TraitItemFn = TraitItemFn::parse(item.to_token_stream()).unwrap();
//         // item.
//         let fun: TraitItemFn = syn::parse(item.to_token_stream().into()).unwrap();
//         let rtt: Option<Type> = match &fun.sig.output {
//             ReturnType::Default => None,
//             ReturnType::Type(_, ty) => Some(ty.as_ref().clone()),
//         };
//         writeln!(debug, "{:#?}", rtt.to_token_stream().to_string()).unwrap();
//         writeln!(debug, "{:#?}", fun.sig.to_token_stream().to_string()).unwrap();

//         let mut params = HashMap::new();
//         for item in fun.sig.inputs {
//             match item {
//                 FnArg::Receiver(_receiver) => (),
//                 FnArg::Typed(pat_type) => {
//                     let name = pat_type.pat.to_token_stream().to_string();
//                     let ty = pat_type.ty.to_token_stream().to_string();
//                     params.insert(name, ty);
//                 }
//             };
//             // params.insert(item., v)
//         }

//         descriptors.push(ProcDescriptor {
//             name: fun.sig.ident.to_string(),
//             params: params,
//             returns: rtt.to_token_stream().to_string(),
//         });

//         // let ty: syn::TypePath = syn::parse2(fun.sig.output.to_token_stream()).unwrap();
//         // writeln!(debug, "{:#?}", ty.to_token_stream()).unwrap();
//         // writeln!(debug, "{:?}", item.into_token_stream()).unwrap();
//     }
//     writeln!(debug, "{:#?}", descriptors).unwrap();
//     debug.flush().unwrap();

//     let enum_tok: proc_macro::TokenStream = emit_enum(tree.ident.to_string(), &descriptors).into();

//     // dump("__proc_macro_dump_2_parsed.txt", tree.clone()).unwrap();
//     item.extend(enum_tok);
//     item
// }
