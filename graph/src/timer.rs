use wasm_bindgen::prelude::JsValue;
use web_time::Instant;

pub struct Timer {
    cb: Option<js_sys::Function>,

    lines: Vec<String>,
    indent: usize,
    last_step: Option<(String, usize, Instant)>,
    // For each nested step, the index into lines and when it started
    stack: Vec<(usize, Instant)>,
}

impl Timer {
    pub fn new<I: Into<String>>(overall_name: I, cb: Option<js_sys::Function>) -> Self {
        let mut timer = Self {
            cb,

            lines: Vec::new(),
            indent: 0,
            last_step: None,
            stack: Vec::new(),
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
                "{}{}: {}",
                "  ".repeat(indent),
                step,
                format_duration(Instant::now() - start)
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
        self.stack.push((self.lines.len(), Instant::now()));
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
        self.finish_nested();
        self.indent -= 1;
    }

    fn finish_nested(&mut self) {
        if let Some((idx, start)) = self.stack.pop() {
            let line = &mut self.lines[idx];
            line.push_str(&format!(": {}", format_duration(Instant::now() - start)));
        }
    }

    pub fn done(mut self) {
        self.record_last();
        if self.indent != 1 {
            error!("Timer done() called improperly");
        }
        // Only the overall step should be left, but close everything to be safe
        while !self.stack.is_empty() {
            self.finish_nested();
        }

        for x in self.lines {
            info!("{x}");
        }
    }
}

fn format_duration(d: std::time::Duration) -> String {
    let secs = d.as_secs_f64();
    if secs >= 1.0 {
        format!("{secs:.1}s")
    } else {
        format!("{:.1}ms", secs * 1000.0)
    }
}
