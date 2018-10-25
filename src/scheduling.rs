//! Let factorize a huge amount of scheduling policies into one api.
use depjoin;
use rayon;
//use rayon::current_num_threads;
use std::cell::RefCell;
use std::cmp::min;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use traits::{Divisible, DivisibleAtIndex};

// we use this boolean to prevent fine grain parallelism when coarse grain
// parallelism is still available in composed algorithms.
thread_local!(static SEQUENCE: RefCell<bool> = RefCell::new(false));

/// All scheduling available scheduling policies.
#[derive(Copy, Clone)]
pub enum Policy {
    /// Do all computations sequentially.
    Sequential,
    /// Recursively cut in two with join until given block size.
    Join(usize),
    /// Recursively cut in two with join_context until given block size.
    JoinContext(usize),
    /// Recursively cut in two with depjoin until given block size.
    DepJoin(usize),
    /// Advance locally with increasing block sizes. When stolen create tasks
    /// We need an initial block size.
    Adaptive(usize),
}

pub(crate) fn schedule<I, WF, MF, RF, O>(
    input: I,
    work_function: &WF,
    map_function: &MF,
    reduce_function: &RF,
    policy: Policy,
) -> O
where
    I: Divisible,
    WF: Fn(I, usize) -> I + Sync,
    MF: Fn(I) -> O + Sync,
    RF: Fn(O, O) -> O + Sync,
    O: Send + Sized,
{
    match policy {
        Policy::Sequential => schedule_sequential(input, work_function, map_function),
        Policy::Join(block_size) => schedule_join(
            input,
            work_function,
            map_function,
            reduce_function,
            block_size,
        ),
        Policy::JoinContext(block_size) => schedule_join_context(
            input,
            work_function,
            map_function,
            reduce_function,
            block_size,
        ),
        Policy::DepJoin(block_size) => schedule_depjoin(
            input,
            work_function,
            map_function,
            reduce_function,
            block_size,
        ),
        Policy::Adaptive(block_size) => SEQUENCE.with(|s| {
            if *s.borrow() {
                schedule_sequential(input, work_function, map_function)
            } else {
                schedule_adaptive(
                    input,
                    work_function,
                    map_function,
                    reduce_function,
                    block_size,
                )
            }
        }),
    }
}

fn schedule_sequential<I, WF, MF, O>(input: I, work_function: &WF, map_function: &MF) -> O
where
    I: Divisible,
    WF: Fn(I, usize) -> I,
    MF: Fn(I) -> O,
{
    let len = input.len();
    map_function(work_function(input, len))
}

fn schedule_join<I, WF, MF, RF, O>(
    input: I,
    work_function: &WF,
    map_function: &MF,
    reduce_function: &RF,
    block_size: usize,
) -> O
where
    I: Divisible,
    WF: Fn(I, usize) -> I + Sync,
    MF: Fn(I) -> O + Sync,
    RF: Fn(O, O) -> O + Sync,
    O: Send + Sized,
{
    let len = input.len();
    if len <= block_size {
        map_function(work_function(input, len))
    } else {
        let (i1, i2) = input.split();
        let (r1, r2) = rayon::join(
            || schedule_join(i1, work_function, map_function, reduce_function, block_size),
            || schedule_join(i2, work_function, map_function, reduce_function, block_size),
        );
        reduce_function(r1, r2)
    }
}

fn schedule_join_context<I, WF, MF, RF, O>(
    input: I,
    work_function: &WF,
    map_function: &MF,
    reduce_function: &RF,
    block_size: usize,
) -> O
where
    I: Divisible,
    WF: Fn(I, usize) -> I + Sync,
    MF: Fn(I) -> O + Sync,
    RF: Fn(O, O) -> O + Sync,
    O: Send + Sized,
{
    let len = input.len();
    if len <= block_size {
        map_function(work_function(input, len))
    } else {
        let (i1, i2) = input.split();
        let (r1, r2) = rayon::join_context(
            |_| schedule_join_context(i1, work_function, map_function, reduce_function, block_size),
            |c| {
                if c.migrated() {
                    schedule_join_context(
                        i2,
                        work_function,
                        map_function,
                        reduce_function,
                        block_size,
                    )
                } else {
                    let len = i2.len();
                    map_function(work_function(i2, len))
                }
            },
        );
        reduce_function(r1, r2)
    }
}

fn schedule_depjoin<I, WF, MF, RF, O>(
    input: I,
    work_function: &WF,
    map_function: &MF,
    reduce_function: &RF,
    block_size: usize,
) -> O
where
    I: Divisible,
    WF: Fn(I, usize) -> I + Sync,
    MF: Fn(I) -> O + Sync,
    RF: Fn(O, O) -> O + Sync,
    O: Send + Sized,
{
    let len = input.len();
    if len <= block_size {
        map_function(work_function(input, len))
    } else {
        let (i1, i2) = input.split();
        depjoin(
            || schedule_depjoin(i1, work_function, map_function, reduce_function, block_size),
            || schedule_depjoin(i2, work_function, map_function, reduce_function, block_size),
            reduce_function,
        )
    }
}

struct AdaptiveWorker<
    'a,
    'b,
    I: Divisible,
    WF: Fn(I, usize) -> I + Sync + 'b,
    MF: Fn(I) -> O + Sync + 'b,
    RF: Fn(O, O) -> O + Sync + 'b,
    O: Send + Sized,
