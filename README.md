# Orinoco

Welcome to Orinico! This crate is *extremely* early in development and will face breaking changes as it takes shape. 

This crate exists to solve the problem of "I have a `impl MyTrait` that exists somewhere else, but I want to be able to interact with it as if it's here."

Specificially, at minimum it derives 2 enums (function inputs and return values) which have a variant for each function in the trait body.

For the `#[orinoco_thread_rpc(MyTraitMsgName)]` macro, it also derives a Server (which itself takes a boxed closure that will create the `impl MyTrait` to be accessed) which returns a clonable Client upon creation. 

The magic here is that this `ThreadMyTraitClient` itself implements `MyTrait`. When you call one of the `MyTrait` methods on the client, it will send a message with the parameters to the server using an mpsc channel and listen for the return value on a single-use callback channel. 

Provided that each method of `MyTrait` takes `&self` as the first parameter, this *should* just work. 

If that explanation made no sense, check out the singular test in `lib.rs`. 

There's a prototype of the same functionality implemented for TCP connections, but I'm not quite happy with the semantics yet. 

*Hopefully* the woeful state of the code here will serve as definitive proof that this project is fully human-made. 

# Why not just use a `Mutex<T>` for the Multithreaded case?

Great question! The initial inspiration for this crate was that `gdal` gets very angry if you access the same `GDALDataset` from different threads, even if a `Mutex<T>` is used. I figure there may be other similar situations with bindings to C libraries that make similar assumptions. 