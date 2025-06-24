
use actix_web::{HttpRequest, HttpResponse, Responder, post, rt::task::JoinHandle, web};

use std::{collections::HashMap, sync::Arc};

use crate::{auth::check_auth::is_authorized, models::app_state::AppState};

/*
|--------------------------------------------------------------------------
| This just start the build process
|--------------------------------------------------------------------------
|
*/

#[post("/build")]
pub async fn build_initialize(
    req: HttpRequest,
    payload: web::Query<HashMap<String, String>>,
    state: web::Data<AppState>,
) -> impl Responder {

  
    if !is_authorized(&req,state.clone()).await {
        return HttpResponse::Unauthorized().body("Unauthorized");
    }

    let unique_id = payload.get(&state.config.project.build.unique_build_key);

    if unique_id.is_none() {
        return HttpResponse::BadRequest().body("Empty package name");
    }

    {
        let mut package_name_g = state.package_name.lock().await;
        *package_name_g = Some(package_name);

    }
   

    /*
    |--------------------------------------------------------------------------
    | Clear previous build logs only on next build
    |--------------------------------------------------------------------------
    |
    */

    let mut buf = state.buffer.lock().await;
    buf.clear();
    let process_state = state.get_ref().clone();

    let mut flag = state.is_building.lock().await;
    if *flag {
        let token = state.token.lock().await;

        let payload = BuildState {
            token: Some(token.clone().unwrap().to_string()),
            is_running: true,
        };

        let json_str = serde_json::to_string(&payload).unwrap();

        return HttpResponse::Ok().body(json_str);
    }
    *flag = true; // set as updating

    let process_state_clone = Arc::clone(&state);
    let handle_curent: JoinHandle<()> = actix_web::rt::spawn(async move {
        build(process_state_clone).await;
    });
    let mut handle = process_state.builder_handle.lock().await;
    *handle = Some(handle_curent);

    let mut token = state.token.lock().await;

    let new_token = generate_token(32);
    *token = Some(new_token.clone());

    let payload = BuildState {
        token: Some("".to_string() + &new_token.clone()),
        is_running: true,
    };

    let json_str = serde_json::to_string(&payload).unwrap();

    HttpResponse::Ok().body(json_str)
}