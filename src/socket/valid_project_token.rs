use std::collections::HashMap;

use actix_web::{web, HttpRequest, HttpResponse, Responder};

use crate::{auth::check_auth::is_authorized, models::{app_state::{AppState, BuildResponse}, status::Status}};



pub async fn is_valid_project_token(req: HttpRequest,payload: web::Json<HashMap<String,String>>,state: web::Data<AppState>,)-> impl Responder {

    if !is_authorized(&req,state.clone()).await {
        let res = BuildResponse{
            message: "Unauthorized Access".to_string(),
            status: Status::Unauthorized,
            build_id: None,
            token: None
        };
        return HttpResponse::Unauthorized().json(res);
    }

    

    let project_token_s = payload.get("project_token");
    if project_token_s.is_none() {
        let res = BuildResponse{
            message: "Missing project token".to_string(),
            status: Status::MissingProjectToken,
            build_id: None,
            token: None
        };
        return HttpResponse::Unauthorized().json(res);
    }

    let mut project_token = state.project_token.lock().await;

    let project_token_s = project_token_s.unwrap();
    if project_token_s == project_token.as_ref().unwrap() {
        let res = BuildResponse{
            message: "Valid Project Token".to_string(),
            status: Status::Success,
            build_id: None,
            token: None
        };
        return HttpResponse::Ok().json(res);
    }

    let change_project_token = payload.get("change_project_token");

    if change_project_token.is_none() {
        let res = BuildResponse{
            message: "Missing change project token".to_string(),
            status: Status::MissingProjectToken,
            build_id: None,
            token: None
        };
        return HttpResponse::Unauthorized().json(res);
    }

    let change_project_token = change_project_token.unwrap();


    project_token.replace(change_project_token.to_string());

        let res = BuildResponse{
            message: "Change the project token".to_string(),
            status: Status::ChangeProjectToken,
            build_id: None,
            token: None
        };
        return HttpResponse::Unauthorized().json(res);
   



}