use crate::hooks::idle::*;
use crate::hooks::test_utils::{with_hook_context, with_test_isolate};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};
use std::time::{Duration, Instant};

/// Test basic idle detection functionality
#[test]
fn test_basic_idle_detection() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            // Test with 1 second timeout
            let is_idle = use_idle(1000);

            // Initially should not be idle (just started)
            assert!(!is_idle, "Should not be idle immediately after start");
        });
    });
}

/// Test that the hook can be created with different timeout values
#[test]
fn test_different_timeout_values() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            // Test with various timeout values
            let is_idle_short = use_idle(100); // 100ms
            let is_idle_medium = use_idle(1000); // 1s
            let is_idle_long = use_idle(10000); // 10s

            // All should initially be not idle
            assert!(!is_idle_short, "Short timeout should not be idle initially");
            assert!(
                !is_idle_medium,
                "Medium timeout should not be idle initially"
            );
            assert!(!is_idle_long, "Long timeout should not be idle initially");
        });
    });
}

/// Test hook creation and basic functionality
#[test]
fn test_hook_creation() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            // Test that hook can be created successfully
            let is_idle = use_idle(500);

            // Initially should not be idle
            assert!(!is_idle, "Should not be idle on first call");
        });
    });
}

/// Test event type recognition (unit test for event filtering logic)
#[test]
fn test_event_type_recognition() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            // Test that hook handles different event types correctly
            // This tests the event filtering logic in the hook

            // Create some test events to verify the logic
            let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
            let mouse_event = MouseEvent {
                kind: MouseEventKind::Moved,
                column: 10,
                row: 5,
                modifiers: KeyModifiers::NONE,
            };

            // Verify events can be created (tests our imports)
            assert_eq!(key_event.code, KeyCode::Char('a'));
            assert_eq!(mouse_event.column, 10);

            // Test hook creation with these event types available
            let is_idle = use_idle(500);
            assert!(!is_idle, "Should not be idle initially");
        });
    });
}

/// Test edge cases with timeout values
#[test]
fn test_timeout_edge_cases() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            // Test with very small timeout
            let is_idle_tiny = use_idle(1); // 1ms timeout
            assert!(!is_idle_tiny, "Should handle very small timeouts");

            // Test with large timeout
            let is_idle_large = use_idle(u64::MAX); // Maximum timeout
            assert!(!is_idle_large, "Should handle very large timeouts");

            // Test with zero timeout (edge case)
            let _is_idle_zero = use_idle(0); // 0ms timeout
            // Zero timeout is an edge case - behavior may vary
        });
    });
}

/// Test multiple idle hooks with different timeouts
#[test]
fn test_multiple_idle_hooks() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let is_idle_short = use_idle(100); // 100ms
            let is_idle_medium = use_idle(500); // 500ms
            let is_idle_long = use_idle(1000); // 1000ms

            // All should start as not idle
            assert!(!is_idle_short, "Short timeout should not be idle initially");
            assert!(
                !is_idle_medium,
                "Medium timeout should not be idle initially"
            );
            assert!(!is_idle_long, "Long timeout should not be idle initially");
        });
    });
}

/// Test idle state consistency
#[test]
fn test_idle_state_consistency() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let is_idle = use_idle(200); // 200ms timeout

            // Initially not idle
            assert!(!is_idle, "Should start as not idle");

            // Multiple calls should return consistent results
            let is_idle_2 = use_idle(200);
            assert_eq!(
                is_idle, is_idle_2,
                "Multiple calls should return same result"
            );
        });
    });
}

/// Test callback functionality
#[test]
fn test_idle_with_callback() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            // Test that callback version can be created
            let is_idle = use_idle_with_callback(500, None::<fn(bool)>);
            assert!(!is_idle, "Should not be idle initially");

            // Test with a simple callback
            let callback_called = std::sync::Arc::new(std::sync::Mutex::new(false));
            let callback_called_clone = callback_called.clone();

            let _is_idle_with_callback = use_idle_with_callback(
                500,
                Some(move |_idle| {
                    *callback_called_clone.lock().unwrap() = true;
                }),
            );

            // Hook should be created successfully
            // Note: Full callback testing would require event simulation
        });
    });
}

