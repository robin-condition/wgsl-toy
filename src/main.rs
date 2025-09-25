use leptos::{logging, prelude::*};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use openidconnect::{
    core::{
        CoreAuthDisplay, CoreClaimName, CoreClaimType, CoreClient, CoreClientAuthMethod,
        CoreGrantType, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm,
        CoreJweKeyManagementAlgorithm, CoreResponseMode, CoreResponseType,
        CoreSubjectIdentifierType,
    }, reqwest::{self, Client}, AuthenticationFlow, ClientId, CsrfToken, EmptyAdditionalProviderMetadata, IssuerUrl, Nonce, ProviderMetadata, RedirectUrl, Scope
};
use serde::Deserialize;

use crate::shader_editor::ShaderEditor;
pub mod shader_editor;

type CognitoProviderMetadata = ProviderMetadata<
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
    client_id: ClientId
}

#[component]
fn App() -> impl IntoView {
    let cfg = include_str!("private-config.json");
    let cfg_file: MyOICDCfgFile = serde_json::from_str::<MyOICDCfgFile>(cfg).unwrap();

    provide_context(cfg_file);

    // https://github.com/ramosbugs/openidconnect-rs/blob/main/examples/google.rs

    provide_context(reqwest::ClientBuilder::new().build().unwrap());

    let resource = LocalResource::new(move || {
        async move {
            let ctx = expect_context::<MyOICDCfgFile>();

            let issuer_url = IssuerUrl::new(ctx.issuer).unwrap();
            let client_id = ClientId::new(ctx.client_id);
            let redirect_url = RedirectUrl::new(ctx.redirect_uri).unwrap();

            let http_client = expect_context::<Client>();
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

            logging::log!("Auth: {authorize_url}");
        }
    }
    );

    view! {
        <Router>
            <Routes fallback=|| {
                view! { hi }
            }>
                <Route
                    path=path!("/")
                    view=|| {
                        view! { <ShaderEditor /> }
                    }
                />
            </Routes>
        </Router>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| App())
}
