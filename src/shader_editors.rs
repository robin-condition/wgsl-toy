use leptos::prelude::*;
use leptos::{component, view, IntoView, Params};
use leptos_router::hooks::use_params;
use leptos_router::params::Params;
use serde::Deserialize;

use crate::shader_editor::ShaderEditor;
use shaderwheels_logic::web::SHADER_API_ENDPOINT;

#[derive(Params, PartialEq)]
struct UrlParams {
    id: Option<u32>,
}

#[derive(Deserialize)]
struct ShaderInfo {
    contents: String,

}

#[component]
pub fn ShaderEditorFromExplicitId(#[prop(into)] id: Signal<u32>) -> impl IntoView {
    let read_shader = LocalResource::new(move || async move {
        let http_client = reqwest::ClientBuilder::new().build().unwrap();//expect_context::<Client>();
        let url = SHADER_API_ENDPOINT.to_string() + "/get-shader-text/" + id.get().to_string().as_str();
        let ret = http_client.get(url).send().await;//.map(|f| f.text().unwrap());
        let shader_contents = match ret {
            Ok(resp) => {
                resp.json::<String>().await
            },
            Err(e) => Err(e)
        };

        shader_contents.map(|contents| ShaderInfo{contents})
    });

    let rendered_stuff = move || {
        match read_shader.read().as_ref().as_ref() {
            Some(Ok(shader_info)) => view! { <ShaderEditor starting_text=shader_info.contents.clone() /> }.into_any(),
            Some(Err(_)) => view! { <p>Failed to retrieve shader!</p> }.into_any(),
            None => view! { <div>"Loading..."</div> }.into_any(),
        }
    };
    view! { {rendered_stuff} }
}

#[component]
pub fn ShaderEditorFromId() -> impl IntoView {
    let use_id = use_params::<UrlParams>();
    let sig = Signal::derive(move || use_id.read().as_ref().unwrap().id.unwrap());
    view! {
        <Show
            when=move || { use_id.read().as_ref().is_ok_and(|f| f.id.is_some()) }
            fallback=move || {
                view! { "Failed to load id parameter. How did you get here?" }
            }
        >
            "hi shader #"
            {sig.get()}
            <ShaderEditorFromExplicitId id=sig />
        </Show>
    }
}
