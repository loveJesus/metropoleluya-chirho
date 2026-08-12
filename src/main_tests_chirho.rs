// For God so loved the world, that He gave His only Begotten Son, that whosoever believeth in Him should not perish but have everlasting life. - John 3:16 (KJV)

use super::*;

static TMUX_TEST_SEQUENCE_CHIRHO: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
const TMUX_TEST_STEP_TIMEOUT_CHIRHO: Duration = Duration::from_secs(2);
const TMUX_TEST_HARNESS_TIMEOUT_CHIRHO: Duration = Duration::from_secs(5);

fn send_test_signal_chirho(
    sender_chirho: &std::sync::mpsc::SyncSender<()>,
    label_chirho: &str,
) -> Result<(), String> {
    sender_chirho
        .send(())
        .map_err(|_| format!("{label_chirho} signal receiver disconnected_chirho"))
}

fn wait_for_test_signal_chirho(
    receiver_chirho: &std::sync::mpsc::Receiver<()>,
    timeout_chirho: Duration,
    label_chirho: &str,
) -> Result<(), String> {
    match receiver_chirho.recv_timeout(timeout_chirho) {
        Ok(()) => Ok(()),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(format!(
            "{label_chirho} timed out after {timeout_chirho:?}_chirho"
        )),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            Err(format!("{label_chirho} disconnected_chirho"))
        }
    }
}

fn spawn_tmux_worker_chirho<WorkChirho>(
    result_sender_chirho: std::sync::mpsc::Sender<Result<(), String>>,
    work_chirho: WorkChirho,
) where
    WorkChirho: FnOnce() -> Result<(), String> + Send + 'static,
{
    let _worker_handle_chirho = std::thread::spawn(move || {
        let result_chirho = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work_chirho))
            .unwrap_or_else(|_| Err("tmux test worker panicked_chirho".to_string()));
        let _result_send_chirho = result_sender_chirho.send(result_chirho);
    });
}

fn collect_tmux_worker_results_chirho(
    result_receiver_chirho: &std::sync::mpsc::Receiver<Result<(), String>>,
    expected_count_chirho: usize,
    timeout_chirho: Duration,
) -> Result<(), String> {
    let deadline_chirho = Instant::now()
        .checked_add(timeout_chirho)
        .ok_or_else(|| "tmux test deadline overflowed_chirho".to_string())?;
    for worker_index_chirho in 0..expected_count_chirho {
        let remaining_chirho = deadline_chirho.saturating_duration_since(Instant::now());
        let worker_result_chirho = match result_receiver_chirho.recv_timeout(remaining_chirho) {
            Ok(worker_result_chirho) => worker_result_chirho,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                return Err(format!(
                    "tmux worker result {}/{} timed out after {timeout_chirho:?}_chirho",
                    worker_index_chirho + 1,
                    expected_count_chirho
                ));
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                return Err(format!(
                    "tmux worker result {}/{} disconnected_chirho",
                    worker_index_chirho + 1,
                    expected_count_chirho
                ));
            }
        };
        worker_result_chirho?;
    }
    Ok(())
}

struct IsolatedTmuxServerChirho {
    socket_name_chirho: String,
    session_name_chirho: String,
    first_pane_chirho: String,
}

impl IsolatedTmuxServerChirho {
    fn new_chirho() -> Self {
        let sequence_chirho =
            TMUX_TEST_SEQUENCE_CHIRHO.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let socket_name_chirho = format!(
            "metropoleluya-buffer-test-{}-{sequence_chirho}-chirho",
            std::process::id()
        );
        let session_name_chirho = "delivery-test-chirho".to_string();
        let output_chirho = std::process::Command::new("tmux")
            .args([
                "-L",
                &socket_name_chirho,
                "new-session",
                "-d",
                "-P",
                "-F",
                "#{pane_id}",
                "-x",
                "160",
                "-y",
                "40",
                "-s",
                &session_name_chirho,
                "-n",
                "legacy-a-chirho",
                "cat",
            ])
            .output()
            .unwrap();
        assert!(
            output_chirho.status.success(),
            "isolated tmux server failed: {}",
            String::from_utf8_lossy(&output_chirho.stderr)
        );
        let first_pane_chirho = String::from_utf8(output_chirho.stdout)
            .unwrap()
            .trim()
            .to_string();
        assert!(!first_pane_chirho.is_empty());
        std::thread::sleep(Duration::from_millis(50));
        Self {
            socket_name_chirho,
            session_name_chirho,
            first_pane_chirho,
        }
    }

    fn add_pane_chirho(&self, window_name_chirho: &str) -> String {
        let output_chirho = std::process::Command::new("tmux")
            .args([
                "-L",
                &self.socket_name_chirho,
                "new-window",
                "-d",
                "-P",
                "-F",
                "#{pane_id}",
                "-t",
                &self.session_name_chirho,
                "-n",
                window_name_chirho,
                "cat",
            ])
            .output()
            .unwrap();
        assert!(
            output_chirho.status.success(),
            "isolated tmux pane failed: {}",
            String::from_utf8_lossy(&output_chirho.stderr)
        );
        String::from_utf8(output_chirho.stdout)
            .unwrap()
            .trim()
            .to_string()
    }

