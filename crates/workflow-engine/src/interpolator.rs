use std::collections::HashMap;
use regex::Regex;

pub struct VariableInterpolator;

impl VariableInterpolator {
    /// Replaces `{{ params.key }}` and `{{ env.KEY }}` placeholders in a string
    pub fn interpolate(
        template: &str,
        params: &HashMap<String, String>,
        env_vars: &HashMap<String, String>,
    ) -> String {
        let re = Regex::new(r"\{\{\s*([a-zA-Z0-9_\.]+)\s*\}\}").unwrap();
        re.replace_all(template, |caps: &regex::Captures| {
            let key = &caps[1];
            if let Some(param_name) = key.strip_prefix("params.") {
                params.get(param_name).cloned().unwrap_or_else(|| caps[0].to_string())
            } else if let Some(env_name) = key.strip_prefix("env.") {
                env_vars.get(env_name).cloned().unwrap_or_else(|| caps[0].to_string())
            } else if let Some(val) = params.get(key) {
                val.clone()
            } else {
                caps[0].to_string()
            }
        }).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_interpolation() {
        let mut params = HashMap::new();
        params.insert("date".to_string(), "2026-08-15".to_string());
        params.insert("target".to_string(), "prod_db".to_string());

        let mut env = HashMap::new();
        env.insert("REGION".to_string(), "us-east-1".to_string());

        let template = "run-job --date={{ params.date }} --target={{ target }} --region={{ env.REGION }}";
        let result = VariableInterpolator::interpolate(template, &params, &env);

        assert_eq!(result, "run-job --date=2026-08-15 --target=prod_db --region=us-east-1");
    }
}
