// For God so loved the world, that He gave His only Begotten Son, that whosoever believeth in Him should not perish but have everlasting life. - John 3:16 (KJV)

use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex, Weak};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const BUFFER_NAME_STEM_CHIRHO: &str = "metropoleluya-delivery";
static BUFFER_SEQUENCE_CHIRHO: AtomicU64 = AtomicU64::new(0);
static TARGET_DELIVERY_LOCKS_CHIRHO: LazyLock<Mutex<HashMap<String, Weak<Mutex<()>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

struct TargetDeliveryLeaseChirho {
    target_key_chirho: String,
    target_lock_chirho: Arc<Mutex<()>>,
}

impl Drop for TargetDeliveryLeaseChirho {
    fn drop(&mut self) {
        let mut registry_chirho = TARGET_DELIVERY_LOCKS_CHIRHO
            .lock()
            .unwrap_or_else(|poisoned_chirho| poisoned_chirho.into_inner());
        let own_weak_chirho = Arc::downgrade(&self.target_lock_chirho);
        let owns_entry_chirho =
            registry_chirho
                .get(&self.target_key_chirho)
                .is_some_and(|registered_weak_chirho| {
                    Weak::ptr_eq(registered_weak_chirho, &own_weak_chirho)
                });
        if owns_entry_chirho && Arc::strong_count(&self.target_lock_chirho) == 1 {
            registry_chirho.remove(&self.target_key_chirho);
        }
    }
}

fn acquire_target_delivery_lease_chirho(target_key_chirho: String) -> TargetDeliveryLeaseChirho {
    let mut registry_chirho = TARGET_DELIVERY_LOCKS_CHIRHO
        .lock()
        .unwrap_or_else(|poisoned_chirho| poisoned_chirho.into_inner());
    let target_lock_chirho = match registry_chirho
        .get(&target_key_chirho)
        .and_then(Weak::upgrade)
    {
        Some(target_lock_chirho) => target_lock_chirho,
        None => {
            let target_lock_chirho = Arc::new(Mutex::new(()));
            registry_chirho.insert(
                target_key_chirho.clone(),
                Arc::downgrade(&target_lock_chirho),
            );
            target_lock_chirho
        }
    };
    drop(registry_chirho);
    TargetDeliveryLeaseChirho {
        target_key_chirho,
        target_lock_chirho,
    }
}

#[derive(Debug)]
pub(crate) struct TmuxProbeChirho {
    pub(crate) alive_chirho: bool,
    pub(crate) session_chirho: Option<String>,
    pub(crate) window_index_chirho: Option<String>,
    pub(crate) pane_id_chirho: Option<String>,
    pub(crate) window_name_chirho: Option<String>,
    pub(crate) error_chirho: Option<String>,
}

/// The window half of a `SESSION:WINDOW` target, with any `.pane` suffix and
/// `=` exact-match prefix removed. `None` means the target names no window (a
/// bare session, or a tmux id such as `%12` / `@3` / `$1`), so there is nothing
/// structural to verify beyond what tmux itself enforces.
fn requested_window_chirho(target_chirho: &str) -> Option<(&str, &str)> {
    let target_chirho = target_chirho.strip_prefix('=').unwrap_or(target_chirho);
    if target_chirho.starts_with(['%', '@', '$']) {
        return None;
    }
    let (session_chirho, window_chirho) = target_chirho.split_once(':')?;
    let window_chirho = window_chirho
        .split_once('.')
        .map_or(window_chirho, |(window_chirho, _pane_chirho)| window_chirho);
    (!window_chirho.is_empty()).then_some((session_chirho, window_chirho))
}

/// tmux resolves `SESSION:INDEX` for an index that does not exist by silently
/// falling back to the session's *current* window and exiting 0. A stale
/// registration then delivers to whichever pane the operator happens to be
/// looking at, which reads as "everyone got the message". Every resolution
/// therefore proves tmux returned the window we actually asked for.
fn probe_matches_request_chirho(target_chirho: &str, probe_chirho: &TmuxProbeChirho) -> bool {
    let Some((session_chirho, window_chirho)) = requested_window_chirho(target_chirho) else {
        return true;
    };
    if !session_chirho.is_empty() && probe_chirho.session_chirho.as_deref() != Some(session_chirho)
    {
        return false;
    }
    let observed_chirho = if window_chirho
        .chars()
        .all(|char_chirho| char_chirho.is_ascii_digit())
    {
        probe_chirho.window_index_chirho.as_deref()
    } else {
        probe_chirho.window_name_chirho.as_deref()
    };
    observed_chirho == Some(window_chirho)
}

