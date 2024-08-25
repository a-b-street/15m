use wasm_bindgen::prelude::JsValue;
use web_time::Instant;

pub struct Timer {
    cb: Option<js_sys::Function>,

    lines: Vec<String>,
    indent: usize,
    last_step: Option<(String, usize, Instant)>,
}

impl Timer {
    pub fn new<I: Into<String>>(overall_name: I, cb: Option<js_sys::Function>) -> Self {
        let mut timer = Self {
            cb,

            lines: Vec::new(),
            indent: 0,
            last_step: None,
        };
        timer.push(overall_name);
        timer
    }

    pub fn log<I: Into<String>>(&self, msg: I) {
        let msg = msg.into();
        info!("{msg}");
        if let Some(ref cb) = self.cb {
            if let Err(err) = cb.call1(&JsValue::null(), &JsValue::from(msg)) {
                error!("JS progress callback broke: {err:?}");
            }
        }
    }

    fn record_last(&mut self) {
        if let Some((step, indent, start)) = self.last_step.take() {
            self.lines.push(format!(
                "{}{}: {:?}",
                "  ".repeat(indent),
                step,
                Instant::now() - start
            ));
        }
    }

    /// Start a new step, with no nesting
    pub fn step<I: Into<String>>(&mut self, step: I) {
        let step = step.into();
        self.log(step.clone());

        self.record_last();
        self.last_step = Some((step, self.indent, Instant::now()));
    }

    /// Start a new step with nested steps following it
    pub fn push<I: Into<String>>(&mut self, step: I) {
        let step = step.into();
        self.log(step.clone());

        self.record_last();
        self.lines
            .push(format!("{}{}", "  ".repeat(self.indent), step));
        self.indent += 1;
    }

    /// Stop a nested step
    pub fn pop(&mut self) {
        if self.indent == 0 {
            error!("Timer stop() called improperly");
            return;
        }
        self.record_last();
        self.indent -= 1;
    }

    pub fn done(mut self) {
        self.record_last();
        if self.indent != 1 {
            error!("Timer done() called improperly");
        }

        for x in self.lines {
            info!("{x}");
        }
    }
}
