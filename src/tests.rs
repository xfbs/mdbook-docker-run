use super::*;

#[tokio::test]
async fn test() {
    let config = Config {
        parallel: 4,
        ..Config::default()
    };
    let context = Context::new(&config).await.unwrap();
    let instance = Instance {
        image: Some("alpine".into()),
        script: vec!["echo hello".into()],
        ..Default::default()
    };
    let output = context.run(&instance).await.unwrap();
    assert_eq!(output, "hello\n");
}
