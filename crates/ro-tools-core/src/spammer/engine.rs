use crate::error::ToolsError;
use crate::ports::InputWriter;
use crate::spammer::config::SpammerConfig;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SpammerTick {
    pub cycled: bool,
    pub cycle_count: u64,
}

pub struct SpammerEngine<I: InputWriter> {
    input: I,
    config: SpammerConfig,
    cycle_count: u64,
}

impl<I: InputWriter> SpammerEngine<I> {
    pub fn new(input: I, config: SpammerConfig) -> Self {
        Self {
            input,
            config: config.clamped(),
            cycle_count: 0,
        }
    }

    pub fn update_config(&mut self, config: SpammerConfig) {
        self.config = config.clamped();
    }

    pub fn config(&self) -> &SpammerConfig {
        &self.config
    }

    pub fn cycle_count(&self) -> u64 {
        self.cycle_count
    }

    /// Ciclo IPC-mode: KEYDOWN → click → KEYUP.
    /// El grab lo hace ro-inputd; cada ciclo es un press discreto independiente del estado físico.
    pub fn tick(&mut self) -> Result<SpammerTick, ToolsError> {
        self.input.key_down("F1")?;
        self.input.click_left()?;
        self.input.key_up("F1")?;

        self.cycle_count += 1;
        Ok(SpammerTick {
            cycled: true,
            cycle_count: self.cycle_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::InputWriter;
    use std::sync::Mutex;

    struct MockInput {
        log: Mutex<Vec<String>>,
    }

    impl InputWriter for MockInput {
        fn key_down(&self, key: &str) -> Result<(), ToolsError> {
            self.log.lock().unwrap().push(format!("down:{key}"));
            Ok(())
        }

        fn key_up(&self, key: &str) -> Result<(), ToolsError> {
            self.log.lock().unwrap().push(format!("up:{key}"));
            Ok(())
        }

        fn press_key(&self, key: &str) -> Result<(), ToolsError> {
            self.log.lock().unwrap().push(format!("key:{key}"));
            Ok(())
        }

        fn click_left(&self) -> Result<(), ToolsError> {
            self.log.lock().unwrap().push("click".into());
            Ok(())
        }
    }

    #[test]
    fn spammer_key_and_click() {
        let input = MockInput {
            log: Mutex::new(vec![]),
        };
        let mut engine = SpammerEngine::new(
            input,
            SpammerConfig {
                enabled: true,
                delay_ms: 10,
            },
        );

        let tick = engine.tick().unwrap();
        assert!(tick.cycled);
        assert_eq!(tick.cycle_count, 1);

        let log = engine.input.log.lock().unwrap();
        assert_eq!(log.as_slice(), &["down:F1", "click", "up:F1"]);
    }

    #[test]
    fn spammer_executes_when_tick_called() {
        let input = MockInput {
            log: Mutex::new(vec![]),
        };
        let mut engine = SpammerEngine::new(input, SpammerConfig::default());
        let tick = engine.tick().unwrap();
        assert!(tick.cycled);
    }
}
