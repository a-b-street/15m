use web_time::Instant;

pub struct Timer {
    lines: Vec<String>,
    indent: usize,
    last_step: Option<(String, usize, Instant)>,
}

impl Timer {
    pub fn new<I: Into<String>>(overall_name: I) -> Self {
        let mut timer = Self {
            lines: Vec::new(),
            indent: 0,
            last_step: None,
        };
        timer.push(overall_name);
        timer
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
        self.record_last();
        self.last_step = Some((step.into(), self.indent, Instant::now()));
    }

    /// Start a new step with nested steps following it
    pub fn push<I: Into<String>>(&mut self, step: I) {
        self.record_last();
        self.lines
            .push(format!("{}{}", "  ".repeat(self.indent), step.into()));
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
