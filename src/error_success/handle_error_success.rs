use std::{fs, path::{ Path}, time::Duration};

use actix_web::web;
use tokio::time::sleep;

use crate::{ helpers::utils::{save_log, secure_join_path, send_to_other_server}, models::{app_state::{AppState, BuildProcess}, config::PayloadType}};



/// handle the success,error log to be send to the other server of the build
pub async fn handle_error_success(state: web::Data<AppState>,current_build: BuildProcess) {


    let log_str = serde_json::to_string(&current_build).unwrap();

    let mut buld = current_build.clone();

        if state.config.enable_logs{
            save_log(&state.config.log_path, log_str.clone(), current_build.unique_id.clone()).await;
        }
        

        let url = state.config.project.build.on_success_failure.clone();
        let state_clone = state.clone();

        for out_paylaod in state.config.project.build.on_success_error_payload.clone(){
            
            if out_paylaod.r#type == PayloadType::File{

                let file_path = if out_paylaod.key2.is_some() {
                    out_paylaod.key2.as_ref().unwrap()
                } else {
                    out_paylaod.key1.as_str()
                };
                let path_relative = secure_join_path(&state.config.project.project_path, &file_path);
                if path_relative.is_none(){
                    println!("Failed to create payload file: Path is not secure");
                    continue;
                }
                let path_relative = path_relative.unwrap();
                // let path_relative = format!("{}/{}", state.config.project.project_path, file_path);
                // println!("path_relative {}", path_relative);
                let path = Path::new(path_relative.as_str());

                if path.exists() {
                    let string_opt =  fs::read_to_string(path).ok();
                    if string_opt.is_some() {
                        let string = string_opt.unwrap();
                        buld.out_payload.insert(out_paylaod.key1.to_string(), string);
                }
            }
                continue;

            }//handle file reading here

            let env_name = if out_paylaod.key2.is_some() {
                out_paylaod.key2.as_ref().unwrap()
            } else {
                out_paylaod.key1.as_str()
            };

            let env_value = current_build.payload.get(env_name);

            if env_value.is_none() {
                continue;
            }
            buld.out_payload.insert(env_name.to_string(), env_value.unwrap().to_string());


        }
    
        // println!("out_payload {:?}", buld.out_payload);
        let _ = tokio::spawn(async move {
            let is_send = send_to_other_server(url.clone(), log_str.clone()).await;
            
            if !is_send {
                println!("Retrying to send data to other server");
                sleep(Duration::from_secs(10)).await;
                send_to_other_server(url.clone(), log_str.clone()).await;
        
                let mut error_logs = state_clone.builds.failed_history.lock().await;
                error_logs.push(buld);
                
            }
            else{
            println!("Done everything");
            }

        }).await; //wait here until the thread is done


}
