#![feature(coroutines, coroutine_trait)]
use rand::Rng;
use std::{
    ops::{Coroutine, CoroutineState},
    pin::Pin,
    thread::sleep,
    time::Duration,
};

struct RandCoroutine {
    pub value: u8,
    pub live: bool,
}

impl RandCoroutine {
    fn new() -> Self {
        let mut coroutine = Self {
            value: 0,
            live: true,
        };

        coroutine.generate();
        coroutine
    }

    fn generate(&mut self) {
        let mut rng = rand::thread_rng();
        self.value = rng.gen_range(0..=10);
    }
}

impl Coroutine<()> for RandCoroutine {
    type Yield = u8;
    type Return = ();

    fn resume(mut self: Pin<&mut Self>, _arg: ()) -> CoroutineState<Self::Yield, Self::Return> {
        self.generate();
        CoroutineState::Yielded(self.value)
    }
}

fn main() {
    let mut coroutines: Vec<RandCoroutine> = vec![];

    for _ in 0..10 {
        coroutines.push(RandCoroutine::new());
    }

    let mut total: u32 = 0;

    loop {
        let mut all_dead = true;
        for mut coroutine in coroutines.iter_mut() {
            if coroutine.live {
                match Pin::new(&mut coroutine).resume(()) {
                    CoroutineState::Yielded(value) => {
                        total += value as u32;
                    }
                    CoroutineState::Complete(()) => {
                        panic!("Coroutine should not be completed");
                    }
                }
                all_dead = false;

                if coroutine.value < 9 {
                    coroutine.live = false;
                }
            }
        }

        if all_dead {
            break;
        }
    }

    println!("Total: {}", total);

    let (sender, receiver) = std::sync::mpsc::channel::<RandCoroutine>();

    let _ = std::thread::spawn(move || loop {
        let mut coroutine = match receiver.recv() {
            Ok(coroutine) => coroutine,
            Err(_) => break,
        };

        match Pin::new(&mut coroutine).resume(()) {
            CoroutineState::Yielded(result) => {
                println!("Coroutine yielded: {}", result);
            }
            CoroutineState::Complete(_) => {
                panic!("Coroutine should not be completed");
            }
        }
    });

    sleep(Duration::from_secs(1));

    sender.send(RandCoroutine::new()).unwrap();
    sender.send(RandCoroutine::new()).unwrap();
    sender.send(RandCoroutine::new()).unwrap();

    sleep(Duration::from_secs(1));
}
