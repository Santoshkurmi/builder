use std::{collections::HashMap, hash::Hash, process::Stdio};

use actix_web::web;
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command};

use crate::{helpers::utils::{read_stderr, read_stdout}, models::{app_state::{self, AppState, BuildLog, BuildProcess, ChannelMessage}, config::PayloadType, status::Status}};


pub async fn run_build(state: web::Data<AppState>) {

    let mut env_map: HashMap<String, String> = HashMap::new();

    for payload in &state.config.project.build.payload {
        if payload.r#type != PayloadType::Env{
            continue;
        }
        let env_name = if payload.key2.is_some() {
            payload.key2.as_ref().unwrap()
        } else {
            payload.key1.as_str()
        };

        let mut  current_build = state.builds.current_build.lock().await;
        let  current_build = current_build.as_mut().unwrap();
        let env_value = current_build.payload.get(payload.key1.as_str()).unwrap();
        env_map.insert(env_name.to_string(), env_value.to_string());
    }


    let mut step = 1;
    for command in &state.config.project.build.commands {

        let command_with_env = format!("{} && echo '+_+_+_\n' && env", command.command);
        println!("Running command: {}", command_with_env);
        let mut child = Command::new("bash")
            .arg("-c")
            .envs(&env_map)
            .arg( &command_with_env )
            
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