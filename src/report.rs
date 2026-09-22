use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Pass,
    Warn,
    Fail,
    Info,
}

#[derive(Debug, Serialize)]
pub struct Finding {
    pub id: &'static str,
    pub severity: Severity,
    pub summary: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub pass: usize,
    pub warn: usize,
    pub fail: usize,
    pub info: usize,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_url: Option<String>,
    pub protocol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_mode: Option<String>,
    pub summary: Summary,
    pub findings: Vec<Finding>,
}

impl Report {
    pub fn new(target: String, protocol: String) -> Self {
        Self {
            target,
            effective_url: None,
            protocol,
            response_mode: None,
            summary: Summary {
                pass: 0,
                warn: 0,
                fail: 0,
                info: 0,
            },
            findings: Vec::new(),
        }
    }

    pub fn push(&mut self, finding: Finding) {
        match finding.severity {
            Severity::Pass => self.summary.pass += 1,
            Severity::Warn => self.summary.warn += 1,
            Severity::Fail => self.summary.fail += 1,
            Severity::Info => self.summary.info += 1,
        }
        self.findings.push(finding);
    }

    pub fn has_failures(&self) -> bool {
        self.summary.fail > 0
    }

    pub fn has_warnings(&self) -> bool {
        self.summary.warn > 0
    }
}

pub fn pass(id: &'static str, summary: impl Into<String>, detail: impl Into<String>) -> Finding {
    Finding {
        id,
        severity: Severity::Pass,
        summary: summary.into(),
        detail: detail.into(),
        remediation: None,
    }
}

pub fn warn(
    id: &'static str,
    summary: impl Into<String>,
    detail: impl Into<String>,
    remediation: impl Into<String>,
) -> Finding {
    Finding {
        id,
        severity: Severity::Warn,
        summary: summary.into(),
        detail: detail.into(),
        remediation: Some(remediation.into()),
    }
}

pub fn fail(
    id: &'static str,
    summary: impl Into<String>,
    detail: impl Into<String>,
    remediation: impl Into<String>,
) -> Finding {
    Finding {
        id,
        severity: Severity::Fail,
        summary: summary.into(),
        detail: detail.into(),
        remediation: Some(remediation.into()),
    }
}

pub fn info(id: &'static str, summary: impl Into<String>, detail: impl Into<String>) -> Finding {
    Finding {
        id,
        severity: Severity::Info,
        summary: summary.into(),
        detail: detail.into(),
        remediation: None,
    }
}

pub fn print_human(report: &Report) {
    println!("MCPDoctor");
    println!("Target:   {}", report.target);
    if let Some(effective_url) = &report.effective_url {
        if effective_url != &report.target {
            println!("Final URL: {effective_url}");
        }
    }
    println!("Protocol: {}", report.protocol);
    if let Some(response_mode) = &report.response_mode {
        println!("Response: {response_mode}");
    }
    println!();

    for finding in &report.findings {
        let marker = match finding.severity {
            Severity::Pass => "PASS",
            Severity::Warn => "WARN",
            Severity::Fail => "FAIL",
            Severity::Info => "INFO",
        };

        println!("[{marker}] {} — {}", finding.id, finding.summary);
        println!("       {}", finding.detail);
        if let Some(remediation) = &finding.remediation {
            println!("       Fix: {remediation}");
        }
    }

    println!();
    println!(
        "Summary: {} pass, {} warn, {} fail, {} info",
        report.summary.pass, report.summary.warn, report.summary.fail, report.summary.info
    );

    if report.has_failures() {
        println!("Result: blocking compatibility findings detected");
    } else if report.has_warnings() {
        println!("Result: no blocking findings; warnings remain");
    } else {
        println!("Result: no blocking findings detected");
    }
}