    fn capture_pane_chirho(&self, pane_chirho: &str) -> String {
        let output_chirho = std::process::Command::new("tmux")
            .args([
                "-L",
                &self.socket_name_chirho,
                "capture-pane",
                "-p",
                "-t",
                pane_chirho,
            ])
            .output()
            .unwrap();
        assert!(
            output_chirho.status.success(),
            "isolated tmux capture failed: {}",
            String::from_utf8_lossy(&output_chirho.stderr)
        );
        String::from_utf8(output_chirho.stdout).unwrap()
    }

    fn send_enter_chirho(&self, pane_chirho: &str) {
        let output_chirho = std::process::Command::new("tmux")
            .args([
                "-L",
                &self.socket_name_chirho,
                "send-keys",
                "-t",
                pane_chirho,
                "Enter",
            ])
            .output()
            .unwrap();
        assert!(
            output_chirho.status.success(),
            "isolated tmux send-keys failed: {}",
            String::from_utf8_lossy(&output_chirho.stderr)
        );
    }

    fn window_index_chirho(&self, pane_chirho: &str) -> String {
        let output_chirho = std::process::Command::new("tmux")
            .args([
                "-L",
                &self.socket_name_chirho,
                "display-message",
                "-p",
                "-t",
                pane_chirho,
                "#{window_index}",
            ])
            .output()
            .unwrap();
        assert!(
            output_chirho.status.success(),
            "isolated tmux window lookup failed: {}",
            String::from_utf8_lossy(&output_chirho.stderr)
        );
        String::from_utf8(output_chirho.stdout)
            .unwrap()
            .trim()
            .to_string()
    }

    fn buffer_exists_chirho(&self, buffer_name_chirho: &str) -> bool {
        std::process::Command::new("tmux")
            .args([
                "-L",
                &self.socket_name_chirho,
                "show-buffer",
                "-b",
                buffer_name_chirho,
            ])
            .output()
            .is_ok_and(|output_chirho| output_chirho.status.success())
    }
}

impl Drop for IsolatedTmuxServerChirho {
    fn drop(&mut self) {
        let _cleanup_result_chirho = std::process::Command::new("tmux")
            .args(["-L", &self.socket_name_chirho, "kill-server"])
            .output();
    }
}

