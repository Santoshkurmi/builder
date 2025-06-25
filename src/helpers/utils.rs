use std::sync::Arc;
use std::time::Duration;

use rand::{distributions::Alphanumeric, Rng};
use reqwest::Client;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{ChildStderr, ChildStdout};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;
use crate::models::app_state::ChannelMessage;
use crate::models::app_state::{AppState, BuildLog};
use crate::models::status::Status;

pub fn generate_token(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}


pub async fn read_output_lines(
    stream: Option<impl tokio::io::AsyncRead + Unpin>,
    step: usize,
    status: Status,
    state: &Arc<AppState>,
) {
    if let Some(output) = stream {
        let reader = BufReader::new(output);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            send_output(state, step, &status, &line).await;
        }
    }
}



pub async fn read_stdout( stdout: ChildStdout,step: usize,state: &Arc<AppState>) {
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    while let Ok(bytes_read) = reader.read_line(&mut line).await {
        if bytes_read == 0 {
            break; // EOF
        }

        {
            if *state.is_terminated.lock().await {
                let mut  current_build = state.builds.current_build.lock().await;
                let  current_build = current_build.as_mut().unwrap();
                current_build.status = Status::Aborted;
                break;
            }
        }//lock can be taken in above I think


        let log = BuildLog {
            timestamp: chrono::Utc::now(),
            status: Status::Success,
            step: step,
            message: line.trim().to_string(),
        };
        println!("Output: {}", line);
        let json_str = serde_json::to_string(&log).unwrap();
        let mut  current_build = state.builds.current_build.lock().await;
        let  current_build = current_build.as_mut().unwrap();
        current_build.logs.push(log);
        let _ = state.build_sender.send( ChannelMessage::Data( json_str ));
        line.clear();
        
    }
}

pub async fn read_stderr( stderr: ChildStderr,step: usize,state: &Arc<AppState>) {
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();
    while let Ok(bytes_read) = reader.read_line(&mut line).await {
        if bytes_read == 0 {
            break; // EOF
        }

        {
            if *state.is_terminated.lock().await {
                let mut  current_build = state.builds.current_build.lock().await;
                let  current_build = current_build.as_mut().unwrap();
                current_build.status = Status::Aborted;
                break;
            }
        }//lock can be taken in above I think


        let log = BuildLog {
            timestamp: chrono::Utc::now(),
            status: Status::Error,
            step: step,
            message: line.trim().to_string(),
        };
        println!("Output: {}", line);
        let json_str = serde_json::to_string(&log).unwrap();
        let mut  current_build = state.builds.current_build.lock().await;
        let  current_build = current_build.as_mut().unwrap();
        current_build.logs.push(log);
        let _ = state.build_sender.send( ChannelMessage::Data( json_str ));
        line.clear();
        
    }
}



pub async fn send_output(state: &Arc<AppState>, step: usize, status: &Status, message: &str) {
    let msg = BuildLog {
        step: step,
        status: status.clone(),//need to change here
        message: message.to_string(),
        timestamp: chrono::Utc::now(),
    };
    let json_str = serde_json::to_string(&msg).unwrap();

    let _ = state.build_sender.send( ChannelMessage::Data( json_str.clone() ));

    let mut guard = state.builds.current_build.lock().await;
    
    if let Some(current_build) = &mut *guard {
        current_build.logs.push(msg);
    }

    // buf.push(msg);
}


pub async  fn save_log(log_path:String,logs:String,token:String){

   
    // let home_dir = dirs::home_dir().expect("Home directory not found");

    // let full_path = format!("{}/{}",home_dir.to_string_lossy(),log_path);


    let full_path = log_path;

    // Create logs directory if it doesn't exist
    fs::create_dir_all( &full_path).expect("Failed to create logs directory");

    // Create a file inside ~/logs
    let mut file_path = PathBuf::from(&full_path);

    let now = Local::now();
    file_path.push(format!("{}_{}.log", now.format("%Y-%m-%d_%H-%M-%S"), token ));

    println!("File path: {}", file_path.to_str().unwrap());

    let mut file = File::create(file_path).expect("Failed to create file");
    // writeln!(file, "This is a new log entry!").expect("Failed to write to file");

    file.write_all(logs.as_bytes()).expect("Failed to write to file");

   
}


pub async fn send_to_other_server(path:String,data:String) ->bool{
    let client = Client::new();
    println!("{}",path);
    let res = client
        .post(path)
        .body(data)
        .header("Content-Type", "application/json")
        .timeout(Duration::new(5, 0))
        .send()
        .await;
    match res {
        Ok(response) => {
            let status = response.status();
            if  !status.is_success(){
                println!("failed to send data to other server: {}", status);
                let body = response.text().await.unwrap_or_default();
                println!("{}",body);
                return  false;
            }
            let body = response.text().await.unwrap_or_default();
            println!("Successfully sent data to other server: {}", status);
            println!("Response body: {}", body);
            return  true;
        }
        Err(err) => {
            println!("failed to send data to other server: {}", err);
            return  false;
        } 
    }


}