/// Resolves the live-pane branch in
/// `spec-chirho/workflows-chirho/metropoleluya-http-broker-flow-chirho.md`.
pub(crate) fn probe_tmux_target_chirho(target_chirho: &str) -> TmuxProbeChirho {
    probe_tmux_target_on_server_chirho(&[], target_chirho)
}

pub(crate) fn probe_tmux_target_on_server_chirho(
    server_args_chirho: &[&str],
    target_chirho: &str,
) -> TmuxProbeChirho {
    let output_chirho = Command::new("tmux")
        .args(server_args_chirho)
        .args([
            "display-message",
            "-p",
            "-t",
            target_chirho,
            "#{session_name}\t#{window_index}\t#{pane_id}\t#{window_name}",
        ])
        .output();
    match output_chirho {
        Ok(output_chirho) if output_chirho.status.success() => {
            let text_chirho = String::from_utf8_lossy(&output_chirho.stdout);
            let parts_chirho: Vec<&str> = text_chirho.trim_end_matches('\n').split('\t').collect();
            let field_chirho = |index_chirho: usize| {
                parts_chirho
                    .get(index_chirho)
                    .filter(|value_chirho| !value_chirho.is_empty())
                    .map(|value_chirho| value_chirho.to_string())
            };
            let mut probe_chirho = TmuxProbeChirho {
                alive_chirho: false,
                session_chirho: field_chirho(0),
                window_index_chirho: field_chirho(1),
                pane_id_chirho: field_chirho(2),
                window_name_chirho: field_chirho(3),
                error_chirho: None,
            };
            let resolved_chirho = probe_chirho.session_chirho.is_some()
                && probe_chirho.window_index_chirho.is_some()
                && probe_chirho.pane_id_chirho.is_some();
            if !resolved_chirho {
                probe_chirho.error_chirho =
                    Some("tmux target resolved without a concrete pane_chirho".to_string());
                return probe_chirho;
            }
            if !probe_matches_request_chirho(target_chirho, &probe_chirho) {
                probe_chirho.error_chirho = Some(format!(
                    "tmux target {target_chirho} does not exist; tmux fell back to {}:{} ({})",
                    probe_chirho.session_chirho.as_deref().unwrap_or("?"),
                    probe_chirho.window_index_chirho.as_deref().unwrap_or("?"),
                    probe_chirho.pane_id_chirho.as_deref().unwrap_or("?")
                ));
                probe_chirho.pane_id_chirho = None;
                return probe_chirho;
            }
            probe_chirho.alive_chirho = true;
            probe_chirho
        }
        Ok(output_chirho) => TmuxProbeChirho {
            alive_chirho: false,
            session_chirho: None,
            window_index_chirho: None,
            pane_id_chirho: None,
            window_name_chirho: None,
            error_chirho: Some(
                String::from_utf8_lossy(&output_chirho.stderr)
                    .trim()
                    .to_string(),
            ),
        },
        Err(err_chirho) => TmuxProbeChirho {
            alive_chirho: false,
            session_chirho: None,
            window_index_chirho: None,
            pane_id_chirho: None,
            window_name_chirho: None,
            error_chirho: Some(err_chirho.to_string()),
        },
    }
}

pub(crate) fn next_tmux_buffer_name_chirho() -> String {
    let sequence_chirho = BUFFER_SEQUENCE_CHIRHO.fetch_add(1, Ordering::Relaxed);
    let epoch_nanos_chirho = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    format!(
        "{BUFFER_NAME_STEM_CHIRHO}-{}-{epoch_nanos_chirho}-{sequence_chirho}-chirho",
        std::process::id()
    )
}

fn delete_tmux_buffer_chirho(server_args_chirho: &[&str], buffer_name_chirho: &str) {
    let _delete_result_chirho = run_tmux_chirho(
        server_args_chirho,
        &["delete-buffer", "-b", buffer_name_chirho],
    );
}

