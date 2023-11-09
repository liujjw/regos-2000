use core::time::Duration;
// use simple_logger::SimpleLogger;
use chrono::Local;
use pasts::{prelude::*, Loop};

use async_std::task;

async fn sleeper() -> &'static str {
    println!("Start: {}", Local::now());
    println!("Waiting 2 seconds…");

    task::sleep(Duration::new(2, 0)).await;

    println!("End: {}", Local::now());
    println!("Waited 2 seconds.");
    "Done"
}

async fn run_sync() {
    sleeper().await;
    sleeper().await;
}

async fn run() {
    struct State {
        tasks: [LocalBoxNotify<'static, &'static str>; 2],
    }
    
    impl State {
        fn task_done(&mut self, (_id, text): (usize, &str)) -> Poll {
            Pending
        }
    }

    let task_one = Box::pin(sleeper().fuse());
    let task_two = Box::pin(sleeper().fuse());

    let state = &mut State {
        tasks: [task_one, task_two],
    };

    // log::warn!("start");
    // println!("Start: {}", Local::now());
    Loop::new(state)
        .on(|s| s.tasks.as_mut_slice(), State::task_done)
        .await;
    // println!("End: {}", Local::now());
    // log::warn!("end");
}

fn main() {
    // let log = SimpleLogger::new().init().unwrap();  
    let fut = run();
    let fut_sync = run_sync();
    
    println!("sync");
    pasts::Executor::default().block_on(fut_sync);
    println!("async");
    pasts::Executor::default().block_on(fut);
}