> {
    input: I,
    initial_block_size: usize,
    current_block_size: usize,
    stolen: &'a AtomicBool,
    sender: Sender<Option<I>>,
    work_function: &'b WF,
    map_function: &'b MF,
    reduce_function: &'b RF,
    phantom: PhantomData<(O)>,
}

impl<'a, 'b, I, WF, MF, RF, O> AdaptiveWorker<'a, 'b, I, WF, MF, RF, O>
where
    I: Divisible,
    WF: Fn(I, usize) -> I + Sync,
    MF: Fn(I) -> O + Sync,
    RF: Fn(O, O) -> O + Sync + 'b,
    O: Send + Sized,
{
    fn new(
        input: I,
        initial_block_size: usize,
        stolen: &'a AtomicBool,
        sender: Sender<Option<I>>,
        work_function: &'b WF,
        map_function: &'b MF,
        reduce_function: &'b RF,
    ) -> Self {
        // adjust block size to fit on boundaries
        let blocks_number = (((input.len() as f64) / initial_block_size as f64 + 1.0).log(2.0)
            - 1.0)
            .floor() as i32;
        let current_block_size =
            ((input.len() as f64) / (2.0f64.powi(blocks_number + 1) - 1.0)).ceil() as usize;

        AdaptiveWorker {
            input,
            initial_block_size,
            current_block_size,
            stolen,
            sender,
            work_function,
            map_function,
            reduce_function,
            phantom: PhantomData,
        }
    }
    fn is_stolen(&self) -> bool {
        self.stolen.load(Ordering::Relaxed)
    }
    fn answer_steal(self) -> O {
        let (my_half, his_half) = self.input.split();
        self.sender.send(Some(his_half)).expect("sending failed");
        schedule_adaptive(
            my_half,
            self.work_function,
            self.map_function,
            self.reduce_function,
            self.initial_block_size,
        )
    }

    fn cancel_stealing_task(&mut self) {
        self.sender.send(None).expect("canceling task failed");
    }

    //TODO: we still need macro blocks
    fn schedule(mut self) -> O {
        // TODO: automate this min everywhere ?
        // TODO: factorize a little bit
        // start by computing a little bit in order to get a first output
        let size = min(self.input.len(), self.current_block_size);

        if self.input.len() <= self.current_block_size {
            self.cancel_stealing_task(); // no need to keep people waiting for nothing
            self.input = (self.work_function)(self.input, size);
            return (self.map_function)(self.input);
        } else {
            self.input = (self.work_function)(self.input, size);
            if self.input.len() == 0 {
                // it's over
                self.cancel_stealing_task();
                return (self.map_function)(self.input);
            }
        }

        // loop while not stolen or something left to do
        loop {
            if self.is_stolen() && self.input.len() > self.initial_block_size {
                return self.answer_steal();
            }
            self.current_block_size *= 2;
            let size = min(self.input.len(), self.current_block_size);

            if self.input.len() <= self.current_block_size {
                self.cancel_stealing_task(); // no need to keep people waiting for nothing
                self.input = (self.work_function)(self.input, size);
                return (self.map_function)(self.input);
            }
            SEQUENCE.with(|s| *s.borrow_mut() = true); // we force subtasks to work sequentially
            self.input = (self.work_function)(self.input, size);
            SEQUENCE.with(|s| *s.borrow_mut() = false);
            if self.input.len() == 0 {
                // it's over
                self.cancel_stealing_task();
                return (self.map_function)(self.input);
            }
        }
    }
}

fn schedule_adaptive<I, WF, MF, RF, O>(
    input: I,
    work_function: &WF,
    map_function: &MF,
    reduce_function: &RF,
    initial_block_size: usize,
) -> O
where
    I: Divisible,
    WF: Fn(I, usize) -> I + Sync,
    MF: Fn(I) -> O + Sync,
    RF: Fn(O, O) -> O + Sync,
    O: Send + Sized,
{
    let size = input.len();
    if size <= initial_block_size {
        map_function(work_function(input, size))
    } else {
        let stolen = &AtomicBool::new(false);
        let (sender, receiver) = channel();

        let worker = AdaptiveWorker::new(
            input,
            initial_block_size,
            stolen,
            sender,
            work_function,
            map_function,
            reduce_function,
        );

        //TODO depjoin instead of join
        let (o1, maybe_o2) = rayon::join(
            move || worker.schedule(),
            move || {
                stolen.store(true, Ordering::Relaxed);
                #[cfg(feature = "logs")]
                let input =
                    rayon::sequential_task(1, 1, || receiver.recv().expect("receiving failed"))?;
                #[cfg(not(feature = "logs"))]
                let input = receiver.recv().expect("receiving failed")?;
                assert!(input.len() > 0);
                Some(schedule_adaptive(
                    input,
                    work_function,
                    map_function,
                    reduce_function,
                    initial_block_size,
                ))
            },
        );

        let fusion_needed = maybe_o2.is_some();
        if fusion_needed {
            reduce_function(o1, maybe_o2.unwrap())
        } else {
            o1
        }
    }
}

pub fn fully_adaptive_schedule<I, WF, RETF>(input: I, work_function: &WF, retrieve_function: &RETF)
where
    I: Divisible,
    WF: Fn(I, usize) -> I + Sync,
    RETF: Fn(I, I, I) -> I + Sync,
{
    unimplemented!()
}
