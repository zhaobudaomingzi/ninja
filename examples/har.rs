use openai::{
    arkose::{ArkoseContext, ArkoseToken, Type},
    context::{self, args::Args},
    with_context,
};

#[tokio::main]
async fn main() {
    env_logger::init();
    context::init(Args::builder().build());
    for _ in 0..100 {
        match ArkoseToken::new_from_har(
            &mut ArkoseContext::builder()
                .client(with_context!(arkose_client))
                .typed(Type::GPT4)
                .build(),
        )
        .await
        {
            Ok(token) => {
                println!("{}", token.json());
            }
            Err(err) => {
                println!("{}", err);
            }
        };
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}
