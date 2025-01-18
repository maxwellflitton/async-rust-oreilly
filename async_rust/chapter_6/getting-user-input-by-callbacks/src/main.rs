use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicI16, AtomicBool};
use core::sync::atomic::Ordering;
use std::future::Future;
use std::task::{Poll, Context};
use std::pin::Pin;
use std::time::{Instant, Duration};
use device_query::{DeviceEvents, DeviceState};
use std::io::{self, Write};
use std::sync::LazyLock;

// Atomic shared states
static TEMP: LazyLock<Arc<AtomicI16>> = LazyLock::new(|| {
    Arc::new(AtomicI16::new(2090))
});
static DESIRED_TEMP: LazyLock<Arc<AtomicI16>> = LazyLock::new(|| {
    Arc::new(AtomicI16::new(2100))
});
static HEAT_ON: LazyLock<Arc<AtomicBool>> = LazyLock::new(|| {
    Arc::new(AtomicBool::new(false))
});
static INPUT: LazyLock<Arc<Mutex<String>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(String::new()))
});

// Conditional compilation for DEVICE_STATE
#[cfg(target_os = "macos")]
static DEVICE_STATE: LazyLock<Arc<DeviceState>> = LazyLock::new(|| {
    Arc::new(DeviceState::new())
});

#[cfg(not(target_os = "macos"))]
thread_local! {
    static DEVICE_STATE: DeviceState = DeviceState::new();
}

// DisplayFuture definition
pub struct DisplayFuture {
    pub temp_snapshot: i16,
}

impl DisplayFuture {
    pub fn new() -> Self {
        DisplayFuture {
            temp_snapshot: TEMP.load(Ordering::SeqCst),
        }
    }
}

impl Future for DisplayFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current_snapshot = TEMP.load(Ordering::SeqCst);
        let desired_temp = DESIRED_TEMP.load(Ordering::SeqCst);
        let heat_on = HEAT_ON.load(Ordering::SeqCst);

        if current_snapshot == self.temp_snapshot {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        if current_snapshot < desired_temp && !heat_on {
            HEAT_ON.store(true, Ordering::SeqCst);
        } else if current_snapshot > desired_temp && heat_on {
            HEAT_ON.store(false, Ordering::SeqCst);
        }

        clearscreen::clear().unwrap();
        println!(
            "Temperature: {}\nDesired Temp: {}\nHeater On: {}",
            current_snapshot as f32 / 100.0,
            desired_temp as f32 / 100.0,
            heat_on
        );

        self.temp_snapshot = current_snapshot;
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

// HeaterFuture definition
pub struct HeaterFuture {
    pub time_snapshot: Instant,
}

impl HeaterFuture {
    pub fn new() -> Self {
        HeaterFuture {
            time_snapshot: Instant::now(),
        }
    }
}

impl Future for HeaterFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !HEAT_ON.load(Ordering::SeqCst) {
            self.time_snapshot = Instant::now();
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        let current_snapshot = Instant::now();
        if current_snapshot.duration_since(self.time_snapshot) < Duration::from_secs(3) {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }

        TEMP.fetch_add(3, Ordering::SeqCst);
        self.time_snapshot = Instant::now();
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

// HeatLossFuture definition
pub struct HeatLossFuture {
    pub time_snapshot: Instant,
}

impl HeatLossFuture {
    pub fn new() -> Self {
        HeatLossFuture {
            time_snapshot: Instant::now(),
        }
    }
}

impl Future for HeatLossFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current_snapshot = Instant::now();
        if current_snapshot.duration_since(self.time_snapshot) > Duration::from_secs(3) {
            TEMP.fetch_sub(1, Ordering::SeqCst);
            self.time_snapshot = Instant::now();
        }

        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

// Render function
pub fn render(temp: i16, desired_temp: i16, heat_on: bool, input: String) {
    clearscreen::clear().unwrap();
    println!(
        "Temperature: {}\nDesired Temp: {}\nHeater On: {}\nInput: {}",
        temp as f32 / 100.0,
        desired_temp as f32 / 100.0,
        heat_on,
        input
    );
}

// Main function
#[tokio::main]
async fn main() {
    #[cfg(target_os = "macos")]
    let _guard = DEVICE_STATE.on_key_down(|key| {
        let mut input = INPUT.lock().unwrap();
        input.push_str(&key.to_string());
        render(
            TEMP.load(Ordering::SeqCst),
            DESIRED_TEMP.load(Ordering::SeqCst),
            HEAT_ON.load(Ordering::SeqCst),
            INPUT.lock().unwrap().clone(),
        );
    });

    #[cfg(not(target_os = "macos"))]
    DEVICE_STATE.with(|device_state| {
        device_state.on_key_down(|key| {
            let mut input = INPUT.lock().unwrap();
            input.push_str(&key.to_string());
            render(
                TEMP.load(Ordering::SeqCst),
                DESIRED_TEMP.load(Ordering::SeqCst),
                HEAT_ON.load(Ordering::SeqCst),
                INPUT.lock().unwrap().clone(),
            );
        });
    });

    let display = tokio::spawn(async {
        DisplayFuture::new().await;
    });
    let heat_loss = tokio::spawn(async {
        HeatLossFuture::new().await;
    });
    let heater = tokio::spawn(async {
        HeaterFuture::new().await;
    });

    display.await.unwrap();
    heat_loss.await.unwrap();
    heater.await.unwrap();
}
