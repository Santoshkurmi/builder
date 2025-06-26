use std::{collections::HashMap, time::Duration};

use actix_web::web;
use tokio::time::sleep;

use crate::{error_success::handle_error_success::{self, handle_error_success}, helpers::utils::{save_log, send_to_other_server}, models::{app_state::{self, AppState, BuildProcess, ChannelMessage, ProjectLog}, status::Status}};

use super::run_build::run_build;


pub async fn build_manager(state: web::Data<AppState>) {
    
    {
        let mut is_queue_running = state.is_queue_running.lock().await;
        *is_queue_running = true;
    }
   

    loop{
        println!("Starting build manager and executing build");
        let mut build_queue = state.builds.build_queue.lock().await;
        
        if build_queue.is_empty() {
            break;
        }

        let build = build_queue.remove(0);

        let build_process = BuildProcess{
            id: build.id.clone(),
            unique_id: build.unique_id.clone(),
            status: crate::models::status::Status::Building,
            current_step: 1,
            total_steps: state.config.project.build.commands.len() as usize,
            started_at: chrono::Utc::now(),
            end_at: chrono::Utc::now(),
            duration_seconds: 0,
            socket_token: build.socket_token.clone(),
            logs: Vec::new(),
            payload: build.payload.clone(),
            out_payload: HashMap::new(),
        };

        {
            state.builds.current_build.lock().await.replace(build_process);
        }

        // start the thread to perform the build operation here
        // and await
        let project_log = ProjectLog{
            id: build.id.clone(),
            unique_id: build.unique_id.clone(),
            socket_token: build.socket_token.clone(),
            step: 0,
            state: Status::Building,
        };

        drop(build_queue);

        
        {
            let mut project_logs = state.project_logs.lock().await;
            project_logs.push(project_log);
        }

        run_build(state.clone()).await;

        let _ = state.project_sender.send(ChannelMessage::Data("Build started".to_string()));
        
        

        //check the status of the build whether its failed or success
        {
            let mut current_build = state.builds.current_build.lock().await;
        
            let cur_build = current_build.as_mut().unwrap();
            let cur_build_clone = cur_build.clone();

            drop(current_build);

            handle_error_success(state.clone(),cur_build_clone.clone()).await;

            
            let _ = state.build_sender.send(ChannelMessage::Shutdown);
        }

        {

        }
            let mut current_build = state.builds.current_build.lock().await;
        
            *current_build = None;

        {

            let build_queue = state.builds.build_queue.lock().await;
            
            if build_queue.is_empty() {
                break;
            }

        }

            
            if state.config.project.next_build_delay > 0 {
                println!("Sleeping for {} seconds", state.config.project.next_build_delay);
                tokio::time::sleep(std::time::Duration::from_secs(state.config.project.next_build_delay as u64)).await;
            }

        

    }//loop forever(or until shutdown)

    
 
    {
        let mut is_queue_running = state.is_queue_running.lock().await;
        *is_queue_running = false;
    }
    
    {
        let mut project_logs = state.project_logs.lock().await;
        project_logs.clear();
    }

    //delte all the logs once get
    // let mut build_queue = state.builds.build_queue.lock().await;
    // *build_queue = Vec::new();

    println!("End of all builds");


}