#[test]
fn concurrent_tmux_buffers_isolate_targets_and_expose_legacy_failure_chirho() {
    let server_chirho = IsolatedTmuxServerChirho::new_chirho();
    let legacy_pane_a_chirho = &server_chirho.first_pane_chirho;
    let legacy_pane_b_chirho = server_chirho.add_pane_chirho("legacy-b-chirho");
    let fixed_pane_a_chirho = server_chirho.add_pane_chirho("fixed-a-chirho");
    let fixed_pane_b_chirho = server_chirho.add_pane_chirho("fixed-b-chirho");
    let legacy_payload_a_chirho = "legacy-distinct-a-payload-chirho";
    let legacy_payload_b_chirho = "legacy-distinct-b-payload-chirho";
    let legacy_buffer_name_chirho = "metropoleluya-chirho";

    // Plant the old construction deterministically: A loads the one shared
    // buffer, B overwrites it, and only then may either target paste it.
    let (legacy_a_loaded_sender_chirho, legacy_a_loaded_receiver_chirho) =
        std::sync::mpsc::sync_channel(1);
    let (legacy_b_loaded_sender_chirho, legacy_b_loaded_receiver_chirho) =
        std::sync::mpsc::sync_channel(1);
    let (legacy_result_sender_chirho, legacy_result_receiver_chirho) = std::sync::mpsc::channel();
    let socket_for_a_chirho = server_chirho.socket_name_chirho.clone();
    let pane_for_a_chirho = legacy_pane_a_chirho.clone();
    spawn_tmux_worker_chirho(legacy_result_sender_chirho.clone(), move || {
        tmux_transport_chirho::paste_tmux_buffer_for_test_chirho(
            &socket_for_a_chirho,
            &pane_for_a_chirho,
            legacy_payload_a_chirho,
            legacy_buffer_name_chirho,
            false,
            move || {
                send_test_signal_chirho(&legacy_a_loaded_sender_chirho, "legacy A loaded_chirho")?;
                wait_for_test_signal_chirho(
                    &legacy_b_loaded_receiver_chirho,
                    TMUX_TEST_STEP_TIMEOUT_CHIRHO,
                    "legacy B load_chirho",
                )
            },
        )
    });
    let socket_for_b_chirho = server_chirho.socket_name_chirho.clone();
    let pane_for_b_chirho = legacy_pane_b_chirho.clone();
    spawn_tmux_worker_chirho(legacy_result_sender_chirho.clone(), move || {
        wait_for_test_signal_chirho(
            &legacy_a_loaded_receiver_chirho,
            TMUX_TEST_STEP_TIMEOUT_CHIRHO,
            "legacy A load_chirho",
        )?;
        tmux_transport_chirho::paste_tmux_buffer_for_test_chirho(
            &socket_for_b_chirho,
            &pane_for_b_chirho,
            legacy_payload_b_chirho,
            legacy_buffer_name_chirho,
            false,
            move || {
                send_test_signal_chirho(&legacy_b_loaded_sender_chirho, "legacy B loaded_chirho")
            },
        )
    });
    drop(legacy_result_sender_chirho);
    collect_tmux_worker_results_chirho(
        &legacy_result_receiver_chirho,
        2,
        TMUX_TEST_HARNESS_TIMEOUT_CHIRHO,
    )
    .unwrap();
    let legacy_capture_a_chirho = server_chirho.capture_pane_chirho(legacy_pane_a_chirho);
    assert!(legacy_capture_a_chirho.contains(legacy_payload_b_chirho));
    assert!(!legacy_capture_a_chirho.contains(legacy_payload_a_chirho));

    // The repaired construction reaches the same load/load/paste interleaving,
    // but each target owns a distinct buffer which paste-buffer deletes.
    let fixed_payload_a_chirho = "fixed-distinct-a-payload-chirho";
    let fixed_payload_b_chirho = "fixed-distinct-b-payload-chirho";
    let fixed_buffer_a_chirho = tmux_transport_chirho::next_tmux_buffer_name_chirho();
    let fixed_buffer_b_chirho = tmux_transport_chirho::next_tmux_buffer_name_chirho();
    assert_ne!(fixed_buffer_a_chirho, fixed_buffer_b_chirho);
    let (fixed_a_loaded_sender_chirho, fixed_a_loaded_receiver_chirho) =
        std::sync::mpsc::sync_channel(1);
    let (fixed_b_loaded_sender_chirho, fixed_b_loaded_receiver_chirho) =
        std::sync::mpsc::sync_channel(1);
    let (fixed_result_sender_chirho, fixed_result_receiver_chirho) = std::sync::mpsc::channel();
    let socket_for_a_chirho = server_chirho.socket_name_chirho.clone();
    let pane_for_a_chirho = fixed_pane_a_chirho.clone();
    let buffer_for_a_chirho = fixed_buffer_a_chirho.clone();
    spawn_tmux_worker_chirho(fixed_result_sender_chirho.clone(), move || {
        tmux_transport_chirho::paste_tmux_buffer_for_test_chirho(
            &socket_for_a_chirho,
            &pane_for_a_chirho,
            fixed_payload_a_chirho,
            &buffer_for_a_chirho,
            true,
            move || {
                send_test_signal_chirho(&fixed_a_loaded_sender_chirho, "fixed A loaded_chirho")?;
                wait_for_test_signal_chirho(
                    &fixed_b_loaded_receiver_chirho,
                    TMUX_TEST_STEP_TIMEOUT_CHIRHO,
                    "fixed B load_chirho",
                )
            },
        )
    });
    let socket_for_b_chirho = server_chirho.socket_name_chirho.clone();
    let pane_for_b_chirho = fixed_pane_b_chirho.clone();
    let buffer_for_b_chirho = fixed_buffer_b_chirho.clone();
    spawn_tmux_worker_chirho(fixed_result_sender_chirho.clone(), move || {
        tmux_transport_chirho::paste_tmux_buffer_for_test_chirho(
            &socket_for_b_chirho,
            &pane_for_b_chirho,
            fixed_payload_b_chirho,
            &buffer_for_b_chirho,
            true,
            move || {
                send_test_signal_chirho(&fixed_b_loaded_sender_chirho, "fixed B loaded_chirho")?;
                wait_for_test_signal_chirho(
                    &fixed_a_loaded_receiver_chirho,
                    TMUX_TEST_STEP_TIMEOUT_CHIRHO,
                    "fixed A load_chirho",
                )
            },
        )
    });
    drop(fixed_result_sender_chirho);
    collect_tmux_worker_results_chirho(
        &fixed_result_receiver_chirho,
        2,
        TMUX_TEST_HARNESS_TIMEOUT_CHIRHO,
    )
    .unwrap();
    let fixed_capture_a_chirho = server_chirho.capture_pane_chirho(&fixed_pane_a_chirho);
    let fixed_capture_b_chirho = server_chirho.capture_pane_chirho(&fixed_pane_b_chirho);
    assert!(fixed_capture_a_chirho.contains(fixed_payload_a_chirho));
    assert!(!fixed_capture_a_chirho.contains(fixed_payload_b_chirho));
    assert!(fixed_capture_b_chirho.contains(fixed_payload_b_chirho));
    assert!(!fixed_capture_b_chirho.contains(fixed_payload_a_chirho));
    assert!(!server_chirho.buffer_exists_chirho(&fixed_buffer_a_chirho));
    assert!(!server_chirho.buffer_exists_chirho(&fixed_buffer_b_chirho));

    let failed_buffer_chirho = tmux_transport_chirho::next_tmux_buffer_name_chirho();
    assert!(tmux_transport_chirho::paste_tmux_buffer_for_test_chirho(
        &server_chirho.socket_name_chirho,
        "missing-target-chirho:997",
        "failed-paste-payload-chirho",
        &failed_buffer_chirho,
        true,
        || Ok(()),
    )
    .is_err());
    assert!(!server_chirho.buffer_exists_chirho(&failed_buffer_chirho));
}

