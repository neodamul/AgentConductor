use std::{
    sync::Arc,
    time::{Duration, Instant},
};

pub fn close<T>(room: Arc<T>) {
    let deadline = Instant::now() + Duration::from_secs(30);
    // A disconnected worker releases its Room asynchronously; Windows keeps its files locked.
    while Arc::strong_count(&room) > 1 {
        assert!(
            Instant::now() < deadline,
            "native request worker did not release the Room"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    drop(room);
}
