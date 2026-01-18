mod common;

use axum::http::StatusCode;
use common::{TestContext, TestFixture};
use serde_json::json;
use vostuff_api::api::models::{LoginResponse, OrgSelectionResponse, UserInfo};

#[tokio::test]
async fn test_login_single_org() {
    let ctx = TestContext::new().await;

    // Create org and user
    let org_id = ctx.create_organization("TestCo", "Test Company").await;
    let user_id = ctx
        .create_user("Alice", "alice@test.com", "password123")
        .await;
    ctx.add_user_to_org(user_id, org_id, vec!["USER".to_string()])
        .await;

    // Login without specifying org (should auto-select)
    let response = ctx
        .post(
            "/api/auth/login",
            &json!({
                "identity": "alice@test.com",
                "password": "password123"
            }),
            None,
        )
        .await;

    response.assert_status(StatusCode::OK);

    let login_resp: LoginResponse = response.json();
    assert!(!login_resp.token.is_empty());
    assert_eq!(login_resp.user.identity, "alice@test.com");
    assert_eq!(login_resp.user.organization.id, org_id);
}

#[tokio::test]
async fn test_login_multi_org_requires_selection() {
    let ctx = TestContext::new().await;

    // Create two orgs
    let org1_id = ctx.create_organization("Org1", "First Org").await;
    let org2_id = ctx.create_organization("Org2", "Second Org").await;

    // Create user in both orgs
    let user_id = ctx.create_user("Bob", "bob@test.com", "password123").await;
    ctx.add_user_to_org(user_id, org1_id, vec!["USER".to_string()])
        .await;
    ctx.add_user_to_org(user_id, org2_id, vec!["ADMIN".to_string()])
        .await;

    // Login without specifying org
    let response = ctx
        .post(
            "/api/auth/login",
            &json!({
                "identity": "bob@test.com",
                "password": "password123"
            }),
            None,
        )
        .await;

    response.assert_status(StatusCode::OK);

    let org_selection: OrgSelectionResponse = response.json();
    assert!(!org_selection.follow_on_token.is_empty());
    assert_eq!(org_selection.organizations.len(), 2);
    assert!(org_selection.organizations.iter().any(|o| o.id == org1_id));
    assert!(org_selection.organizations.iter().any(|o| o.id == org2_id));
}

#[tokio::test]
async fn test_login_with_org_selection() {
    let ctx = TestContext::new().await;

    // Create two orgs
    let org1_id = ctx.create_organization("Org1", "First Org").await;
    let org2_id = ctx.create_organization("Org2", "Second Org").await;

    // Create user in both orgs with different roles
    let user_id = ctx
        .create_user("Charlie", "charlie@test.com", "password123")
        .await;
    ctx.add_user_to_org(user_id, org1_id, vec!["USER".to_string()])
        .await;
    ctx.add_user_to_org(
        user_id,
        org2_id,
        vec!["USER".to_string(), "ADMIN".to_string()],
    )
    .await;

    // Login with explicit org selection (org2)
    let response = ctx
        .post(
            "/api/auth/login",
            &json!({
                "identity": "charlie@test.com",
                "password": "password123",
                "organization_id": org2_id
            }),
            None,
        )
        .await;

    response.assert_status(StatusCode::OK);

    let login_resp: LoginResponse = response.json();
    assert_eq!(login_resp.user.organization.id, org2_id);
    assert_eq!(login_resp.user.roles, vec!["USER", "ADMIN"]);
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let ctx = TestContext::new().await;

    let org_id = ctx.create_organization("TestCo", "Test Company").await;
    let user_id = ctx
        .create_user("Alice", "alice@test.com", "password123")
        .await;
    ctx.add_user_to_org(user_id, org_id, vec!["USER".to_string()])
        .await;

    // Wrong password
    let response = ctx
        .post(
            "/api/auth/login",
            &json!({
                "identity": "alice@test.com",
                "password": "wrongpassword"
            }),
            None,
        )
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_select_org_after_multi_org_login() {
    let ctx = TestContext::new().await;

    // Create two orgs
    let org1_id = ctx.create_organization("Org1", "First Org").await;
    let org2_id = ctx.create_organization("Org2", "Second Org").await;

    // Create user in both orgs
    let user_id = ctx
        .create_user("Dave", "dave@test.com", "password123")
        .await;
    ctx.add_user_to_org(user_id, org1_id, vec!["USER".to_string()])
        .await;
    ctx.add_user_to_org(user_id, org2_id, vec!["ADMIN".to_string()])
        .await;

    // First login to get follow-on token
    let login_response = ctx
        .post(
            "/api/auth/login",
            &json!({
                "identity": "dave@test.com",
                "password": "password123"
            }),
            None,
        )
        .await;

    login_response.assert_status(StatusCode::OK);
    let org_selection: OrgSelectionResponse = login_response.json();

    // Now select org2
    let select_response = ctx
        .post(
            "/api/auth/select-org",
            &json!({
                "follow_on_token": org_selection.follow_on_token,
                "organization_id": org2_id
            }),
            None,
        )
        .await;

    select_response.assert_status(StatusCode::OK);

    let final_login: LoginResponse = select_response.json();
    assert_eq!(final_login.user.organization.id, org2_id);
    assert_eq!(final_login.user.roles, vec!["ADMIN"]);
}

#[tokio::test]
async fn test_auth_me_endpoint() {
    let fixture = TestFixture::new().await;

    // Call /api/auth/me with user1's token
    let response = fixture
        .ctx
        .get("/api/auth/me", Some(&fixture.user1_token))
        .await;

    response.assert_success();

    let user_info: UserInfo = response.json();
    assert_eq!(user_info.id, fixture.user1_id);
    assert_eq!(user_info.identity, "user1@test.com");
    assert_eq!(user_info.organization.id, fixture.org1_id);
    assert_eq!(user_info.roles, vec!["USER"]);
}

#[tokio::test]
async fn test_auth_me_without_token() {
    let ctx = TestContext::new().await;

    // Call /api/auth/me without token
    let response = ctx.get("/api/auth/me", None).await;

    // Should return 401 Unauthorized
    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_me_with_invalid_token() {
    let ctx = TestContext::new().await;

    // Call /api/auth/me with invalid token
    let response = ctx.get("/api/auth/me", Some("invalid_token_here")).await;

    // Should return 401 Unauthorized
    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_auth_me_returns_correct_org() {
    let fixture = TestFixture::new().await;

    // Call /api/auth/me with user3's token (different org)
    let response = fixture
        .ctx
        .get("/api/auth/me", Some(&fixture.user3_token))
        .await;

    response.assert_success();

    let user_info: UserInfo = response.json();
    assert_eq!(user_info.id, fixture.user3_id);
    assert_eq!(user_info.organization.id, fixture.org2_id);
    assert_eq!(user_info.roles, vec!["USER"]);
}