#[test]
fn same_pane_transactions_remain_distinct_and_expose_unlocked_failure_chirho() {
    let server_chirho = IsolatedTmuxServerChirho::new_chirho();
    let unlocked_pane_chirho = server_chirho.first_pane_chirho.clone();
    let fixed_pane_chirho = server_chirho.add_pane_chirho("same-pane-fixed-chirho");
    let unlocked_payload_a_chirho = "unlocked-same-pane-a-chirho";
    let unlocked_payload_b_chirho = "unlocked-same-pane-b-chirho";
    let unlocked_buffer_a_chirho = tmux_transport_chirho::next_tmux_buffer_name_chirho();
    let unlocked_buffer_b_chirho = tmux_transport_chirho::next_tmux_buffer_name_chirho();
    let (unlocked_result_sender_chirho, unlocked_result_receiver_chirho) =
        std::sync::mpsc::channel();

    // Plant the old unlocked transaction: two distinct buffers paste into the
    // same draft before either delivery gets to send its first Enter.
    let socket_for_a_chirho = server_chirho.socket_name_chirho.clone();
    let pane_for_a_chirho = unlocked_pane_chirho.clone();
    spawn_tmux_worker_chirho(unlocked_result_sender_chirho.clone(), move || {
        tmux_transport_chirho::paste_tmux_buffer_for_test_chirho(
            &socket_for_a_chirho,
            &pane_for_a_chirho,
            unlocked_payload_a_chirho,
            &unlocked_buffer_a_chirho,
            true,
            || Ok(()),
        )
    });
    let socket_for_b_chirho = server_chirho.socket_name_chirho.clone();
    let pane_for_b_chirho = unlocked_pane_chirho.clone();
    spawn_tmux_worker_chirho(unlocked_result_sender_chirho.clone(), move || {
        tmux_transport_chirho::paste_tmux_buffer_for_test_chirho(
            &socket_for_b_chirho,
            &pane_for_b_chirho,
            unlocked_payload_b_chirho,
            &unlocked_buffer_b_chirho,
            true,
            || Ok(()),
        )
    });
    drop(unlocked_result_sender_chirho);
    collect_tmux_worker_results_chirho(
        &unlocked_result_receiver_chirho,
        2,
        TMUX_TEST_HARNESS_TIMEOUT_CHIRHO,
    )
    .unwrap();
    server_chirho.send_enter_chirho(&unlocked_pane_chirho);
    server_chirho.send_enter_chirho(&unlocked_pane_chirho);
    std::thread::sleep(Duration::from_millis(25));
    let unlocked_capture_chirho = server_chirho.capture_pane_chirho(&unlocked_pane_chirho);
    assert!(unlocked_capture_chirho.lines().any(|line_chirho| {
        line_chirho.contains(unlocked_payload_a_chirho)
            && line_chirho.contains(unlocked_payload_b_chirho)
    }));

    // The repaired full delivery calls run concurrently, but their physical
    // pane lease covers paste, first Enter, settle delay, and second Enter.
    let fixed_payload_a_chirho = "fixed-same-pane-a-chirho";
    let fixed_payload_b_chirho = "fixed-same-pane-b-chirho";
    let (fixed_result_sender_chirho, fixed_result_receiver_chirho) = std::sync::mpsc::channel();
    let socket_for_a_chirho = server_chirho.socket_name_chirho.clone();
    let pane_for_a_chirho = fixed_pane_chirho.clone();
    spawn_tmux_worker_chirho(fixed_result_sender_chirho.clone(), move || {
        tmux_transport_chirho::send_tmux_message_for_test_chirho(
            &socket_for_a_chirho,
            &pane_for_a_chirho,
            fixed_payload_a_chirho,
            Duration::from_millis(25),
        )
    });
    let socket_for_b_chirho = server_chirho.socket_name_chirho.clone();
    let pane_for_b_chirho = format!(
        "{}:same-pane-fixed-chirho",
        server_chirho.session_name_chirho
    );
    spawn_tmux_worker_chirho(fixed_result_sender_chirho.clone(), move || {
        tmux_transport_chirho::send_tmux_message_for_test_chirho(
            &socket_for_b_chirho,
            &pane_for_b_chirho,
            fixed_payload_b_chirho,
            Duration::from_millis(25),
        )
    });
    drop(fixed_result_sender_chirho);
    collect_tmux_worker_results_chirho(
        &fixed_result_receiver_chirho,
        2,
        TMUX_TEST_HARNESS_TIMEOUT_CHIRHO,
    )
    .unwrap();
    std::thread::sleep(Duration::from_millis(25));
    let fixed_capture_chirho = server_chirho.capture_pane_chirho(&fixed_pane_chirho);
    assert!(fixed_capture_chirho
        .lines()
        .any(|line_chirho| line_chirho.trim() == fixed_payload_a_chirho));
    assert!(fixed_capture_chirho
        .lines()
        .any(|line_chirho| line_chirho.trim() == fixed_payload_b_chirho));
    assert!(!fixed_capture_chirho.lines().any(|line_chirho| {
        line_chirho.contains(fixed_payload_a_chirho) && line_chirho.contains(fixed_payload_b_chirho)
    }));
    assert_eq!(
        tmux_transport_chirho::target_delivery_lock_count_for_test_chirho(
            &server_chirho.socket_name_chirho
        ),
        0
    );
}

