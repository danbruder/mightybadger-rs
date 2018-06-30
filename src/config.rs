use std::env;
use std::sync::{RwLock, RwLockReadGuard};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    pub api_key: Option<String>,
    pub env: Option<String>,
    pub report_data: Option<bool>,
    pub root: Option<String>,
    pub revision: Option<String>,
    pub hostname: Option<String>,
    pub request: RequestConfig,
    #[doc(hidden)]
    pub _non_exhaustive: (),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RequestConfig {
    pub filter_keys: Option<Vec<String>>,
    #[doc(hidden)]
    pub _non_exhaustive: (),
}

impl RequestConfig {
    pub(crate) fn filter_key(&self, key: &str) -> bool {
        if let Some(ref filter_keys) = self.filter_keys {
            filter_keys.iter().any(|s| key.contains(s))
        } else {
            ["password", "HTTP_AUTHORIZATION"]
                .iter()
                .any(|s| key.contains(s))
        }
    }
}

lazy_static! {
    static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
    static ref CONFIG_PROXY: RwLock<Config> = RwLock::new(Config::default());
}

pub fn configure_from_env() {
    fn set_string(entry: &mut Option<String>, env_name: &str) {
        if entry.is_none() {
            *entry = env::var_os(env_name).map(|s| s.to_string_lossy().to_string());
        }
    }

    fn set_bool(entry: &mut Option<bool>, env_name: &str) {
        if entry.is_none() {
            *entry = env::var_os(env_name).map(|s| {
                let s = s.to_string_lossy().to_string();
                ["true", "t", "1"].iter().any(|t| s.eq_ignore_ascii_case(t))
            });
        }
    }

    fn set_string_array(entry: &mut Option<Vec<String>>, env_name: &str) {
        if entry.is_none() {
            *entry = env::var_os(env_name).map(|s| {
                let s = s.to_string_lossy().to_string();
                s.split(",")
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<_>>()
            });
        }
    }

    configure(|config| {
        set_string(&mut config.api_key, "HONEYBADGER_API_KEY");
        set_string(&mut config.env, "HONEYBADGER_ENV");
        set_bool(&mut config.report_data, "HONEYBADGER_REPORT_DATA");
        set_string(&mut config.root, "HONEYBADGER_ROOT");
        set_string(&mut config.revision, "HONEYBADGER_REVISION");
        set_string(&mut config.hostname, "HONEYBADGER_HOSTNAME");
        set_string_array(
            &mut config.request.filter_keys,
            "HONEYBADGER_REQUEST_FILTER_KEYS",
        );
    })
}

pub fn configure<F>(f: F)
where
    F: FnOnce(&mut Config),
{
    let new_config = {
        let mut config_proxy = CONFIG_PROXY.write().unwrap();
        f(&mut config_proxy);
        config_proxy.clone()
    };
    let mut config = CONFIG.write().unwrap();
    *config = new_config;
}

pub(crate) fn read_config_safe() -> RwLockReadGuard<'static, Config> {
    CONFIG.read().unwrap()
}

pub fn read_config() -> RwLockReadGuard<'static, Config> {
    CONFIG_PROXY.read().unwrap()
}