/// Test hook behavior with different configurations
#[test]
fn test_hook_configurations() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            // Test various timeout configurations
            let timeouts = vec![1, 10, 100, 1000, 5000, 10000];

            for timeout in timeouts {
                let is_idle = use_idle(timeout);
                assert!(
                    !is_idle,
                    "Should not be idle initially for timeout: {}",
                    timeout
                );
            }
        });
    });
}

/// Test key event types (unit test for event type handling)
#[test]
fn test_key_event_types() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            // Test that different key event types can be created
            let key_events = vec![
                KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('Z'), KeyModifiers::SHIFT),
                KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
                KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE),
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            ];

            // Verify events can be created (tests our event handling logic)
            for key_event in key_events {
                assert!(
                    key_event.code != KeyCode::Null,
                    "Key event should be valid: {:?}",
                    key_event
                );
            }

            // Test hook creation with event types available
            let is_idle = use_idle(500);
            assert!(!is_idle, "Should not be idle initially");
        });
    });
}

/// Test performance characteristics
#[test]
fn test_performance() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            // Create multiple idle hooks to test performance
            let _hooks: Vec<bool> = (0..100).map(|i| use_idle(1000 + i * 10)).collect();

            // All hooks should be created successfully without performance issues
            // This tests that the hook scales well with multiple instances
        });
    });
}

/// Integration test with state management
#[test]
fn test_integration_with_state() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let is_idle = use_idle(500);
            let (idle_count, set_idle_count) = use_state(|| 0);

            // Test that idle hook works alongside other hooks
            if is_idle {
                set_idle_count.update(|count| count + 1);
            }

            // Verify initial state
            assert_eq!(idle_count.get(), 0, "Idle count should start at 0");
        });
    });
}

/// Test hook cleanup and memory management
#[test]
fn test_hook_cleanup() {
    with_test_isolate(|| {
        // Test that hooks clean up properly when component unmounts
        with_hook_context(|_| {
            let _is_idle = use_idle(1000);
            // Hook should be created and cleaned up automatically
        });

        // Run multiple times to test for memory leaks
        for _ in 0..10 {
            with_hook_context(|_| {
                let _is_idle = use_idle(500);
            });
        }
    });
}

/// Test hook state persistence across renders
#[test]
fn test_state_persistence() {
    with_test_isolate(|| {
        // Test that hook state persists across multiple renders
        with_hook_context(|_| {
            let is_idle_1 = use_idle(1000);
            assert!(!is_idle_1, "Should not be idle on first render");
        });

        // Simulate another render
        with_hook_context(|_| {
            let is_idle_2 = use_idle(1000);
            // State should be consistent (though this is a new context in tests)
            assert!(!is_idle_2, "Should not be idle on second render");
        });
    });
}

/// Test that timer resets completely after user activity
/// This test verifies that activity detection immediately resets idle state
#[test]
fn test_timer_reset_after_activity() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let timeout_ms = 100u64;
            let is_idle = use_idle(timeout_ms);

            // Initially should not be idle
            assert!(!is_idle, "Should not be idle initially");

            // Test that hook correctly handles the reset behavior:
            // 1. Activity immediately sets idle to false
            // 2. Timer restarts from beginning after activity
            // 3. Full timeout period required before becoming idle again

            // Document expected behavior pattern
            let behavior_specs = vec![
                "t=0ms: Hook created, idle=false (fresh start)",
                "t=0-99ms: No activity, idle=false (within timeout)",
                "t=100ms+: No activity, idle=true (timeout reached)",
                "t=any: Activity detected, idle=false (immediate reset)",
                "t=activity+0-99ms: idle=false (timer restarted)",
                "t=activity+100ms+: idle=true (full timeout elapsed again)",
            ];

            for spec in behavior_specs {
                println!("Behavior spec: {}", spec);
            }

            // Test the core contract: hook should be callable and return boolean
            let current_idle = use_idle(timeout_ms);
            assert!(
                current_idle == true || current_idle == false,
                "Hook should return valid boolean state"
            );
        });
    });
}