#[test]
fn unrelated_pane_locks_remain_concurrent_and_retire_chirho() {
    let server_chirho = IsolatedTmuxServerChirho::new_chirho();
    let pane_a_chirho = server_chirho.first_pane_chirho.clone();
    let pane_b_chirho = server_chirho.add_pane_chirho("unrelated-lock-b-chirho");
    let (a_locked_sender_chirho, a_locked_receiver_chirho) = std::sync::mpsc::sync_channel(1);
    let (b_locked_sender_chirho, b_locked_receiver_chirho) = std::sync::mpsc::sync_channel(1);
    let (result_sender_chirho, result_receiver_chirho) = std::sync::mpsc::channel();

    let socket_for_a_chirho = server_chirho.socket_name_chirho.clone();
    spawn_tmux_worker_chirho(result_sender_chirho.clone(), move || {
        tmux_transport_chirho::hold_tmux_target_lock_for_test_chirho(
            &socket_for_a_chirho,
            &pane_a_chirho,
            move || {
                send_test_signal_chirho(&a_locked_sender_chirho, "pane A locked_chirho")?;
                wait_for_test_signal_chirho(
                    &b_locked_receiver_chirho,
                    TMUX_TEST_STEP_TIMEOUT_CHIRHO,
                    "unrelated pane B lock_chirho",
                )
            },
        )
    });
    let socket_for_b_chirho = server_chirho.socket_name_chirho.clone();
    spawn_tmux_worker_chirho(result_sender_chirho.clone(), move || {
        wait_for_test_signal_chirho(
            &a_locked_receiver_chirho,
            TMUX_TEST_STEP_TIMEOUT_CHIRHO,
            "pane A lock_chirho",
        )?;
        tmux_transport_chirho::hold_tmux_target_lock_for_test_chirho(
            &socket_for_b_chirho,
            &pane_b_chirho,
            move || send_test_signal_chirho(&b_locked_sender_chirho, "pane B locked_chirho"),
        )
    });
    drop(result_sender_chirho);
    collect_tmux_worker_results_chirho(
        &result_receiver_chirho,
        2,
        TMUX_TEST_HARNESS_TIMEOUT_CHIRHO,
    )
    .unwrap();
    assert_eq!(
        tmux_transport_chirho::target_delivery_lock_count_for_test_chirho(
            &server_chirho.socket_name_chirho
        ),
        0
    );
}

#[test]
fn timeout_gate_and_worker_collector_reject_missing_participants_chirho() {
    let (missing_signal_sender_chirho, missing_signal_receiver_chirho) =
        std::sync::mpsc::sync_channel(1);
    let signal_start_chirho = Instant::now();
    let signal_error_chirho = wait_for_test_signal_chirho(
        &missing_signal_receiver_chirho,
        Duration::from_millis(25),
        "planted missing gate peer_chirho",
    )
    .unwrap_err();
    assert!(signal_error_chirho.contains("timed out"));
    assert!(signal_start_chirho.elapsed() < Duration::from_secs(1));
    drop(missing_signal_sender_chirho);

    let (missing_result_sender_chirho, missing_result_receiver_chirho) = std::sync::mpsc::channel();
    missing_result_sender_chirho.send(Ok(())).unwrap();
    let result_start_chirho = Instant::now();
    let result_error_chirho = collect_tmux_worker_results_chirho(
        &missing_result_receiver_chirho,
        2,
        Duration::from_millis(25),
    )
    .unwrap_err();
    assert!(result_error_chirho.contains("timed out"));
    assert!(result_start_chirho.elapsed() < Duration::from_secs(1));
    drop(missing_result_sender_chirho);
}

#[test]
fn bind_listener_binds_and_rebinds_chirho() {
    // socket2 path binds an ephemeral port and, after drop, rebinds the same
    // port — the mechanic SO_REUSEADDR guarantees across a broker restart.
    let listener_chirho = bind_listener_chirho("127.0.0.1:0").unwrap();
    let addr_chirho = listener_chirho.local_addr().unwrap();
    drop(listener_chirho);
    assert!(bind_listener_chirho(&addr_chirho.to_string()).is_ok());
}

