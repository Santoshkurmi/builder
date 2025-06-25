use std::process::Stdio;

use actix_web::web;
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command};

use crate::{helpers::utils::{read_stderr, read_stdout}, models::{app_state::{self, AppState, BuildLog, BuildProcess, ChannelMessage}, status::Status}};


pub async fn run_build(state: web::Data<AppState>) {


    let mut step = 1;
    for command in &state.config.project.build.commands {

        let mut child = Command::new("bash")
            .arg("-c")
            .arg(&command.command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        

        let  stdout = child.stdout.take().unwrap();
        let  stderr = child.stderr.take().unwrap();

       
        tokio::join!(
            read_stdout(stdout, step, &state),
            read_stderr(stderr, step, &state)
        );
        
        if * state.is_terminated.lock().await {
            child.kill().await.unwrap();
            break;
        }

       
        let status = child.wait().await.expect("Failed to wait on child");
        if status.success() {
            let mut  current_build = state.builds.current_build.lock().await;
            let  current_build = current_build.as_mut().unwrap();
            current_build.status = Status::Success; //nothing much to do
            
        } else {

            let mut  current_build = state.builds.current_build.lock().await;
            let  current_build = current_build.as_mut().unwrap();
            current_build.status = Status::Error; //
            if command.on_error == "abort" {
                *state.is_terminated.lock().await = true;
                break;
            }//handle the case here all the other will also be terminated, handle here
        }

        step += 1;


    }//loop each command




}