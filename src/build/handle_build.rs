
use actix_web::{HttpRequest, HttpResponse, Responder, post, rt::task::JoinHandle, web};
use uuid::Uuid;

use std::{collections::HashMap, sync::Arc};

use crate::{auth::check_auth::is_authorized, build::build_manager::{self, build_manager}, helpers::utils::generate_token, models::{app_state::{AppState, BuildProcess, BuildRequest, BuildResponse, BuildState}, config::Config, status::Status}};



pub async fn build_initialize(
    req: HttpRequest,
    payload: web::Json<HashMap<String, String>>,
    state: web::Data<AppState>,
) -> impl Responder {

  
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

    if state.config.project.max_pending_build == state.builds.build_queue.lock().await.len() as u32{
        let res = BuildResponse{
            message: format!("Max Pending Reached: {}", state.config.project.max_pending_build),
            build_id: None,
            token: None,
            status: Status::MaxPending,
        };
        return HttpResponse::Conflict().json(res);
    }

    let mut is_already_running = false;
    if let Some(current_build) = &*guard {
        if &current_build.unique_id == unique_id.unwrap()  {
            let res = BuildResponse{
            message: format!("Build already in progress: {}", current_build.unique_id),
            build_id: Some(current_build.id.clone()),
            token: Some( current_build.socket_token.clone() ),
            status: Status::AlreadyBuilding,
        };
            return HttpResponse::Conflict().json(res);
        }
        is_already_running = true;
    }//if current build exists

    drop(guard);

    let mut build_queue = state.builds.build_queue.lock().await;
    
    let is_already_queued = build_queue.iter().any(|build| {
        &build.unique_id == unique_id.unwrap()
    });

    if is_already_queued {
        let res = BuildResponse{
            message: format!("Build already in queue"),
            token: None,
            build_id: None,
            status: Status::AlreadyQueue,
        };
        return HttpResponse::Conflict().json(res);
    }
   

    let new_token = generate_token(32);
    let id = Uuid::new_v4();

    let build_state =  BuildRequest{
        id: id.to_string(),
        unique_id: unique_id.unwrap().to_string(),
        payload: payload.clone(),
        socket_token: new_token.clone(),
    };

    build_queue.push(build_state);
    drop(build_queue);

    if !*state.is_queue_running.lock().await{


        tokio::spawn(async move {
            build_manager(state.clone()).await;
        });
    }
     
    let res = BuildResponse{
        message:  if is_already_running {"Build is in pending state".to_string()} else {"Build started".to_string()},
        token: Some(new_token.clone()),
        build_id: Some( id.to_string() ),
        status:if is_already_running {Status::Pending} else {Status::Building},
    };
    HttpResponse::Ok().json(res)
}