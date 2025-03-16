use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use crate::app_discovery::AppInfo;

pub fn search_apps(apps: &[AppInfo], query: &str) -> Vec<AppInfo> {
    if query.is_empty() {
        return Vec::new();
    }
    
    let matcher = SkimMatcherV2::default();
    
    let mut matches: Vec<(i64, AppInfo)> = apps
        .iter()
        .filter_map(|app| {
            let name_score = matcher.fuzzy_match(&app.name.to_lowercase(), &query.to_lowercase());
            let path_score = matcher.fuzzy_match(&app.path.to_lowercase(), &query.to_lowercase())
                .map(|score| score / 2); 
            
            let score = match (name_score, path_score) {
                (Some(n), Some(p)) => Some(std::cmp::max(n, p)),
                (Some(n), None) => Some(n),
                (None, Some(p)) => Some(p),
                (None, None) => None,
            };
            
            score.map(|score| (score, app.clone()))
        })
        .collect();
    
    matches.sort_by(|a, b| b.0.cmp(&a.0));
    matches.into_iter().map(|(_, app)| app).collect()
}
