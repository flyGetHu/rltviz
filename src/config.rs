use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
        }
    }
}

impl HttpMethod {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "GET" => Some(Self::GET),
            "POST" => Some(Self::POST),
            "PUT" => Some(Self::PUT),
            "DELETE" => Some(Self::DELETE),
            "PATCH" => Some(Self::PATCH),
            "HEAD" => Some(Self::HEAD),
            "OPTIONS" => Some(Self::OPTIONS),
            _ => None,
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HttpConfig {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Vec<(String, String)>,
    pub body: String,
    #[serde(default)]
    pub insecure: bool,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            url: "https://httpbin.org/get".to_string(),
            method: HttpMethod::GET,
            headers: vec![],
            body: String::new(),
            insecure: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RampUpConfig {
    pub start_concurrency: u32,
    pub end_concurrency: u32,
    pub steps: u32,
    pub step_duration_secs: u64,
}

impl Default for RampUpConfig {
    fn default() -> Self {
        Self {
            start_concurrency: 10,
            end_concurrency: 100,
            steps: 5,
            step_duration_secs: 30,
        }
    }
}

impl RampUpConfig {
    pub fn total_stages(&self) -> u32 {
        self.steps + 1
    }

    pub fn concurrency_step_size(&self) -> u32 {
        if self.steps == 0 {
            return 0;
        }
        (self.end_concurrency.saturating_sub(self.start_concurrency)) / self.steps
    }

    pub fn concurrency_at_stage(&self, stage: u32) -> u32 {
        if stage >= self.total_stages() {
            return self.end_concurrency;
        }
        self.start_concurrency + self.concurrency_step_size() * stage
    }

    pub fn total_duration_secs(&self) -> u64 {
        self.step_duration_secs * self.total_stages() as u64
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub http: HttpConfig,
    pub ramp_up: RampUpConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rampup_total_stages() {
        let cfg = RampUpConfig {
            start_concurrency: 10,
            end_concurrency: 100,
            steps: 5,
            step_duration_secs: 30,
        };
        assert_eq!(cfg.total_stages(), 6);
    }

    #[test]
    fn test_rampup_concurrency_at_stage() {
        let cfg = RampUpConfig {
            start_concurrency: 10,
            end_concurrency: 100,
            steps: 5,
            step_duration_secs: 30,
        };
        assert_eq!(cfg.concurrency_at_stage(0), 10);
        assert_eq!(cfg.concurrency_at_stage(5), 100);
        assert_eq!(cfg.concurrency_at_stage(10), 100);
    }

    #[test]
    fn test_rampup_step_size_zero_steps() {
        let cfg = RampUpConfig {
            start_concurrency: 50,
            end_concurrency: 100,
            steps: 0,
            step_duration_secs: 30,
        };
        assert_eq!(cfg.concurrency_step_size(), 0);
        assert_eq!(cfg.total_stages(), 1);
    }

    #[test]
    fn test_rampup_total_duration() {
        let cfg = RampUpConfig {
            start_concurrency: 10,
            end_concurrency: 100,
            steps: 5,
            step_duration_secs: 30,
        };
        assert_eq!(cfg.total_duration_secs(), 180);
    }

    #[test]
    fn test_http_method_display() {
        assert_eq!(HttpMethod::GET.to_string(), "GET");
        assert_eq!(HttpMethod::POST.to_string(), "POST");
        assert_eq!(HttpMethod::PUT.to_string(), "PUT");
        assert_eq!(HttpMethod::DELETE.to_string(), "DELETE");
    }

    #[test]
    fn test_rampup_step_size_normal() {
        let cfg = RampUpConfig {
            start_concurrency: 10,
            end_concurrency: 100,
            steps: 3,
            step_duration_secs: 30,
        };
        assert_eq!(cfg.concurrency_step_size(), 30); // (100-10)/3 = 30
        assert_eq!(cfg.concurrency_at_stage(0), 10);
        assert_eq!(cfg.concurrency_at_stage(1), 40);
        assert_eq!(cfg.concurrency_at_stage(2), 70);
        assert_eq!(cfg.concurrency_at_stage(3), 100);
    }

    #[test]
    fn test_http_method_from_str() {
        assert_eq!(HttpMethod::from_str("GET"), Some(HttpMethod::GET));
        assert_eq!(HttpMethod::from_str("POST"), Some(HttpMethod::POST));
        assert_eq!(HttpMethod::from_str("PUT"), Some(HttpMethod::PUT));
        assert_eq!(HttpMethod::from_str("DELETE"), Some(HttpMethod::DELETE));
        assert_eq!(HttpMethod::from_str("PATCH"), Some(HttpMethod::PATCH));
        assert_eq!(HttpMethod::from_str("HEAD"), Some(HttpMethod::HEAD));
        assert_eq!(HttpMethod::from_str("OPTIONS"), Some(HttpMethod::OPTIONS));
        assert_eq!(HttpMethod::from_str(""), None);
    }

    #[test]
    fn test_defaults() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.http.method, HttpMethod::GET);
        assert_eq!(cfg.ramp_up.start_concurrency, 10);
    }
}