fn load_and_paste_tmux_buffer_chirho<AfterLoadChirho>(
    server_args_chirho: &[&str],
    target_chirho: &str,
    text_chirho: &str,
    buffer_name_chirho: &str,
    delete_after_paste_chirho: bool,
    after_load_chirho: AfterLoadChirho,
) -> Result<(), String>
where
    AfterLoadChirho: FnOnce() -> Result<(), String>,
{
    let mut child_chirho = Command::new("tmux")
        .args(server_args_chirho)
        .args(["load-buffer", "-b", buffer_name_chirho, "-"])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err_chirho| err_chirho.to_string())?;
    let mut stdin_chirho = match child_chirho.stdin.take() {
        Some(stdin_chirho) => stdin_chirho,
        None => {
            let _kill_result_chirho = child_chirho.kill();
            let _wait_result_chirho = child_chirho.wait();
            delete_tmux_buffer_chirho(server_args_chirho, buffer_name_chirho);
            return Err("failed to open tmux load-buffer stdin".to_string());
        }
    };
    let write_result_chirho = stdin_chirho.write_all(text_chirho.as_bytes());
    drop(stdin_chirho);
    let output_chirho = match child_chirho.wait_with_output() {
        Ok(output_chirho) => output_chirho,
        Err(err_chirho) => {
            delete_tmux_buffer_chirho(server_args_chirho, buffer_name_chirho);
            return Err(err_chirho.to_string());
        }
    };
    if let Err(err_chirho) = write_result_chirho {
        delete_tmux_buffer_chirho(server_args_chirho, buffer_name_chirho);
        return Err(err_chirho.to_string());
    }
    if !output_chirho.status.success() {
        delete_tmux_buffer_chirho(server_args_chirho, buffer_name_chirho);
        let stderr_chirho = String::from_utf8_lossy(&output_chirho.stderr)
            .trim()
            .to_string();
        return Err(if stderr_chirho.is_empty() {
            "tmux load-buffer failed".to_string()
        } else {
            stderr_chirho
        });
    }

    if let Err(err_chirho) = after_load_chirho() {
        delete_tmux_buffer_chirho(server_args_chirho, buffer_name_chirho);
        return Err(err_chirho);
    }
    let paste_result_chirho = if delete_after_paste_chirho {
        run_tmux_chirho(
            server_args_chirho,
            &[
                "paste-buffer",
                "-d",
                "-b",
                buffer_name_chirho,
                "-t",
                target_chirho,
            ],
        )
    } else {
        run_tmux_chirho(
            server_args_chirho,
            &[
                "paste-buffer",
                "-b",
                buffer_name_chirho,
                "-t",
                target_chirho,
            ],
        )
    };
    if paste_result_chirho.is_err() {
        delete_tmux_buffer_chirho(server_args_chirho, buffer_name_chirho);
    }
    paste_result_chirho
}

/// Resolves aliases to the physical-pane identity used by the target-lock
/// branch in `spec-chirho/workflows-chirho/metropoleluya-http-broker-flow-chirho.md`.
/// Delivery always addresses the physical `%pane_id`, and only after the probe
/// has proven that pane really is the registered window. A stale registration
/// fails loudly here instead of silently pasting into the session's current
/// window.
fn resolve_delivery_target_chirho(
    server_args_chirho: &[&str],
    target_chirho: &str,
) -> Result<String, String> {
    let probe_chirho = probe_tmux_target_on_server_chirho(server_args_chirho, target_chirho);
    match probe_chirho.pane_id_chirho {
        Some(pane_id_chirho) if probe_chirho.alive_chirho => Ok(pane_id_chirho),
        _ => Err(probe_chirho
            .error_chirho
            .unwrap_or_else(|| format!("tmux target {target_chirho} is not deliverable"))),
    }
}

fn target_delivery_key_chirho(server_args_chirho: &[&str], target_chirho: &str) -> String {
    let server_key_chirho = if server_args_chirho.is_empty() {
        "default-tmux-server-chirho".to_string()
    } else {
        server_args_chirho.join("\u{1f}")
    };
    format!("{server_key_chirho}\u{1f}{target_chirho}")
}

/// Holds one cold-retiring physical-pane lease across the complete delivery
/// transaction in `spec-chirho/workflows-chirho/metropoleluya-http-broker-flow-chirho.md`.
fn with_target_delivery_lock_chirho<WorkChirho>(
    server_args_chirho: &[&str],
    target_chirho: &str,
    work_chirho: WorkChirho,
) -> Result<(), String>
where
    WorkChirho: FnOnce(&str) -> Result<(), String>,
{
    let resolved_target_chirho = resolve_delivery_target_chirho(server_args_chirho, target_chirho)?;
    let target_key_chirho = target_delivery_key_chirho(server_args_chirho, &resolved_target_chirho);
    let lease_chirho = acquire_target_delivery_lease_chirho(target_key_chirho);
    let target_guard_chirho = lease_chirho
        .target_lock_chirho
        .lock()
        .unwrap_or_else(|poisoned_chirho| poisoned_chirho.into_inner());
    let result_chirho = work_chirho(&resolved_target_chirho);
    drop(target_guard_chirho);
    drop(lease_chirho);
    result_chirho
}

