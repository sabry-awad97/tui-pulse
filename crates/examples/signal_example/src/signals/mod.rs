use pulse::prelude::*;

// Define global signals
pub static COUNTER: GlobalSignal<i32> = Signal::global(|| 0);
pub static USER_NAME: GlobalSignal<String> = Signal::global(|| String::from("Guest"));
