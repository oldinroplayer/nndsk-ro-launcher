use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaunchReservation {
    generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcessState {
    Idle,
    Launching { generation: u64 },
    Running { pid: u32, generation: u64 },
}

#[derive(Clone)]
pub struct GameProcessHandle {
    state: Arc<Mutex<ProcessState>>,
    next_generation: Arc<AtomicU64>,
}

impl GameProcessHandle {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ProcessState::Idle)),
            next_generation: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn begin_launch(&self) -> Result<LaunchReservation, String> {
        let mut state = self.lock()?;
        if *state != ProcessState::Idle {
            return Err("Ya hay un juego iniciándose o en ejecución".to_string());
        }
        let generation = self.next_generation.fetch_add(1, Ordering::Relaxed);
        *state = ProcessState::Launching { generation };
        Ok(LaunchReservation { generation })
    }

    pub fn mark_running(&self, reservation: LaunchReservation, pid: u32) -> Result<(), String> {
        let mut state = self.lock()?;
        match *state {
            ProcessState::Launching { generation } if generation == reservation.generation => {
                *state = ProcessState::Running { pid, generation };
                Ok(())
            }
            _ => Err("La reserva de lanzamiento ya no es válida".to_string()),
        }
    }

    pub fn cancel_launch(&self, reservation: LaunchReservation) {
        if let Ok(mut state) = self.state.lock() {
            if matches!(
                *state,
                ProcessState::Launching { generation } if generation == reservation.generation
            ) {
                *state = ProcessState::Idle;
            }
        }
    }

    pub fn running_pid(&self) -> Result<Option<u32>, String> {
        match *self.lock()? {
            ProcessState::Idle => Ok(None),
            ProcessState::Launching { .. } => Err("El juego todavía se está iniciando".to_string()),
            ProcessState::Running { pid, .. } => Ok(Some(pid)),
        }
    }

    pub fn finish(&self, reservation: LaunchReservation) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        if matches!(
            *state,
            ProcessState::Running { generation, .. } if generation == reservation.generation
        ) {
            *state = ProcessState::Idle;
            return true;
        }
        false
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, ProcessState>, String> {
        self.state
            .lock()
            .map_err(|_| "El estado del proceso del juego está bloqueado".to_string())
    }
}

impl Default for GameProcessHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_parallel_launches_and_recovers_after_cancel() {
        let process = GameProcessHandle::new();
        let first = process.begin_launch().unwrap();
        assert!(process.begin_launch().is_err());
        process.cancel_launch(first);
        assert!(process.begin_launch().is_ok());
    }

    #[test]
    fn only_the_current_generation_can_finish_the_process() {
        let process = GameProcessHandle::new();
        let first = process.begin_launch().unwrap();
        process.mark_running(first, 42).unwrap();

        assert_eq!(process.running_pid().unwrap(), Some(42));
        assert!(process.finish(first));
        assert_eq!(process.running_pid().unwrap(), None);

        let second = process.begin_launch().unwrap();
        process.mark_running(second, 84).unwrap();
        assert!(!process.finish(first));
        assert_eq!(process.running_pid().unwrap(), Some(84));
    }
}
