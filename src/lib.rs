pub use orinoco_proc_macro::orinoco_thread_rpc;

// #[cfg(test)]
mod test {
    use orinoco_proc_macro::orinoco_thread_rpc;

    #[orinoco_thread_rpc(TestRpcMsg)]
    pub trait TestRpc {
        fn hello(&self, input: String) -> anyhow::Result<String>;
        fn goodbye(&self, other: bool, input: String) -> anyhow::Result<String>;
    }
    struct TestObj {}
    impl TestRpc for TestObj {
        fn hello(&self, input: String) -> anyhow::Result<String> {
            println!("hello {input}");
            Ok(input)
        }
        fn goodbye(&self, other: bool, input: String) -> anyhow::Result<String> {
            println!("goodbye {other} {input}");
            Ok(input)
        }
    }

    #[test]
    fn test_rpc() {
        let server = ThreadRpcTestRpcServer::launch(
            Some("Test".to_string()),
            Box::new(|| Box::new(TestObj {})),
        );

        server.hello("Hello!".to_string()).unwrap();
    }
}