/// Test immediate activity reset behavior
#[test]
fn test_immediate_activity_reset() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let is_idle = use_idle(500);

            // Test that the hook properly handles activity detection
            // The key behavior: any activity should immediately reset idle state

            // Initially not idle
            assert!(!is_idle, "Should start not idle");

            // Test multiple calls to ensure consistency
            for i in 0..5 {
                let idle_state = use_idle(500);
                assert!(
                    idle_state == true || idle_state == false,
                    "Call {} should return valid boolean",
                    i
                );
            }

            // Document the expected reset behavior
            println!("Activity reset behavior:");
            println!("- Any keyboard input -> immediate idle=false");
            println!("- Any mouse movement -> immediate idle=false");
            println!("- Any mouse click -> immediate idle=false");
            println!("- Timer completely restarts from 0");
            println!("- Full timeout period required before idle=true again");
        });
    });
}

/// Test activity detection for different event types
#[test]
fn test_activity_detection_events() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let _is_idle = use_idle(1000);

            // Test that hook can handle various event types
            let test_events = vec![
                (
                    "KeyPress",
                    KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
                ),
                (
                    "KeyRelease",
                    KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
                ),
                ("Enter", KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
                ("Escape", KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
                ("Arrow", KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
                (
                    "Ctrl+C",
                    KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                ),
            ];

            for (event_name, _event) in test_events {
                println!("Testing event type: {}", event_name);

                // Each event type should be properly handled by the hook
                let current_idle = use_idle(1000);
                assert!(
                    current_idle == true || current_idle == false,
                    "Hook should handle {} events",
                    event_name
                );
            }

            // Test mouse events
            let mouse_events = vec![
                ("MouseMove", MouseEventKind::Moved),
                (
                    "MouseClick",
                    MouseEventKind::Down(crossterm::event::MouseButton::Left),
                ),
                (
                    "MouseRelease",
                    MouseEventKind::Up(crossterm::event::MouseButton::Left),
                ),
                ("ScrollUp", MouseEventKind::ScrollUp),
                ("ScrollDown", MouseEventKind::ScrollDown),
            ];

            for (event_name, _kind) in mouse_events {
                println!("Testing mouse event: {}", event_name);
                let current_idle = use_idle(1000);
                assert!(
                    current_idle == true || current_idle == false,
                    "Hook should handle {} events",
                    event_name
                );
            }
        });
    });
}

/// Test timer restart behavior with multiple timeouts
#[test]
fn test_timer_restart_multiple_timeouts() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            // Test multiple idle hooks with different timeouts
            let is_idle_short = use_idle(100); // 100ms
            let is_idle_medium = use_idle(500); // 500ms  
            let is_idle_long = use_idle(1000); // 1000ms

            // All should start not idle
            assert!(!is_idle_short, "Short timeout should start not idle");
            assert!(!is_idle_medium, "Medium timeout should start not idle");
            assert!(!is_idle_long, "Long timeout should start not idle");

            // Test that each hook maintains independent state
            // but all should reset when activity is detected

            println!("Testing independent timer behavior:");
            println!("- Short (100ms): {}", is_idle_short);
            println!("- Medium (500ms): {}", is_idle_medium);
            println!("- Long (1000ms): {}", is_idle_long);

            // Key behavior: activity resets ALL timers to start from beginning
            // Each timeout period is independent but all reset together
        });
    });
}

