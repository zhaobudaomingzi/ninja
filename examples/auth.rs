use std::{str::FromStr, time};

use openai::{
    arkose::funcaptcha::solver::{ArkoseSolver, Solver},
    auth::{
        model::{AuthAccount, AuthStrategy},
        provide::AuthProvider,
        AuthClientBuilder,
    },
};
use reqwest::impersonate::Impersonate;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let client_key = std::env::var("KEY").expect("Need solver client key");
    let solver = Solver::from_str(&std::env::var("SOLVER").expect("Need solver"))
        .expect("Not support solver");

    let ctx = openai::context::args::Args::builder()
        .arkose_solver(ArkoseSolver::new(solver, client_key, None, 1))
        .build();
    openai::context::init(ctx);

    let email = std::env::var("EMAIL")?;
    let password = std::env::var("PASSWORD")?;
    let auth = AuthClientBuilder::builder()
        .impersonate(Impersonate::Chrome100)
        .timeout(time::Duration::from_secs(30))
        .connect_timeout(time::Duration::from_secs(10))
        .build();
    let token = auth
        .do_access_token(
            &AuthAccount::builder()
                .username(email)
                .password(password)
                .option(AuthStrategy::Web)
                .build(),
        )
        .await?;
    let auth_token = openai::token::model::Token::try_from(token)?;
    println!("AuthenticationToken: {:#?}", auth_token);
    println!("AccessToken: {}", auth_token.access_token());

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    if let Some(refresh_token) = auth_token.refresh_token() {
        println!("RefreshToken: {}", refresh_token);
        let refresh_token = auth.do_refresh_token(refresh_token).await?;
        if let Some(refresh_token) = refresh_token.refresh_token {
            println!("RefreshToken: {}", refresh_token);
            auth.do_revoke_token(&refresh_token).await?;
        }
    }

    Ok(())
}
