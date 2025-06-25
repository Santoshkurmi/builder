use actix_web::web;

use crate::models::{app_state::{self, AppState, BuildProcess, ChannelMessage}, status::Status};

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
            let mut is_queue_running = state.is_queue_running.lock().await;
            *is_queue_running = false;
            break;
        }

        let build = build_queue.remove(0);
        drop(build_queue);

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
        };

        {
            state.builds.current_build.lock().await.replace(build_process);
        }

        // start the thread to perform the build operation here
        // and await

        run_build(state.clone()).await;

        let _ = state.project_sender.send(ChannelMessage::Data("Build started".to_string()));
        
        //check the status of the build whether its failed or success
        let  current_build = state.builds.current_build.lock().await;



        if let Some(cur_build) = current_build.as_ref() {
            if cur_build.status == Status::Error {
                // *current_build = None; // ✅ Clear the Option
                break;
            }
            else{
                //send here success logs to the main server
                //and continue
            }
        }

        drop(current_build);


        {
            if *state.is_terminated.lock().await {
                //handle termination here too for terminate all
                *state.is_queue_running.lock().await = false;
                break;
            } 
            if state.config.project.next_build_delay > 0 {
                println!("Sleeping for {} seconds", state.config.project.next_build_delay);
                tokio::time::sleep(std::time::Duration::from_secs(state.config.project.next_build_delay as u64)).await;
            }
        }

        let _ = state.build_sender.send(ChannelMessage::Shutdown);
        

    }//loop forever(or until shutdown)

    let mut queue = state.builds.build_queue.lock().await;
    queue.clear();

    let mut is_queue_running = state.is_queue_running.lock().await;
    *is_queue_running = false;

    let mut current_build = state.builds.current_build.lock().await;
    *current_build = None;

    let mut build_queue = state.builds.build_queue.lock().await;
    *build_queue = Vec::new();

    println!("End of all builds");


}