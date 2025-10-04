use openidconnect::{
    AuthenticationFlow, AuthorizationCode, ClientId, CsrfToken, EmptyAdditionalClaims,
    EmptyAdditionalProviderMetadata, IssuerUrl, Nonce, OAuth2TokenResponse, ProviderMetadata,
    RedirectUrl, Scope,
    core::{
        CoreAuthDisplay, CoreClaimName, CoreClaimType, CoreClient, CoreClientAuthMethod,
        CoreGenderClaim, CoreGrantType, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm,
        CoreJweKeyManagementAlgorithm, CoreResponseMode, CoreResponseType,
        CoreSubjectIdentifierType,
    },
    reqwest,
};
use serde::Deserialize;

pub type CognitoProviderMetadata = ProviderMetadata<
    EmptyAdditionalProviderMetadata,
    CoreAuthDisplay,
    CoreClientAuthMethod,
    CoreClaimName,
    CoreClaimType,
    CoreGrantType,
    CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm,
    CoreJsonWebKey,
    CoreResponseMode,
    CoreResponseType,
    CoreSubjectIdentifierType,
>;

pub const SHADER_API_ENDPOINT: &str = include_str!("api-endpoint.txt");

#[derive(Deserialize, Clone)]
pub struct MyOICDCfgFile {
    client_id: String,
    issuer: String,
    redirect_uri: String,
    post_logout_redirect_uri: String,
    scope: String,
}

pub struct OICDCtx {
    issuer_url: IssuerUrl,
    client_id: ClientId,
}

pub struct AuthReturn {
    code: Option<String>,
    state: Option<String>,
}

pub async fn doauth() {
    let cfg = include_str!("private-config.json");
    let ctx: MyOICDCfgFile = serde_json::from_str::<MyOICDCfgFile>(cfg).unwrap();

    let issuer_url = IssuerUrl::new(ctx.issuer).unwrap();
    let client_id = ClientId::new(ctx.client_id);
    let redirect_url = RedirectUrl::new(ctx.redirect_uri).unwrap();

    let http_client = reqwest::ClientBuilder::new().build().unwrap();
    let provider_meta = CognitoProviderMetadata::discover_async(issuer_url, &http_client)
        .await
        .unwrap();

    let client = CoreClient::from_provider_metadata(provider_meta, client_id, None)
        .set_redirect_uri(redirect_url);

    let (authorize_url, csrf_state, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        // This example is requesting access to the "calendar" features and the user's profile.
        .add_scope(Scope::new("email".to_string()))
        .url();

    println!("Auth: {authorize_url}");

    let authret: Result<AuthReturn, std::io::Error> = Ok(AuthReturn {
        code: None,
        state: None,
    }); //use_query::<AuthReturn>();
    if authret.is_err() || authret.as_ref().unwrap().code.is_none() {
        return;
    }
    let authreturn = authret.unwrap();
    let code = AuthorizationCode::new(authreturn.code.unwrap());
    let state = CsrfToken::new(authreturn.state.unwrap());

    let token_resp = client
        .exchange_code(code)
        .unwrap()
        .request_async(&http_client)
        .await
        .unwrap();
    println!("Scopes: {:?}", token_resp.scopes());

    // Should verify in here

    let uinf: openidconnect::UserInfoClaims<EmptyAdditionalClaims, CoreGenderClaim> = client
        .user_info(token_resp.access_token().clone(), None)
        .unwrap()
        .request_async(&http_client)
        .await
        .unwrap();

    //token_resp.

    println!("Userinfo: {:?}", uinf);
}
