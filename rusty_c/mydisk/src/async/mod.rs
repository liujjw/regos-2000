#![cfg_attr(not(unix), no_std)]
extern crate alloc;
use crate::common::*;

use alloc::string::ToString;
use async_main::{async_main, LocalSpawner};
use pasts::{prelude::*, Loop};

struct State {
    // Spawned tasks
    tasks: [LocalBoxNotify<'static, &'static str>; 2],
}

impl State {
    fn task_done(&mut self, (_id, text): (usize, &str)) -> Poll {
        Pending
    }
}

async fn task_one() -> &'static str {
    "Hello"
}

async fn task_two() -> &'static str {
    "World"
}

#[async_main]
async fn main(_spawner: LocalSpawner) {
    // create two tasks to spawn
    let task_one = Box::pin(task_one().fuse());
    let task_two = Box::pin(task_two().fuse());

    // == Allocations end ==

    // create array of tasks to spawn
    let state = &mut State {
        tasks: [task_one, task_two],
    };

    Loop::new(state)
        .on(|s| s.tasks.as_mut_slice(), State::task_done)
        .await;
}

mod main {
    #[no_mangle]
    extern "C" fn main() -> ! {
        super::main();

        loop {}
    }
}


