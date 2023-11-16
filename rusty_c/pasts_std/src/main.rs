use core::time::Duration;
// use simple_logger::SimpleLogger;
use chrono::Local;
use pasts::{notify, prelude::*, Loop, Executor};

use async_std::task;

// see counter.rs, spawn.rs, slices.rs, and tasks.rs

struct State_With_Upcalls<'a> {
    counter: usize,
    one: &'a mut (dyn Notify<Event = ()> + Unpin),
    two: &'a mut (dyn Notify<Event = ()> + Unpin),
}

impl State_With_Upcalls<'_> {
    fn one(&mut self, (): ()) -> Poll {
        println!("Upcall one with shared counter value {}", self.counter);
        self.counter += 1;
        Pending
    }

    fn two(&mut self, (): ()) -> Poll {
        println!("Upcall two with shared counter value {}", self.counter);
        self.counter += 2;
        Pending
    }

    async fn run_sync_async_with_upcalls(&mut self) {
        Loop::new(self)
            .on(|s| &mut s.one, Self::one)
            .on(|s| &mut s.two, Self::two)
            .await;
    }
}

struct State {
    tasks: [LocalBoxNotify<'static, &'static str>; 2],
}

impl State {
    fn task_done(&mut self, (_id, text): (usize, &str)) -> Poll {
        Pending
    }
}

async fn sleeper(sig: &'static str) -> &'static str {
    println!("Start {}: {}", sig, Local::now());

    task::sleep(Duration::new(2, 0)).await;

    println!("End {}: {}", sig, Local::now());
    "Done"
}
// sync code does not work!
async fn sync_sleeper() -> &'static str {
    let mut i: u32 = 0;
    let iterations = 3_000_000_000;
    println!("Start loop: {}", Local::now());
    while i < iterations {
        i += 1;
    }
    println!("End loop: {}", Local::now());
    "Done"
}

async fn run_sync() {
    sleeper("sync").await;
    sleeper("sync").await;
}

// static task set with upcalls
async fn run_static_async() {
    let task_one = Box::pin(sleeper("async").fuse());
    let task_two = Box::pin(sleeper("async").fuse());

    let state = &mut State {
        tasks: [task_one, task_two],
    };

    Loop::new(state)
        .on(|s| s.tasks.as_mut_slice(), State::task_done)
        .await;
}

// dynamic task set
async fn run_dynamic_async() {
    sleeper("spawn").await;
}

// does not work
async fn run_sync_async() {
    let task_one = Box::pin(sync_sleeper().fuse());
    let task_two = Box::pin(sync_sleeper().fuse());

    let state = &mut State {
        tasks: [task_one, task_two],
    };

    Loop::new(state)
        .on(|s| s.tasks.as_mut_slice(), State::task_done)
        .await;
}

fn main() {
    // let log = SimpleLogger::new().init().unwrap();  
    let executor = Executor::default();
    
    // does not work with sync code! TODO
    // println!("-----------------------sync_async-------------------------");
    // executor.clone().block_on(run_sync_async());


    // println!("-----------------------sync-------------------------");
    // executor.clone().block_on(run_sync());
    // println!("-----------------------async: dynamic task set created and run with static task set-----------------------");
    // executor.clone().spawn_boxed(run_dynamic_async());
    // executor.clone().spawn_boxed(run_dynamic_async());
    // executor.clone().block_on(run_static_async());
    println!("-----------------------async with different upcalls-------------------------");
    executor.clone().block_on(async {
        let sleep = |seconds| task::sleep(Duration::from_secs_f64(seconds));
        let mut state = State_With_Upcalls {
            counter: 0,
            one: &mut notify::future_fn(|| Box::pin(sleep(1.0))),
            two: &mut notify::future_fn(|| Box::pin(sleep(2.0))),
        };
        state.run_sync_async_with_upcalls().await;
    });
}