#[test]
fn deliver_parallel_handles_empty_targets_chirho() {
    // No subscribers -> no work, no panic, no threads spawned.
    assert!(deliver_parallel_chirho(&[], "body-chirho").is_empty());
}

#[test]
fn tmux_probe_rejects_empty_success_for_missing_target_chirho() {
    let probe_chirho = probe_tmux_target_chirho("__METRO_SMOKE_MISSING_CHIRHO__:997");
    assert!(!probe_chirho.alive_chirho);
    assert!(probe_chirho.session_chirho.is_none());
    assert!(probe_chirho.error_chirho.is_some());
}

#[test]
fn supervisor_backs_off_then_gives_up_chirho() {
    let mut crashes_chirho = 0u32;
    // Rapid crashes (ran < 10s) grow the backoff: 2s, 4s, 8s, 16s...
    for expected_secs_chirho in [2u64, 4, 8, 16] {
        assert_eq!(
            next_supervisor_action_chirho(Duration::from_secs(1), &mut crashes_chirho),
            SupervisorActionChirho::RestartAfterChirho(Duration::from_secs(expected_secs_chirho))
        );
    }
    // ...then give up rather than restart-bomb.
    assert_eq!(
        next_supervisor_action_chirho(Duration::from_secs(1), &mut crashes_chirho),
        SupervisorActionChirho::GiveUpChirho
    );
}

#[test]
fn supervisor_resets_after_healthy_run_chirho() {
    let mut crashes_chirho = 4u32; // on the brink of giving up
                                   // A healthy run (>= 10s) clears the counter and restarts promptly.
    assert_eq!(
        next_supervisor_action_chirho(Duration::from_secs(30), &mut crashes_chirho),
        SupervisorActionChirho::RestartAfterChirho(Duration::from_secs(1))
    );
    assert_eq!(crashes_chirho, 0);
}

#[test]
fn identity_uses_session_slash_agent_chirho() {
    assert_eq!(
        identity_chirho("PROJECT_CHIRHO", "gpt_chirho"),
        "PROJECT_CHIRHO/gpt_chirho"
    );
}

#[test]
fn percent_encoding_round_trips_room_names_chirho() {
    let value_chirho = "project chirho/topic";
    let encoded_chirho = percent_encode_chirho(value_chirho);
    assert_eq!(
        percent_decode_chirho(&encoded_chirho).unwrap(),
        value_chirho
    );
}

#[test]
fn timestamp_formatter_uses_eastern_text_chirho() {
    // Storage is epoch UTC; display localizes to America/New_York (DST-correct).
    // Epoch 0 = 1970-01-01T00:00Z, which is EST (UTC-5) in New York.
    assert_eq!(format_timestamp_chirho(0), "1969-12-31 19:00:00.000 EST");
    // Winter instant -> EST (-5).
    assert_eq!(
        format_timestamp_chirho(1_704_067_200_123),
        "2023-12-31 19:00:00.123 EST"
    );
    // Summer instant -> EDT (-4): proves DST handling, not a hard-coded offset.
    assert_eq!(
        format_timestamp_chirho(1_719_792_000_000),
        "2024-06-30 20:00:00.000 EDT"
    );
}

#[test]
fn body_validation_allows_multiline_agent_messages_chirho() {
    validate_body_chirho("PROJECT_CHIRHO/GPT SENDS: line one\n\nDetails line two.").unwrap();
    assert!(validate_body_chirho("").is_err());
    assert!(validate_body_chirho("bad\0body").is_err());
}

#[test]
fn authorization_errors_use_forbidden_http_status_chirho() {
    assert_eq!(
        channels_chirho::http_error_status_chirho("authorization denied: private room"),
        403
    );
    assert_eq!(
        channels_chirho::http_error_status_chirho("unknown route"),
        404
    );
    assert_eq!(
        channels_chirho::http_error_status_chirho("room_chirho is required"),
        400
    );
}

#[test]
fn in_memory_db_registers_agent_and_room_chirho() {
    let conn_chirho = Connection::open_in_memory().unwrap();
    init_db_chirho(&conn_chirho).unwrap();
    let response_chirho = register_agent_chirho(
        &conn_chirho,
        RegisterRequestChirho {
            session_chirho: "TEST_CHIRHO".to_string(),
            agent_chirho: "gpt_chirho".to_string(),
            tmux_target_chirho: "missing-session-chirho:1".to_string(),
            rooms_chirho: vec!["room-chirho".to_string()],
            topics_chirho: vec!["audit-chirho".to_string()],
            notify_actor_chirho: None,
            private_chirho: false,
            ttl_seconds_chirho: None,
        },
    )
    .unwrap();
    assert_eq!(response_chirho["identity_chirho"], "TEST_CHIRHO/gpt_chirho");
    let agents_chirho = list_agents_chirho(&conn_chirho, Some("room-chirho")).unwrap();
    assert_eq!(agents_chirho["agents_chirho"].as_array().unwrap().len(), 1);
}