fn send_tmux_message_on_server_chirho(
    server_args_chirho: &[&str],
    target_chirho: &str,
    text_chirho: &str,
    settle_duration_chirho: Duration,
) -> Result<(), String> {
    with_target_delivery_lock_chirho(
        server_args_chirho,
        target_chirho,
        |resolved_target_chirho| {
            let buffer_name_chirho = next_tmux_buffer_name_chirho();
            load_and_paste_tmux_buffer_chirho(
                server_args_chirho,
                resolved_target_chirho,
                text_chirho,
                &buffer_name_chirho,
                true,
                || Ok(()),
            )?;
            run_tmux_chirho(
                server_args_chirho,
                &["send-keys", "-t", resolved_target_chirho, "Enter"],
            )?;
            thread::sleep(settle_duration_chirho);
            run_tmux_chirho(
                server_args_chirho,
                &["send-keys", "-t", resolved_target_chirho, "Enter"],
            )
        },
    )
}

/// Implements the isolated transport branch in
/// `spec-chirho/workflows-chirho/metropoleluya-http-broker-flow-chirho.md`.
/// Every attempt owns a collision-resistant named buffer; `paste-buffer -d`
/// retires it on success and the error path deletes it best-effort. The full
/// pane transaction is serialized by physical pane while unrelated panes keep
/// independent locks; the lock registry retires an entry after its last lease.
pub(crate) fn send_tmux_message_chirho(
    target_chirho: &str,
    text_chirho: &str,
) -> Result<(), String> {
    send_tmux_message_on_server_chirho(&[], target_chirho, text_chirho, Duration::from_secs(1))
}

#[cfg(test)]
pub(crate) fn paste_tmux_buffer_for_test_chirho<AfterLoadChirho>(
    socket_name_chirho: &str,
    target_chirho: &str,
    text_chirho: &str,
    buffer_name_chirho: &str,
    delete_after_paste_chirho: bool,
    after_load_chirho: AfterLoadChirho,
) -> Result<(), String>
where
    AfterLoadChirho: FnOnce() -> Result<(), String>,
{
    load_and_paste_tmux_buffer_chirho(
        &["-L", socket_name_chirho],
        target_chirho,
        text_chirho,
        buffer_name_chirho,
        delete_after_paste_chirho,
        after_load_chirho,
    )
}

#[cfg(test)]
pub(crate) fn send_tmux_message_for_test_chirho(
    socket_name_chirho: &str,
    target_chirho: &str,
    text_chirho: &str,
    settle_duration_chirho: Duration,
) -> Result<(), String> {
    send_tmux_message_on_server_chirho(
        &["-L", socket_name_chirho],
        target_chirho,
        text_chirho,
        settle_duration_chirho,
    )
}

#[cfg(test)]
pub(crate) fn hold_tmux_target_lock_for_test_chirho<WorkChirho>(
    socket_name_chirho: &str,
    target_chirho: &str,
    work_chirho: WorkChirho,
) -> Result<(), String>
where
    WorkChirho: FnOnce() -> Result<(), String>,
{
    with_target_delivery_lock_chirho(&["-L", socket_name_chirho], target_chirho, |_| {
        work_chirho()
    })
}

#[cfg(test)]
pub(crate) fn target_delivery_lock_count_for_test_chirho(socket_name_chirho: &str) -> usize {
    let server_key_chirho = ["-L", socket_name_chirho].join("\u{1f}");
    let key_prefix_chirho = format!("{server_key_chirho}\u{1f}");
    TARGET_DELIVERY_LOCKS_CHIRHO
        .lock()
        .unwrap_or_else(|poisoned_chirho| poisoned_chirho.into_inner())
        .keys()
        .filter(|target_key_chirho| target_key_chirho.starts_with(&key_prefix_chirho))
        .count()
}

fn run_tmux_chirho(server_args_chirho: &[&str], args_chirho: &[&str]) -> Result<(), String> {
    let output_chirho = Command::new("tmux")
        .args(server_args_chirho)
        .args(args_chirho)
        .output()
        .map_err(|err_chirho| err_chirho.to_string())?;
    if output_chirho.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output_chirho.stderr)
            .trim()
            .to_string())
    }
}
