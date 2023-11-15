# info 
build concurency into the kernel directly to speed up io
use a single thread, set of upcalls, these upcalls can be nested, values passed, and look synchronous if desired
implement a rw-lock on inodes
the upcalls can also be unrelated, be synchronized like the the fork-join pattern from c
basically, a scheduler runtime will determine which task to run, and yield the task when waiting on disk for example

# c api?
syntax and syntactic sugar
in c to simulate, `run("read", 1, 1, buf).then().then().then()...`

```
c_async_b("read", 1, 1, buf, upcall1), 
 upcall1 = int x = 1; c_await_b("read", 2, 3, buf, upcall2); int y = 2; 
   upcall2 = int z = 2; c_await_b("read", 4, 5, buf, upcall3);
     ...
```
the semantics of the above will be eager blocking synchrnonous execution of the first read,
however we cannot pass values to nested upcalls

then blocking asynchronous execution within the nested upcalls as needed we could also do
```
void join(an array of function pointers)
-or- 
c_async_b_join([
 c_await_nb("read", 1, 1, buf, null),
 c_await_nb("read", 1, 2, buf, null),
 c_await_nb("read", 1, 3, buf, null) 
])
```
`void async_run() { async_nb(), async_nb(), async_nb() }`
`void async_run() { async_b(), async_b(), async_b() }` // eager


# just rust
initially i am designing a c api, but stuck because C lacks a lot of language f
eatures which makes expressing  what i want to do difficult, for example 
its not clear how to pass values to nested upcalls without a promise library, 
lack of method syntax, we need too much to add to c.
but we need to call this code from C, or do we?

what if the only way to access the async code is through rust, 
so we can only use rust to get async io. we offer a synchronous filesystem api offered to
the c code, and an asynchrnous api when you use pure rust, and of course we have that
the c and rust code both  

# async rust api
rust: we use the built in async/await syntax 
`async fn run(type: &str) -> () {if type == "read" read.await?; }` // c wrapper (deprecated)
we can simply create another trait, `Async_Stackable`, which is imcompatible with `Stackable`
the following could be the signatures
`async fn read(ino: u32, offset: u32, buf: &mut Block) -> Result<i32, Error> {}`

in std, we have the following distinctions:
std::futures/futures-rs only contain types, not the runtime
lazy(Rust Tokio) vs eager(JS) engine 

in nostd, runtime/executor/engine: pasts vs Tokio

in tokio:
`async fn run() { tokio::join! {task1, task2, task3} }` // async and unrelated
`async fn run() { tokio::spawn(async {task1.await?; task2.await?; task3.await?;}) }` // eager, illusion of synchronous
let future = run(); // lazy future
let rt = runtime::new();
rt.block_on(future) // blocking
the executor must go over the specific tasks and run them, the executor does not go over all your code, hence top level call is blocking

# progress
toy code running on x86
REWRITE NECESSAERY

# riscv
works on x86, not riscv yet

# rw-lock
the instructions for riscv 

# meeting nov 9
* reentrancy of upcalls, timer?
* queue of func arg (timer) pointer pairs (thunks), an eager event loop that checks the queue of thunks algol60
* the timer is for relative order in simulations
* read(ino, offset, thunk) // the thunks can create further downcalls or further thunks, put new thunks at the end of the queue, 
* no nesting (if calling up that would be reentrant, so just call down and put a thunk at the end of the queue) 
* continuations with async/awaits? saving context and restoring context so bad, prev system not necessary
* use a makefile to encode 

# code progess for nov 16
clock cache will contain an event loop, if the elem is not in the cache, then we will submit those disk ops to the event loop, 
we can only have a fixed vector to loop on, so everytime we add a set of operations to our queue, we will loop some of them to run, and then keep looping until there are no more left (batch processing, i cant add to the event loop while its already running a batch, ie not dynamic)

need to figure out how long one of them takes though, ideally by running on the arduino board,
or i can just set configuration parameters and benchmark the best batch size

we target x86 no_std for now

compiling using nightly
nightly-x86_64-unknown-linux-gnu (default)
rustc 1.74.0-nightly (9f5fc1bd4 2023-09-02)