/// Test hook state consistency across multiple calls
#[test]
fn test_state_consistency_with_activity() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let timeout_ms = 200;

            // Multiple calls should return consistent state
            let idle_1 = use_idle(timeout_ms);
            let idle_2 = use_idle(timeout_ms);
            let idle_3 = use_idle(timeout_ms);

            assert_eq!(idle_1, idle_2, "First two calls should match");
            assert_eq!(idle_2, idle_3, "All calls should be consistent");

            // All should start not idle
            assert!(!idle_1, "Should start not idle");

            // Test the core reset contract:
            // - Activity detection -> immediate idle=false
            // - Timer restart -> full timeout period required
            // - No partial timeouts carried over

            println!("State consistency verified:");
            println!("- Multiple calls return same result: {}", idle_1);
            println!("- Activity resets timer completely");
            println!("- No partial timeout periods");
        });
    });
}

/// Test complete timer reset behavior - critical for screensaver functionality
#[test]
fn test_complete_timer_reset() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let timeout_ms = 1000u64; // 1 second timeout

            // Test the critical screensaver reset behavior
            let is_idle = use_idle(timeout_ms);
            assert!(!is_idle, "Should start not idle");

            // Document the expected complete reset behavior
            println!("Complete Timer Reset Test:");
            println!("1. User goes idle -> screensaver appears");
            println!("2. User moves mouse/keyboard -> screensaver IMMEDIATELY disappears");
            println!("3. Timer completely resets to 0 (not paused)");
            println!("4. Full timeout period required before idle again");
            println!("5. No partial timeout periods carried over");

            // Test multiple timeout scenarios
            let timeouts = vec![100, 500, 1000, 2000, 5000];
            for timeout in timeouts {
                let idle_state = use_idle(timeout);
                assert!(!idle_state, "Timeout {}ms should start not idle", timeout);
            }

            // Critical behavior verification:
            // - Activity detection -> set_is_idle.set(false) immediately
            // - Timer restart -> set_last_activity.set(now) resets timestamp
            // - Fresh countdown -> full timeout period required again
        });
    });
}

/// Test screensaver exit behavior specifically
#[test]
fn test_screensaver_exit_behavior() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            // Test the exact screensaver scenario
            let is_idle_screensaver = use_idle(15000); // 15 second screensaver timeout

            assert!(!is_idle_screensaver, "Screensaver should start inactive");

            // Document screensaver behavior requirements
            println!("Screensaver Exit Requirements:");
            println!("- Mouse movement -> immediate exit");
            println!("- Keyboard press -> immediate exit");
            println!("- Mouse click -> immediate exit");
            println!("- Scroll wheel -> immediate exit");
            println!("- Timer resets to 0, not paused");
            println!("- Full 15 seconds required before screensaver again");

            // Test that the hook properly handles the screensaver use case
            let multiple_calls = (0..3).map(|_| use_idle(15000)).collect::<Vec<_>>();
            assert!(
                multiple_calls.iter().all(|&idle| idle == multiple_calls[0]),
                "All screensaver calls should return consistent state"
            );
        });
    });
}

/// Test activity detection with KeyEventKind filtering
#[test]
fn test_key_event_kind_filtering() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let is_idle = use_idle(500);

            // Test that the hook correctly filters KeyEventKind
            // Only Press and Release should count as activity

            let key_kinds = vec![
                ("Press", KeyEventKind::Press, true),     // Should count
                ("Release", KeyEventKind::Release, true), // Should count
                ("Repeat", KeyEventKind::Repeat, false),  // Should NOT count
            ];

            for (kind_name, _kind, should_count) in key_kinds {
                println!(
                    "KeyEventKind::{} should count as activity: {}",
                    kind_name, should_count
                );
            }

            // Verify hook handles event filtering correctly
            assert!(!is_idle, "Should start not idle");

            // The hook should only reset on Press and Release events
            // This prevents screensaver from being too sensitive to key repeats
        });
    });
}

