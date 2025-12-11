use atomx::signal;

#[derive(Clone, Default, PartialEq, Debug)]
struct TestData {
    value: i32,
}

#[test]
fn equals_last_returns_true_on_initial_default() {
    let (source, _sink) = signal::create::<TestData>();
    assert!(source.equals_last(&TestData::default()));
}

#[test]
fn equals_last_returns_false_after_different_data() {
    let (mut source, _sink) = signal::create::<TestData>();
    let first = TestData { value: 42 };
    let second = TestData { value: 99 };

    source.send(&first);
    assert!(source.equals_last(&first));
    assert!(!source.equals_last(&second));
}

#[test]
fn equals_last_works_correctly_with_sends() {
    let (mut source, _sink) = signal::create::<TestData>();
    let data1 = TestData { value: 42 };
    let data2 = TestData { value: 99 };

    // After first send, equals_last should return true for same data
    source.send(&data1);
    assert!(source.equals_last(&data1));
    assert!(!source.equals_last(&data2));

    // After second send, equals_last updates
    source.send(&data2);
    assert!(!source.equals_last(&data1));
    assert!(source.equals_last(&data2));
}

#[test]
fn send_swaps_on_changed_data() {
    let (mut source, sink) = signal::create::<TestData>();

    source.send(&TestData { value: 42 });
    let mut last_value = 0;
    sink.process(&mut |d| last_value = d.value);
    assert_eq!(last_value, 42);

    source.send(&TestData { value: 99 });
    sink.process(&mut |d| last_value = d.value);
    assert_eq!(last_value, 99);
}

#[test]
fn modify_always_swaps_regardless_of_changes() {
    let (mut source, sink) = signal::create::<TestData>();
    let data = TestData { value: 42 };

    source.send(&data);
    let mut process_count = 0;
    sink.process(&mut |_| process_count += 1);
    assert_eq!(process_count, 1);

    // Modify to same value - still swaps
    source.modify(&mut |d| d.value = 42);
    sink.process(&mut |_| process_count += 1);
    assert_eq!(process_count, 2); // modify always swaps
}

// Sync signal tests

#[test]
fn sync_equals_last_returns_true_on_initial_default() {
    let (source, _sink) = signal::sync::create::<TestData>();
    assert!(source.equals_last(&TestData::default()));
}

#[test]
fn sync_equals_last_returns_false_when_receiving() {
    let (mut source, sink) = signal::sync::create::<TestData>();
    let first = TestData { value: 42 };
    let second = TestData { value: 99 };

    // After send, we are in Receiving state - equals_last returns false
    source.send(&first);
    assert!(!source.equals_last(&first));  // Receiving: always false
    assert!(!source.equals_last(&second)); // Receiving: always false

    // After sink acknowledges, we are Ready - equals_last works normally
    sink.process(&mut |_| {});
    assert!(source.equals_last(&first));   // Ready: compares with buffer
    assert!(!source.equals_last(&second)); // Ready: compares with buffer
}

#[test]
fn sync_send_skips_on_unchanged_data() {
    let (mut source, sink) = signal::sync::create::<TestData>();
    let data = TestData { value: 42 };

    use signal::sync::State;

    // First send
    let state = source.send(&data);
    assert!(matches!(state, State::Ready));
    sink.process(&mut |_| {});

    // Second send with same data - should skip
    let state = source.send(&data);
    assert!(matches!(state, State::Ready)); // Returns Ready without swap
}

#[test]
fn sync_send_swaps_on_changed_data() {
    let (mut source, sink) = signal::sync::create::<TestData>();

    source.send(&TestData { value: 42 });
    sink.process(&mut |_| {});

    let mut last_value = 0;
    source.send(&TestData { value: 99 });
    sink.process(&mut |d| last_value = d.value);
    assert_eq!(last_value, 99);
}
