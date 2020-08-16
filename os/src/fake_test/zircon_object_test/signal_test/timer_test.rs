/* #[cfg(test)]
mod tests {
    use super::*;
    use kernel_hal::timer_now;

    #[test]
    fn one_shot() {
        let timer = Timer::one_shot(timer_now() + Duration::from_millis(15));
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(timer.signal(), Signal::empty());

        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(timer.signal(), Signal::SIGNALED);
    }

    #[test]
    fn set() {
        let timer = Timer::new();
        timer.set(timer_now() + Duration::from_millis(10), Duration::default());
        timer.set(timer_now() + Duration::from_millis(20), Duration::default());

        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(timer.signal(), Signal::empty());

        std::thread::sleep(Duration::from_millis(15));
        assert_eq!(timer.signal(), Signal::SIGNALED);

        timer.set(timer_now() + Duration::from_millis(10), Duration::default());
        assert_eq!(timer.signal(), Signal::empty());
    }

    #[test]
    fn cancel() {
        let timer = Timer::new();
        timer.set(timer_now() + Duration::from_millis(10), Duration::default());

        std::thread::sleep(Duration::from_millis(5));
        timer.cancel();

        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(timer.signal(), Signal::empty());
    }
}
 */