#[test]
fn request_connections_skip_migration_and_use_busy_timeout_chirho() {
    let db_path_chirho = std::env::temp_dir().join(format!(
        "metropoleluya-request-connection-{}-{}-chirho.sqlite",
        std::process::id(),
        now_ms_chirho()
    ));
    drop(open_db_chirho(Some(db_path_chirho.clone())).unwrap());
    let conn_chirho = open_request_db_chirho(Some(db_path_chirho.clone())).unwrap();
    let busy_timeout_ms_chirho: i64 = conn_chirho
        .query_row("pragma busy_timeout", [], |row_chirho| row_chirho.get(0))
        .unwrap();
    assert_eq!(busy_timeout_ms_chirho, 5_000);
    let schema_count_chirho: i64 = conn_chirho
        .query_row(
            "select count(*) from sqlite_master where type = 'table' and name = 'agents_chirho'",
            [],
            |row_chirho| row_chirho.get(0),
        )
        .unwrap();
    assert_eq!(schema_count_chirho, 1);
    drop(conn_chirho);
    std::fs::remove_file(db_path_chirho).unwrap();
}

#[test]
fn sqlite_messages_view_exposes_readable_timestamp_chirho() {
    let conn_chirho = Connection::open_in_memory().unwrap();
    init_db_chirho(&conn_chirho).unwrap();
    conn_chirho
        .execute(
            r#"
            insert into messages_chirho (
                at_ms_chirho, from_identity_chirho, room_chirho, topic_chirho, body_chirho
            ) values (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                1_704_067_200_123_i64,
                "PROJECT_CHIRHO/gpt_chirho",
                "project-chirho",
                "audit-chirho",
                "body-chirho"
            ],
        )
        .unwrap();
    let at_text_chirho: String = conn_chirho
        .query_row(
            "select at_text_chirho from messages_with_time_chirho where id_chirho = 1",
            [],
            |row_chirho| row_chirho.get(0),
        )
        .unwrap();
    // The SQL view keeps canonical UTC (Z-suffixed) for raw DB inspection.
    assert_eq!(at_text_chirho, "2024-01-01 00:00:00.123Z");
    // The API/TUI display path localizes the same instant to America/New_York.
    let listed_chirho =
        channels_chirho::list_room_messages_chirho(&conn_chirho, "project-chirho", 0, None)
            .unwrap();
    assert_eq!(
        listed_chirho["messages_chirho"][0]["at_text_chirho"],
        "2023-12-31 19:00:00.123 EST"
    );
}

#[test]
fn remove_agent_deletes_only_targeted_room_chirho() {
    let conn_chirho = Connection::open_in_memory().unwrap();
    init_db_chirho(&conn_chirho).unwrap();
    register_agent_chirho(
        &conn_chirho,
        RegisterRequestChirho {
            session_chirho: "TEST_CHIRHO".to_string(),
            agent_chirho: "gpt_chirho".to_string(),
            tmux_target_chirho: "missing-session-chirho:1".to_string(),
            rooms_chirho: vec!["room-a-chirho".to_string(), "room-b-chirho".to_string()],
            topics_chirho: vec![],
            notify_actor_chirho: None,
            private_chirho: false,
            ttl_seconds_chirho: None,
        },
    )
    .unwrap();
    let response_chirho = remove_agent_chirho(
        &conn_chirho,
        RemoveRequestChirho {
            from_session_chirho: "TEST_CHIRHO".to_string(),
            from_agent_chirho: "operator_chirho".to_string(),
            session_chirho: "TEST_CHIRHO".to_string(),
            agent_chirho: "gpt_chirho".to_string(),
            room_chirho: "room-a-chirho".to_string(),
        },
    )
    .unwrap();
    assert_eq!(response_chirho["ok_chirho"], true);
    assert_eq!(response_chirho["removed_count_chirho"], 1);
    assert_eq!(response_chirho["notified_chirho"], false);
    let room_a_count_chirho: i64 = conn_chirho
        .query_row(
            "select count(*) from subscriptions_chirho where identity_chirho = ?1 and room_chirho = ?2",
            params!["TEST_CHIRHO/gpt_chirho", "room-a-chirho"],
            |row_chirho| row_chirho.get(0),
        )
        .unwrap();
    assert_eq!(room_a_count_chirho, 0);
    let room_b_count_chirho: i64 = conn_chirho
        .query_row(
            "select count(*) from subscriptions_chirho where identity_chirho = ?1 and room_chirho = ?2",
            params!["TEST_CHIRHO/gpt_chirho", "room-b-chirho"],
            |row_chirho| row_chirho.get(0),
        )
        .unwrap();
    assert_eq!(room_b_count_chirho, 1);
    let agent_rows_chirho: i64 = conn_chirho
        .query_row(
            "select count(*) from agents_chirho where identity_chirho = ?1",
            params!["TEST_CHIRHO/gpt_chirho"],
            |row_chirho| row_chirho.get(0),
        )
        .unwrap();
    assert_eq!(agent_rows_chirho, 1);
}