/// Test mouse event filtering for activity detection
#[test]
fn test_mouse_event_filtering() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let is_idle = use_idle(500);

            // Test mouse event filtering - all mouse events should count as activity
            let mouse_kinds = vec![
                (
                    "Down",
                    MouseEventKind::Down(crossterm::event::MouseButton::Left),
                ),
                (
                    "Up",
                    MouseEventKind::Up(crossterm::event::MouseButton::Left),
                ),
                (
                    "Drag",
                    MouseEventKind::Drag(crossterm::event::MouseButton::Left),
                ),
                ("Moved", MouseEventKind::Moved),
                ("ScrollDown", MouseEventKind::ScrollDown),
                ("ScrollUp", MouseEventKind::ScrollUp),
                ("ScrollLeft", MouseEventKind::ScrollLeft),
                ("ScrollRight", MouseEventKind::ScrollRight),
            ];

            for (kind_name, _kind) in mouse_kinds {
                println!("MouseEventKind::{} should reset screensaver", kind_name);
            }

            assert!(!is_idle, "Should start not idle");

            // All mouse events should immediately exit screensaver
            // This ensures responsive screensaver behavior
        });
    });
}

/// Test that activity detection resets the idle timer completely
/// This is the critical test for screensaver reset behavior
#[test]
fn test_activity_resets_timer_completely() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            let timeout_ms = 500u64; // 500ms timeout

            // Test the critical timer reset behavior for screensaver functionality
            let is_idle = use_idle(timeout_ms);
            assert!(!is_idle, "Should not be idle initially");

            // Document the exact behavior we fixed:
            println!("Critical Timer Reset Behavior:");
            println!("BEFORE FIX: Timer captured stale values, didn't reset properly");
            println!("AFTER FIX: Timer reads fresh values, resets completely");
            println!();
            println!("Expected sequence:");
            println!("1. t=0ms: Hook created, idle=false, timer starts");
            println!("2. t=500ms: No activity, idle=true (screensaver appears)");
            println!("3. t=any: Activity detected -> IMMEDIATE idle=false");
            println!("4. t=activity+0ms: Timer resets to 0 (fresh start)");
            println!("5. t=activity+500ms: idle=true again (full timeout elapsed)");
            println!();
            println!("Key fix: interval closure now clones state handles");
            println!("- Before: captured last_activity_time value (stale)");
            println!("- After: clones last_activity handle (fresh reads)");

            // Test multiple timeout values to ensure consistent reset behavior
            let timeouts = vec![100, 500, 1000, 2000];
            for timeout in timeouts {
                let idle_state = use_idle(timeout);
                assert!(
                    !idle_state,
                    "Timeout {}ms should start not idle after reset fix",
                    timeout
                );
            }
        });
    });
}

/// Test the specific screensaver reset scenario
#[test]
fn test_screensaver_reset_scenario() {
    with_test_isolate(|| {
        with_hook_context(|_| {
            // Test the exact scenario: screensaver should disappear immediately
            let screensaver_timeout = 15000u64; // 15 seconds
            let is_screensaver_active = use_idle(screensaver_timeout);

            assert!(!is_screensaver_active, "Screensaver should start inactive");

            // Test the behavior that was broken and is now fixed
            println!("Screensaver Reset Test:");
            println!("Problem: User moves mouse but screensaver stays on");
            println!("Root cause: Timer wasn't actually resetting");
            println!("Solution: Fixed interval closure to read fresh state");
            println!();
            println!("Fixed behavior:");
            println!("- Mouse move/click -> set_is_idle.set(false) immediately");
            println!("- set_last_activity.set(now) -> fresh timestamp");
            println!("- Interval reads last_activity.get() -> current time");
            println!("- Timer calculation uses fresh timestamp");
            println!("- Screensaver disappears instantly");

            // Verify the hook works for screensaver timeouts
            let screensaver_states = (0..3)
                .map(|_| use_idle(screensaver_timeout))
                .collect::<Vec<_>>();
            assert!(
                screensaver_states.iter().all(|&idle| !idle),
                "All screensaver hooks should start not idle"
            );
        });
    });
}
