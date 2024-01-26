use openai::{
    context::{self},
    with_context,
};

#[test]
fn test_preauth_cookie_provider() {
    let time = openai::now_duration().unwrap();

    with_context!(
        push_preauth_cookie,
        &format!("id0:{}-xxx", time.as_secs()),
        Some(3600)
    );
    with_context!(
        push_preauth_cookie,
        &format!("id1:{}-yyy", time.as_secs()),
        Some(3600)
    );
}