#[test]
fn remove_of_non_member_is_noop_chirho() {
    let conn_chirho = Connection::open_in_memory().unwrap();
    init_db_chirho(&conn_chirho).unwrap();
    let response_chirho = remove_agent_chirho(
        &conn_chirho,
        RemoveRequestChirho {
            from_session_chirho: "TEST_CHIRHO".to_string(),
            from_agent_chirho: "operator_chirho".to_string(),
            session_chirho: "TEST_CHIRHO".to_string(),
            agent_chirho: "ghost_chirho".to_string(),
            room_chirho: "room-a-chirho".to_string(),
        },
    )
    .unwrap();
    assert_eq!(response_chirho["ok_chirho"], true);
    assert_eq!(response_chirho["removed_count_chirho"], 0);
    assert_eq!(response_chirho["notified_chirho"], false);
}

#[test]
fn register_notify_actor_reports_unnotified_for_dead_pane_chirho() {
    let conn_chirho = Connection::open_in_memory().unwrap();
    init_db_chirho(&conn_chirho).unwrap();
    let response_chirho = register_agent_chirho(
        &conn_chirho,
        RegisterRequestChirho {
            session_chirho: "TEST_CHIRHO".to_string(),
            agent_chirho: "gpt_chirho".to_string(),
            tmux_target_chirho: "missing-session-chirho:1".to_string(),
            rooms_chirho: vec!["room-chirho".to_string()],
            topics_chirho: vec![],
            notify_actor_chirho: Some("TEST_CHIRHO/operator_chirho".to_string()),
            private_chirho: false,
            ttl_seconds_chirho: None,
        },
    )
    .unwrap();
    assert_eq!(response_chirho["notified_chirho"], false);
}

/// A registration whose window index no longer exists must never deliver.
/// `tmux display-message -t SESSION:97` answers with the session's *current*
/// window and exits 0, so resolving a target through it used to paste a
/// private notice into whichever pane the operator happened to be looking at:
/// removing one agent appeared to notify the whole room. Delivery now proves
/// the pane it resolved really is the registered window, and refuses otherwise.
#[test]
fn stale_window_target_refuses_delivery_instead_of_hitting_current_window_chirho() {
    let server_chirho = IsolatedTmuxServerChirho::new_chirho();
    let live_pane_chirho = server_chirho.first_pane_chirho.clone();
    let live_index_chirho = server_chirho.window_index_chirho(&live_pane_chirho);
    let stale_target_chirho = format!("{}:97", server_chirho.session_name_chirho);
    let socket_args_chirho = ["-L", server_chirho.socket_name_chirho.as_str()];
    let stale_payload_chirho = "stale-target-must-not-arrive-chirho";
    let live_payload_chirho = "live-target-must-arrive-chirho";

    // tmux performs the silent fallback this guard exists to catch.
    let fallback_probe_chirho =
        tmux_transport_chirho::probe_tmux_target_on_server_chirho(&socket_args_chirho, "invalid");
    assert!(
        !fallback_probe_chirho.alive_chirho,
        "a target tmux cannot resolve must not read as alive"
    );
    let stale_probe_chirho = tmux_transport_chirho::probe_tmux_target_on_server_chirho(
        &socket_args_chirho,
        &stale_target_chirho,
    );
    assert!(
        !stale_probe_chirho.alive_chirho,
        "a missing window index must not read as alive"
    );
    assert!(
        stale_probe_chirho.pane_id_chirho.is_none(),
        "a missing window index must not surrender a pane to deliver into"
    );

    let stale_result_chirho = tmux_transport_chirho::send_tmux_message_for_test_chirho(
        &server_chirho.socket_name_chirho,
        &stale_target_chirho,
        stale_payload_chirho,
        Duration::from_millis(50),
    );
    assert!(
        stale_result_chirho.is_err(),
        "delivery to a missing window must fail loudly, not fall back"
    );

    // The registered window still delivers, so the guard costs no reachability.
    let live_target_chirho = format!("{}:{live_index_chirho}", server_chirho.session_name_chirho);
    tmux_transport_chirho::send_tmux_message_for_test_chirho(
        &server_chirho.socket_name_chirho,
        &live_target_chirho,
        live_payload_chirho,
        Duration::from_millis(50),
    )
    .expect("the registered window must still receive deliveries");

    std::thread::sleep(Duration::from_millis(200));
    let transcript_chirho = server_chirho.capture_pane_chirho(&live_pane_chirho);
    assert!(
        !transcript_chirho.contains(stale_payload_chirho),
        "stale target leaked into the session's current window: {transcript_chirho}"
    );
    assert!(
        transcript_chirho.contains(live_payload_chirho),
        "registered window lost its delivery: {transcript_chirho}"
    );
}
