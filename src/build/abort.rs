use std::collections::HashMap;

use actix_web::{web, HttpRequest, HttpResponse, Responder};

use crate::{auth::check_auth::is_authorized, models::{app_state::{AppState, BuildResponse}, status::Status}};



pub async fn abort(req: HttpRequest,payload: web::Json<HashMap<String, String>>,state: web::Data<AppState>,)-> impl Responder {

    if !is_authorized(&req,state.clone()).await {
        let res = BuildResponse{
            message: "Unauthorized Access".to_string(),
            status: Status::Unauthorized,
            build_id: None,
            token: None
        };
        return HttpResponse::Unauthorized().json(res);
    }

    let unique_id = payload.get(&state.config.project.build.unique_build_key);

    if unique_id.is_none() {
        let res = BuildResponse{
            message: format!("Missing unique build key: {}", state.config.project.build.unique_build_key),
            status: Status::MissingUniqueId,
            build_id: None,
            token: None
        };
        return HttpResponse::BadRequest().json(res);
    }

    let guard = state.builds.current_build.lock().await;

    if let Some(current_build) = &*guard {
        if &current_build.unique_id == unique_id.unwrap()  {
            let mut is_terminated = state.is_terminated.lock().await;
            *is_terminated = true;
            println!("Terminating build");
            return HttpResponse::Ok().json("Aborted");
        }
    }

    let mut queue = state.builds.build_queue.lock().await;
    if let Some(index) = queue.iter().position(|build| {
        &build.unique_id == unique_id.unwrap()
    }) {
        queue.remove(index);
        return HttpResponse::Ok().json("Aborted Pending Build");
    }


    HttpResponse::Unauthorized().json("Aborted No Build Found")


}


pub async fn abort_all(req: HttpRequest,payload: web::Json<HashMap<String, String>>,state: web::Data<AppState>,)-> impl Responder {

    if !is_authorized(&req,state.clone()).await {
        let res = BuildResponse{
            message: "Unauthorized Access".to_string(),
            status: Status::Unauthorized,
            build_id: None,
            token: None
        };
        return HttpResponse::Unauthorized().json(res);
    }


    let mut is_terminated = state.is_terminated.lock().await;
    *is_terminated = true;
   

    

    HttpResponse::Unauthorized().json("Aborted All Builds")


}